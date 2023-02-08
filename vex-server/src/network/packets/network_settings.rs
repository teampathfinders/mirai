use bytes::{BufMut, BytesMut};

use vex_common::{ClientThrottleSettings, CompressionAlgorithm, Encodable, VResult, WriteExtensions};

use crate::network::packets::GamePacket;
use crate::util::WriteExtensions;

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
