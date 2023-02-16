use bytes::{Buf, BufMut, BytesMut};
use common::{bail, Decodable, Encodable, VError, VResult};
use crate::network::packets::GamePacket;

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

impl GamePacket for SimpleEvent {
    const ID: u32 = 0x40;
}

impl Encodable for SimpleEvent {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(2);

        buffer.put_i16_le(self as i16);

        Ok(buffer)
    }
}

impl Decodable for SimpleEvent {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        Self::try_from(buffer.get_i16_le())
    }
}