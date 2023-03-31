use util::bytes::{BinaryWrite, MutableBuffer, size_of_varint};
use util::Result;
use util::Serialize;

use crate::network::ConnectedPacket;

/// Opens a dialog showing details about a player's Xbox account.
#[derive(Debug, Clone)]
pub struct ShowProfile<'a> {
    /// XUID of the profile to display.
    pub xuid: &'a str,
}

impl<'a> ConnectedPacket for ShowProfile<'a> {
    const ID: u32 = 0x68;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.xuid.len() as u32) + self.xuid.len()
    }
}

impl<'a> Serialize for ShowProfile<'a> {
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_str(self.xuid)
    }
}
