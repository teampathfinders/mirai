use util::{bail, Deserialize, Error, Result, Serialize};
use util::{BinaryRead, BinaryWrite, MutableBuffer, SharedBuffer};

use crate::bedrock::ConnectedPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleEvent {
    CommandsEnabled = 1,
    CommandsDisabled,
    UnlockWorldTemplateSettings,
}

impl TryFrom<i16> for SimpleEvent {
    type Error = anyhow::Error;

    fn try_from(value: i16) -> anyhow::Result<Self> {
        Ok(match value {
            1 => Self::CommandsEnabled,
            2 => Self::CommandsDisabled,
            3 => Self::UnlockWorldTemplateSettings,
            _ => bail!(Malformed, "Invalid simple event type {value}")
        })
    }
}

impl ConnectedPacket for SimpleEvent {
    const ID: u32 = 0x40;

    fn serialized_size(&self) -> usize {
        2
    }
}

impl Serialize for SimpleEvent {
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_i16_le(*self as i16)
    }
}

impl<'a> Deserialize<'a> for SimpleEvent {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        Self::try_from(reader.read_i16_le()?)
    }
}