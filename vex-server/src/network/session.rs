use std::io::Write;
use std::net::SocketAddr;
use std::num::NonZeroU64;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use bytes::{BufMut, BytesMut};
use flate2::Compression;
use flate2::write::DeflateEncoder;
use parking_lot::{Mutex, RwLock};
use tokio::net::UdpSocket;
use tokio::sync::OnceCell;
use tokio_util::sync::CancellationToken;

use vex_common::{CompressionAlgorithm, Encodable, error, SERVER_CONFIG, VResult};

use crate::crypto::{Encryptor, IdentityData, UserData};
use crate::network::Packet;
use crate::network::packets::{CompressionAlgorithm, DeviceOS, Disconnect, GamePacket, Packet};
use crate::network::session::compound_collector::CompoundCollector;
use crate::network::session::order_channel::OrderChannel;
use crate::network::session::recovery_queue::RecoveryQueue;
use crate::network::session::send_queue::SendQueue;
use crate::util::AsyncDeque;

/// Sessions directly correspond to clients connected to the server.
///
/// Anything that has to do with specific clients must be communicated with their associated sessions.
/// The server does not interact with clients directly, everything is done through these sessions.
///
#[derive(Debug)]
pub struct Session {
    /// Identity data such as XUID and display name.
    pub identity: OnceCell<IdentityData>,
    /// Extra user data, such as device OS and skin.
    pub user_data: OnceCell<UserData>,
    /// Used to encrypt and decrypt packets.
    pub encryptor: OnceCell<Encryptor>,
    /// Indicates whether this session is active.
    pub active: CancellationToken,
    /// Whether packets should be compressed.
    /// Even if this is set to true server-wide, it will only be enabled once compression has been configured for this session.
    pub compression_enabled: AtomicBool,
    pub raknet_session: vex_raknet::Session,
}

impl Session {
    /// Creates a new session.
    pub fn new(ipv4_socket: Arc<UdpSocket>, address: SocketAddr, mtu: u16, guid: u64) -> Arc<Self> {
        let session = Arc::new(Self {
            identity: OnceCell::new(),
            user_data: OnceCell::new(),
            encryptor: OnceCell::new(),
            active: CancellationToken::new(),
            compression_enabled: AtomicBool::new(false),
            raknet_session: vex_raknet::Session::new(ipv4_socket, address, mtu, guid),
        });

        session
    }

    pub async fn flush(&self) -> VResult<()> {
        self.raknet_session.flush().await
    }

    /// Sends a game packet with default settings
    /// (reliable ordered and medium priority)
    pub fn send_packet<T: GamePacket + Encodable>(&self, packet: T) -> VResult<()> {
        self.send_packet_with_config(packet, DEFAULT_CONFIG)
    }

    /// Sends a game packet with custom reliability and priority
    pub fn send_packet_with_config<T: GamePacket + Encodable>(
        &self,
        packet: T,
        config: PacketConfig,
    ) -> VResult<()> {
        let packet = Packet::new(packet).subclients(0, 0);

        let mut buffer = BytesMut::new();
        buffer.put_u8(GAME_PACKET_ID);

        let mut packet_buffer = packet.encode()?;
        tracing::info!("Buffer: {:?}", packet_buffer.as_ref());

        if self.compression_enabled.load(Ordering::SeqCst) {
            let (algorithm, threshold) = {
                let config = SERVER_CONFIG.read();
                (config.compression_algorithm, config.compression_threshold)
            };

            if packet_buffer.len() > threshold as usize {
                // Compress packet
                match SERVER_CONFIG.read().compression_algorithm {
                    CompressionAlgorithm::Snappy => {
                        todo!("Snappy compression")
                    }
                    CompressionAlgorithm::Deflate => {
                        let mut writer = DeflateEncoder::new(Vec::new(), Compression::fast());

                        writer.write_all(packet_buffer.as_ref())?;
                        // .context("Failed to compress packet using Deflate")?;

                        packet_buffer = BytesMut::from(writer.finish()?.as_slice());
                        // .context("Failed to flush Deflate encoder")?.as_slice());
                    }
                }
            }
        }

        if let Some(encryptor) = self.encryptor.get() {
            packet_buffer = encryptor.encrypt(packet_buffer);
        }

        buffer.put(packet_buffer);

        self.send_raw_buffer_with_config(buffer, config);
        Ok(())
    }

    /// Retrieves the identity of the client.
    ///
    /// Warning: An internal RwLock is kept in a read state until the return value of this function is dropped.
    pub fn get_identity(&self) -> VResult<&str> {
        let identity = self
            .identity
            .get()
            .ok_or_else(|| error!(NotInitialized, "Identity ID has not been initialised yet"))?;
        Ok(identity.identity.as_str())
    }

    /// Retrieves the XUID of the client.
    ///
    /// Warning: An internal RwLock is kept in a read state until the return value of this function is dropped.
    pub fn get_xuid(&self) -> VResult<u64> {
        let identity = self
            .identity
            .get()
            .ok_or_else(|| error!(NotInitialized, "XUID has not been initialised yet"))?;
        Ok(identity.xuid)
    }

    /// Retrieves the display name of the client.
    ///
    /// Warning: An internal RwLock is kept in a read state until the return value of this function is dropped.
    pub fn get_display_name(&self) -> VResult<&str> {
        let identity = self
            .identity
            .get()
            .ok_or_else(|| error!(NotInitialized, "Display name has not been initialised yet"))?;
        Ok(identity.display_name.as_str())
    }

    pub fn get_encryptor(&self) -> VResult<&Encryptor> {
        self.encryptor
            .get()
            .ok_or_else(|| error!(NotInitialized, "Encryption has not been initialised yet"))
    }

    pub fn get_device_os(&self) -> VResult<DeviceOS> {
        let data = self
            .user_data
            .get()
            .ok_or_else(|| error!(NotInitialized, "User data has not been initialised yet"))?;
        Ok(data.device_os)
    }

    /// Returns the randomly generated GUID given by the client itself.
    pub const fn get_guid(&self) -> u64 {
        self.raknet_session.get_guid()
    }

    /// Kicks the session from the server, displaying the given menu.
    pub fn kick<S: Into<String>>(&self, message: S) -> VResult<()> {
        let disconnect_packet = Disconnect {
            kick_message: message.into(),
            hide_disconnect_screen: false,
        };
        self.send_packet(disconnect_packet)?;
        // self.flag_for_close();
        // FIXME: Client sends disconnect and acknowledgement packet after closing.

        Ok(())
    }

    /// Returns whether the session is currently active.
    ///
    /// If this returns false, any remaining associated processes should be stopped as soon as possible.
    #[inline]
    pub fn is_active(&self) -> bool {
        !self.active.is_cancelled()
    }
}
