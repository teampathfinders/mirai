use std::{borrow::Cow, sync::Arc, time::Duration};

use anyhow::Context;
use dashmap::DashMap;
use parking_lot::RwLock;
use proto::bedrock::{Command, CommandOutputMessage, CommandRequest, ParseError, ParseErrorKind, ParseResult, ParsedCommand};
use tokio::{sync::{mpsc, oneshot}, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use util::Joinable;

const SERVICE_TIMEOUT: Duration = Duration::from_millis(10);

/// Represents a single output message in the command service response.
#[derive(Debug)]
pub struct CommandOutput {
    /// Output of the command.
    pub message: Cow<'static, str>,
    /// Optional parameters used in the command output.
    pub parameters: Vec<Cow<'static, str>>
}

/// The result of a command execution.
pub type ExecutionResult = Result<CommandOutput, CommandOutput>;

/// A request that can be sent to the command [`Service`].
#[derive(Debug)]
pub struct ServiceRequest {
    command: String,
    callback: oneshot::Sender<ExecutionResult>
}

/// Type used to communicate with the command [`Service`].
pub struct ServiceEndpoint {
    sender: mpsc::Sender<ServiceRequest>
}

impl ServiceEndpoint {
    /// Request execution of a command.
    /// 
    /// This method will return a receiver that will receive the output when the command has been executed.
    /// Execution of the command might not happen within the same tick.
    pub async fn request(&self, command: String) -> anyhow::Result<oneshot::Receiver<ExecutionResult>> {
        let (sender, receiver) = oneshot::channel();
        let request = ServiceRequest { command, callback: sender };

        self.sender.send_timeout(request, SERVICE_TIMEOUT).await.context("Command service request timed out")?;

        Ok(receiver)
    }
}

impl Clone for ServiceEndpoint {
    fn clone(&self) -> ServiceEndpoint {
        ServiceEndpoint { sender: self.sender.clone() }
    }
}

trait CommandHandler: Send + Sync {
    /// Executes the command using this handlers.
    /// This function also performs parsing of the input.
    fn call(&self, input: &str) -> ExecutionResult;
    /// Returns the syntax structure of the command.
    fn structure(&self) -> &Command;
}

struct HandlerImpl<F> 
where
    F: Fn(ParsedCommand) -> ExecutionResult + Send + Sync
{
    handler: F,
    structure: Command,
}

impl<F> CommandHandler for HandlerImpl<F> 
where
    F: Fn(ParsedCommand) -> ExecutionResult + Send + Sync
{
    fn call(&self, input: &str) -> ExecutionResult {
        // Parse command with default parser.
        let parsed = match ParsedCommand::default_parser(&self.structure, input) {
            Ok(cmd) => cmd,
            Err(err) => {
                return Err(CommandOutput {
                    message: err.description,
                    parameters: Vec::new()
                })
            }
        };

        (self.handler)(parsed)
    }

    fn structure(&self) -> &Command {
        &self.structure
    }
}

struct ParserHandlerImpl<F, P> 
where
    F: Fn(ParsedCommand) -> ExecutionResult + Send + Sync,
    P: Fn(&str) -> ParseResult + Send + Sync
{
    handler: F,
    parser: P,
    structure: Command
}

impl<F, P> CommandHandler for ParserHandlerImpl<F, P> 
where
    F: Fn(ParsedCommand) -> ExecutionResult + Send + Sync,
    P: Fn(&str) -> ParseResult + Send + Sync
{
    fn call(&self, input: &str) -> ExecutionResult {
        // Parse command with a custom parser.
        let parsed = match (self.parser)(input) {
            Ok(cmd) => cmd,
            Err(err) => {
                return Err(CommandOutput {
                    message: err.description,
                    parameters: Vec::new()
                })
            }
        };

        (self.handler)(parsed)
    }

    fn structure(&self) -> &Command {
        &self.structure
    }
}

/// Service that manages command execution.
pub struct Service {
    token: CancellationToken,
    handle: RwLock<Option<JoinHandle<()>>>,
    registry: DashMap<String, Arc<dyn CommandHandler>>,

    /// Simply stored here so it can be used for endpoints.
    sender: mpsc::Sender<ServiceRequest>
}

impl Service {
    /// Creates a new command service.
    pub fn new(token: CancellationToken) -> Arc<Service> {
        let (sender, receiver) = mpsc::channel(10);
        let service = Arc::new(Service {
            token, handle: RwLock::new(None), sender, registry: DashMap::new()
        });

        let clone = Arc::clone(&service);
        let handle = tokio::spawn(async move {
            clone.execution_job(receiver).await
        });

        *service.handle.write() = Some(handle);
        service
    }

    /// Registers a new command with the default syntax parser. 
    pub fn register<F>(&self, structure: Command, handler: F) 
    where
        F: Fn(ParsedCommand) -> ExecutionResult + Send + Sync + 'static
    {   
        let aliases = structure.aliases.clone();
        let name = structure.name.clone();

        let wrap = Arc::new(HandlerImpl {
            handler, structure
        });

        for alias in aliases {
            self.registry.insert(alias, wrap.clone());
        }
        self.registry.insert(name, wrap);
    }

    /// Registers a new command with a custom parser.
    pub fn register_with_parser<F, P>(&self, structure: Command, handler: F, parser: P) 
    where
        F: Fn(ParsedCommand) -> ExecutionResult + Send + Sync + 'static,
        P: Fn(&str) -> ParseResult
    {
        // let aliases = structure.aliases.clone();
        // let name = structure.name.clone();

        // let wrap = Arc::new(CommandHandlerImpl {
        //     handler, structure
        // });

        // for alias in aliases {
        //     self.registry.insert(alias, wrap.clone());
        // }
        // self.registry.insert(name, wrap);
    }

    /// Creates a new [`ServiceEndpoint`].
    pub fn create_endpoint(&self) -> ServiceEndpoint {
        ServiceEndpoint { sender: self.sender.clone() }
    }

    /// Parses the syntactic structure of a command before sending it off to a custom handler.
    fn execute_handler(&self, command: &str) -> ExecutionResult {
        let command_name = {
            let mut split = command.split(' ');
            let first = split
                .next()
                .ok_or_else(|| {
                    CommandOutput {
                        message: Cow::Borrowed("Expected command name after /"),
                        parameters: vec![]
                    }
                })?;

            // Get rid of slash in front of name.
            let mut chars = first.chars();
            chars.next();
            chars.as_str()
        };
        
        let Some(handler) = self.registry.get(command_name) else {
            return Err(CommandOutput {
                message: Cow::Owned(format!("Unknown command {command_name}. Make sure the command exists and you have permission to use it.")),
                parameters: vec![]
            })
        };
        
        handler.call(command)
    }

    /// Runs the service execution job.
    async fn execution_job(self: Arc<Service>, mut receiver: mpsc::Receiver<ServiceRequest>) {
        loop {
            tokio::select! {
                opt = receiver.recv() => {
                    let Some(request) = opt else {
                        tracing::error!("Command service lost all references, this is a bug. Shutting down the service");
                        break
                    };

                    let clone = Arc::clone(&self);
                    tokio::spawn(async move {
                        let result = clone.execute_handler(&request.command);
                        // Error can be ignored because it only occurs if the receiver does not exist anymore.
                        let _: Result<(), ExecutionResult> = request.callback.send(result);
                    });
                }
                _ = self.token.cancelled() => {
                    // Stop accepting requests.
                    receiver.close();
                    break
                }   
            }
        }

        tracing::info!("Command service closed");
    }
}

impl Joinable for Service {
    async fn join(&self) -> anyhow::Result<()> {
        let handle = self.handle.write().take();
        match handle {
            Some(handle) => Ok(handle.await?),
            None => anyhow::bail!("This command service has already been joined.")
        }
    }
}