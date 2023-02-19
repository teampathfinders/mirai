use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use bytes::{BytesMut, Bytes};
use parking_lot::RwLock;
use rand::Rng;
use tokio::net::UdpSocket;
use tokio::signal;
use tokio::sync::{OnceCell, mpsc};
use tokio::sync::oneshot::Receiver;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::command::{
    Command, CommandDataType, CommandEnum, CommandOverload, CommandParameter,
    CommandPermissionLevel,
};
use crate::config::SERVER_CONFIG;
use crate::level_manager::LevelManager;
use crate::network::packets::{
    GameRule, BOOLEAN_GAME_RULES, CLIENT_VERSION_STRING, INTEGER_GAME_RULES,
    NETWORK_VERSION,
};
use crate::network::raknet::packets::IncompatibleProtocol;
use crate::network::raknet::packets::OfflinePing;
use crate::network::raknet::packets::OfflinePong;
use crate::network::raknet::packets::OpenConnectionReply1;
use crate::network::raknet::packets::OpenConnectionReply2;
use crate::network::raknet::packets::OpenConnectionRequest1;
use crate::network::raknet::packets::OpenConnectionRequest2;
use crate::network::raknet::BufPacket;
use crate::network::raknet::RAKNET_VERSION;
use crate::network::session::SessionManager;
use common::{AsyncDeque, bail};
use common::{error, VResult};
use common::{Deserialize, Serialize};

/// Local IPv4 address
pub const IPV4_LOCAL_ADDR: Ipv4Addr = Ipv4Addr::UNSPECIFIED;
/// Local IPv6 address
pub const IPV6_LOCAL_ADDR: Ipv6Addr = Ipv6Addr::UNSPECIFIED;
/// Size of the UDP receive buffer.
const RECV_BUF_SIZE: usize = 4096;
/// Refresh rate of the server's metadata.
/// This data is displayed in the server menu.
const METADATA_REFRESH_INTERVAL: Duration = Duration::from_secs(2);

/// Global instance that manages all data and services of the server.
#[derive(Debug)]
pub struct InstanceManager {
    /// Randomised GUID, required by Minecraft
    guid: i64,
    /// String containing info displayed in the server tab.
    metadata: RwLock<String>,
    /// IPv4 UDP socket
    ipv4_socket: Arc<UdpSocket>,
    /// Port the IPv4 service is hosted on.
    ipv4_port: u16,
    /// Queue for incoming packets.
    inward_queue: Arc<AsyncDeque<BufPacket>>,
    /// Queue for packets waiting to be sent.
    outward_queue: Arc<AsyncDeque<BufPacket>>,
    /// Token indicating whether the server is still running.
    /// All services listen to this token to determine whether they should shut down.
    token: CancellationToken,
    /// Service that manages all player sessions.
    session_manager: Arc<SessionManager>,
    /// Manages the level.
    level_manager: RwLock<Option<Arc<LevelManager>>>,
    /// Channel that the LevelManager sends a message to when it has fully shutdown.
    /// This is to make sure that the world has been saved and safely shut down before shutting down the server.
    level_notifier: Receiver<()>
}

impl InstanceManager {
    /// Creates a new server.
    pub async fn run() -> VResult<()> {
        let (ipv4_port, _ipv6_port) = {
            let lock = SERVER_CONFIG.read();
            (lock.ipv4_port, lock.ipv6_port)
        };

        let global_token = CancellationToken::new();
        let ipv4_socket = Arc::new(
            UdpSocket::bind(SocketAddrV4::new(IPV4_LOCAL_ADDR, ipv4_port))
                .await?,
        );

        let session_manager =
            Arc::new(SessionManager::new(global_token.clone()));

        let (level_manager, level_notifier) = LevelManager::new(session_manager.clone())?;
        level_manager.add_command(Command {
            name: "gamerule".to_owned(),
            description: "Sets or queries a game rule value.".to_owned(),
            permission_level: CommandPermissionLevel::Normal,
            aliases: vec![],
            overloads: vec![
                // Boolean game rules.
                CommandOverload {
                    parameters: vec![
                        CommandParameter {
                            data_type: CommandDataType::String,
                            name: "rule".to_owned(),
                            suffix: "".to_owned(),
                            command_enum: Some(CommandEnum {
                                dynamic: false,
                                enum_id: "boolean gamerule".to_owned(),
                                options: BOOLEAN_GAME_RULES
                                    .iter()
                                    .map(|g| g.to_string())
                                    .collect::<Vec<_>>(),
                            }),
                            optional: false,
                            options: 0,
                        },
                        CommandParameter {
                            data_type: CommandDataType::String,
                            name: "value".to_owned(),
                            suffix: "".to_owned(),
                            command_enum: Some(CommandEnum {
                                dynamic: false,
                                enum_id: "boolean".to_owned(),
                                options: vec![
                                    "true".to_owned(),
                                    "false".to_owned(),
                                ],
                            }),
                            optional: true,
                            options: 0,
                        },
                    ],
                },
                // Integral game rules.
                CommandOverload {
                    parameters: vec![
                        CommandParameter {
                            data_type: CommandDataType::String,
                            name: "rule".to_owned(),
                            suffix: "".to_owned(),
                            command_enum: Some(CommandEnum {
                                dynamic: false,
                                enum_id: "integral gamerule".to_owned(),
                                options: INTEGER_GAME_RULES
                                    .iter()
                                    .map(|g| g.to_string())
                                    .collect::<Vec<_>>(),
                            }),
                            optional: false,
                            options: 0,
                        },
                        CommandParameter {
                            data_type: CommandDataType::Int,
                            name: "value".to_owned(),
                            suffix: "this is a suffix".to_owned(),
                            command_enum: None,
                            optional: true,
                            options: 0,
                        },
                    ],
                },
            ],
        });
        level_manager.add_command(Command {
            name: "daylock".to_owned(),
            description: "Locks and unlocks the day-night cycle.".to_owned(),
            aliases: vec![],
            permission_level: CommandPermissionLevel::Normal,
            overloads: vec![CommandOverload {
                parameters: vec![CommandParameter {
                    data_type: CommandDataType::String,
                    name: "lock".to_owned(),
                    suffix: "".to_owned(),
                    command_enum: Some(CommandEnum {
                        dynamic: false,
                        enum_id: "boolean".to_owned(),
                        options: vec!["true".to_owned(), "false".to_owned()],
                    }),
                    optional: true,
                    options: 0,
                }],
            }],
        });

        session_manager.set_level_manager(Arc::downgrade(&level_manager))?;

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
            tokio::spawn(
                async move { controller.metadata_refresh_task().await },
            );
        }

        tracing::info!("Server started");
        // The metadata task is not important for shutdown, we don't have to wait for it.
        let _ = tokio::join!(receiver_task, sender_task);

        let server = Arc::new(Self {
            guid: rand::thread_rng().gen(),
            metadata: RwLock::new(String::new()),

            ipv4_socket,
            ipv4_port,

            inward_queue: Arc::new(AsyncDeque::new(10)),
            outward_queue: Arc::new(AsyncDeque::new(10)),

            session_manager,
            level_manager: RwLock::new(Some(level_manager)),
            token: global_token,
            level_notifier
        });

        Ok(())
    }

    /// Shut down the server by cancelling the global token
    pub async fn shutdown(mut self) {
        tracing::info!("Disconnecting all clients");
        self.session_manager.kick_all("Server closed").await;
        tracing::info!("Notifying services");
        self.token.cancel();

        // Destroy level manager
        tracing::info!("Shutting down chunk manager");
        *self.level_manager.write() = None;
        let _ = self.level_notifier.await;
    }

    /// Generates a response to the [`OfflinePing`] packet with [`OfflinePong`].
    #[inline]
    fn process_unconnected_ping(
        pk: BufPacket,
        server_guid: u64,
        metadata: &str
    ) -> VResult<BufPacket> {
        let ping = OfflinePing::deserialize(pk.buf)?;
        pk.buf = OfflinePong {
            time: ping.time,
            server_guid,
            metadata,
        }
        .serialize()?;

        Ok(pk)
    }

    /// Generates a response to the [`OpenConnectionRequest1`] packet with [`OpenConnectionReply1`].
    #[inline]
    fn process_open_connection_request1(
        pk: BufPacket,
        server_guid: u64
    ) -> VResult<()> {
        let request =
            OpenConnectionRequest1::deserialize(pk.buf)?;

        if request.protocol_version != RAKNET_VERSION {
            pk.buf = IncompatibleProtocol { server_guid }.serialize()?;
            return Ok(pk)
        }

        pk.buf = OpenConnectionReply1 { 
            mtu: request.mtu, server_guid 
        }.serialize()?;

        Ok(pk)
    }

    /// Responds to the [`OpenConnectionRequest2`] packet with [`OpenConnectionReply2`].
    /// This is also when a session is created for the client.
    /// From this point, all packets are encoded in a [`Frame`](crate::network::raknet::Frame).
    #[inline]
    fn process_open_connection_request2(
        udp_socket: Arc<UdpSocket>,
        sess_manager: Arc<SessionManager>,
        pk: BufPacket,
    ) -> VResult<()> {
        let request =
            OpenConnectionRequest2::deserialize(pk.buf)?;

        pk.buf = OpenConnectionReply2 {
            server_guid,
            mtu: request.mtu,
            client_address: &pk.addr,
        }
        .serialize()?;

        sess_manager.add_session(
            udp_socket,
            pk.addr,
            request.mtu,
            request.client_guid,
        );

        Ok(pk)
    }

    /// Receives packets from IPv4 clients and adds them to the receive queue
    async fn udp_recv_job(
        udp_socket: Arc<UdpSocket>,
        sess_manager: Arc<SessionManager>,
        server_guid: u64,
        metadata: Arc<RwLock<String>>
    ) {
        let mut recv_buf = [0u8; RECV_BUF_SIZE];

        loop {
            let (n, address) = tokio::select! {
                r = udp_socket.recv_from(&mut recv_buf) => {
                    match r {
                        Ok(r) => r,
                        Err(e) => {
                            tracing::error!("UdpSocket recv_from failed: {e}");
                            continue
                        }
                    }
                }
            };

            let mut pk = BufPacket {
                buf: Bytes::from(&recv_buf[..n]),
                addr: address,
            };     

            if pk.is_unconnected() {
                let udp_socket = udp_socket.clone();
                
                tokio::spawn(async move {
                    let id = if let Some(id) = pk.packet_id() {
                        id
                    } else {
                        tracing::error!("Unconnected packet was empty");
                        return
                    };
                    
                    pk = match id {
                        OfflinePing::ID => Self::process_unconnected_ping(
                            pk, server_guid, metadata.read().as_str()
                        ),
                        OpenConnectionRequest1::ID => Self::process_open_connection_request1(
                            pk, server_guid
                        ),
                        OpenConnectionRequest2::ID => Self::process_open_connection_request2(
                            pk, sess_manager, udp_socket
                        ),
                        _ => tracing::error!("Invalid unconnected packet ID: {id:x}")
                    };

                    match udp_socket.send_to(pk.buf.as_ref(), pk.addr).await {
                        Ok(_) => (),
                        Err(e) => {
                            tracing::error!("Unable to send unconnected packet to client: {e}");
                        }
                    }
                });
            }    
        }
    }

    /// Sends packets from the send queue
    async fn udp_send_job(
        udp_socket: Arc<UdpSocket>, 
        mut queue: mpsc::Receiver<BufPacket>
    ) {
        loop {
            if let Some(sendable) = queue.recv().await {
                match udp_socket.send_to(&sendable.buf, sendable.addr).await {
                    Ok(_) => (),
                    Err(e) => {
                        tracing::error!("Failed to send packet: {e:?}");
                    }
                }
            } else {
                // Channel has been closed, shut down task.
                return
            }
        }
    }

    /// Refreshes the server description and player counts on a specified interval.
    async fn metadata_refresh_task(self: Arc<Self>) {
        let mut interval = tokio::time::interval(METADATA_REFRESH_INTERVAL);
        while !self.token.is_cancelled() {
            let description =
                format!("{} players", self.session_manager.session_count());
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
            self.session_manager.session_count(),
            self.session_manager.max_session_count(),
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
    // fn register_shutdown_handler(instance: Arc<Self>) {
    //     tokio::spawn(async move {
    //         tokio::select! {
    //             _ = signal::ctrl_c() => {
    //                 tracing::info!("Shutting down...");
    //                 instance.shutdown().await
    //             },
    //             _ = instance.token.cancelled() => {
    //                 // Token has been cancelled by something else, this service is no longer needed
    //             }
    //         }
    //     });
    // }
}
