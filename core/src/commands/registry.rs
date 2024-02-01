use std::sync::Arc;

use dashmap::DashMap;
use proto::bedrock::{Command, CommandRequest};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::net::BedrockUser;

#[derive(Debug, Clone)]
pub enum CommandExecutionSource {
    Player(u64)
}

#[derive(Debug, Clone)]
pub struct CommandExecutionData {
    source: CommandExecutionSource
}

struct CommandRegistryEntry {
    command: Command,
}

pub struct CommandServiceRequest {
    
}

/// Services that schedules execution of commands.
pub struct CommandService {
    receiver: mpsc::Receiver<CommandServiceRequest>,

    registry: DashMap<String, CommandRegistryEntry>,
    token: CancellationToken
}

impl CommandService {
    pub fn new(token: CancellationToken) -> (Arc<CommandService>, mpsc::Sender<CommandServiceRequest>) {
        let (sender, receiver) = mpsc::channel(10);
        let service = Arc::new(CommandService { registry: DashMap::new(), token, receiver });

        let clone = service.clone();
        tokio::spawn(async move {
            clone.execution_job().await
        });

        (service, sender)
    }

    /// Registers a new command in the registry with the specified channel.
    pub fn register(&self, command: Command, channel: mpsc::Sender<CommandExecutionData>) {
        let entry = CommandRegistryEntry {
            command
        };

        self.registry.insert(command.name.clone(), entry);
    }

    /// Deregisters a command from the registry.
    pub fn deregister(&self, name: &str) -> Option<Command> {
        self.registry.remove(name).map(|(_, v)| v)
    }

    async fn execution_job(self: Arc<Self>) {
        loop {
            tokio::select! {

            }
        }

        tracing::info!("Command execution service exited");
    }
}