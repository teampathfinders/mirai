//! Contains the server instance.

use anyhow::Context;

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4};
use std::sync::Arc;
use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::sync::oneshot::Receiver;
use tokio_util::sync::CancellationToken;

use util::MutableBuffer;
use util::{Deserialize, Serialize};

use crate::config::SERVER_CONFIG;
use crate::level::LevelManager;
use crate::network::{UserMap, UserCreateInfo};
use crate::raknet::ForwardablePacket;
use proto::bedrock::{
    Command, CommandDataType, CommandEnum, CommandOverload, CommandParameter, CommandPermissionLevel, BOOLEAN_GAME_RULES, CLIENT_VERSION_STRING,
    INTEGER_GAME_RULES, MOBEFFECT_NAMES, NETWORK_VERSION,
};
use proto::raknet::{
    IncompatibleProtocol, OpenConnectionReply1, OpenConnectionReply2, OpenConnectionRequest1, OpenConnectionRequest2, UnconnectedPing,
    UnconnectedPong, RAKNET_VERSION,
};
use replicator::Replicator;

/// Local IPv4 address
pub const IPV4_LOCAL_ADDR: Ipv4Addr = Ipv4Addr::UNSPECIFIED;
/// Local IPv6 address
pub const IPV6_LOCAL_ADDR: Ipv6Addr = Ipv6Addr::UNSPECIFIED;
/// Size of the UDP receive buffer.
const RECV_BUF_SIZE: usize = 4096;
/// Refresh rate of the server's metadata.
/// This data is displayed in the server menu.
const METADATA_REFRESH_INTERVAL: Duration = Duration::from_secs(2);

async fn signal_listener(token: CancellationToken) -> anyhow::Result<()> {
    tokio::select! {
        _ = token.cancelled() => (),
        _ = tokio::signal::ctrl_c() => ()
    }

    // #[cfg(windows)]
    // tokio::select! {
    //     _ = token.cancelled() => (),
    //     _ = tokio::signal::ctrl_c() => ()
    // }

    // #[cfg(unix)]
    // {
    //     use tokio::signal::unix::{signal, SignalKind};

    //     let mut sig = signal(SignalKind::terminate())?;
    //     tokio::select! {
    //         _ = token.cancelled() => (),
    //         _ = tokio::signal::ctrl_c() => (),
    //         _ = sig.recv() => ()
    //     }
    // }

    Ok(())
}

/// Manages all the processes running within the server.
///
/// The instance is what makes sure that every job is started and that the server
/// shuts down properly when requested. It does this by signalling different jobs in the correct order.
/// For example, the [`SessionManager`] is the first thing that is shut down to kick all the players from
/// the server before continuing with the shutdown.
pub struct ServerInstance {
    /// IPv4 UDP socket
    udp4_socket: Arc<UdpSocket>,
    /// Token indicating whether the server is still running.
    /// All services listen to this token to determine whether they should shut down.
    token: CancellationToken,
    /// Service that manages all player sessions.
    session_manager: Arc<UserMap>,
    /// Manages the level.
    level_manager: Arc<LevelManager>,
    /// Channel that the LevelManager sends a message to when it has fully shutdown.
    /// This is to make sure that the world has been saved and safely shut down before shutting down the server.
    level_notifier: Receiver<()>,
}

impl ServerInstance {
    /// Creates a new server.
    ///
    /// This method is asynchronous and completes when the server is fully shut down again.
    pub async fn run() -> anyhow::Result<()> {
        let (ipv4_port, _ipv6_port) = {
            let lock = SERVER_CONFIG.read();
            (lock.ipv4_port, lock.ipv6_port)
        };

        let token = CancellationToken::new();

        let udp_socket = Arc::new(
            UdpSocket::bind(SocketAddrV4::new(IPV4_LOCAL_ADDR, ipv4_port))
                .await
                .context("Unable to create UDP socket")?,
        );

        let replicator = Replicator::new().await.context("Cannot create replication layer")?;
        let session_manager = Arc::new(UserMap::new(replicator));

        let level = LevelManager::new(session_manager.clone(), token.clone())?;

        level.add_command(Command {
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
                                options: BOOLEAN_GAME_RULES.iter().map(|g| g.to_string()).collect::<Vec<_>>(),
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
                                options: vec!["true".to_owned(), "false".to_owned()],
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
                                options: INTEGER_GAME_RULES.iter().map(|g| g.to_string()).collect::<Vec<_>>(),
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

        level.add_command(Command {
            name: String::from("effect"),
            aliases: vec![],
            description: String::from("Adds or removes the status effects of players and other entities."),
            overloads: vec![
                CommandOverload {
                    parameters: vec![
                        CommandParameter {
                            name: String::from("target"),
                            data_type: CommandDataType::Target,
                            command_enum: None,
                            suffix: String::new(),
                            optional: false,
                            options: 0,
                        },
                        CommandParameter {
                            name: String::from("effect"),
                            data_type: CommandDataType::String,
                            command_enum: Some(CommandEnum {
                                enum_id: String::from("effect_clear"),
                                options: vec![String::from("clear")],
                                dynamic: false,
                            }),
                            suffix: String::new(),
                            optional: false,
                            options: 0,
                        },
                    ],
                },
                CommandOverload {
                    parameters: vec![
                        CommandParameter {
                            name: String::from("target"),
                            data_type: CommandDataType::Target,
                            command_enum: None,
                            suffix: String::new(),
                            optional: false,
                            options: 0,
                        },
                        CommandParameter {
                            name: String::from("effect"),
                            data_type: CommandDataType::String,
                            command_enum: Some(CommandEnum {
                                enum_id: String::from("effect"),
                                options: MOBEFFECT_NAMES.iter().map(|s| String::from(*s)).collect(),
                                dynamic: false,
                            }),
                            suffix: String::new(),
                            optional: false,
                            options: 0,
                        },
                        CommandParameter {
                            name: String::from("duration"),
                            data_type: CommandDataType::Int,
                            command_enum: None,
                            suffix: String::new(),
                            optional: true,
                            options: 0,
                        },
                        CommandParameter {
                            name: String::from("amplifier"),
                            data_type: CommandDataType::Int,
                            command_enum: None,
                            suffix: String::new(),
                            optional: true,
                            options: 0,
                        },
                        CommandParameter {
                            name: String::from("hideParticles"),
                            data_type: CommandDataType::String,
                            command_enum: Some(CommandEnum {
                                enum_id: String::from("boolean"),
                                dynamic: false,
                                options: vec![String::from("true"), String::from("false")],
                            }),
                            suffix: String::new(),
                            optional: true,
                            options: 0,
                        },
                    ],
                },
            ],
            permission_level: CommandPermissionLevel::Normal,
        });

        // session_manager.set_level_manager(Arc::downgrade(&level))?;

        // UDP receiver job.
        let receiver_task = {
            let udp_socket = udp_socket.clone();
            let session_manager = session_manager.clone();
            let token = token.clone();

            tokio::spawn(async move { Self::udp_recv_job(token, udp_socket, session_manager).await })
        };

        tracing::info!("Ready on localhost:{}!", ipv4_port);

        // Wait for a shutdown signal...
        signal_listener(token.clone()).await?;

        tracing::info!("Shutting down server. This can take several seconds...");

        // ...then shut down all services.
        if let Err(e) = session_manager.kick_all("Server closed") {
            tracing::error!("Failed to kick remaining sessions: {e}");
        }

        token.cancel();

        drop(session_manager);
        // drop(level_manager);

        let _ = tokio::join!(receiver_task /*, level_notifier*/);

        Ok(())
    }

    /// Generates a response to the [`UnconnectedPing`] packet with [`UnconnectedPong`].
    #[inline]
    fn process_unconnected_ping(packet: ForwardablePacket, server_guid: u64, metadata: &str) -> anyhow::Result<ForwardablePacket> {
        let ping = UnconnectedPing::deserialize(packet.buf.snapshot())?;
        let pong = UnconnectedPong { time: ping.time, server_guid, metadata };

        let mut serialized = MutableBuffer::with_capacity(pong.serialized_size());
        pong.serialize(&mut serialized)?;

        let packet = ForwardablePacket { buf: serialized, addr: packet.addr };

        Ok(packet)
    }

    /// Generates a response to the [`OpenConnectionRequest1`] packet with [`OpenConnectionReply1`].
    #[inline]
    fn process_open_connection_request1(mut packet: ForwardablePacket, server_guid: u64) -> anyhow::Result<ForwardablePacket> {
        let request = OpenConnectionRequest1::deserialize(packet.buf.snapshot())?;

        packet.buf.clear();
        if request.protocol_version != RAKNET_VERSION {
            let reply = IncompatibleProtocol { server_guid };

            packet.buf.clear();
            packet.buf.reserve_to(reply.serialized_size());
            reply.serialize(&mut packet.buf)?;
        } else {
            let reply = OpenConnectionReply1 { mtu: request.mtu, server_guid };

            packet.buf.clear();
            packet.buf.reserve_to(reply.serialized_size());
            reply.serialize(&mut packet.buf)?;
        }

        Ok(packet)
    }

    /// Responds to the [`OpenConnectionRequest2`] packet with [`OpenConnectionReply2`].
    /// This is also when a session is created for the client.
    /// From this point, all raknet are encoded in a [`Frame`](crate::raknet::Frame).
    #[inline]
    fn process_open_connection_request2(
        mut packet: ForwardablePacket,
        udp_socket: Arc<UdpSocket>,
        user_manager: Arc<UserMap>,
        server_guid: u64,
    ) -> anyhow::Result<ForwardablePacket> {
        let request = OpenConnectionRequest2::deserialize(packet.buf.snapshot())?;
        let reply = OpenConnectionReply2 {
            server_guid,
            mtu: request.mtu,
            client_address: packet.addr,
        };

        packet.buf.clear();
        packet.buf.reserve_to(reply.serialized_size());
        reply.serialize(&mut packet.buf)?;

        user_manager.insert(UserCreateInfo {
            address: packet.addr,
            guid: request.client_guid,
            mtu: request.mtu,
            socket: udp_socket
        });

        Ok(packet)
    }

    /// Receives raknet from IPv4 clients and adds them to the receive queue
    async fn udp_recv_job(token: CancellationToken, udp_socket: Arc<UdpSocket>, user_manager: Arc<UserMap>) {
        let server_guid = rand::random();

        // TODO: Customizable server description.
        let metadata = Self::refresh_metadata(
            &String::from_utf8_lossy(&[0xee, 0x84, 0x88, 0x20]),
            server_guid,
            user_manager.count(),
            user_manager.max_count(),
        );

        // This is heap-allocated because stack data is stored inline in tasks.
        // If it were to be stack-allocated, Tokio would have to copy the entire buffer each time
        // the task is moved across threads.
        let mut recv_buf = vec![0u8; RECV_BUF_SIZE];

        loop {
            let (n, address) = tokio::select! {
                r = udp_socket.recv_from(&mut recv_buf) => {
                    match r {
                        Ok(r) => r,
                        Err(e) => {
                            tracing::error!("Failed to receive UDP packet from client: {e}");
                            continue
                        }
                    }
                },
                _ = token.cancelled() => break
            };

            let packet = ForwardablePacket {
                buf: MutableBuffer::from(recv_buf[..n].to_vec()),
                addr: address,
            };

            if packet.is_unconnected() {
                let udp_socket = udp_socket.clone();
                let session_manager = user_manager.clone();
                let metadata = metadata.clone();

                tokio::spawn(async move {
                    let id = if let Some(id) = packet.packet_id() {
                        id
                    } else {
                        tracing::error!("Unconnected packet was empty");
                        return;
                    };

                    let pk_result = match id {
                        UnconnectedPing::ID => Self::process_unconnected_ping(packet, server_guid, &metadata),
                        OpenConnectionRequest1::ID => Self::process_open_connection_request1(packet, server_guid),
                        OpenConnectionRequest2::ID => {
                            Self::process_open_connection_request2(packet, udp_socket.clone(), session_manager, server_guid)
                        }
                        _ => {
                            tracing::error!("Invalid unconnected packet ID: {id:x}");
                            return;
                        }
                    };

                    match pk_result {
                        Ok(packet) => match udp_socket.send_to(packet.buf.as_ref(), packet.addr).await {
                            Ok(_) => (),
                            Err(e) => {
                                tracing::error!("Unable to send unconnected packet to client: {e}");
                            }
                        },
                        Err(e) => {
                            tracing::error!("{e}");
                        }
                    }
                });
            } else {
                user_manager.forward(packet);
            }
        }
    }

    /// Generates a new metadata string using the given description and new player count.
    fn refresh_metadata(description: &str, server_guid: u64, session_count: usize, max_session_count: usize) -> String {
        format!(
            "MCPE;{};{};{};{};{};{};{};Survival;1;{};{};",
            description,
            NETWORK_VERSION,
            CLIENT_VERSION_STRING,
            session_count,
            max_session_count,
            server_guid,
            SERVER_CONFIG.read().server_name,
            19132,
            19133
        )
    }
}
