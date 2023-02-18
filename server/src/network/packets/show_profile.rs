use bytes::BytesMut;
use common::{VResult, WriteExtensions, size_of_var};

use common::Encodable;

use super::GamePacket;

/// Opens a dialog showing details about a player's Xbox account.
#[derive(Debug, Clone)]
pub struct ShowProfile<'s> {
    /// XUID of the profile to display.
    pub xuid: &'s str,
}

impl GamePacket for ShowProfile<'_> {
    const ID: u32 = 0x68;
}

impl Encodable for ShowProfile<'_> {
    fn encode(&self) -> VResult<BytesMut> {
        let packet_size = size_of_var(self.xuid.len() as u32) + self.xuid.len();
        let mut buffer = BytesMut::with_capacity(packet_size);

        buffer.put_string(self.xuid);

        Ok(buffer)
    }
}
