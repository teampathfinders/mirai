use std::future::Future;
use std::io::{Read, Write};

use std::sync::{Arc, OnceLock};
use std::sync::atomic::{
    AtomicBool, AtomicI64, AtomicU32, Ordering
};
use std::time::Instant;

use anyhow::Context;
use flate2::Compression;
use flate2::write::DeflateEncoder;
use parking_lot::RwLock;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::{linear_buckets, Histogram};
use raknet::{BroadcastPacket, RaknetCommand, RaknetUser, SendConfig, DEFAULT_SEND_CONFIG};
use systemstat::Duration;
use tokio::sync::{broadcast, mpsc};
use proto::bedrock::{CommandPermissionLevel, Disconnect, GameMode, PermissionLevel, Skin, ConnectedPacket, CONNECTED_PACKET_ID, CompressionAlgorithm, Header, RequestNetworkSettings, Login, ClientToServerHandshake, CacheStatus, ResourcePackClientResponse, ViolationWarning, ChunkRadiusRequest, Interact, TextMessage, SetLocalPlayerAsInitialized, MovePlayer, PlayerAction, RequestAbility, Animate, CommandRequest, SettingsCommand, ContainerClose, FormResponseData, TickSync, UpdateSkin, PlayerAuthInput, DisconnectReason};
use proto::crypto::{Encryptor, BedrockIdentity, BedrockClientInfo};
use proto::uuid::Uuid;
use replicator::Replicator;

use tokio::task::JoinHandle;
use util::{AtomicFlag, BinaryRead, BinaryWrite, Deserialize, Joinable, Serialize, Vector};

use crate::command;
use crate::config::SERVER_CONFIG;
use crate::forms;

use lazy_static::lazy_static;

const REQUEST_TIMEOUT: Duration = Duration::from_millis(50);

lazy_static! {
    #[doc(hidden)]
    pub static ref CONNECTED_CLIENTS_METRIC: Gauge::<i64, AtomicI64> = Gauge::default();
    #[doc(hidden)]
    pub static ref RESPONSE_TIMES_METRIC: Histogram = Histogram::new(linear_buckets(0.0, 1.0, 25));
}

/// Represents a user connected to the server.
pub struct BedrockUser {
    pub(super) encryptor: OnceLock<Encryptor>,
    pub(super) identity: OnceLock<BedrockIdentity>,
    pub(super) client_info: OnceLock<BedrockClientInfo>,

    /// Next packet that the server is expecting to receive.
    pub(crate) expected: AtomicU32,
    /// Whether compression has been configured.
    pub(crate) should_decompress: AtomicFlag,
    /// Whether the client supports the blob cache.
    pub(crate) supports_cache: AtomicBool,
    /// Replication layer.
    pub(crate) replicator: Arc<Replicator>,
    pub(crate) raknet: Arc<RaknetUser>,
    pub(crate) player: OnceLock<PlayerData>,

    pub(crate) forms: forms::Subscriber,
    pub(crate) commands: command::ServiceEndpoint,

    pub(crate) broadcast: broadcast::Sender<BroadcastPacket>,
    pub(crate) job_handle: RwLock<Option<JoinHandle<()>>>
}

impl BedrockUser {
    /// Creates a new user.
    pub fn new(
        raknet: Arc<RaknetUser>,
        replicator: Arc<Replicator>,
        receiver: mpsc::Receiver<RaknetCommand>,
        commands: command::ServiceEndpoint,
        broadcast: broadcast::Sender<BroadcastPacket>
    ) -> Arc<Self> {
        CONNECTED_CLIENTS_METRIC.inc();

        let user = Arc::new(Self {
            encryptor: OnceLock::new(),
            identity: OnceLock::new(),
            client_info: OnceLock::new(),
            expected: AtomicU32::new(RequestNetworkSettings::ID),
            should_decompress: AtomicFlag::new(),
            supports_cache: AtomicBool::new(false),
            
            replicator,
            raknet,
            player: OnceLock::new(),

            forms: forms::Subscriber::new(),
            commands,
            
            broadcast,
            job_handle: RwLock::new(None)
        });

        let clone = Arc::clone(&user);
        let handle = tokio::spawn(async move {
            clone.receiver(receiver).await;
        });

        *user.job_handle.write() = Some(handle);
        user
    }

    /// The worker that processes incoming packets.
    #[tracing::instrument(
        skip_all,
        name = "BedrockUser::receiver",
        fields(
            address = %self.raknet.address
        )
    )]
    async fn receiver(self: &Arc<Self>, mut receiver: mpsc::Receiver<RaknetCommand>) {
        let mut broadcast = self.broadcast.subscribe();
        
        let mut should_run = true;
        while should_run {
            tokio::select! {
                cmd = receiver.recv() => {  
                    let Some(cmd) = cmd else {
                        // Channel has been closed.
                        break
                    };

                    match cmd {
                        RaknetCommand::Received(packet) => {
                            if let Err(err) = self.handle_encrypted_frame(packet).await {
                                tracing::error!("Failed to handle protocol packet: {err:#}");
                            }
                        },
                        RaknetCommand::BudgetExhausted => {
                            if self.kick_with_reason("Exhausted request budget", DisconnectReason::NotAllowed).await.is_err() {
                                // If kicking does not work, force disconnect them.
                                self.raknet.disconnect();
                            }
                        },
                        RaknetCommand::Disconnected => {
                            tracing::warn!("Raknet has reported a disconnect status, destroying user");
                            break
                        }
                    }
                },
                packet = broadcast.recv() => {
                    if let Ok(packet) = packet {
                        if let Err(err) = self.handle_broadcast(packet) {
                            tracing::error!("Failed to handle broadcast: {err:#}");
                        }
                    }
                },
                // Use `should_run` variable to trigger one final processing run when shutting down.
                _ = self.raknet.active.cancelled() => should_run = false
            };
        }

        tracing::info!("{} has disconnected", self.name().unwrap_or("<unknown>"));
        CONNECTED_CLIENTS_METRIC.dec();
    }

    /// Handles a packet broadcasted by another user.
    fn handle_broadcast(&self, packet: BroadcastPacket) -> anyhow::Result<()> {
        let should_send = packet.sender.map(|sender| sender != self.raknet.address).unwrap_or(true);
        if should_send {
            let header = Header {
                id: packet.id, sender_subclient: 0, target_subclient: 0
            };
    
            let mut body = Vec::new();
            header.serialize_into(&mut body)?;
            body.write_all(&packet.content)?;
    
            let mut full = Vec::with_capacity(body.len() + 5);
            full.write_var_u32(body.len() as u32)?;
            full.write_all(&body)?;

            self.send_serialized(full, DEFAULT_SEND_CONFIG)?;
        }

        Ok(())
    }

    /// Sends a form to the client and asynchronously waits for a response.
    /// 
    /// In case it is more convenient to use a channel receiver instead, use the [`subscribe`](Subscriber::subscribe)
    /// method on the `forms` field of the user.
    #[allow(clippy::future_not_send)]
    pub async fn send_form<F: forms::SubmittableForm>(&self, form: F) -> anyhow::Result<forms::Response> {
        let recv = self.forms.subscribe(self, form)?;
        let resp = recv.await?;

        Ok(resp)
    }

    /// Kicks a player from the server and displays the specified message to them.
    #[inline]
    pub fn kick<'a>(&'a self, message: &'a str) -> impl Future<Output = anyhow::Result<()>> + 'a {
        // This function returns a future object directly to reduce code bloat from async.
        self.kick_with_reason(message, DisconnectReason::Kicked)
    }

    /// Kicks a player from the server and displays the specified message to them.
    /// This also adds a reason to the kick, which is used for telemetry purposes.
    #[tracing::instrument(
        name = "BedrockUser::kick_with_reason",
        skip(self, message, reason)
        fields(
            username = %self.name().unwrap_or("<unknown>"),
            reason = %message
        )
    )]
    pub async fn kick_with_reason(&self, message: &str, reason: DisconnectReason) -> anyhow::Result<()> {
        let disconnect_packet = Disconnect {
            reason, message, hide_message: false
        };
        self.send(disconnect_packet)?;

        tracing::info!("User has been kicked");
        self.raknet.join().await
    }

    /// Sends a packet to all initialised sessions including self.
    pub fn broadcast<P: ConnectedPacket + Serialize + Clone>(
        &self,
        packet: P,
    ) -> anyhow::Result<()> {
        self.broadcast.send(BroadcastPacket::new(packet, None)?)?;
        Ok(())
    }

    /// Sends a packet to all initialised sessions other than self.
    pub fn broadcast_others<P: ConnectedPacket + Serialize + Clone>(
        &self,
        packet: P,
    ) -> anyhow::Result<()> {
        self.broadcast.send(BroadcastPacket::new(packet, Some(self.raknet.address))?)?;
        Ok(())
    }

    /// Sends a game packet with default settings
    /// (reliable ordered and medium priority)
    pub fn send<T: ConnectedPacket + Serialize>(&self, packet: T) -> anyhow::Result<()> {
        let header = Header {
            id: T::ID, sender_subclient: 0, target_subclient: 0
        };

        let mut body = Vec::new();
        header.serialize_into(&mut body)?;
        packet.serialize_into(&mut body)?;

        let mut full = Vec::with_capacity(body.len() + 5);
        full.write_var_u32(body.len() as u32)?;
        full.write_all(&body)?;

        self.send_serialized(full, DEFAULT_SEND_CONFIG)
    }

    /// Sends a game packet with custom reliability and priority
    pub fn send_serialized<B>(&self, packet: B, config: SendConfig) -> anyhow::Result<()>
        where
            B: AsRef<[u8]>
    {
        let mut out = Vec::new();
        if self.should_decompress.get() {
            let (algorithm, threshold) = {
                let config = SERVER_CONFIG.read();
                (config.compression_algorithm, config.compression_threshold)
            };

            if packet.as_ref().len() > threshold as usize {
                // Compress packet
                match algorithm {
                    CompressionAlgorithm::Snappy => {
                        unimplemented!("Snappy compression");
                    }
                    CompressionAlgorithm::Flate => {
                        let mut writer = DeflateEncoder::new(Vec::new(), Compression::best());

                        writer.write_all(packet.as_ref())?;
                        let compressed_body = writer.finish()?;

                        out.write_u8(CONNECTED_PACKET_ID)?;
                        out.write_u8(algorithm as u8)?;
                        out.write_all(&compressed_body)?;
                    }
                }
            } else {
                // Also reserve capacity for checksum even if encryption is disabled,
                // preventing allocations.
                out = Vec::with_capacity(1 + packet.as_ref().len() + 8);
                out.write_u8(CONNECTED_PACKET_ID)?;
                out.write_all(packet.as_ref())?;
            }
        } else {
            // Also reserve capacity for checksum even if encryption is disabled,
            // preventing allocations.
            out = Vec::with_capacity(1 + packet.as_ref().len() + 8);
            out.write_u8(CONNECTED_PACKET_ID)?;
            out.write_all(packet.as_ref())?;
        };

        if let Some(encryptor) = self.encryptor.get() {
            encryptor.encrypt(&mut out).context("Failed to encrypt packet")?;
        }

        self.raknet.send_raw_buffer_with_config(out, config);
        Ok(())
    }

    /// Handles a received encrypted frame.
    /// 
    /// This is the first function that is called when a packet is received from the RakNet processing layer.
    /// It first decrypts the packet if encryption has been enabled and then optionally decompresses it.
    /// 
    /// After processing, this function sends the processed packet to [`handle_frame_body`](Self::handle_frame_body)
    /// function,
    async fn handle_encrypted_frame(self: &Arc<Self>, mut packet: Vec<u8>) -> anyhow::Result<()> {
        let timestamp = Instant::now();

        if packet[0] != 0xfe {
            anyhow::bail!("First byte in a Bedrock proto packet should be 0xfe");
        }

        packet.remove(0);

        // Decrypt if encryption is enabled.
        if let Some(encryptor) = self.encryptor.get() {
            encryptor.decrypt(&mut packet).context("Failed to decrypt packet")?;
        }

        let out = if self.should_decompress.get() {
            if packet[0] == 0xff {
                packet.remove(0);
                self.handle_frame_body(packet).await
            } else {
                let algorithm = CompressionAlgorithm::try_from(packet[0])?;
                packet.remove(0);

                match algorithm {
                    CompressionAlgorithm::Flate => {
                        let mut reader = flate2::read::DeflateDecoder::new(packet.as_slice());

                        let mut decompressed = Vec::new();
                        reader.read_to_end(&mut decompressed)?;
                        self.handle_frame_body(decompressed).await
                    },
                    CompressionAlgorithm::Snappy => {
                        anyhow::bail!("Snappy compression is not implemented");
                    }
                }
            }
        } else {
            self.handle_frame_body(packet).await
        };

        RESPONSE_TIMES_METRIC.observe(timestamp.elapsed().as_millis() as f64);

        out
    }

    /// Handles the body of a frame.
    /// 
    /// This function does the actual processing of the content of the frame and responds to it.
    #[tracing::instrument(
        skip_all,
        name = "BedrockUser::handle_frame_body"
        fields(
            username = self.name().unwrap_or("<unknown>")
        )
    )]
    async fn handle_frame_body(self: &Arc<Self>, mut packet: Vec<u8>) -> anyhow::Result<()> {
        let start_len = packet.len();
        let mut reader: &[u8] = packet.as_ref();
        let _length = reader.read_var_u32()?;
        let header = Header::deserialize_from(&mut reader)?;

        packet.drain(0..(start_len - reader.remaining()));

        let expected = self.expected();
        if expected != u32::MAX && header.id != expected {
            // Server received an unexpected packet.
            tracing::warn!(
                "Client sent unexpected packet while logging in (expected {:#04x}, got {:#04x})",
                expected, header.id
            );
            
            self.kick_with_reason("Unexpected packet", DisconnectReason::UnexpectedPacket).await?;
        }

        let this = Arc::clone(self);
        let future = async move {
            match header.id {
                PlayerAuthInput::ID => this.handle_auth_input(packet),
                RequestNetworkSettings::ID => {
                    this.handle_network_settings_request(packet)
                }
                Login::ID => this.handle_login(packet).await,
                ClientToServerHandshake::ID => {
                    this.handle_client_to_server_handshake(packet)
                }
                CacheStatus::ID => this.handle_cache_status(packet),
                ResourcePackClientResponse::ID => {
                    this.handle_resource_client_response(packet)
                }
                ViolationWarning::ID => this.handle_violation_warning(packet).await,
                ChunkRadiusRequest::ID => this.handle_chunk_radius_request(packet),
                Interact::ID => this.handle_interaction(packet),
                TextMessage::ID => this.handle_text_message(packet).await,
                SetLocalPlayerAsInitialized::ID => {
                    this.handle_local_initialized(packet)
                }
                MovePlayer::ID => this.handle_move_player(packet),
                PlayerAction::ID => this.handle_player_action(packet),
                RequestAbility::ID => this.handle_ability_request(packet),
                Animate::ID => this.handle_animation(packet),
                // Command request does not return a result because it does not fail.
                CommandRequest::ID => Ok(this.handle_command_request(packet)),
                UpdateSkin::ID => this.handle_skin_update(packet),
                SettingsCommand::ID => this.handle_settings_command(packet),
                ContainerClose::ID => this.handle_container_close(packet),
                FormResponseData::ID => this.handle_form_response(packet),
                TickSync::ID => this.handle_tick_sync(packet),
                id => anyhow::bail!("Invalid game packet: {id:#04x}"),
            }
        };
        
        let timeout = tokio::time::timeout(REQUEST_TIMEOUT, future);
        match timeout.await {
            Ok(result) => result,
            Err(_) => {
                tracing::error!("Request timed out");
                anyhow::bail!("Request timed out")
            }
        }
    }

    /// Returns the forms handler.
    #[inline]
    pub const fn forms(&self) -> &forms::Subscriber {
        &self.forms
    }

    /// This function panics if the identity was not set.
    #[inline]
    pub fn identity(&self) -> anyhow::Result<&BedrockIdentity> {
        self.identity.get().ok_or_else(|| anyhow::anyhow!("Identity unknown: user has not logged in yet"))
    }

    /// This function panics if the name was not set.
    #[inline]
    pub fn name(&self) -> anyhow::Result<&str> {
        self.identity().map(|id| id.name.as_str())
    }

    /// This function panics if the XUID was not set.
    #[inline]
    pub fn xuid(&self) -> anyhow::Result<u64> {
        self.identity().map(|id| id.xuid)
    }

    /// This function panics if the UUID was not set.
    #[inline]
    pub fn uuid(&self) -> anyhow::Result<&Uuid> {
        self.identity().map(|id| &id.uuid)
    }

    /// This function panics if the encryptor was not set.
    #[inline]
    pub fn encryptor(&self) -> anyhow::Result<&Encryptor> {
        self.encryptor.get().ok_or_else(|| anyhow::anyhow!("Encryption handshake has not been performed yet"))
    }

    /// Returns the next expected packet for this session.
    /// The expected packet will be [`u32::MAX`] if the user is fully
    /// initialized and therefore doesn't follow a strict packet order anymore.
    #[inline]
    pub fn expected(&self) -> u32 {
        self.expected.load(Ordering::SeqCst)
    }

    /// Returns whether the user is fully initialized.
    #[inline]
    pub fn initialized(&self) -> bool {
        self.expected() == u32::MAX
    }

    /// This functions panic if the player data was not initialized.
    pub fn player(&self) -> anyhow::Result<&PlayerData> {
        self.player.get().ok_or_else(|| anyhow::anyhow!("Player data unavailable"))
    }
}

impl Joinable for BedrockUser {
    #[tracing::instrument(
        skip(self),
        name = "BedrockUser::join"
    )]
    async fn join(&self) -> anyhow::Result<()> {
        let handle = self.job_handle.write().take();
        if let Some(handle) = handle {
            // Error logged by RakNet join method.
            let _: anyhow::Result<()> = self.raknet.join().await;

            match handle.await {
                Ok(_) => Ok(()),
                Err(err) => {
                    tracing::error!("Error occurred while awaiting Bedrock user service shutdown: {err:#?}");
                    Ok(())
                }
            }
        } else {
            tracing::error!("This user service has already been joined");
            anyhow::bail!("User service already joined");
        }        
    }
}

/// Contains data that is mostly related to the player in the vanilla game.
/// 
/// Unlike [`BedrockUser`], most of this data is not related to the Bedrock protocol itself.
pub struct PlayerData {
    /// Whether the player's inventory is currently open.
    pub is_inventory_open: AtomicBool,
    /// Position of the player.
    pub position: Vector<f32, 3>,
    /// Rotation of the player.
    /// x and y components are general rotation.
    /// z component is head yaw.
    pub rotation: Vector<f32, 3>,
    /// Game mode.
    pub game_mode: GameMode,
    /// General permission level.
    pub permission_level: PermissionLevel,
    /// Command permission level
    pub command_permission_level: CommandPermissionLevel,
    /// The client's skin.
    pub skin: RwLock<Skin>,
    /// Runtime ID.
    pub runtime_id: u64,
}

impl PlayerData {
    /// Creates a new player data struct.
    pub fn new(skin: Skin) -> Self {
        Self {
            is_inventory_open: AtomicBool::new(false),
            position: Vector::from([0.0, 50.0, 0.0]),
            rotation: Vector::from([0.0; 3]),
            game_mode: GameMode::Creative,
            permission_level: PermissionLevel::Operator,
            command_permission_level: CommandPermissionLevel::Owner,
            skin: RwLock::new(skin),
            runtime_id: 1
        }
    }

    /// The gamemode the player is currently in.
    pub const fn gamemode(&self) -> GameMode {
        self.game_mode
    }

    /// The runtime ID of the player.
    pub const fn runtime_id(&self) -> u64 {
        self.runtime_id
    }

    /// The permission level of the player.
    pub const fn permission_level(&self) -> PermissionLevel {
        self.permission_level
    }

    /// The command permission level of the player.
    pub const fn command_permission_level(&self) -> CommandPermissionLevel {
        self.command_permission_level
    }
}