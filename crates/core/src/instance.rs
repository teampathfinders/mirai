//! Contains the server instance.

use anyhow::Context;

use parking_lot::RwLock;
use raknet::RakNetCreateDescription;
use tokio::task::JoinHandle;

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::net::UdpSocket;

use tokio_util::sync::CancellationToken;

use util::{CowString, Deserialize, Joinable, RVec, ReserveTo, Serialize};

use crate::command::{self, HandlerOutput, HandlerResult, ParsedCommand};
use crate::config::Config;
use crate::data::{BlockStates, CreativeItems, ItemStates};
use crate::net::{Clients, ForwardablePacket};
use proto::bedrock::{
    Command, CommandDataType, CommandEnum, CommandOverload, CommandParameter, CommandPermissionLevel, CreditsStatus, CreditsUpdate,
    CLIENT_VERSION_STRING, PROTOCOL_VERSION,
};
use proto::raknet::{
    IncompatibleProtocol, OpenConnectionReply1, OpenConnectionReply2, OpenConnectionRequest1, OpenConnectionRequest2, UnconnectedPing,
    UnconnectedPong, RAKNET_VERSION,
};

/// Local IPv4 address
pub const IPV4_LOCAL_ADDR: Ipv4Addr = Ipv4Addr::UNSPECIFIED;
/// Local IPv6 address
pub const IPV6_LOCAL_ADDR: Ipv6Addr = Ipv6Addr::UNSPECIFIED;
/// Size of the UDP receive buffer.
const RECV_BUF_SIZE: usize = 2048;
/// Refresh rate of the server's metadata.
/// This data is displayed in the server menu.
const METADATA_REFRESH_INTERVAL: Duration = Duration::from_secs(2);

/// Configures and instance and constructs it.
pub struct InstanceBuilder(Config);

impl InstanceBuilder {
    /// Creates a new instance builder. This is the same as calling [`builder`](Instance::builder) on [`Instance`].
    pub fn new() -> InstanceBuilder {
        Instance::builder()
    }

    /// Sets the path to the level.
    pub fn level_path<P: Into<String>>(mut self, path: P) -> InstanceBuilder {
        self.0.level.path = path.into();
        self
    }

    /// Sets the IPv4 address of the instance.
    pub fn ipv4_address<A: Into<SocketAddrV4>>(mut self, addr: A) -> InstanceBuilder {
        self.0.ipv4_addr = addr.into();
        self
    }

    /// Sets the IPv6 address of the instance.
    pub fn ipv6_addr<A: Into<SocketAddrV6>>(mut self, addr: A) -> InstanceBuilder {
        self.0.ipv6_addr = Some(addr.into());
        self
    }

    /// Produces an [`Instance`] with the configured options, consuming the builder.
    pub async fn build(self) -> anyhow::Result<Arc<Instance>> {
        tracing::info!(
            "Mirai server v{} (rev. {}) built for MCBE {CLIENT_VERSION_STRING} (prot. {PROTOCOL_VERSION})",
            Instance::SERVER_VERSION,
            Instance::GIT_REV
        );

        let item_states = ItemStates::new()?;
        let block_states = BlockStates::new()?;
        let creative_items = CreativeItems::new(&item_states, &block_states)?;

        let ipv4_socket = UdpSocket::bind(self.0.ipv4_addr).await.context("Unable to create IPv4 UDP socket")?;
        let ipv6_socket = match self.0.ipv6_addr {
            Some(addr) => Some(UdpSocket::bind(addr).await.context("Unable to create IPv6 UDP socket")?),
            None => None,
        };

        let ipv4_socket = Arc::new(ipv4_socket);
        let ipv6_socket = ipv6_socket.map(Arc::new);

        let running_token = CancellationToken::new();

        let command_service = crate::command::Service::new(running_token.clone());
        let level_service = crate::level::Service::new(crate::level::ServiceOptions {
            instance_token: running_token.clone(),
            level_path: self.0.level.path.clone(),
        })?;

        let user_map = Arc::new(Clients::new(Arc::clone(&command_service), Arc::clone(&level_service)));
        let user_map = Arc::new(Clients::new(Arc::clone(&command_service), Arc::clone(&level_service)));
        let instance = Instance {
            ipv4_socket,
            ipv6_socket,
            clients: user_map,
            command_service,
            level_service,
            config: self.0,

            raknet_guid: rand::random(),
            current_motd: RwLock::new(String::new()),
            running_token,
            shutdown_token: CancellationToken::new(),
            startup_token: CancellationToken::new(),

            // Data
            creative_items,
            block_states
        };

        let instance = Arc::new(instance);
        instance.refresh_motd();

        Ok(instance)
    }
}

impl Default for InstanceBuilder {
    fn default() -> Self {
        Self::new()
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
    clients: Arc<Clients>,
    /// Keeps track of all available commands.
    command_service: Arc<crate::command::Service>,
    /// Keeps track of the level state.
    level_service: Arc<crate::level::Service>,
    /// Keeps track of the current configuration of the server.
    config: Config,
    /// Cancelled when the server has started up successfully.
    startup_token: CancellationToken,
    /// Cancelled when the server is in the process of shutting down.
    running_token: CancellationToken,
    /// Cancelled when the server has fully shut down.
    shutdown_token: CancellationToken,
    /// The RakNet GUID of the server. This is literally just randomly generated on startup.
    raknet_guid: u64,
    /// The current message of the day. Update every [`METADATA_REFRESH_INTERVAL`] seconds.
    current_motd: RwLock<String>,
    pub(crate) creative_items: CreativeItems,
    block_states: BlockStates
}

impl Instance {
    /// The current version of the server.
    pub const SERVER_VERSION: &'static str = env!("CARGO_PKG_VERSION");
    /// The Git revision of the server.
    pub const GIT_REV: &'static str = env!("VERGEN_GIT_DESCRIBE");
    /// The client version string (i.e "1.20")
    pub const CLIENT_VERSION_STRING: &'static str = CLIENT_VERSION_STRING;
    /// The network protocol version.
    pub const PROTOCOL_VERSION: u32 = PROTOCOL_VERSION;

    /// Creates a new [`InstanceBuilder`].
    pub fn builder() -> InstanceBuilder {
        InstanceBuilder(Config::new())
    }

    /// Gets the current configuration of the instance.
    #[inline]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    /// Gets the command service of this instance.
    #[inline]
    pub const fn commands(&self) -> &Arc<crate::command::Service> {
        &self.command_service
    }

    /// Gets the level service of this instance.
    #[inline]
    pub const fn level(&self) -> &Arc<crate::level::Service> {
        &self.level_service
    }

    /// Gets the client list of this instance.
    #[inline]
    pub const fn clients(&self) -> &Arc<crate::net::Clients> {
        &self.clients
    }

    /// Refreshes the message of the day by calling the generating function again.
    pub fn refresh_motd(self: &Arc<Instance>) {
        let motd: CowString<'_> = (self.config.motd_callback)(self);
        let metadata = format!(
            "MCPE;{};{};{};{};{};{};{};Survival;1;{};{};",
            motd.as_str(),
            PROTOCOL_VERSION,
            CLIENT_VERSION_STRING,
            self.clients.total_connected(),
            self.config.max_connections(),
            self.raknet_guid,
            self.config.name.as_str(),
            self.config.ipv4_addr.port(),
            self.config.ipv6_addr.map(|addr| addr.port()).unwrap_or(0)
        );

        *self.current_motd.write() = metadata;
    }

    /// Signals the server to start shutting down.
    ///
    /// This function returns `None` if the server is already shutting down.
    /// Otherwise a handle to the task performing the shutdown is returned.
    /// This handle can be used to await a full shutdown.
    ///
    /// The returned handle can optionally be used to await a full shutdown.
    /// If the server is already in the process of shutting down, the handle will return an error.
    pub fn shutdown(self: &Arc<Instance>) -> Option<JoinHandle<anyhow::Result<()>>> {
        if self.running_token.is_cancelled() {
            // Server is already shutting down
            return None;
        }

        let this = Arc::clone(self);
        let handle = tokio::spawn(async move {
            let handle = this.clients.shutdown();
            match handle.await {
                Ok(_) => (),
                Err(e) => {
                    tracing::error!("User map shutdown task panicked: {e:#}");
                }
            }

            // Wait for user map to shut down before cancelling general token.
            this.running_token.cancel();

            this.level_service.join().await?;
            this.command_service.join().await?;

            // Awaiting shutdown of the IPv4 and IPv6 receivers is not important
            // because they shut down instantly and don't contain any important data
            // that might need to be saved such as with the level service.

            this.shutdown_token.cancel();

            Ok(())
        });

        Some(handle)
    }

    /// Starts the server and immediately returns when the server has successfully started
    pub fn start(self: &Arc<Instance>) -> anyhow::Result<()> {
        self.clients.set_instance(self)?;
        self.command_service.set_instance(self)?;
        self.level_service.set_instance(self)?;

        self.command_service.register(
            Command {
                aliases: vec!["shutdown".to_owned(), "banjo".to_owned()],
                description: "Shuts down the server".to_owned(),
                name: "shutdown".to_owned(),
                overloads: vec![CommandOverload { parameters: Vec::new() }],
                permission_level: CommandPermissionLevel::Normal,
            },
            |_input, ctx| {
                ctx.instance.shutdown();

                Ok(command::HandlerOutput {
                    message: CowString::new("Server is shutting down"),
                    parameters: Vec::new(),
                })
            },
        )?;

        self.command_service.register(
            Command {
                aliases: vec![],
                description: "Shows the credits".to_owned(),
                name: "credits".to_owned(),
                overloads: vec![CommandOverload { parameters: vec![] }],
                permission_level: CommandPermissionLevel::Normal,
            },
            |_input, ctx| {
                let _ = ctx.caller.send(CreditsUpdate {
                    runtime_id: 1,
                    status: CreditsStatus::Start,
                });

                Ok(HandlerOutput { message: "".into(), parameters: vec![] })
            },
        )?;

        self.command_service.register(
            Command {
                aliases: vec![],
                description: "autocompletion example".to_owned(),
                name: "autocomplete".to_owned(),
                overloads: vec![
                    CommandOverload {
                        parameters: vec![CommandParameter {
                            name: "param1".to_owned(),
                            command_enum: Some(CommandEnum {
                                dynamic: false,
                                enum_id: "options".to_owned(),
                                options: vec!["option1".to_owned(), "option2".to_owned()],
                            }),
                            data_type: CommandDataType::String,
                            optional: true,
                            options: 0,
                            suffix: "".to_owned(),
                        }],
                    },
                    CommandOverload {
                        parameters: vec![
                            CommandParameter {
                                name: "param1".to_owned(),
                                command_enum: Some(CommandEnum {
                                    dynamic: false,
                                    enum_id: "options".to_owned(),
                                    options: vec!["option1".to_owned(), "option2".to_owned()],
                                }),
                                data_type: CommandDataType::String,
                                optional: false,
                                options: 0,
                                suffix: "".to_owned(),
                            },
                            CommandParameter {
                                name: "param2".to_owned(),
                                command_enum: Some(CommandEnum {
                                    dynamic: false,
                                    enum_id: "options2".to_owned(),
                                    options: vec!["option3".to_owned(), "option4".to_owned()],
                                }),
                                data_type: CommandDataType::String,
                                optional: true,
                                options: 0,
                                suffix: "".to_owned(),
                            },
                        ],
                    },
                ],
                permission_level: CommandPermissionLevel::Normal,
            },
            |input, _ctx| {
                tracing::info!("Requested command is: {input:?}");

                Ok(HandlerOutput {
                    message: "this is a command response".into(),
                    parameters: vec![],
                })
            },
        )?;

        static COUNTER: AtomicUsize = AtomicUsize::new(1);

        fn create_fn(_: ParsedCommand, ctx: &command::Context) -> HandlerResult {
            let _ = ctx.instance.commands().register(
                Command {
                    aliases: Vec::new(),
                    description: "hello".to_owned(),
                    name: format!("hello-{}", COUNTER.fetch_add(1, Ordering::Relaxed)),
                    overloads: vec![CommandOverload { parameters: Vec::new() }],
                    permission_level: CommandPermissionLevel::Normal,
                },
                create_fn,
            );

            Ok(command::HandlerOutput {
                message: CowString::new("Created a new command"),
                parameters: Vec::new(),
            })
        }

        self.command_service.register(
            Command {
                aliases: Vec::new(),
                description: "Shuts down the server".to_owned(),
                name: "hello-0".to_owned(),
                overloads: vec![CommandOverload { parameters: Vec::new() }],
                permission_level: CommandPermissionLevel::Normal,
            },
            create_fn,
        )?;

        {
            let socket = Arc::clone(&self.ipv4_socket);
            let this = Arc::clone(self);

            tokio::spawn(Instance::net_receiver(this, socket));
            tracing::info!("IPv4 listener ready");
        }

        if let Some(ipv6_socket) = &self.ipv6_socket {
            let socket = Arc::clone(ipv6_socket);
            let this = Arc::clone(self);

            tokio::spawn(Instance::net_receiver(this, socket));
            tracing::info!("IPv6 listener ready");
        }

        {
            let this = Arc::clone(self);
            tokio::spawn(async move {
                if let Err(err) = tokio::signal::ctrl_c().await {
                    tracing::error!("Failed to create Ctrl-C signal handler: {err:#}");
                } else {
                    this.shutdown();
                }
            });
        }

        self.startup_token.cancel();

        Ok(())
    }

    /// Generates a response to the [`UnconnectedPing`] packet with [`UnconnectedPong`].
    #[inline]
    #[tracing::instrument(
        skip_all,
        name = "Instance::process_unconnected_ping",
        fields(
            %packet.addr
        )
    )]
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
    #[tracing::instrument(
        skip_all,
        name = "Instance::process_open_connection_request1",
        fields(
            %packet.addr
        )
    )]
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
    #[tracing::instrument(
        skip_all,
        name = "Instance::process_open_connection_request2",
        fields(
            %packet.addr
        )
    )]
    fn process_open_connection_request2(
        mut packet: ForwardablePacket,
        udp_socket: Arc<UdpSocket>,
        user_manager: Arc<Clients>,
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

        user_manager.insert(RakNetCreateDescription {
            address: packet.addr,
            guid: request.client_guid,
            mtu: request.mtu,
            socket: udp_socket,
        });

        Ok(packet)
    }

    /// Receives raknet from IPv4 clients and adds them to the receive queue
    async fn net_receiver(self: Arc<Instance>, udp_socket: Arc<UdpSocket>) {
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
                _ = self.running_token.cancelled() => break
            };

            let packet = ForwardablePacket {
                buf: RVec::alloc_from_slice(&recv_buf[..n]),
                addr: address,
            };

            if packet.is_unconnected() {
                let udp_socket = Arc::clone(&udp_socket);
                let session_manager = Arc::clone(&self.clients);
                let metadata = self.current_motd.read().clone();

                let this = Arc::clone(&self);
                tokio::spawn(async move {
                    let Some(id) = packet.packet_id() else {
                        tracing::warn!("Unconnected packet was empty");
                        return;
                    };

                    let pk_result = match id {
                        UnconnectedPing::ID => Instance::process_unconnected_ping(packet, this.raknet_guid, &metadata),
                        OpenConnectionRequest1::ID => Instance::process_open_connection_request1(packet, this.raknet_guid),
                        OpenConnectionRequest2::ID => {
                            Instance::process_open_connection_request2(packet, Arc::clone(&udp_socket), session_manager, this.raknet_guid)
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
            } else if let Err(e) = self.clients.forward(packet).await {
                tracing::error!("{e:#}");
            }
        }

        tracing::info!("Network receiver closed");
    }
}

impl Joinable for Instance {
    /// Waits for the instance to shut down.
    ///
    /// This function is safe to call multiple times and will always return `Ok`.
    async fn join(&self) -> anyhow::Result<()> {
        self.shutdown_token.cancelled().await;
        Ok(())
    }
}
