use bytes::BytesMut;
use common::{bail, ReadExtensions, VError, VResult, WriteExtensions, size_of_var};

use common::{Deserialize, Serialize};

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
            _ => bail!(BadPacket, "Invalid credits status {value}"),
        })
    }
}

/// Displays the Minecraft credits to the client.
#[derive(Debug, Clone)]
pub struct CreditsUpdate {
    pub runtime_id: u64,
    /// Status update to apply.
    pub status: CreditsStatus,
}

impl GamePacket for CreditsUpdate {
    const ID: u32 = 0x4b;
}

impl Serialize for CreditsUpdate {
    fn serialize(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(
            size_of_var(self.runtime_id) + size_of_var(self.status as i32)
        );

        buffer.put_var_u64(self.runtime_id);
        buffer.put_var_i32(self.status as i32);

        Ok(buffer)
    }
}

impl Deserialize for CreditsUpdate {
    fn deserialize(mut buffer: BytesMut) -> VResult<Self> {
        let runtime_id = buffer.get_var_u64()?;
        let status = CreditsStatus::try_from(buffer.get_var_i32()?)?;

        Ok(Self { runtime_id, status })
    }
}
