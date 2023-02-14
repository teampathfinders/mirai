use bytes::BytesMut;
use common::{VResult, WriteExtensions};

use common::Encodable;

use super::GamePacket;

/// Opens a dialog showing details about a player's Xbox account.
#[derive(Debug)]
pub struct ShowProfile<'s> {
    /// XUID of the profile to display.
    pub xuid: &'s str,
}

impl GamePacket for ShowProfile<'_> {
    const ID: u32 = 0x68;
}

impl Encodable for ShowProfile<'_> {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(1 + self.xuid.len());

        buffer.put_string(self.xuid);

        Ok(buffer)
    }
}
