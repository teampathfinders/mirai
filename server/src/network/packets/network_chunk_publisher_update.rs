use bytes::{BufMut, BytesMut, Bytes};
use common::{BlockPosition, VResult, WriteExtensions, size_of_var};

use common::Serialize;

use super::ConnectedPacket;

#[derive(Debug, Clone)]
pub struct NetworkChunkPublisherUpdate {
    pub position: BlockPosition,
    pub radius: u32,
}

impl ConnectedPacket for NetworkChunkPublisherUpdate {
    const ID: u32 = 0x79;

    fn serialized_size(&self) -> usize {
        size_of_var(self.position.x) +
        size_of_var(self.position.y) +
        size_of_var(self.position.z) +
        size_of_var(self.radius) + 4
    }
}

impl Serialize for NetworkChunkPublisherUpdate {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_block_pos(&self.position);
        buffer.put_var_u32(self.radius);

        // No saved chunks.
        buffer.put_u32(0);
    }
}
