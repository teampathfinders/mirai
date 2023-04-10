//! Contains the server instance.

use std::iter::Once;
use std::marker::PhantomData;
use anyhow::Context;
use ext::{Plugin, PluginRuntime};
use level::Dimension;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use rand::Rng;
use tokio::net::{UdpSocket, ToSocketAddrs};
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
use crate::network::SessionMap;
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

/// Controls the UDP socket, managing incoming and outgoing traffic.
#[derive(Debug)]
pub struct UdpController {
    metadata: Arc<RwLock<String>>,
    session_map: Arc<SessionMap>,
    /// Unique GUID of the server.
    /// This is randomly generated on startup and sent together with every RakNet packet.
    /// This doesn't really seem to have a purpose.
    server_guid: u64,
    /// UDP socket used for IPv4 communications.
    ipv4_socket: UdpSocket,
    /// Token that can be cancelled by the instance to
    /// shut down the controller.
    token: CancellationToken,
    /// Handle to the receiving task.
    ///
    /// `tokio::join` requires owned access to the handles,
    /// therefore these are wrapped in an RwLock and option.
    recv_handle: RwLock<Option<JoinHandle<()>>>
}

impl UdpController {
    /// Creates a new UDP controller.
    ///
    /// The async task is automatically started, the socket is also created.
    pub async fn new(
        server_guid: u64,
        metadata: Arc<RwLock<String>>,
        session_map: Arc<SessionMap>,
        ipv4_port: u16, token: CancellationToken
    ) -> anyhow::Result<Arc<Self>> {
        let ipv4_address = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            ipv4_port
        );

        let ipv4_socket = UdpSocket::bind(ipv4_address).await?;
        let controller = Arc::new(Self {
            metadata, session_map,
            server_guid,
            ipv4_socket, token,
            recv_handle: RwLock::new(None)
        });

        let clone = controller.clone();
        let recv_handle = tokio::spawn(async move {
            clone.receive_task().await
        });

        *controller.recv_handle.write() = Some(recv_handle);

        Ok(controller)
    }

    /// Sends the given buffer to the specified IP addresses.
    pub async fn send_to<B, A>(&self, buf: B, addrs: A) -> anyhow::Result<()>
    where
        B: AsRef<[u8]>,
        A: ToSocketAddrs
    {
        let buf = buf.as_ref();

        self.ipv4_socket.send_to(buf, addrs).await?;
        Ok(())
    }

    /// Waits for the controller tasks to shut down.
    ///
    /// # Errors
    ///
    /// This method requires `self` to be the only reference left.
    /// If there are still multiple strong references to this controller,
    /// this method will fail.
    pub async fn shutdown(self: Arc<Self>) -> anyhow::Result<()> {
        if let Ok(inner) = Arc::try_unwrap(self) {
            // These can be unwrapped without panicking because shutdown requires exclusive
            // access to `self`.
            // It is statically guaranteed that this method can only be called once.
            let recv_handle = inner.recv_handle.write().take().unwrap();
            tokio::join!(recv_handle);

            Ok(())
        } else {
            anyhow::bail!("Cannot shutdown UdpController while there are still references to it")
        }
    }

    /// Starts the receiving task of this controller.
    ///
    /// The task runs until the `token` is cancelled.
    async fn receive_task(self: Arc<Self>) {
        // Heap allocated to prevent storing large values on the stack.
        let mut recv_buf = vec![0u8; RECV_BUF_SIZE];

        loop {
            let (n, addr) = tokio::select! {
                res = self.ipv4_socket.recv_from(&mut recv_buf) => {
                    match res {
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
                let clone = Arc::clone(&self);

                tokio::spawn(async move {
                    let id = raw_packet.id().unwrap();
                    let result = match id {
                        UnconnectedPing::ID =>
                            clone.process_unconnected_ping(&mut raw_packet),
                        OpenConnectionRequest1::ID =>
                            clone.process_open_connection_request1(&mut raw_packet),
                        OpenConnectionRequest2::ID =>
                            clone.process_open_connection_request2(&mut raw_packet),
                        _ => {
                            tracing::error!("Invalid unconnected packet ID: {id:x}");
                            return
                        }
                    };

                    match result {
                        Ok(_) => match clone.ipv4_socket.send_to(raw_packet.buf.as_ref(), raw_packet.addr).await {
                            Ok(_) => (),
                            Err(e) => {
                                tracing::error!("Failed to send unconnected packet: {e:?}");
                            }
                        },
                        Err(e) => {
                            tracing::error!("{e:?}");
                        }
                    }
                });
            } else {
                let clone = Arc::clone(&self);
                tokio::spawn(async move {
                    let result = clone.session_map.forward(raw_packet).await;
                    if let Err(err) = result {
                        tracing::error!("{err:?}");
                    }
                });
            }
        }
    }

    /// Processes an [`UnconnectedPing`] packet.
    ///
    /// This is an unconnected packet, which is handled by the controller directly instead
    /// of going through sessions.
    #[inline]
    fn process_unconnected_ping(self: &Arc<Self>, packet: &mut RawPacket) -> anyhow::Result<()> {
        let ping = UnconnectedPing::deserialize(packet.buf.as_ref().into())?; // TODO
        let pong = UnconnectedPong {
            time: ping.time,
            server_guid: self.server_guid,
            metadata: &self.metadata.read()
        };

        packet.buf.clear();
        packet.buf.reserve_to(pong.serialized_size());
        pong.serialize(&mut packet.buf)
    }

    /// Processes an [`OpenConnectionRequest1`] packet.
    ///
    /// This is an unconnected packet, which is handled by the controller directly instead
    /// of going through sessions.
    #[inline]
    fn process_open_connection_request1(self: &Arc<Self>, packet: &mut RawPacket) -> anyhow::Result<()> {
        let request = OpenConnectionRequest1::deserialize(packet.buf.as_ref().into())?; // TODO

        packet.buf.clear();
        if request.protocol_version != RAKNET_VERSION {
            let reply = IncompatibleProtocol {
                server_guid: self.server_guid
            };

            packet.buf.reserve_to(reply.serialized_size());
            reply.serialize(&mut packet.buf)
        } else {
            let reply = OpenConnectionReply1 {
                mtu: request.mtu, server_guid: self.server_guid
            };

            packet.buf.reserve_to(reply.serialized_size());
            reply.serialize(&mut packet.buf)
        }
    }

    /// Processes an [`OpenConnectionRequest2`] packet.
    ///
    /// This is an unconnected packet, which is handled by the controller directly instead
    /// of going through sessions.
    #[inline]
    fn process_open_connection_request2(self: &Arc<Self>, packet: &mut RawPacket) -> anyhow::Result<()> {
        let request = OpenConnectionRequest2::deserialize(packet.buf.as_ref().into())?; // TODO
        let reply = OpenConnectionReply2 {
            server_guid: self.server_guid,
            mtu: request.mtu,
            client_address: packet.addr
        };

        packet.buf.clear();
        packet.buf.reserve_to(reply.serialized_size());
        reply.serialize(&mut packet.buf)?;

        self.session_map.insert(Arc::clone(self), packet.addr, request.mtu, request.client_guid);

        Ok(())
    }
}

pub struct ServerInstance {
    metadata: Arc<RwLock<String>>,

    /// Controls the UDP socket.
    ///
    /// This is wrapped in an `Arc` because the controller contains async tasks
    /// that require access to the controller.
    udp_controller: Arc<UdpController>,
    sessions: Arc<SessionMap>,
    plugin_runtime: PluginRuntime,

    token: CancellationToken
}

impl ServerInstance {
    /// Creates a new server instance running on the specified port.
    pub async fn new(ipv4_port: u16, max_players: usize) -> anyhow::Result<Self> {
        let server_guid = rand::thread_rng().gen();

        let token = CancellationToken::new();
        let sessions = Arc::new(SessionMap::new(token.clone(), max_players));
        let metadata = Arc::new(RwLock::new(String::new()));

        let udp_controller = UdpController::new(
            server_guid,
            metadata.clone(), sessions.clone(),
            ipv4_port, token.clone()
        ).await?;

        // let item_registry = Arc::new(ItemRegistry::new()?);

        let plugin_runtime = PluginRuntime::new()
            .context("Failed to start plugin runtime")?;

        {
            let sessions = sessions.clone();
            let metadata = metadata.clone();
            let token = token.clone();

            tokio::spawn(async move {
                let mut interval = tokio::time::interval(METADATA_REFRESH_INTERVAL);
                loop {
                    let count = sessions.count();
                    let max_count = sessions.max_count();
                    let description = "description"; // TODO: Call plugin method or use default description.

                    let new = Self::regenerate_metadata(description, server_guid, count, max_count);
                    *metadata.write() = new;

                    tokio::select! {
                        _ = token.cancelled() => break,
                        _ = interval.tick() => continue
                    }
                }
            });
        }

        Ok(Self {
            metadata,
            udp_controller,
            sessions,
            plugin_runtime,
            token,
        })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        tokio::select! {
            _ = self.token.cancelled() => (),
            _ = tokio::signal::ctrl_c() => ()
        }
        self.token.cancel();

        self.udp_controller.shutdown().await
    }

    pub async fn shutdown(self) -> anyhow::Result<()> {
        self.token.cancel();

        self.udp_controller.shutdown().await
    }

    /// Generates a new metadata string using the given description and new player count.
    fn regenerate_metadata(
        description: &str, server_guid: u64, session_count: usize, max_session_count: usize
    ) -> String {
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
