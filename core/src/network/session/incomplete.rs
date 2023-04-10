use std::{sync::{atomic::{AtomicU32, AtomicBool, AtomicI32, Ordering}, Arc}, net::SocketAddr};

use tokio::sync::{mpsc, OnceCell, broadcast};
use util::{bytes::MutableBuffer, Deserialize, Serialize};
use uuid::Uuid;

use crate::{network::{Disconnect, ConnectedPacket, Skin, DeviceOS, ChunkRadiusRequest, CacheStatus, ChunkRadiusReply, RequestNetworkSettings, PlayStatus, NETWORK_VERSION, Status, NetworkSettings}, raknet::{RakNetSession, RakNetMessage, PacketConfig, Reliability, SendPriority, RakNetSessionBuilder, BroadcastPacket, RawPacket}, crypto::{UserData, IdentityData}, config::SERVER_CONFIG, instance::UdpController};

use super::{SessionLike, SessionRef};

const RAKNET_MESSAGE_CHANNEL_SIZE: usize = 5;

#[derive(Default)]
pub struct SessionBuilder {
    addr: Option<SocketAddr>,
    udp: Option<Arc<UdpController>>,
    sender: Option<mpsc::Sender<RawPacket>>,
    receiver: Option<mpsc::Receiver<RawPacket>>,
    broadcast: Option<broadcast::Sender<BroadcastPacket>>,
    guid: u64
}

impl SessionBuilder {
    /// Creates a new `SessionBuilder`.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures the Raknet GUID of the session.
    #[inline]
    pub fn guid(&mut self, guid: u64) -> &mut Self {
        self.guid = guid;
        self
    }

    #[inline]
    pub fn udp(&mut self, controller: Arc<UdpController>) -> &mut Self {
        self.udp = Some(controller);
        self
    }

    #[inline]
    pub fn addr(&mut self, addr: SocketAddr) -> &mut Self {
        self.addr = Some(addr);
        self
    }

    /// Configures the channel used for packet sending/receiving.
    #[inline]
    pub fn channel(
        &mut self,
        (sender, receiver): (mpsc::Sender<RawPacket>, mpsc::Receiver<RawPacket>)
    ) -> &mut Self {
        self.receiver = Some(receiver);
        self.sender = Some(sender);
        self
    }

    /// Configures the channel used for packet broadcasting.
    #[inline]
    pub fn broadcast(&mut self, broadcast: broadcast::Sender<BroadcastPacket>) -> &mut Self {
        self.broadcast = Some(broadcast);
        self
    }

    /// Builds a [`SessionRef`] and consumes this builder.
    ///
    /// # Panics
    ///
    /// This method panics if several required options were not set.
    #[inline]
    pub fn build(mut self) -> SessionRef<IncompleteSession> {
        let sender = self.sender.take().unwrap();
        let session = IncompleteSession::from(self);

        SessionRef {
            sender, session
        }
    }
}

#[derive(Debug)]
pub struct IncompleteSession {
    pub expected: AtomicU32,
    pub cache_support: AtomicBool,
    pub render_distance: AtomicI32,
    pub guid: u64,
    pub compression: AtomicBool,
    pub sender: mpsc::Sender<RakNetMessage>,
    pub receiver: mpsc::Receiver<RakNetMessage>,
    pub raknet: RakNetSession,
    pub identity: OnceCell<IdentityData>,
    /// Extra user data, such as device OS and language.
    pub user_data: OnceCell<UserData>,
    pub skin: OnceCell<Skin>
}

impl IncompleteSession {
    #[inline]
    pub fn get_identity_data(&self) -> anyhow::Result<&IdentityData> {
        self.identity.get().ok_or_else(|| {
            anyhow::anyhow!("Identity data has not been initialised yet")
        })
    }

    #[inline]
    pub fn get_user_data(&self) -> anyhow::Result<&UserData> {
        self.user_data.get().ok_or_else(|| {
            anyhow::anyhow!("User data has not been initialised yet")
        })
    }

    #[inline]
    pub fn get_device_os(&self) -> anyhow::Result<DeviceOS> {
        let data = self.user_data.get().ok_or_else(|| {
            anyhow::anyhow!(
                "Device OS data has not been initialised yet"
            )
        })?;
        Ok(data.build_platform)
    }

    /// Returns the randomly generated GUID given by the client itself.
    #[inline]
    pub fn get_guid(&self) -> u64 {
        self.raknet.guid
    }

    /// Retrieves the identity of the client.
    #[inline]
    pub fn get_uuid(&self) -> anyhow::Result<&Uuid> {
        let identity = self.identity.get().ok_or_else(|| {
            anyhow::anyhow!(
                "Identity ID data has not been initialised yet"
            )
        })?;
        Ok(&identity.uuid)
    }

    /// Retrieves the XUID of the client.
    #[inline]
    pub fn get_xuid(&self) -> anyhow::Result<u64> {
        let identity = self.identity.get().ok_or_else(|| {
            anyhow::anyhow!("XUID data has not been initialised yet")
        })?;
        Ok(identity.xuid)
    }

    /// Retrieves the display name of the client.
    #[inline]
    pub fn get_display_name(&self) -> anyhow::Result<&str> {
        let identity = self.identity.get().ok_or_else(|| {
            anyhow::anyhow!(
                "Display name data has not been initialised yet"
            )
        })?;
        Ok(&identity.display_name)
    }

    pub fn on_cache_status(&mut self, packet: MutableBuffer) -> anyhow::Result<()> {
        let status = CacheStatus::deserialize(packet.as_ref())?;
        self.cache_support.store(status.support, Ordering::Relaxed);

        Ok(())
    }

    pub fn on_radius_request(&mut self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = ChunkRadiusRequest::deserialize(packet.as_ref())?;
        let radius = std::cmp::min(SERVER_CONFIG.read().allowed_render_distance, request.radius);

        if request.radius <= 0 {
            anyhow::bail!("Render distance must be greater than 0");
        }

        self.send(ChunkRadiusReply {
            radius
        })?;

        self.render_distance.store(radius, Ordering::Relaxed);
        Ok(())
    }

    pub fn on_settings_request(&mut self, packet: MutableBuffer) -> anyhow::Result<()> {
        let request = RequestNetworkSettings::deserialize(packet.as_ref())?;
        if request.protocol_version > NETWORK_VERSION {
            self.send(PlayStatus {
                status: Status::FailedServer
            })?;

            anyhow::bail!(format!(
                "Client is using a newer protocol version: {} vs. {NETWORK_VERSION}",
                request.protocol_version
            ));
        } else if request.protocol_version < NETWORK_VERSION {
            self.send(PlayStatus {
                status: Status::FailedClient
            })?;

            anyhow::bail!(format!(
                "Client is using an older protocol version: {} vs. {NETWORK_VERSION}",
                request.protocol_version
            ));
        }

        let response = {
            let lock = SERVER_CONFIG.read();

            NetworkSettings {
                compression_algorithm: lock.compression_algorithm,
                compression_threshold: lock.compression_threshold,
                client_throttle: lock.client_throttle
            }
        };

        self.compression.store(true, Ordering::Relaxed);
        self.send(response)
    }
}

impl SessionLike for IncompleteSession {
    fn send<T>(&self, packet: T) -> anyhow::Result<()>
    where
        T: ConnectedPacket + Serialize
    {
        self.raknet.send(packet)
    }

    fn send_buf<A>(&self, buf: A) -> anyhow::Result<()>
    where
        A: AsRef<[u8]>
    {
        self.raknet.send_buf(buf, PacketConfig {
            reliability: Reliability::ReliableOrdered,
            priority: SendPriority::Medium
        })
    }

    fn broadcast<T>(&self, packet: T) -> anyhow::Result<()>
    where
        T: ConnectedPacket + Serialize
    {
        self.raknet.broadcast(packet)
    }

    fn broadcast_others<T>(&self, packet: T) -> anyhow::Result<()>
    where
        T: ConnectedPacket + Serialize,
    {
        self.raknet.broadcast_others(packet)
    }

    fn kick<S>(&self, reason: S) -> anyhow::Result<()>
    where
        S: AsRef<str>
    {
        let disconnect_packet = Disconnect {
            reason: reason.as_ref(),
            hide_reason: false
        };
        self.send(disconnect_packet)?;
        self.raknet.token.cancel();

        Ok(())
    }
}

impl From<SessionBuilder> for IncompleteSession {
    fn from(builder: SessionBuilder) -> Self {
        let (sessionSender, raknetReceiver) = mpsc::channel(RAKNET_MESSAGE_CHANNEL_SIZE);
        let (raknetSender, sessionReceiver) = mpsc::channel(RAKNET_MESSAGE_CHANNEL_SIZE);

        let raknet = RakNetSessionBuilder::new()
            .udp(builder.udp.unwrap())
            .addr(builder.addr.unwrap())
            .broadcast(builder.broadcast.unwrap())
            .packet_receiver(builder.receiver.unwrap())
            .guid(builder.guid)
            .channel((raknetSender, raknetReceiver))
            .build();

        Self {
            expected: AtomicU32::new(0),
            cache_support: AtomicBool::new(false),
            render_distance: AtomicI32::new(0),
            guid: builder.guid,
            compression: AtomicBool::new(false),
            raknet,
            sender: sessionSender,
            receiver: sessionReceiver,
            identity: OnceCell::new(),
            user_data: OnceCell::new(),
            skin: OnceCell::new()
        }
    }
}