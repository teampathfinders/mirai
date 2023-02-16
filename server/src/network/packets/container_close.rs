use bytes::{BufMut, BytesMut};
use common::{Encodable, VResult, WriteExtensions};
use crate::network::packets::GamePacket;

#[derive(Debug, Clone)]
pub struct ContainerClose {
    /// Equal to the window ID sent in the [`ContainerOpen`](super::ContainerOpen) packet.
    pub window_id: u8,
    /// Whether the server force-closed the container.
    pub server_initiated: bool
}

impl GamePacket for ContainerClose {
    const ID: u32 = 0x2f;
}

impl Encodable for ContainerClose {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(2);

        buffer.put_u8(self.window_id);
        buffer.put_bool(self.server_initiated);

        Ok(buffer)
    }
}