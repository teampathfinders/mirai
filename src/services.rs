use crate::config::{ServerConfig, CLIENT_VERSION_STRING, NETWORK_VERSION};
use crate::error::{VexError, VexResult};
use crate::raknet::packets::{
    Decodable, RaknetPacket, RawPacket, UnconnectedPing, UnconnectedPong,
};
use crate::raknet::SessionController;
use crate::util::AsyncDeque;
use bytes::BytesMut;
use rand::Rng;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4};
use std::sync::{Arc, RwLock};
use tokio::net::UdpSocket;
use tokio::signal;
use tokio_util::sync::CancellationToken;

const IPV4_LOCAL_ADDR: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const IPV6_LOCAL_ADDR: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);

const RECV_BUF_SIZE: usize = 4096;

pub struct ServerController {
    guid: i64,
    metadata: RwLock<String>,

    ipv4_socket: Arc<UdpSocket>,
    ipv4_port: u16,

    inward_queue: Arc<AsyncDeque<RawPacket>>,
    outward_queue: Arc<AsyncDeque<RawPacket>>,

    global_token: CancellationToken,
    session_controller: Arc<SessionController>,
}

impl ServerController {
    pub async fn new(config: ServerConfig) -> VexResult<Arc<Self>> {
        tracing::info!("Setting up services...");

        let global_token = CancellationToken::new();
        let ipv4_socket =
            Arc::new(UdpSocket::bind(SocketAddrV4::new(IPV4_LOCAL_ADDR, config.ipv4_port)).await?);

        let server = Self {
            guid: rand::thread_rng().gen(),
            metadata: RwLock::new(String::new()),

            ipv4_socket,
            ipv4_port: config.ipv4_port,

            inward_queue: Arc::new(AsyncDeque::new(10)),
            outward_queue: Arc::new(AsyncDeque::new(10)),

            session_controller: Arc::new(SessionController::new(
                global_token.clone(),
                config.max_players,
            )?),
            global_token,
        };
        server.refresh_metadata("Default description")?;

        Ok(Arc::new(server))
    }

    pub async fn run(self: Arc<Self>) -> VexResult<()> {
        ServerController::register_shutdown_handler(self.global_token.clone());

        let receiver_task = {
            let controller = self.clone();
            tokio::spawn(async move { controller.v4_receiver_task().await })
        };

        let sender_task = {
            let controller = self.clone();
            tokio::spawn(async move { controller.v4_sender_task().await })
        };

        let session_handle = self.session_controller.start();

        let _ = tokio::join!(receiver_task, sender_task, session_handle);

        Ok(())
    }

    /// Shut down the server by cancelling the global token
    pub async fn shutdown(&self) {
        self.global_token.cancel();
    }

    async fn handle_offline_packet(self: Arc<Self>, packet: RawPacket) -> VexResult<()> {
        let id = packet
            .packet_id()
            .ok_or(VexError::InvalidRequest("Packet is empty".to_string()))?;
        tracing::info!("{id:0x?}");

        match id {
            UnconnectedPing::ID => self.handle_unconnected_ping(packet).await?,
            OpenConnectionRequest1::ID => self.handle_open_connection_request1(packet).await?,
            OpenConnectionRequest2::ID => self.handle_open_connection_request2(packet).await?,
            _ => todo!("Packet type not implemented"),
        }

        Ok(())
    }

    async fn handle_unconnected_ping(self: Arc<Self>, packet: RawPacket) -> VexResult<()> {
        let ping = UnconnectedPing::decode(packet.buffer.clone())?;
        let pong = UnconnectedPong {
            time: ping.time,
            server_guid: self.guid,
            metadata: self.metadata(),
        }
        .encode();

        self.ipv4_socket
            .send_to(pong.as_ref(), packet.address)
            .await?;
        Ok(())
    }

    async fn handle_open_connection_request1(self: Arc<Self>, packet: RawPacket) -> VexResult<()> {
        let request = OpenConnectionRequest1::decode(packet.buffer.clone())?;
        let reply = OpenConnectionReply1 {
            mtu: request.mtu,
            server_guid: self.guid,
        }
        .encode();

        self.ipv4_socket
            .send_to(reply.as_ref(), packet.address)
            .await?;
        Ok(())
    }

    async fn handle_open_connection_request2(self: Arc<Self>, packet: RawPacket) -> VexResult<()> {
        let request = OpenConnectionRequest2::decode(packet.buffer.clone())?;
        let reply = OpenConnectionReply2 {
            server_guid: self.guid,
            mtu: request.mtu,
            client_address: packet.address,
        }
        .encode();

        self.ipv4_socket
            .send_to(reply.as_ref(), packet.address)
            .await?;
        Ok(())
    }

    /// Receives packets from IPv4 clients and adds them to the receive queue
    async fn v4_receiver_task(self: Arc<Self>) {
        tracing::info!("Inward v4 service online");

        let mut receive_buffer = [0u8; RECV_BUF_SIZE];

        loop {
            // Wait on both the cancellation token and socket at the same time.
            // The token will immediately take over and stop the task when the server is shutting down.
            let (n, address) = tokio::select! {
                result = self.ipv4_socket.recv_from(&mut receive_buffer) => {
                     match result {
                        Ok(r) => r,
                        Err(e) => {
                            tracing::error!("Failed to receive packet: {e:?}");
                            continue;
                        }
                    }
                },
                _ = self.global_token.cancelled() => {
                    break
                }
            };

            let mut raw_packet = RawPacket {
                buffer: BytesMut::from(&receive_buffer[..n]),
                address,
            };

            if raw_packet.is_offline_packet() {
                let controller = self.clone();
                tokio::spawn(async move {
                    controller.handle_offline_packet(raw_packet).await;
                });
            } else {
                todo!("Send packet to session");
            }
        }

        tracing::info!("Inward v4 service shut down");
    }

    /// Sends packets from the send queue
    async fn v4_sender_task(self: Arc<Self>) {
        tracing::info!("Outward v4 service online");

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

        tracing::info!("Outward v4 service shut down");
    }

    fn refresh_metadata(&self, description: &str) -> VexResult<()> {
        let new_id = format!(
            "MCPE;Vex Dedicated Server;{};{};{};{};{};{};Survival;1;{};{};",
            NETWORK_VERSION,
            CLIENT_VERSION_STRING,
            // self.session_controller.player_count(),
            // self.session_controller.max_player_count(),
            0,
            10,
            self.guid,
            description,
            // self.ipv4_port,
            19132,
            19133
        );

        let mut lock = self.metadata.write()?;
        *lock = new_id;

        Ok(())
    }

    fn metadata(&self) -> VexResult<String> {
        let lock = self.metadata.read()?;
        Ok((*lock).clone())
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
