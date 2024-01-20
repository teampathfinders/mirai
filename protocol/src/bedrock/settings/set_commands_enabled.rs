use util::{BinaryWrite, MutableBuffer};
use util::Result;
use util::Serialize;

use crate::bedrock::ConnectedPacket;

/// Enables or disables the usage of commands.
///
/// If commands are disabled, the client will prevent itself from even sending any.
#[derive(Debug, Clone)]
pub struct SetCommandsEnabled {
    /// Whether commands are enabled.
    pub enabled: bool,
}

impl ConnectedPacket for SetCommandsEnabled {
    const ID: u32 = 0x3b;

    fn serialized_size(&self) -> usize {
        1
    }
}

impl Serialize for SetCommandsEnabled {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_bool(self.enabled)
    }
}
