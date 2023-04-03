use util::{Result, Serialize, Deserialize};
use util::bytes::{BinaryWrite, MutableBuffer, SharedBuffer, BinaryRead};

use crate::network::ConnectedPacket;

#[derive(Default, Debug, Clone)]
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

impl<'a> Deserialize<'a> for ContainerClose {
    fn deserialize(mut buffer: SharedBuffer<'a>) -> anyhow::Result<Self> {
        let window_id = buffer.read_u8()?;
        let server_initiated = buffer.read_bool()?;

        Ok(Self {
            window_id, server_initiated
        })
    }
}

impl Serialize for ContainerClose {
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_u8(self.window_id)?;
        buffer.write_bool(self.server_initiated)
    }
}