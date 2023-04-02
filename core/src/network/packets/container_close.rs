use util::{Result, Serialize};
use util::bytes::{BinaryWrite, MutableBuffer};

use crate::network::ConnectedPacket;

#[derive(Debug, Clone)]
pub struct ContainerClose {
    /// Equal to the window ID sent in the [`ContainerOpen`](crate::ContainerOpen) packet.
    pub window_id: u8,
    /// Whether the server force-closed the container.
    pub server_initiated: bool,
}

impl ConnectedPacket for ContainerClose {
    const ID: u32 = 0x2f;

    fn serialized_size(&self) -> usize {
        2
    }
}

impl Serialize for ContainerClose {
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_u8(self.window_id)?;
        buffer.write_bool(self.server_initiated)
    }
}