use crate::config::{ServerConfig, CLIENT_VERSION_STRING, NETWORK_VERSION};
use crate::error::VexResult;
use crate::raknet::{NetController, SessionController};
use rand::Rng;
use std::sync::{Arc, RwLock};
use tokio::signal;
use tokio_util::sync::CancellationToken;

pub struct ServerController {
    guid: i64,
    metadata: RwLock<String>,

    global_token: CancellationToken,
    session_controller: Arc<SessionController>,
    net_controller: Arc<NetController>,
}

impl ServerController {
    pub async fn new(config: ServerConfig) -> VexResult<Self> {
        tracing::info!("Setting up services...");

        let guid = rand::thread_rng().gen();
        tracing::debug!("SERVER GUID: {:#0X}", guid as u64);

        let global_token = CancellationToken::new();

        let controller = Self {
            guid,
            metadata: RwLock::new(String::new()),

            session_controller: Arc::new(SessionController::new(global_token.clone(), config.max_players)?),
            net_controller: Arc::new(NetController::new(global_token.clone(), config.ipv4_port).await?),
            global_token,
        };

        // Generate initial metadata
        controller.refresh_metadata("Default description")?;

        Ok(controller)
    }

    pub async fn run(&self) -> VexResult<()> {
        ServerController::register_shutdown_handler(self.global_token.clone());

        let net_handle = self.net_controller.clone().start();
        let session_handle = self.session_controller.start();

        let _ = tokio::join!(net_handle, session_handle);

        Ok(())
    }

    /// Shut down the server by cancelling the global token
    pub async fn shutdown(&self) {
        self.global_token.cancel();
    }

    fn refresh_metadata(&self, description: &str) -> VexResult<()> {
        let new_id = format!(
            "MCPE;Vex Dedicated Server;{};{};{};{};{};{};Survival;1;{};{};",
            NETWORK_VERSION,
            CLIENT_VERSION_STRING,
            self.session_controller.player_count(),
            self.session_controller.max_player_count(),
            self.guid,
            description,
            self.net_controller.ipv4_port(),
            19133
        );

        let mut lock = self.metadata.write()?;
        *lock = new_id;

        Ok(())
    }

    /// Register handler to shut down server on Ctrl-C signal
    fn register_shutdown_handler(token: CancellationToken) {
        tracing::info!("Registered shutdown handler");

        tokio::spawn(async move {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    tracing::info!("Ctrl-C detected, token cancelled, shutting down services...");
                    token.cancel();
                },
                _ = token.cancelled() => {
                    // Token has been cancelled by something else, this service is no longer needed
                }
            }
        });
    }
}
