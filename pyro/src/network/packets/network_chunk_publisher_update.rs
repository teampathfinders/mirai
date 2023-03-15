use bytes::{BufMut, BytesMut, Bytes};
use util::{BlockPosition, Result};
use util::bytes::{BinaryWriter, MutableBuffer, size_of_varint};

use util::Serialize;

use super::ConnectedPacket;

#[derive(Debug, Clone)]
pub struct NetworkChunkPublisherUpdate {
    pub position: BlockPosition,
    pub radius: u32,
}

impl ConnectedPacket for NetworkChunkPublisherUpdate {
    const ID: u32 = 0x79;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.position.x) +
        size_of_varint(self.position.y) +
        size_of_varint(self.position.z) +
        size_of_varint(self.radius) + 4
    }
}

impl Serialize for NetworkChunkPublisherUpdate {
    fn serialize(&self, buffer: &mut MutableBuffer) {
        buffer.write_block_pos(&self.position);
        buffer.write_var_u32(self.radius);

        // No saved chunks.
        buffer.write_u32_be(0);
    }
}
