use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use bytes::BytesMut;
use parking_lot::RwLock;
use rand::Rng;
use tokio::net::UdpSocket;
use tokio::signal;
use tokio_util::sync::CancellationToken;

use crate::config::SERVER_CONFIG;
use crate::error;
use crate::error::VResult;
use crate::network::{Decodable, Encodable};
use crate::network::packets::{CLIENT_VERSION_STRING, NETWORK_VERSION};
use crate::network::raknet::packets::IncompatibleProtocol;
use crate::network::raknet::packets::OfflinePing;
use crate::network::raknet::packets::OfflinePong;
use crate::network::raknet::packets::OpenConnectionReply1;
use crate::network::raknet::packets::OpenConnectionReply2;
use crate::network::raknet::packets::OpenConnectionRequest1;
use crate::network::raknet::packets::OpenConnectionRequest2;
use crate::network::raknet::RAKNET_VERSION;
use crate::network::raknet::RawPacket;
use crate::network::session::SessionTracker;
use crate::util::AsyncDeque;

/// Local IPv4 address
pub const IPV4_LOCAL_ADDR: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
/// Local IPv6 address
pub const IPV6_LOCAL_ADDR: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
/// Size of the UDP receive buffer.
const RECV_BUF_SIZE: usize = 4096;
/// Refresh rate of the server's metadata.
/// This data is displayed in the server menu.
const METADATA_REFRESH_INTERVAL: Duration = Duration::from_secs(2);

/// Global instance that manages all data and services of the server.
#[derive(Debug)]
pub struct ServerInstance {
    /// Randomised GUID, required by Minecraft
    guid: i64,
    /// String containing info displayed in the server tab.
    metadata: RwLock<String>,
    /// IPv4 UDP socket
    ipv4_socket: Arc<UdpSocket>,
    /// Port the IPv4 service is hosted on.
    ipv4_port: u16,
    /// Queue for incoming packets.
    inward_queue: Arc<AsyncDeque<RawPacket>>,
    /// Queue for packets waiting to be sent.
    outward_queue: Arc<AsyncDeque<RawPacket>>,
    /// Token indicating whether the server is still running.
    /// All services listen to this token to determine whether they should shut down.
    global_token: CancellationToken,
    /// Service that manages all player sessions.
    session_controller: Arc<SessionTracker>,
}

impl ServerInstance {
    /// Creates a new server
    pub async fn new() -> VResult<Arc<Self>> {
        let (ipv4_port, _ipv6_port) = {
            let lock = SERVER_CONFIG.read();
            (lock.ipv4_port, lock.ipv6_port)
        };

        let global_token = CancellationToken::new();
        let ipv4_socket =
            Arc::new(UdpSocket::bind(SocketAddrV4::new(IPV4_LOCAL_ADDR, ipv4_port)).await?);

        let server = Self {
            guid: rand::thread_rng().gen(),
            metadata: RwLock::new(String::new()),

            ipv4_socket,
            ipv4_port,

            inward_queue: Arc::new(AsyncDeque::new(10)),
            outward_queue: Arc::new(AsyncDeque::new(10)),

            session_controller: Arc::new(SessionTracker::new(global_token.clone())),
            global_token,
        };

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

    /// Processes any packets that are sent before a session has been created.
    async fn handle_offline_packet(self: Arc<Self>, packet: RawPacket) -> VResult<()> {
        let id = packet
            .packet_id()
            .ok_or_else(|| error!(BadPacket, "Packet is missing payload"))?;

        match id {
            OfflinePing::ID => self.handle_unconnected_ping(packet).await?,
            OpenConnectionRequest1::ID => self.handle_open_connection_request1(packet).await?,
            OpenConnectionRequest2::ID => self.handle_open_connection_request2(packet).await?,
            _ => unimplemented!("Packet type not implemented"),
        }

        Ok(())
    }

    /// Responds to the [`OfflinePing`] packet with [`OfflinePong`].
    async fn handle_unconnected_ping(self: Arc<Self>, packet: RawPacket) -> VResult<()> {
        let ping = OfflinePing::decode(packet.buffer.clone())?;
        let pong = OfflinePong {
            time: ping.time,
            server_guid: self.guid,
            metadata: self.metadata(),
        }
            .encode()?;

        self.ipv4_socket
            .send_to(pong.as_ref(), packet.address)
            .await?;
        Ok(())
    }

    /// Responds to the [`OpenConnectionRequest1`] packet with [`OpenConnectionReply1`].
    async fn handle_open_connection_request1(self: Arc<Self>, packet: RawPacket) -> VResult<()> {
        let request = OpenConnectionRequest1::decode(packet.buffer.clone())?;
        if request.protocol_version != RAKNET_VERSION {
            let reply = IncompatibleProtocol {
                server_guid: self.guid,
            }
                .encode()?;

            self.ipv4_socket
                .send_to(reply.as_ref(), packet.address)
                .await?;
            return Ok(());
        }

        let reply = OpenConnectionReply1 {
            mtu: request.mtu,
            server_guid: self.guid,
        }
            .encode()?;

        self.ipv4_socket
            .send_to(reply.as_ref(), packet.address)
            .await?;
        Ok(())
    }

    /// Responds to the [`OpenConnectionRequest2`] packet with [`OpenConnectionReply2`].
    /// This is also when a session is created for the client.
    /// From this point, all packets are encoded in a [`Frame`](crate::network::raknet::Frame).

    async fn handle_open_connection_request2(self: Arc<Self>, packet: RawPacket) -> VResult<()> {
        let request = OpenConnectionRequest2::decode(packet.buffer.clone())?;
        let reply = OpenConnectionReply2 {
            server_guid: self.guid,
            mtu: request.mtu,
            client_address: packet.address,
        }
            .encode()?;

        self.session_controller.add_session(
            self.ipv4_socket.clone(),
            packet.address,
            request.mtu,
            request.client_guid,
        );
        self.ipv4_socket
            .send_to(reply.as_ref(), packet.address)
            .await?;

        Ok(())
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
