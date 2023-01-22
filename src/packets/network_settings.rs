use bytes::{BufMut, BytesMut};

use crate::error::VexResult;
use crate::raknet::packets::Encodable;

#[derive(Debug, Copy, Clone)]
pub enum CompressionAlgorithm {
    Flate,
    Snappy,
}

/// Sent by the server to modify network related settings.
#[derive(Debug)]
pub struct NetworkSettings {
    /// Minimum size of a packet that is compressed.
    /// Any packets below this threshold will not be compressed.
    pub compression_threshold: u16,
    /// Algorithm used to compress packets.
    pub compression_algorithm: CompressionAlgorithm,
    /// Regulates whether the client should throttle players when exceeding the threshold.
    /// Players outside the threshold will not be ticked, improving performance on low-end devices.
    pub client_throttle: bool,
    /// Threshold for client throttling. If the number of players in the game exceeds this value,
    /// players will be throttled.
    pub client_throttle_threshold: u8,
    /// Amount of players that are ticked when throttling is enabled.
    pub client_throttle_scalar: f32,
}

impl NetworkSettings {
    pub const ID: u8 = 0x8f;
}

impl Encodable for NetworkSettings {
    fn encode(&self) -> VexResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(1 + 2 + 2 + 1 + 1 + 4);

        buffer.put_u8(Self::ID);
        buffer.put_u16(self.compression_threshold);
        buffer.put_u16(self.compression_algorithm as u16);
        buffer.put_u8(self.client_throttle as u8);
        buffer.put_u8(self.client_throttle_threshold);
        buffer.put_f32(self.client_throttle_scalar);

        Ok(buffer)
    }
}