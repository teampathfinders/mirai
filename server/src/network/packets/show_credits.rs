use bytes::BytesMut;
use common::{VResult, WriteExtensions};

use crate::network::Encodable;

use super::GamePacket;

/// Status of the credits display.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CreditStatus {
    /// Start showing credits.
    Start,
    /// Stop showing credits.
    End,
}

/// Displays the Minecraft credits to the client.
#[derive(Debug)]
pub struct ShowCredits {
    pub runtime_id: u64,
    /// Status update to apply.
    pub status: CreditStatus,
}

impl GamePacket for ShowCredits {
    const ID: u32 = 0x4b;
}

impl Encodable for ShowCredits {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_var_u64(self.runtime_id);
        buffer.put_var_i32(self.status as i32);

        Ok(buffer)
    }
}
