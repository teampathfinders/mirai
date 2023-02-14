use bytes::{BufMut, BytesMut};

use crate::network::packets::GamePacket;
use common::Encodable;
use common::VResult;
use common::WriteExtensions;

/// Supported compression algorithms.
///
/// Snappy is fast, but has produces lower compression ratios.
/// Flate is slow, but produces high compression ratios.
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum CompressionAlgorithm {
    /// The Deflate/Zlib compression algorithm.
    Deflate = 0,
    /// The Snappy compression algorithm.
    /// Available since Minecraft 1.19.30.
    Snappy = 1,
}

/// Settings for client throttling.
///
/// If client throttling is enabled, the client will tick fewer players,
/// improving performance on low-end devices.
#[derive(Debug, Copy, Clone)]
pub struct ClientThrottleSettings {
    /// Regulates whether the client should throttle players.
    pub enabled: bool,
    /// Threshold for client throttling.
    /// If the number of players in the game exceeds this value, players will be throttled.
    pub threshold: u8,
    /// Amount of players that are ticked when throttling is enabled.
    pub scalar: f32,
}

/// Sent by the server to modify network related settings.
#[derive(Debug, Clone)]
pub struct NetworkSettings {
    /// Minimum size of a packet that is compressed.
    /// Any packets below this threshold will not be compressed.
    /// Settings this to 0 disables compression.
    pub compression_threshold: u16,
    /// Algorithm used to compress packets.
    pub compression_algorithm: CompressionAlgorithm,
    /// Client throttling settings.
    pub client_throttle: ClientThrottleSettings,
}

impl GamePacket for NetworkSettings {
    /// Unique ID of this packet.
    const ID: u32 = 0x8f;
}

impl Encodable for NetworkSettings {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(2 + 2 + 1 + 1 + 4);

        buffer.put_u16(self.compression_threshold);
        buffer.put_u16(self.compression_algorithm as u16);
        buffer.put_bool(self.client_throttle.enabled);
        buffer.put_u8(self.client_throttle.threshold);
        buffer.put_f32(self.client_throttle.scalar);

        Ok(buffer)
    }
}
