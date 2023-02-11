use bytes::{Buf, BufMut, BytesMut};
use common::{bail, ReadExtensions, VError, VResult, WriteExtensions};

use crate::network::{Decodable, Encodable};

use super::GamePacket;

#[derive(Debug, Copy, Clone)]
pub enum MessageType {
    Raw,
    Chat,
    Translation,
    Popup,
    JukeboxPopup,
    Tip,
    System,
    Whisper,
    Announcement,
    ObjectWhisper,
    Object,
    ObjectAnnouncement,
}

impl TryFrom<u8> for MessageType {
    type Error = VError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Raw,
            1 => Self::Chat,
            2 => Self::Translation,
            3 => Self::Popup,
            4 => Self::JukeboxPopup,
            5 => Self::Tip,
            6 => Self::System,
            7 => Self::Whisper,
            8 => Self::Announcement,
            9 => Self::ObjectWhisper,
            10 => Self::Object,
            11 => Self::ObjectAnnouncement,
            _ => bail!(BadPacket, "Invalid text message type"),
        })
    }
}

#[derive(Debug)]
pub struct TextMessage {
    /// Type of the message.
    pub message_type: MessageType,
    pub needs_translation: bool,
    /// Source of the message
    pub source_name: String,
    pub message: String,
    pub parameters: Vec<String>,
    pub xuid: String,
    pub platform_chat_id: String,
}

impl GamePacket for TextMessage {
    const ID: u32 = 0x09;
}

impl Encodable for TextMessage {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_u8(self.message_type as u8);
        buffer.put_bool(self.needs_translation);

        match self.message_type {
            MessageType::Chat | MessageType::Whisper | MessageType::Announcement => {
                buffer.put_string(&self.source_name);
                buffer.put_string(&self.message);
            }
            MessageType::Raw
            | MessageType::Tip
            | MessageType::System
            | MessageType::Object
            | MessageType::ObjectWhisper
            | MessageType::ObjectAnnouncement => {
                buffer.put_string(&self.message);
            }
            MessageType::Translation | MessageType::Popup | MessageType::JukeboxPopup => {
                buffer.put_string(&self.message);

                buffer.put_var_u32(self.parameters.len() as u32);
                for param in &self.parameters {
                    buffer.put_string(&param);
                }
            }
        }

        buffer.put_string(&self.xuid);
        buffer.put_string(&self.platform_chat_id);

        Ok(buffer)
    }
}

impl Decodable for TextMessage {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let message_type = MessageType::try_from(buffer.get_u8())?;
        let needs_translation = buffer.get_bool();
        let message;
        let mut source_name = String::new();
        let mut parameters = Vec::new();

        match message_type {
            MessageType::Chat | MessageType::Whisper | MessageType::Announcement => {
                source_name = buffer.get_string()?;
                message = buffer.get_string()?;
            }
            MessageType::Raw
            | MessageType::Tip
            | MessageType::System
            | MessageType::Object
            | MessageType::ObjectWhisper
            | MessageType::ObjectAnnouncement => {
                message = buffer.get_string()?;
            }
            MessageType::Translation | MessageType::Popup | MessageType::JukeboxPopup => {
                message = buffer.get_string()?;

                let count = buffer.get_var_u32()?;
                parameters.reserve(count as usize);

                for _ in 0..count {
                    parameters.push(buffer.get_string()?);
                }
            }
        }

        let xuid = buffer.get_string()?;
        let platform_chat_id = buffer.get_string()?;

        Ok(Self {
            message_type,
            needs_translation,
            source_name,
            message,
            parameters,
            xuid,
            platform_chat_id,
        })
    }
}
