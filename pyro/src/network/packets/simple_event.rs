
use util::{bail, Deserialize, Serialize, Error, Result};
use crate::network::packets::ConnectedPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleEvent {
    CommandsEnabled = 1,
    CommandsDisabled,
    UnlockWorldTemplateSettings
}

impl TryFrom<i16> for SimpleEvent {
    type Error = Error;

    fn try_from(value: i16) -> Result<Self> {
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
    fn serialize(&self, buffer: &mut OwnedBuffer) {
        buffer.write_i16_le()(*self as i16);
    }
}

impl Deserialize for SimpleEvent {
    fn deserialize(mut buffer: SharedBuffer) -> Result<Self> {
        Self::try_from(buffer.get_i16_le())
    }
}