use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4};
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use bytes::BytesMut;
use parking_lot::RwLock;
use rand::Rng;
use tokio::net::UdpSocket;
use tokio::signal;
use tokio_util::sync::CancellationToken;

use vex_common::error;
use vex_common::VResult;
use vex_raknet::Listener;

use crate::player::Player;

/// Global instance that manages all data and services of the server.
#[derive(Debug)]
pub struct ServerInstance {
    /// Token indicating whether the server is still running.
    /// All services listen to this token to determine whether they should shut down.
    token: CancellationToken,
}

impl ServerInstance {
    /// Creates a new server
    pub async fn start() -> VResult<Arc<Self>> {
        let token = CancellationToken::new();
        Self::register_shutdown_handler(token.clone());

        let listener = vex_raknet::Listener::new(token.clone(), |session| {
            tracing::info!("Received game packet");

            Ok(Arc::new(move |packet| {
                let session = session.clone();

                Box::pin(async move {
                    println!("Received packet! {:x?}", packet.as_ref()[0]);

                    let player = Player::new(session);

                    Ok(())
                })
            }))
        }).await?;

        listener.start().await?;

        let server = Self { token };
        Ok(Arc::new(server))
    }

    /// Shut down the server by cancelling the global token
    pub fn shutdown(&self) {
        self.token.cancel();
    }

    /// Register handler to shut down server on Ctrl-C signal
    fn register_shutdown_handler(token: CancellationToken) {
        tokio::spawn(async move {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    tracing::info!("Shutting down...");
                    token.cancel();
                },
                _ = token.cancelled() => {
                    // Token has been cancelled by something else, this service is no longer needed
                }
            }
        });
    }
}
