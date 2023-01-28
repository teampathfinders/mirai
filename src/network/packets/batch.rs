use std::io::Write;

use bytes::{BufMut, BytesMut};
use flate2::Compression;
use flate2::write::DeflateEncoder;

use crate::config::SERVER_CONFIG;
use crate::error::VexResult;
use crate::network::packets::{CompressionAlgorithm, GamePacket, Packet};
use crate::network::traits::Encodable;

/// Batch of game packets.
pub struct PacketBatch {
    compression_enabled: bool,
    packets: Vec<BytesMut>,
}

impl PacketBatch {
    pub const ID: u8 = 0xfe;

    /// Creates a new packet batch.
    pub const fn new(compression_enabled: bool) -> Self {
        Self {
            compression_enabled,
            packets: Vec::new(),
        }
    }

    /// Adds a packet to the batch.
    pub fn add<T: GamePacket + Encodable>(mut self, packet: Packet<T>) -> VexResult<Self> {
        let mut encoded = packet.encode()?;
        self.packets.push(encoded);

        Ok(self)
    }
}

impl Encodable for PacketBatch {
    fn encode(&self) -> VexResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_u8(0xfe);
        for packet in &self.packets {
            buffer.put(packet.as_ref());
        }

        let (algorithm, threshold) = {
            let config = SERVER_CONFIG.read();
            (config.compression_algorithm, config.compression_threshold)
        };

        if self.compression_enabled && buffer.len() > threshold as usize {
            let mut compressed = BytesMut::new();
            compressed.put_u8(0xfe);

            // Compress the packets
            compressed.put(
                match algorithm {
                    CompressionAlgorithm::Deflate => {
                        let mut writer = DeflateEncoder::new(Vec::new(), Compression::best());
                        writer.write_all(&buffer.as_ref()[1..])?;

                        writer.finish()?
                    }
                    CompressionAlgorithm::Snappy => {
                        todo!("Snappy compression");
                    }
                }
                    .as_slice(),
            );

            buffer = compressed;
        }

        Ok(buffer)
    }
}
