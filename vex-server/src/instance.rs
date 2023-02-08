use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4};
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

use crate::config::SERVER_CONFIG;
use crate::network::{Decodable, Encodable};
use crate::util::AsyncDeque;

/// Global instance that manages all data and services of the server.
#[derive(Debug)]
pub struct ServerInstance {
    raknet_listener: Listener,
    /// Token indicating whether the server is still running.
    /// All services listen to this token to determine whether they should shut down.
    global_token: CancellationToken,
}

impl ServerInstance {
    /// Creates a new server
    pub async fn new() -> VResult<Arc<Self>> {
        let (ipv4_port, _ipv6_port) = {
            let lock = SERVER_CONFIG.read();
            (lock.ipv4_port, lock.ipv6_port)
        };

        let global_token = CancellationToken::new();


        Ok(Arc::new(server))
    }

    /// Run the server
    pub async fn run(self: Arc<Self>) -> VResult<()> {
        Self::register_shutdown_handler(self.global_token.clone());

        let receiver_task = {
            let controller = self.clone();
            tokio::spawn(async move { controller.v4_receiver_task().await })
        };

        let sender_task = {
            let controller = self.clone();
            tokio::spawn(async move { controller.v4_sender_task().await })
        };

        {
            let controller = self.clone();
            tokio::spawn(async move { controller.metadata_refresh_task().await });
        }

        tracing::info!("Server started");
        // The metadata task is not important for shutdown, we don't have to wait for it.
        let _ = tokio::join!(receiver_task, sender_task);

        Ok(())
    }

    /// Shut down the server by cancelling the global token
    pub fn shutdown(&self) {
        self.global_token.cancel();
    }

    /// Receives packets from IPv4 clients and adds them to the receive queue
    async fn v4_receiver_task(self: Arc<Self>) {
        let mut receive_buffer = [0u8; RECV_BUF_SIZE];

        loop {
            // Wait on both the cancellation token and socket at the same time.
            // The token will immediately take over and stop the task when the server is shutting down.
            let (n, address) = tokio::select! {
                result = self.ipv4_socket.recv_from(&mut receive_buffer) => {
                     match result {
                        Ok(r) => r,
                        Err(e) => {
                            tracing::error!("Could not receive packet: {e:?}");
                            continue;
                        }
                    }
                },
                _ = self.global_token.cancelled() => {
                    break
                }
            };

            let raw_packet = RawPacket {
                buffer: BytesMut::from(&receive_buffer[..n]),
                address,
            };

            if raw_packet.is_offline_packet() {
                let controller = self.clone();
                tokio::spawn(async move {
                    match controller.handle_offline_packet(raw_packet).await {
                        Ok(_) => (),
                        Err(e) => {
                            tracing::error!(
                                "Error occurred while processing offline packet: {e:?}"
                            );
                        }
                    }
                });
            } else {
                match self.session_controller.forward_packet(raw_packet) {
                    Ok(_) => (),
                    Err(e) => {
                        tracing::error!("{}", e.to_string());
                        continue;
                    }
                }
            }
        }
    }

    /// Sends packets from the send queue
    async fn v4_sender_task(self: Arc<Self>) {
        loop {
            let task = tokio::select! {
                _ = self.global_token.cancelled() => break,
                t = self.outward_queue.pop() => t
            };

            match self.ipv4_socket.send_to(&task.buffer, task.address).await {
                Ok(_) => (),
                Err(e) => {
                    tracing::error!("Failed to send packet: {e:?}");
                }
            }
        }
    }

    /// Refreshes the server description and player counts on a specified interval.
    async fn metadata_refresh_task(self: Arc<Self>) {
        let mut interval = tokio::time::interval(METADATA_REFRESH_INTERVAL);
        while !self.global_token.is_cancelled() {
            let description = format!("{} players", self.session_controller.session_count());
            self.refresh_metadata(&description);
            interval.tick().await;
        }
    }

    /// Generates a new metadata string using the given description and new player count.
    fn refresh_metadata(&self, description: &str) {
        let new_id = format!(
            "MCPE;{};{};{};{};{};{};{};Survival;1;{};{};",
            description,
            NETWORK_VERSION,
            CLIENT_VERSION_STRING,
            self.session_controller.session_count(),
            self.session_controller.max_session_count(),
            self.guid,
            SERVER_CONFIG.read().server_name,
            self.ipv4_port,
            19133
        );

        let mut lock = self.metadata.write();
        *lock = new_id;
    }

    /// Returns the current metadata string.
    #[inline]
    fn metadata(&self) -> String {
        (*self.metadata.read()).clone()
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
