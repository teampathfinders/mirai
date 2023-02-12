use bytes::BytesMut;
use common::{VResult, WriteExtensions};

use crate::network::Encodable;

use super::GamePacket;

/// Opens a dialog showing details about a player's Xbox account.
#[derive(Debug)]
pub struct ShowProfile {
    /// XUID of the profile to display.
    pub xuid: String,
}

impl GamePacket for ShowProfile {
    const ID: u32 = 0x68;
}

impl Encodable for ShowProfile {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(1 + self.xuid.len());

        buffer.put_string(&self.xuid);

        Ok(buffer)
    }
}
