use bytes::BytesMut;
use common::{VResult, WriteExtensions, size_of_var};

use common::Serialize;

use super::ConnectedPacket;

/// Opens a dialog showing details about a player's Xbox account.
#[derive(Debug, Clone)]
pub struct ShowProfile<'s> {
    /// XUID of the profile to display.
    pub xuid: &'s str,
}

impl ConnectedPacket for ShowProfile<'_> {
    const ID: u32 = 0x68;
}

impl Serialize for ShowProfile<'_> {
    fn serialize(&self) -> VResult<BytesMut> {
        let packet_size = size_of_var(self.xuid.len() as u32) + self.xuid.len();
        let mut buffer = BytesMut::with_capacity(packet_size);

        buffer.put_string(self.xuid);

        Ok(buffer)
    }
}
