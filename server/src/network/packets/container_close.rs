use bytes::{BufMut, BytesMut};
use common::{Serialize, VResult, WriteExtensions};
use crate::network::packets::ConnectedPacket;

#[derive(Debug, Clone)]
pub struct ContainerClose {
    /// Equal to the window ID sent in the [`ContainerOpen`](super::ContainerOpen) packet.
    pub window_id: u8,
    /// Whether the server force-closed the container.
    pub server_initiated: bool
}

impl ConnectedPacket for ContainerClose {
    const ID: u32 = 0x2f;
}

impl Serialize for ContainerClose {
    fn serialize(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(2);

        buffer.put_u8(self.window_id);
        buffer.put_bool(self.server_initiated);

        Ok(buffer)
    }
}