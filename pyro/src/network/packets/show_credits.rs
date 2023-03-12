use bytes::{BytesMut, Bytes};
use util::{bail, ReadExtensions, Error, Result, WriteExtensions, size_of_varint};

use util::{Deserialize, Serialize};

use super::ConnectedPacket;

/// Status of the credits display.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CreditsStatus {
    /// Start showing credits.
    Start,
    /// Stop showing credits.
    End,
}

impl TryFrom<i32> for CreditsStatus {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self> {
        Ok(match value {
            0 => Self::Start,
            1 => Self::End,
            _ => bail!(Malformed, "Invalid credits status {value}"),
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

impl ConnectedPacket for CreditsUpdate {
    const ID: u32 = 0x4b;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.runtime_id) + size_of_varint(self.status as i32)
    }
}

impl Serialize for CreditsUpdate {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_var_u64(self.runtime_id);
        buffer.put_var_i32(self.status as i32);
    }
}

impl Deserialize for CreditsUpdate {
    fn deserialize(mut buffer: Bytes) -> Result<Self> {
        let runtime_id = buffer.get_var_u64()?;
        let status = CreditsStatus::try_from(buffer.get_var_i32()?)?;

        Ok(Self { runtime_id, status })
    }
}
