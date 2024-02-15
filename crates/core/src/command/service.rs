use std::{sync::{Arc, OnceLock, Weak}, time::Duration};

use anyhow::Context as _;
use dashmap::DashMap;
use parking_lot::RwLock;
use proto::bedrock::{AvailableCommands, Command, DynamicEnumAction, UpdateDynamicEnum};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use util::Joinable;

use crate::{instance::Instance, net::BedrockClient};

use super::{CommandHandler, Context, HandlerImpl, HandlerOutput, HandlerResult, ParseResult, ParsedCommand, ParserHandlerImpl};

const SERVICE_TIMEOUT: Duration = Duration::from_millis(10);

/// A request that can be sent to the command [`Service`].
pub struct ServiceRequest {
    command: String,
    caller: Arc<BedrockClient>,
    sender: oneshot::Sender<HandlerResult>
}


/// Service that manages command execution.
pub struct Service {
    dynamic_enums: DashMap<String, Vec<String>>,

    /// Cancelled by the instance to trigger a shutdown.
    instance_token: CancellationToken,
    shutdown_token: CancellationToken,

    sender: mpsc::Sender<ServiceRequest>,
    instance: OnceLock<Weak<Instance>>,

    /// Up to date [`AvailableCommands`] packet that can be sent to new users.
    available: RwLock<AvailableCommands<'static>>,
    registry: DashMap<String, Arc<dyn CommandHandler>>
}

impl Service {
    /// Creates a new command service.
    pub(crate) fn new(token: CancellationToken) -> Arc<Service> {
        let (sender, receiver) = mpsc::channel(10);
        let service = Arc::new(Service {
            instance_token: token, sender,
            shutdown_token: CancellationToken::new(),
            registry: DashMap::new(),
            dynamic_enums: DashMap::new(),
            available: RwLock::new(AvailableCommands::empty()),
            instance: OnceLock::new()
        });

        let clone = Arc::clone(&service);
        tokio::spawn(async move {
            clone.service_job(receiver).await
        });

        service
    }

    /// Sets the instance pointer of this service.
    /// 
    /// This is used to create contexts when calling command handlers.
    pub(crate) fn set_instance(&self, instance: &Arc<Instance>) -> anyhow::Result<()> {
        self.instance.set(Arc::downgrade(instance)).map_err(|_| anyhow::anyhow!("Instance was already set"))
    }   

    /// Returns the current [`AvailableCommands`] packet.
    pub(crate) fn available_commands(&self) -> AvailableCommands<'static> {
        self.available.read().clone()
    }

    /// Updates autocompletion entries for the given dynamic enum.
    /// 
    /// This function can only be used with enums that were marked as dynamic on creation.
    /// 
    /// This function returns an error if the dynamic enum does not exist or
    /// if sending the update packet to clients fails.
    pub fn update_enum(&self, update: UpdateDynamicEnum) -> anyhow::Result<()> {
        let Some(mut denum) = self.dynamic_enums.get_mut(update.enum_id) else {
            anyhow::bail!("Dynamic enum does not exist")
        };

        match update.action {
            DynamicEnumAction::Add => {
                denum.extend_from_slice(update.options);
            }
            DynamicEnumAction::Set => {
                denum.clear();
                denum.extend_from_slice(update.options);
            }
            DynamicEnumAction::Remove => {
                denum.retain(|opt| {
                    !update.options.contains(opt)
                })
            }
        }

        self.instance().clients().broadcast(update)
    }

    /// Registers a raw handler with this service.
    /// 
    /// This function returns an error if the service failed to notify clients 
    /// of an updated command list.
    pub fn register_handler(&self, handler: Arc<dyn CommandHandler>) -> anyhow::Result<()> {
        let structure = handler.structure();
        self.available.write().commands.push(structure.clone());

        for alias in &structure.aliases {
            self.registry.insert(alias.clone(), Arc::clone(&handler));
        }

        for overload in &structure.overloads {
            for parameter in &overload.parameters {
                let Some(denum) = &parameter.command_enum else { continue };
                if denum.dynamic {
                    self.dynamic_enums.insert(denum.enum_id.clone(), denum.options.clone());
                }
            }
        }

        self.registry.insert(structure.name.clone(), handler);
        self.instance().clients().broadcast(self.available_commands())
    }

    /// Registers a new command with the default syntax parser. 
    /// 
    /// ## Arguments
    /// 
    /// * `structure` - This is the syntactic structure of the command and is what the game uses
    /// for autocompletion. This grammar is also what is used by the server's command parser to
    /// parse and validate the command.
    /// 
    /// * `handler` - This is the function that is ran by the service when your command is executed
    /// by a client. 
    /// 
    /// This function returns an error if the service failed to notify clients 
    /// of an updated command list.
    pub fn register<F>(&self, structure: Command, handler: F)  -> anyhow::Result<()>
    where
        F: Fn(ParsedCommand, &Context) -> HandlerResult + Send + Sync + 'static
    {   
        let handler = Arc::new(HandlerImpl {
            handler, structure
        });
        
        self.register_handler(handler)
    }

    /// Registers a new command with a custom parser.
    /// 
    /// This function returns an error if the service failed to notify clients 
    /// of an updated command list.
    pub fn register_with_parser<F, P>(&self, structure: Command, handler: F, parser: P) -> anyhow::Result<()>
    where
        F: Fn(ParsedCommand, &Context) -> HandlerResult + Send + Sync + 'static,
        P: Fn(&str, &Context) -> ParseResult + Send + Sync + 'static
    {
        let handler = Arc::new(ParserHandlerImpl {
            handler, structure, parser
        });
        
        self.register_handler(handler)
    }

    /// Removes a command from the registry and returns its handler.
    /// 
    /// This function does not accept command aliases, you should use the original name of the command.
    pub fn unregister<S: AsRef<str>>(&self, name: S) -> Option<Arc<dyn CommandHandler>> {
        todo!("Remove function from commands packet");

        self.registry.remove(name.as_ref()).map(|(_, v)| v)
    }

    /// Request execution of a command.
    /// 
    /// This method will return a receiver that will receive the output when the command has been executed.
    /// Execution of the command might not happen within the same tick.
    pub(crate) async fn execute(&self, caller: Arc<BedrockClient>, command: String) 
        -> anyhow::Result<oneshot::Receiver<HandlerResult>> 
    {
        let (sender, receiver) = oneshot::channel();
        let request = ServiceRequest { command, caller, sender };

        self.sender.send_timeout(request, SERVICE_TIMEOUT).await.context("Command service request timed out")?;

        Ok(receiver)
    }

    /// Returns the instance that owns this service.
    fn instance(&self) -> Arc<Instance> {
        // This will not panic because the instance field is initialised before the first command can be executed.
        #[allow(clippy::unwrap_used)]
        self.instance.get().unwrap().upgrade().unwrap()
    }

    /// Parses the syntactic structure of a command before sending it off to a custom handler.
    fn execute_handler(&self, command: &str, ctx: &Context) -> HandlerResult {
        let command_name = {
            let mut split = command.split(' ');
            let first = split
                .next()
                .ok_or_else(|| {
                    HandlerOutput {
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
            return Err(HandlerOutput {
                message: format!("Unknown command {command_name}. Make sure the command exists and you have permission to use it.").into(),
                parameters: Vec::new()
            })
        };
        
        handler.call(command, ctx)
    }

    /// Runs the service execution job.
    async fn service_job(self: Arc<Service>, mut receiver: mpsc::Receiver<ServiceRequest>) {
        loop {
            tokio::select! {
                opt = receiver.recv() => {
                    let Some(request) = opt else {
                        tracing::error!("Command service lost all references, this is a bug. Shutting down the service");
                        break
                    };

                    let clone = Arc::clone(&self);
                    tokio::spawn(async move {
                        let Some(instance) = clone.instance.get() else {
                            tracing::error!("Command service instance was not set");
                            return;
                        };

                        let Some(instance) = instance.upgrade() else {
                            tracing::error!("Attempt to create command context failed: instance has been dropped");
                            return;
                        };

                        let ctx = Context {
                            caller: request.caller, instance: instance
                        };

                        let result = clone.execute_handler(&request.command, &ctx);
                        // Error can be ignored because it only occurs if the receiver does not exist anymore.
                        let _: Result<(), HandlerResult> = request.sender.send(result);
                    });
                }
                _ = self.instance_token.cancelled() => {
                    // Stop accepting requests.
                    receiver.close();
                    break
                }   
            }
        }

        self.shutdown_token.cancel();
        tracing::info!("Command service closed");
    }
}

impl Joinable for Service {
    async fn join(&self) -> anyhow::Result<()> {
        self.shutdown_token.cancelled().await;
        Ok(())
    }
}