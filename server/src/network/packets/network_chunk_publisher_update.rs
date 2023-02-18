use bytes::{BufMut, BytesMut};
use common::{BlockPosition, VResult, WriteExtensions, size_of_var};

use common::Encodable;

use super::GamePacket;

#[derive(Debug, Clone)]
pub struct NetworkChunkPublisherUpdate {
    pub position: BlockPosition,
    pub radius: u32,
}

impl GamePacket for NetworkChunkPublisherUpdate {
    const ID: u32 = 0x79;
}

impl Encodable for NetworkChunkPublisherUpdate {
    fn encode(&self) -> VResult<BytesMut> {
        let packet_size =
            size_of_var(self.position.x) +
            size_of_var(self.position.y) +
            size_of_var(self.position.z) +
            size_of_var(self.radius) + 4;

        let mut buffer = BytesMut::with_capacity(packet_size);

        buffer.put_block_pos(&self.position);
        buffer.put_var_u32(self.radius);

        // No saved chunks.
        buffer.put_u32(0);

        Ok(buffer)
    }
}
