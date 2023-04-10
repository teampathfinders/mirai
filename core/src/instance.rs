//! Contains the server instance.

use std::iter::Once;
use anyhow::Context;
use ext::{Plugin, PluginRuntime};
use level::Dimension;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use rand::Rng;
use tokio::net::UdpSocket;
use tokio::sync::OnceCell;
use tokio::sync::oneshot::Receiver;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use util::bytes::MutableBuffer;
use util::{Deserialize, Serialize};
use util::{Result, Vector};

use crate::command::{Command, CommandDataType, CommandEnum, CommandOverload, CommandParameter, CommandPermissionLevel};
use crate::config::SERVER_CONFIG;
use crate::item::ItemRegistry;
use crate::level::LevelManager;
use crate::network::SessionList;
use crate::network::{BOOLEAN_GAME_RULES, CLIENT_VERSION_STRING, INTEGER_GAME_RULES, NETWORK_VERSION};
use crate::raknet::IncompatibleProtocol;
use crate::raknet::OpenConnectionReply1;
use crate::raknet::OpenConnectionReply2;
use crate::raknet::OpenConnectionRequest1;
use crate::raknet::OpenConnectionRequest2;
use crate::raknet::RawPacket;
use crate::raknet::UnconnectedPing;
use crate::raknet::UnconnectedPong;
use crate::raknet::RAKNET_VERSION;

/// Local IPv4 address
pub const IPV4_LOCAL_ADDR: Ipv4Addr = Ipv4Addr::UNSPECIFIED;
/// Local IPv6 address
pub const IPV6_LOCAL_ADDR: Ipv6Addr = Ipv6Addr::UNSPECIFIED;
/// Size of the UDP receive buffer.
const RECV_BUF_SIZE: usize = 4096;
/// Refresh rate of the server's metadata.
/// This data is displayed in the server menu.
const METADATA_REFRESH_INTERVAL: Duration = Duration::from_secs(2);

pub struct UdpController {
    ipv4_socket: UdpSocket,
    token: CancellationToken,

    recv_handle: RwLock<Option<JoinHandle<()>>>,
    send_handle: RwLock<Option<JoinHandle<()>>>
}

impl UdpController {
    pub async fn new(ipv4_port: u16, token: CancellationToken) -> anyhow::Result<Arc<Self>> {
        let ipv4_address = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            ipv4_port
        );

        let ipv4_socket = UdpSocket::bind(ipv4_address).await?;
        let controller = Arc::new(Self {
            ipv4_socket, token,
            recv_handle: RwLock::new(None),
            send_handle: RwLock::new(None)
        });

        let clone = controller.clone();
        let recv_handle = tokio::spawn(async move {
            clone.receive_task().await
        });

        *controller.recv_handle.write() = Some(recv_handle);

        let clone = controller.clone();
        let send_handle = tokio::spawn(async move {
            clone.send_task().await
        });

        *controller.send_handle.write() = Some(send_handle);

        Ok(controller)
    }

    /// Waits for the controller tasks to shut down.
    ///
    /// # Errors
    ///
    /// This method requires `self` to be the only reference left.
    /// If there are still multiple strong references to this controller,
    /// this method will fail.
    pub async fn shutdown(self: Arc<Self>) -> anyhow::Result<()> {
        if let Ok(inner) = Arc::into_inner(self) {
            // These can be unwrapped without panicking because shutdown requires exclusive
            // access to `self`.
            // It is statically guaranteed that this method can only be called once.
            let recv_handle = self.recv_handle.write().take().unwrap();
            let send_handle = self.send_handle.write().take().unwrap();

            tokio::join!(recv_handle, send_handle);

            Ok(())
        } else {
            anyhow::bail!("Cannot shutdown UdpController while there are still references to it")
        }
    }

    async fn receive_task(self: Arc<Self>) {
        // Heap allocated to prevent storing large values on the stack.
        let mut recv_buf = vec![0u8; RECV_BUF_SIZE];

        loop {
            let (n, addr) = tokio::select! {
                res = socket.recv_from(&mut recv_buf) => {
                    match result {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!("Failed to receive packet on socket: {e:?}");
                            continue
                        }
                    }
                }
                _ = self.token.cancelled() => break
            };

            if n == 0 {
                tracing::error!("Packet received from client was empty");
                continue
            }

            let mut raw_packet = RawPacket {
                buf: MutableBuffer::from(recv_buf[..n].to_vec()),
                addr
            };

            if raw_packet.is_unconnected() {
                tokio::spawn(async move {
                    let id = raw_packet.id().unwrap();
                    let result = match id {
                        UnconnectedPing::ID =>
                            self.process_unconnected_ping(&mut raw_packet),
                        OpenConnectionRequest1::ID =>
                            self.process_open_connection_request1(&mut raw_packet),
                        OpenConnectionRequest2::ID =>
                            self.process_open_connection_request2(&mut raw_packet),
                        _ => {
                            tracing::error!("Invalid unconnected packet ID: {id:x}");
                            return
                        }
                    };

                    todo!();
                });
            } else {

            }
        }
    }

    async fn send_task(self: Arc<Self>) {
        todo!();
    }

    #[inline]
    fn process_unconnected_ping(self: &Arc<Self>, packet: &mut RawPacket) -> anyhow::Result<()> {
        let ping = UnconnectedPing::deserialize(packet.buf.as_ref())?;
        let pong = UnconnectedPong { time: ping.time, server_guid, metadata };

        packet.buf.clear();
        packet.buf.reserve_to(pong.serialized_size());
        pong.serialize(&mut packet.buf)
    }

    #[inline]
    fn process_open_connection_request1(self: &Arc<Self>, packet: &mut RawPacket) -> anyhow::Result<()> {
        let request = OpenConnectionRequest1::deserialize(packet.buf.as_ref())?;

        packet.buf.clear();
        if request.protocol_version != RAKNET_VERSION {
            let reply = IncompatibleProtocol {
                server_guid
            };

            packet.buf.reserve_to(reply.serialized_size());
            reply.serialize(&mut packet.buf)
        } else {
            let reply = OpenConnectionReply1 {
                mtu: request.mtu, server_guid
            };

            packet.buf.reserve_to(reply.serialized_size());
            reply.serialize(&mut packet.buf)
        }
    }

    #[inline]
    fn process_open_connection_request2(self: &Arc<Self>, packet: &mut RawPacket) -> anyhow::Result<()> {
        let request = OpenConnectionRequest2::deserialize(packet.buf.as_ref())?;
        let reply = OpenConnectionReply2 {
            server_guid, mtu: request.mtu, client_address: packet.addr
        };

        packet.buf.clear();
        packet.buf.reserve_to(reply.serialized_size());
        reply.serialize(&mut packet.buf)?;

        session_list.add_session(
            Arc::clone(self), packet.addr, request.mtu, request.client_guid
        );

        Ok(())
    }
}

pub struct ServerInstance {
    /// Controls the UDP socket.
    ///
    /// This is wrapped in an `Arc` because the controller contains async tasks
    /// that require access to the controller.
    udp_controller: Arc<UdpController>,
    sessions: SessionList,
    plugin_runtime: PluginRuntime,

    token: CancellationToken,

    // World data
    item_registry: Arc<ItemRegistry>,
}

impl ServerInstance {
    pub async fn new(ipv4_port: u16) -> anyhow::Result<Self> {
        let token = CancellationToken::new();
        let udp_controller = UdpController::new(ipv4_port, token.clone()).await?;
        let sessions = SessionList::new(token.clone());

        // let item_registry = Arc::new(ItemRegistry::new()?);


        let plugin_runtime = PluginRuntime::new()
            .context("Failed to start plugin runtime")?;

        Ok(Self {
            udp_controller,
            sessions,
            plugin_runtime,
            token,

            item_registry
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        todo!();
    }

    pub async fn shutdown(self) -> anyhow::Result<()> {
        self.token.cancel();

        self.udp_controller.shutdown().await
    }
}

pub struct InstanceManager {
    item_registry: ItemRegistry,
    /// IPv4 UDP socket
    udp4_socket: Arc<UdpSocket>,
    /// Token indicating whether the server is still running.
    /// All services listen to this token to determine whether they should shut down.
    token: CancellationToken,
    /// Service that manages all player sessions.
    session_manager: Arc<SessionList>,
    /// Manages the level.
    level_manager: Arc<LevelManager>,
    /// Channel that the LevelManager sends a message to when it has fully shutdown.
    /// This is to make sure that the world has been saved and safely shut down before shutting down the server.
    level_notifier: Receiver<()>,
    /// Virtual machine that runs and compiles the plugins.
    plugins: PluginRuntime,
}

impl InstanceManager {
    /// Creates a new server.
    pub async fn run() -> anyhow::Result<()> {
        let extensions = PluginRuntime::new().context("Failed to start extension runtime")?;

        let (ipv4_port, _ipv6_port) = {
            let lock = SERVER_CONFIG.read();
            (lock.ipv4_port, lock.ipv6_port)
        };

        let token = CancellationToken::new();
        let udp_socket = Arc::new(UdpSocket::bind(SocketAddrV4::new(IPV4_LOCAL_ADDR, ipv4_port)).await?);

        let item_registry = Arc::new(ItemRegistry::new()?);
        let session_manager = Arc::new(SessionList::new(item_registry));

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

        session_manager.set_level_manager(Arc::downgrade(&level))?;

        // UDP receiver job.
        let receiver_task = {
            let udp_socket = udp_socket.clone();
            let session_manager = session_manager.clone();
            let token = token.clone();

            tokio::spawn(async move { Self::udp_recv_job(token, udp_socket, session_manager).await })
        };

        tracing::info!("Ready!");

        // Wait for either Ctrl-C or token cancel...
        tokio::select! {
            _ = token.cancelled() => (),
            _ = tokio::signal::ctrl_c() => ()
        }

        drop(extensions);

        // ...then shut down all services.
        if let Err(e) = session_manager.kick_all("Server closed").await {
            tracing::error!("Failed to kick remaining sessions: {e}");
        }

        token.cancel();

        drop(session_manager);
        // drop(level_manager);

        let _ = tokio::join!(receiver_task /*, level_notifier*/);

        Ok(())
    }

    /// Generates a response to the [`OfflinePing`] packet with [`OfflinePong`].
    #[inline]
    fn process_unconnected_ping(packet: RawPacket, server_guid: u64, metadata: &str) -> anyhow::Result<RawPacket> {
        let ping = UnconnectedPing::deserialize(packet.buf.snapshot())?;
        let pong = UnconnectedPong { time: ping.time, server_guid, metadata };

        let mut serialized = MutableBuffer::with_capacity(pong.serialized_size());
        pong.serialize(&mut serialized)?;

        let packet = RawPacket { buf: serialized, addr: packet.addr };

        Ok(packet)
    }

    /// Generates a response to the [`OpenConnectionRequest1`] packet with [`OpenConnectionReply1`].
    #[inline]
    fn process_open_connection_request1(mut packet: RawPacket, server_guid: u64) -> anyhow::Result<RawPacket> {
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
    /// From this point, all packets are encoded in a [`Frame`](crate::Frame).
    #[inline]
    fn process_open_connection_request2(
        mut packet: RawPacket,
        udp_socket: Arc<UdpSocket>,
        sess_manager: Arc<SessionList>,
        server_guid: u64,
    ) -> anyhow::Result<RawPacket> {
        let request = OpenConnectionRequest2::deserialize(packet.buf.snapshot())?;
        let reply = OpenConnectionReply2 {
            server_guid,
            mtu: request.mtu,
            client_address: packet.addr,
        };

        packet.buf.clear();
        packet.buf.reserve_to(reply.serialized_size());
        reply.serialize(&mut packet.buf)?;

        sess_manager.add_session(udp_socket, packet.addr, request.mtu, request.client_guid);

        Ok(packet)
    }

    /// Receives packets from IPv4 clients and adds them to the receive queue
    async fn udp_recv_job(token: CancellationToken, udp_socket: Arc<UdpSocket>, sess_manager: Arc<SessionList>) {
        let server_guid = rand::thread_rng().gen();
        let metadata = Self::refresh_metadata("description", server_guid, sess_manager.session_count(), sess_manager.max_session_count());

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
                            tracing::error!("UdpSocket recv_from failed: {e}");
                            continue
                        }
                    }
                },
                _ = token.cancelled() => break
            };

            let packet = RawPacket {
                buf: MutableBuffer::from(recv_buf[..n].to_vec()),
                addr: address,
            };

            if packet.is_unconnected() {
                let udp_socket = udp_socket.clone();
                let session_manager = sess_manager.clone();
                let metadata = metadata.clone();

                tokio::spawn(async move {
                    let id = if let Some(id) = packet.id() {
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
                sess_manager.forward_packet(packet);
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
