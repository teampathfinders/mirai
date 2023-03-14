use bytes::{BytesMut, Bytes};
use util::{Result, size_of_varint};

use util::Serialize;

use super::ConnectedPacket;

/// Opens a dialog showing details about a player's Xbox account.
#[derive(Debug, Clone)]
pub struct ShowProfile<'s> {
    /// XUID of the profile to display.
    pub xuid: &'s str,
}

impl ConnectedPacket for ShowProfile<'_> {
    const ID: u32 = 0x68;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.xuid.len() as u32) + self.xuid.len()
    }
}

impl Serialize for ShowProfile<'_> {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_string(self.xuid);
    }
}
