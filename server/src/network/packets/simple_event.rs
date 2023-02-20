use bytes::{Buf, BufMut, BytesMut, Bytes};
use common::{bail, Deserialize, Serialize, VError, VResult};
use crate::network::packets::ConnectedPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleEvent {
    CommandsEnabled = 1,
    CommandsDisabled,
    UnlockWorldTemplateSettings
}

impl TryFrom<i16> for SimpleEvent {
    type Error = VError;

    fn try_from(value: i16) -> VResult<Self> {
        Ok(match value {
            1 => Self::CommandsEnabled,
            2 => Self::CommandsDisabled,
            3 => Self::UnlockWorldTemplateSettings,
            _ => bail!(BadPacket, "Invalid simple event type {value}")
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
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_i16_le(*self as i16);
    }
}

impl Deserialize for SimpleEvent {
    fn deserialize(mut buffer: Bytes) -> VResult<Self> {
        Self::try_from(buffer.get_i16_le())
    }
}