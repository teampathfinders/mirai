use util::{bail, Error, Result};
use util::{Deserialize, Serialize};
use util::bytes::{BinaryRead, BinaryWrite, MutableBuffer, SharedBuffer, size_of_varint};

use crate::network::ConnectedPacket;

/// Status of the credits display.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CreditsStatus {
    /// Start showing credits.
    Start,
    /// Stop showing credits.
    End,
}

impl TryFrom<i32> for CreditsStatus {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> anyhow::Result<Self> {
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
    fn serialize<W>(&self, buffer: W) -> anyhow::Result<()> where W: BinaryWrite {
        buffer.write_var_u64(self.runtime_id)?;
        buffer.write_var_i32(self.status as i32)
    }
}

impl Deserialize<'_> for CreditsUpdate {
    fn deserialize(mut buffer: SharedBuffer) -> anyhow::Result<Self> {
        let runtime_id = buffer.read_var_u64()?;
        let status = CreditsStatus::try_from(buffer.read_var_i32()?)?;

        Ok(Self { runtime_id, status })
    }
}
