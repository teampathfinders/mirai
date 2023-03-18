
use util::{bail,  Error, Result};

use util::{Deserialize, Serialize};
use util::bytes::{BinaryReader, BinaryWrite, MutableBuffer, SharedBuffer, VarInt, VarString};

use crate::ConnectedPacket;

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
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
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
            _ => bail!(Malformed, "Invalid text message type"),
        })
    }
}

/// Displays text messages.
#[derive(Debug, Clone)]
pub struct TextMessage<'a> {
    /// Type of the message.
    pub message_type: MessageType,
    /// Whether the message requires translation.
    /// This translation can be performed with messages containing %.
    pub needs_translation: bool,
    /// Source of the message
    pub source_name: &'a str,
    /// Message to display.
    pub message: &'a str,
    /// A list of parameters that are filled into the message. These parameters are only
    /// written if the type of the packet is [`Translation`](MessageType::Translation),
    /// [`Tip`](MessageType::Tip), [`Popup`](MessageType::Popup) or [`JukeboxPopup`](MessageType::JukeboxPopup).
    pub parameters: Vec<&'a str>,
    /// XUID of the sender.
    /// This is only set if the type of the packet is [`Chat`](MessageType::Chat).
    pub xuid: &'a str,
    /// Identifier set for specific platforms that determines whether clients are able to chat with each other.
    pub platform_chat_id: &'a str,
}

impl<'a> ConnectedPacket for TextMessage<'a> {
    const ID: u32 = 0x09;

    fn serialized_size(&self) -> usize {
        1 + 1 + match self.message_type {
            MessageType::Chat
            | MessageType::Whisper
            | MessageType::Announcement => {
                self.source_name.var_len() +
                self.message.var_len()
            }
            MessageType::Raw
            | MessageType::Tip
            | MessageType::System
            | MessageType::Object
            | MessageType::ObjectWhisper
            | MessageType::ObjectAnnouncement => {
                self.message.var_len()
            }
            MessageType::Translation
            | MessageType::Popup
            | MessageType::JukeboxPopup => {
                self.message.var_len() +
                (self.parameters.len() as u32).var_len() +
                self.parameters.iter().fold(
                    0, |acc, p| acc + p.var_len()
                )
            }
        } + self.xuid.var_len() + self.platform_chat_id.var_len()
    }
}

impl<'a> Serialize for TextMessage<'a> {
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_u8(self.message_type as u8);
        buffer.write_bool(self.needs_translation);

        match self.message_type {
            MessageType::Chat
            | MessageType::Whisper
            | MessageType::Announcement => {
                buffer.write_str(&self.source_name);
                buffer.write_str(&self.message);
            }
            MessageType::Raw
            | MessageType::Tip
            | MessageType::System
            | MessageType::Object
            | MessageType::ObjectWhisper
            | MessageType::ObjectAnnouncement => {
                buffer.write_str(&self.message);
            }
            MessageType::Translation
            | MessageType::Popup
            | MessageType::JukeboxPopup => {
                buffer.write_str(&self.message);

                buffer.write_var_u32(self.parameters.len() as u32);
                for param in &self.parameters {
                    buffer.write_str(param);
                }
            }
        }

        buffer.write_str(&self.xuid);
        buffer.write_str(&self.platform_chat_id);

        Ok(())
    }
}

impl<'a> Deserialize<'a> for TextMessage<'a> {
    fn deserialize(mut buffer: SharedBuffer<'a>) -> Result<Self> {
        let message_type = MessageType::try_from(buffer.read_u8()?)?;
        let needs_translation = buffer.read_bool()?;
        let message;
        let mut source_name = "";
        let mut parameters = Vec::new();

        match message_type {
            MessageType::Chat
            | MessageType::Whisper
            | MessageType::Announcement => {
                source_name = buffer.read_str()?;
                message = buffer.read_str()?;
            }
            MessageType::Raw
            | MessageType::Tip
            | MessageType::System
            | MessageType::Object
            | MessageType::ObjectWhisper
            | MessageType::ObjectAnnouncement => {
                message = buffer.read_str()?;
            }
            MessageType::Translation
            | MessageType::Popup
            | MessageType::JukeboxPopup => {
                message = buffer.read_str()?;

                let count = buffer.read_var_u32()?;
                parameters.reserve(count as usize);

                for _ in 0..count {
                    parameters.push(buffer.read_str()?);
                }
            }
        }

        let xuid = buffer.read_str()?;
        let platform_chat_id = buffer.read_str()?;

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
