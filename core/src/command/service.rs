use std::{sync::{Arc, OnceLock, Weak}, time::Duration};

use anyhow::Context as _;
use dashmap::DashMap;
use parking_lot::RwLock;
use proto::bedrock::{AvailableCommands, Command, ParseResult, ParsedCommand};
use tokio::{sync::{mpsc, oneshot}, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use util::{PVec, FastString, Joinable, Pooled};

use crate::instance::Instance;

const SERVICE_TIMEOUT: Duration = Duration::from_millis(10);

/// Represents a single output message in the command service response.
#[derive(Debug)]
pub struct CommandOutput {
    /// Output of the command.
    pub message: FastString<'static>,
    // pub message: Cow<'static, str>
    /// Optional parameters used in the command output.
    // pub parameters: Vec<Cow<'static, str>>
    pub parameters: Vec<FastString<'static>>
}

/// The result of a command execution.
pub type ExecutionResult = Result<CommandOutput, CommandOutput>;

/// A request that can be sent to the command [`Service`].
#[derive(Debug)]
pub struct ServiceRequest {
    command: String,
    callback: oneshot::Sender<ExecutionResult>
}

// /// Contains data accessible to command handlers
// pub struct Context {
//     instance: Arc<Instance>
//     // level: Arc<crate::level::Service>,
//     // commands: Arc<crate::command::Service>
// }

pub type Context = Arc<Instance>;

/// A function that parses and executes a command.
trait CommandHandler: Send + Sync {
    /// Executes the command using this handlers.
    /// This function also performs parsing of the input.
    fn call(&self, input: &str, ctx: &Context) -> ExecutionResult;
    /// Returns the syntactic structure of the command.
    fn structure(&self) -> &Command;
}

/// A handler that uses the built-in command parser.
struct HandlerImpl<F> 
where
    F: Fn(ParsedCommand, &Context) -> ExecutionResult + Send + Sync
{
    handler: F,
    structure: Command,
}

impl<F> CommandHandler for HandlerImpl<F> 
where
    F: Fn(ParsedCommand, &Context) -> ExecutionResult + Send + Sync
{
    fn call(&self, input: &str, ctx: &Context) -> ExecutionResult {
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

        (self.handler)(parsed, ctx)
    }

    fn structure(&self) -> &Command {
        &self.structure
    }
}

/// A handler that uses a custom user-provided parser.
struct ParserHandlerImpl<F, P> 
where
    F: Fn(ParsedCommand, &Context) -> ExecutionResult + Send + Sync,
    P: Fn(&str, &Context) -> ParseResult + Send + Sync
{
    handler: F,
    parser: P,
    structure: Command
}

impl<F, P> CommandHandler for ParserHandlerImpl<F, P> 
where
    F: Fn(ParsedCommand, &Context) -> ExecutionResult + Send + Sync,
    P: Fn(&str, &Context) -> ParseResult + Send + Sync
{
    fn call(&self, input: &str, ctx: &Context) -> ExecutionResult {
        // Parse command with a custom parser.
        let parsed = match (self.parser)(input, ctx) {
            Ok(cmd) => cmd,
            Err(err) => {
                return Err(CommandOutput {
                    message: err.description,
                    parameters: Vec::new()
                })
            }
        };

        (self.handler)(parsed, ctx)
    }

    fn structure(&self) -> &Command {
        &self.structure
    }
}

/// Service that manages command execution.
pub struct Service {
    token: CancellationToken,
    handle: RwLock<Option<JoinHandle<()>>>,

    sender: mpsc::Sender<ServiceRequest>,
    instance: OnceLock<Weak<Instance>>,

    registry: DashMap<String, Arc<dyn CommandHandler>>,
    available: RwLock<Vec<Command>>
}

impl Service {
    /// Creates a new command service.
    pub fn new(token: CancellationToken) -> Arc<Service> {
        let (sender, receiver) = mpsc::channel(10);
        let service = Arc::new(Service {
            token, sender,
            handle: RwLock::new(None), 
            registry: DashMap::new(),
            available: RwLock::new(Vec::new()), 
            instance: OnceLock::new()
        });

        let clone = Arc::clone(&service);
        let handle = tokio::spawn(async move {
            clone.execution_job(receiver).await
        });

        *service.handle.write() = Some(handle);
        service
    }

    /// Sets the instance pointer of this service.
    /// 
    /// This is used to create contexts when calling command handlers.
    pub(crate) fn set_instance(&self, instance: &Arc<Instance>) -> anyhow::Result<()> {
        self.instance.set(Arc::downgrade(instance)).map_err(|_| anyhow::anyhow!("Instance was already set"))
    }

    // /// Returns a list of all registered commands that can be used in an [`AvailableCommands`] packet.
    // #[inline]
    // pub fn available(&self) -> Reusable<Vec<Command>> {
    //     todo!()
    // }

    /// Registers a new command with the default syntax parser. 
    pub fn register<F>(&self, structure: Command, handler: F) 
    where
        F: Fn(ParsedCommand, &Context) -> ExecutionResult + Send + Sync + 'static
    {   
        let aliases = structure.aliases.clone();
        let name = structure.name.clone();

        let wrap = Arc::new(HandlerImpl {
            handler, structure
        });

        for alias in aliases {
            self.registry.insert(alias, Arc::clone(&wrap) as Arc<dyn CommandHandler>);
        }
        self.registry.insert(name, wrap);
    }

    /// Registers a new command with a custom parser.
    pub fn register_with_parser<F, P>(&self, structure: Command, handler: F, parser: P) 
    where
        F: Fn(ParsedCommand, &Context) -> ExecutionResult + Send + Sync + 'static,
        P: Fn(&str, &Context) -> ParseResult + Send + Sync + 'static
    {
        let aliases = structure.aliases.clone();
        let name = structure.name.clone();

        let wrap = Arc::new(ParserHandlerImpl {
            handler, structure, parser
        });

        for alias in aliases {
            self.registry.insert(alias, Arc::clone(&wrap) as Arc<dyn CommandHandler>);
        }
        self.registry.insert(name, wrap);
    }

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

    /// Parses the syntactic structure of a command before sending it off to a custom handler.
    fn execute_handler(&self, command: &str, ctx: &Context) -> ExecutionResult {
        let command_name = {
            let mut split = command.split(' ');
            let first = split
                .next()
                .ok_or_else(|| {
                    CommandOutput {
                        message: "Expected command name after /".into(),
                        parameters: Vec::new()
                    }
                })?;

            // Get rid of slash in front of name.
            let mut chars = first.chars();
            chars.next();
            chars.as_str()
        };
        
        let Some(handler) = self.registry.get(command_name) else {
            return Err(CommandOutput {
                message: format!("Unknown command {command_name}. Make sure the command exists and you have permission to use it.").into(),
                parameters: Vec::new()
            })
        };
        
        handler.call(command, ctx)
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
                        let Some(ctx) = clone.instance.get() else {
                            tracing::error!("Command service instance was not set");
                            return;
                        };

                        let Some(ctx) = ctx.upgrade() else {
                            tracing::error!("Attempt to create command context failed: instance has been dropped");
                            return;
                        };

                        let result = clone.execute_handler(&request.command, &ctx);
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