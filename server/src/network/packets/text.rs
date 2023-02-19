use bytes::{Buf, BufMut, BytesMut};
use common::{bail, ReadExtensions, VError, VResult, WriteExtensions};

use common::{Deserialize, Serialize};

use super::ConnectedPacket;

/// Type of message.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MessageType {
    Raw,
    /// A chat message.
    Chat,
    Translation,
    Popup,
    JukeboxPopup,
    Tip,
    /// Used for messages such as a skin change or a player that joined.
    /// This type indicates a message from the server itself.
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

/// Displays text messages.
#[derive(Debug, Clone)]
pub struct TextMessage {
    /// Type of the message.
    pub message_type: MessageType,
    /// Whether the message requires translation.
    /// This translation can be performed with messages containing %.
    pub needs_translation: bool,
    /// Source of the message
    pub source_name: String,
    /// Message to display.
    pub message: String,
    /// A list of parameters that are filled into the message. These parameters are only
    /// written if the type of the packet is [`Translation`](MessageType::Translation),
    /// [`Tip`](MessageType::Tip), [`Popup`](MessageType::Popup) or [`JukeboxPopup`](MessageType::JukeboxPopup).
    pub parameters: Vec<String>,
    /// XUID of the sender.
    /// This is only set if the type of the packet is [`Chat`](MessageType::Chat).
    pub xuid: String,
    /// Identifier set for specific platforms that determines whether clients are able to chat with each other.
    pub platform_chat_id: String,
}

impl ConnectedPacket for TextMessage {
    const ID: u32 = 0x09;
}

impl Serialize for TextMessage {
    fn serialize(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_u8(self.message_type as u8);
        buffer.put_bool(self.needs_translation);

        match self.message_type {
            MessageType::Chat
            | MessageType::Whisper
            | MessageType::Announcement => {
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
            MessageType::Translation
            | MessageType::Popup
            | MessageType::JukeboxPopup => {
                buffer.put_string(&self.message);

                buffer.put_var_u32(self.parameters.len() as u32);
                for param in &self.parameters {
                    buffer.put_string(param);
                }
            }
        }

        buffer.put_string(&self.xuid);
        buffer.put_string(&self.platform_chat_id);

        Ok(buffer)
    }
}

impl Deserialize for TextMessage {
    fn deserialize(mut buffer: BytesMut) -> VResult<Self> {
        let message_type = MessageType::try_from(buffer.get_u8())?;
        let needs_translation = buffer.get_bool();
        let message;
        let mut source_name = String::new();
        let mut parameters = Vec::new();

        match message_type {
            MessageType::Chat
            | MessageType::Whisper
            | MessageType::Announcement => {
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
            MessageType::Translation
            | MessageType::Popup
            | MessageType::JukeboxPopup => {
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
