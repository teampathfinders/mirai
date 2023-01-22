use bytes::{BufMut, BytesMut};

use crate::error::VexResult;
use crate::packets::{GameEncodable, GamePacket};
use crate::raknet::packets::Encodable;

/// Supported compression algorithms.
///
/// Snappy is fast, but has produces lower compression ratios.
/// Flate is slow, but produces high compression ratios.
#[derive(Debug, Copy, Clone)]
pub enum CompressionAlgorithm {
    /// The Deflate/Zlib compression algorithm.
    Flate,
    /// The Snappy compression algorithm.
    /// Available since Minecraft 1.19.30.
    Snappy,
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
#[derive(Debug)]
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

impl GameEncodable for NetworkSettings {
    fn encode(&self, buffer: &mut BytesMut) -> VexResult<()> {
        buffer.reserve(2 + 2 + 1 + 1 + 4);

        buffer.put_u16(self.compression_threshold);
        buffer.put_u16(self.compression_algorithm as u16);
        buffer.put_u8(self.client_throttle.enabled as u8);
        buffer.put_u8(self.client_throttle.threshold);
        buffer.put_f32(self.client_throttle.scalar);

        Ok(())
    }
}
