use bytes::{BufMut, BytesMut};

use crate::error::VexResult;
use crate::packets::{GamePacket, Packet};
use crate::raknet::packets::Encodable;

pub struct PacketBatch {
    packets: Vec<BytesMut>,
}

impl PacketBatch {
    pub fn new() -> Self {
        Self {
            packets: Vec::new()
        }
    }

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

        Ok(buffer)
    }
}