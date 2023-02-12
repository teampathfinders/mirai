use bytes::BytesMut;
use common::{VResult, WriteExtensions, VError, bail, ReadExtensions};

use crate::network::{Encodable, Decodable};

use super::GamePacket;

/// Status of the credits display.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CreditsStatus {
    /// Start showing credits.
    Start,
    /// Stop showing credits.
    End,
}

impl TryFrom<i32> for CreditsStatus {
    type Error = VError;

    fn try_from(value: i32) -> VResult<Self> {
        Ok(match value {
            0 => Self::Start,
            1 => Self::End,
            _ => bail!(BadPacket, "Invalid credits status {value}")
        })
    }
}

/// Displays the Minecraft credits to the client.
#[derive(Debug)]
pub struct CreditsUpdate {
    pub runtime_id: u64,
    /// Status update to apply.
    pub status: CreditsStatus,
}

impl GamePacket for CreditsUpdate {
    const ID: u32 = 0x4b;
}

impl Encodable for CreditsUpdate {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_var_u64(self.runtime_id);
        buffer.put_var_i32(self.status as i32);

        Ok(buffer)
    }
}

impl Decodable for CreditsUpdate {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let runtime_id = buffer.get_var_u64()?;
        let status = CreditsStatus::try_from(buffer.get_var_i32()?)?;

        Ok(Self {
            runtime_id,
            status
        })
    }
}