use util::{Serialize, Deserialize};
use util::{BinaryWrite, BinaryRead};

use crate::bedrock::ConnectedPacket;

/// A container has been closed.
#[derive(Default, Debug, Clone)]
pub struct ContainerClose {
    /// Equal to the window ID sent in the [`ContainerOpen`](crate::bedrock::ContainerOpen) packet.
    pub window_id: u8,
    pub container_type: u8,
    /// Whether the server force-closed the container.
    pub server_initiated: bool,
}

impl ConnectedPacket for ContainerClose {
    const ID: u32 = 0x2f;
}

impl<'a> Deserialize<'a> for ContainerClose {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let window_id = reader.read_u8()?;
        let container_type = reader.read_u8()?;
        let server_initiated = reader.read_bool()?;

        Ok(Self {
            window_id, container_type, server_initiated
        })
    }
}

impl Serialize for ContainerClose {
    fn size_hint(&self) -> Option<usize> {
        Some(3)
    }

    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u8(self.window_id)?;
        writer.write_u8(self.container_type)?;
        writer.write_bool(self.server_initiated)
    }
}