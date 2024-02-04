//! Contains the server instance.

use anyhow::Context;
use raknet::UserCreateInfo;

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6};
use std::sync::Arc;
use std::time::Duration;

use tokio::net::UdpSocket;

use tokio_util::sync::CancellationToken;

use util::{Deserialize, Joinable, ReserveTo, Serialize};

use crate::command::{self};
use crate::config::SERVER_CONFIG;
use crate::net::{ForwardablePacket, UserMap};
use proto::bedrock::{CompressionAlgorithm, CLIENT_VERSION_STRING, NETWORK_VERSION};
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

    Ok(())
}

/// Configuration for the network components.
pub struct NetConfig {
    /// The host and port to run the server on.
    ///
    /// Default: 0.0.0.0:19132.
    pub ipv4_addr: SocketAddrV4,
    /// An optional IPv6 host and port can be specified to accept IPv6 connections as well.
    /// Setting this to `None` will disable IPv6 functionality.
    ///
    /// Default: None.
    pub ipv6_addr: Option<SocketAddrV6>,
    /// Maximum amount of players that can concurrently be connected to this server.
    ///
    /// Default: 100
    pub max_connections: usize,
    /// The compression algorithm to use for packet compression.
    ///
    /// Default: [`Flate`])(CompressionAlgorithm::Flate).
    pub compression: CompressionAlgorithm,
    /// The packet length compression threshold.
    ///
    /// Packets with a length below this threshold will be left uncompressed.
    /// Setting this to 1 will compress all packets, while setting it to 0 disables compression.
    ///
    /// Default: 1
    pub compression_threshold: u16,
}

impl Default for NetConfig {
    fn default() -> NetConfig {
        NetConfig {
            ipv4_addr: SocketAddrV4::new(IPV4_LOCAL_ADDR, 19132),
            ipv6_addr: None,
            max_connections: 100,
            compression: CompressionAlgorithm::Flate,
            compression_threshold: 1,
        }
    }
}

/// Configuration for the database connection.
pub struct DbConfig<'a> {
    /// Host address of the database server.
    ///
    /// Default: localhost.
    ///
    /// When running the server and database in Docker containers, this
    /// should be set to the Docker network name.
    ///
    /// See [Docker networks](`https://docs.docker.com/network/`) for more information.
    pub host: &'a str,
    /// Port of the database server.
    ///
    /// This should usually be set to 6379 when using a Redis server.
    ///
    /// Default: 6379.
    pub port: u16,
}

impl Default for DbConfig<'static> {
    fn default() -> DbConfig<'static> {
        DbConfig { host: "localhost", port: 6379 }
    }
}

/// Configuration for client options.
pub struct ClientConfig {
    /// Whether the client should throttle other players.
    ///
    /// Default: false.
    pub throttling_enabled: bool,
    /// When the player count exceeds this threshold, the client
    /// will start throttling other players.
    ///
    /// Default: 0.
    pub throttling_threshold: u8,
    /// Amount of players that will be ticked when the client is
    /// actively throttling.
    ///
    /// Default: 0.0.
    pub throttling_scalar: f32,
    /// Maximum server-allow render distance.
    ///
    /// If the client requests a larger render distance, the server
    /// will cap it to this maximum.
    ///
    /// Default: 12.
    pub render_distance: u32,
}

impl Default for ClientConfig {
    fn default() -> ClientConfig {
        ClientConfig {
            throttling_enabled: false,
            throttling_threshold: 0,
            throttling_scalar: 0.0,
            render_distance: 12,
        }
    }
}

/// Builder used to configure a new server instance.
pub struct InstanceBuilder<'a> {
    name: String,
    net_config: NetConfig,
    db_config: DbConfig<'a>,
    client_config: ClientConfig,
}

impl<'a> InstanceBuilder<'a> {
    /// Creates a new instance builder.
    #[inline]
    pub fn new() -> InstanceBuilder<'a> {
        InstanceBuilder::default()
    }

    /// Set the name of the server.
    ///
    /// This is the name that shows up at the top of the member list.
    ///
    /// Default: Server.
    #[inline]
    pub fn name<I: Into<String>>(mut self, name: I) -> InstanceBuilder<'a> {
        self.name = name.into();
        self
    }

    /// Set the network config.
    ///
    /// Default: See [`NetConfig`].
    #[inline]
    pub const fn net_config(mut self, config: NetConfig) -> InstanceBuilder<'a> {
        self.net_config = config;
        self
    }

    /// Set the database config.
    ///
    /// Default: See [`DbConfig`].
    #[inline]
    pub const fn db_config(mut self, config: DbConfig<'a>) -> InstanceBuilder<'a> {
        self.db_config = config;
        self
    }

    /// Set the client config.
    ///
    /// Default: See [`ClientConfig`].
    #[inline]
    pub const fn client_config(mut self, config: ClientConfig) -> InstanceBuilder<'a> {
        self.client_config = config;
        self
    }

    /// Produces an [`Instance`] with the configured options, consuming the builder.
    pub async fn build(self) -> anyhow::Result<Instance> {
        let ipv4_socket = UdpSocket::bind(self.net_config.ipv4_addr)
            .await
            .context("Unable to create IPv4 UDP socket")?;
        let ipv6_socket = match self.net_config.ipv6_addr {
            Some(addr) => Some(UdpSocket::bind(addr).await.context("Unable to create IPv6 UDP socket")?),
            None => None,
        };

        let ipv4_socket = Arc::new(ipv4_socket);
        let ipv6_socket = ipv6_socket.map(Arc::new);

        let replicator = Replicator::new(self.db_config.host, self.db_config.port)
            .await
            .context("Unable to create replicator")?;

        let replicator = Arc::new(replicator);

        let token = CancellationToken::new();
        let command_service = command::Service::new(token.clone());
        let user_map = Arc::new(UserMap::new(replicator, Arc::clone(&command_service)));

        let instance = Instance {
            ipv4_socket,
            ipv6_socket,
            user_map,
            token,
            command_service,
        };

        Ok(instance)
    }
}

impl Default for InstanceBuilder<'static> {
    fn default() -> InstanceBuilder<'static> {
        InstanceBuilder {
            name: String::from("Server"),
            net_config: NetConfig::default(),
            db_config: DbConfig::default(),
            client_config: ClientConfig::default(),
        }
    }
}

/// Manages all the processes running within the server.
///
/// The instance is what makes sure that every job is started and that the server
/// shuts down properly when requested. It does this by signalling different jobs in the correct order.
/// For example, the [`SessionManager`] is the first thing that is shut down to kick all the players from
/// the server before continuing with the shutdown.
pub struct Instance {
    /// IPv4 UDP socket
    ipv4_socket: Arc<UdpSocket>,
    /// IPv6 UDP socket.
    ipv6_socket: Option<Arc<UdpSocket>>,
    /// Service that manages all player sessions.
    user_map: Arc<UserMap>,
    /// Token that can signal other services to stop running.
    token: CancellationToken,
    /// Keeps track of all available commands.
    command_service: Arc<command::Service>,
}

impl Instance {
    /// Runs the server.
    ///
    /// This future resolves when the server has fully shut down again.
    pub async fn run(self) -> anyhow::Result<()> {
        // FIXME: The level module will get a refactor and this will be changed
        // let level = Level::new(self.user_map.clone(), self.token.clone())?;
        // self.user_map.set_level(level);

        let recv_job4 = {
            let socket = Arc::clone(&self.ipv4_socket);
            let user_map = Arc::clone(&self.user_map);
            let token = self.token.clone();

            let handle = tokio::spawn(Instance::net_recv_job(token, socket, user_map));
            tracing::info!("IPv4 listener ready");

            handle
        };

        let recv_job6 = if let Some(socket) = self.ipv6_socket {
            let user_map = Arc::clone(&self.user_map);
            let token = self.token.clone();

            let handle = tokio::spawn(Instance::net_recv_job(token, socket, user_map));
            tracing::info!("IPv6 listener ready");

            Some(handle)
        } else {
            tracing::info!("IPv6 functionality disabled");
            None
        };

        signal_listener(self.token.clone()).await?;

        // Error is logged by user map.
        // The user map requires manual shutdown to make sure it shuts down before all other services.
        _ = self.user_map.join().await;

        // Keep all services running until all clients have been disconnected.
        // Only then start shutting down services.
        self.token.cancel();

        if let Err(err) = recv_job4.await {
            tracing::error!("Error occurred while awaiting IPv4 receiver shutdown: {err:#?}");
        }

        if let Some(job) = recv_job6 {
            if let Err(err) = job.await {
                tracing::error!("Error occurred while awaiting IPv6 receiver shutdown: {err:#?}");
            }
        }

        // Error is logged by command service.
        _ = self.command_service.join().await;

        Ok(())
    }

    /// Generates a response to the [`UnconnectedPing`] packet with [`UnconnectedPong`].
    #[inline]
    fn process_unconnected_ping(mut packet: ForwardablePacket, server_guid: u64, metadata: &str) -> anyhow::Result<ForwardablePacket> {
        let ping = UnconnectedPing::deserialize(packet.buf.as_ref())?;
        let pong = UnconnectedPong { time: ping.time, server_guid, metadata };

        #[cfg(trace_raknet)]
        tracing::debug!("{ping:?}");

        packet.buf.clear();
        packet.buf.reserve_to(pong.size_hint());
        pong.serialize_into(&mut packet.buf)?;

        let packet = ForwardablePacket { buf: packet.buf, addr: packet.addr };

        Ok(packet)
    }

    /// Generates a response to the [`OpenConnectionRequest1`] packet with [`OpenConnectionReply1`].
    #[inline]
    fn process_open_connection_request1(mut packet: ForwardablePacket, server_guid: u64) -> anyhow::Result<ForwardablePacket> {
        let request = OpenConnectionRequest1::deserialize(packet.buf.as_ref())?;

        #[cfg(trace_raknet)]
        tracing::debug!("{request:?}");

        packet.buf.clear();
        if request.protocol_version != RAKNET_VERSION {
            let reply = IncompatibleProtocol { server_guid };

            packet.buf.clear();
            packet.buf.reserve_to(reply.size_hint());
            reply.serialize_into(&mut packet.buf)?;
        } else {
            let reply = OpenConnectionReply1 { mtu: request.mtu, server_guid };

            packet.buf.clear();
            packet.buf.reserve_to(reply.size_hint());
            reply.serialize_into(&mut packet.buf)?;
        }

        Ok(packet)
    }

    /// Responds to the [`OpenConnectionRequest2`] packet with [`OpenConnectionReply2`].
    /// This is also when a session is created for the client.
    /// From this point, all packets are encoded in a [`Frame`](crate::raknet::Frame).
    #[inline]
    fn process_open_connection_request2(
        mut packet: ForwardablePacket,
        udp_socket: Arc<UdpSocket>,
        user_manager: Arc<UserMap>,
        server_guid: u64,
    ) -> anyhow::Result<ForwardablePacket> {
        let request = OpenConnectionRequest2::deserialize(packet.buf.as_ref())?;
        let reply = OpenConnectionReply2 {
            server_guid,
            mtu: request.mtu,
            client_address: packet.addr,
        };

        #[cfg(trace_raknet)]
        tracing::debug!("{request:?}");

        packet.buf.clear();
        packet.buf.reserve_to(reply.size_hint());
        reply.serialize_into(&mut packet.buf)?;

        user_manager.insert(UserCreateInfo {
            address: packet.addr,
            guid: request.client_guid,
            mtu: request.mtu,
            socket: udp_socket,
        });

        Ok(packet)
    }

    /// Receives raknet from IPv4 clients and adds them to the receive queue
    async fn net_recv_job(token: CancellationToken, udp_socket: Arc<UdpSocket>, user_manager: Arc<UserMap>) {
        let server_guid = rand::random();

        // TODO: Customizable server description.
        let metadata = Self::refresh_metadata(
            &String::from_utf8_lossy(&[0xee, 0x84, 0x88, 0x20]),
            server_guid,
            user_manager.connected_count(),
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
                buf: recv_buf[..n].to_vec(),
                addr: address,
            };

            if packet.is_unconnected() {
                let udp_socket = Arc::clone(&udp_socket);
                let session_manager = Arc::clone(&user_manager);
                let metadata = metadata.clone();

                tokio::spawn(async move {
                    let Some(id) = packet.packet_id() else {
                        tracing::error!("Unconnected packet was empty");
                        return;
                    };

                    let pk_result = match id {
                        UnconnectedPing::ID => Self::process_unconnected_ping(packet, server_guid, &metadata),
                        OpenConnectionRequest1::ID => Self::process_open_connection_request1(packet, server_guid),
                        OpenConnectionRequest2::ID => {
                            Self::process_open_connection_request2(packet, Arc::clone(&udp_socket), session_manager, server_guid)
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
            } else if let Err(e) = user_manager.forward(packet).await {
                tracing::error!("{e:#}");
            }
        }

        tracing::info!("Network receiver closed");
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
