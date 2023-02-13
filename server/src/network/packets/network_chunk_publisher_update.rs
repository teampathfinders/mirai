use bytes::{BufMut, BytesMut};
use common::{BlockPosition, VResult, WriteExtensions};

use common::Encodable;

use super::GamePacket;

#[derive(Debug)]
pub struct NetworkChunkPublisherUpdate {
    pub position: BlockPosition,
    pub radius: u32,
}

impl GamePacket for NetworkChunkPublisherUpdate {
    const ID: u32 = 0x79;
}

impl Encodable for NetworkChunkPublisherUpdate {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_block_pos(&self.position);
        buffer.put_var_u32(self.radius);

        // No saved chunks.
        buffer.put_u32(0);

        Ok(buffer)
    }
}
