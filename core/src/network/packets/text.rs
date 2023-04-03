use util::{bail, Error, Result};
use util::{Deserialize, Serialize};
use util::bytes::{BinaryRead, BinaryWrite, MutableBuffer, SharedBuffer, VarInt, VarString};

use crate::network::ConnectedPacket;

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextData<'a> {
    Raw {
        message: &'a str
    } = 0,
    Chat {
        source: &'a str,
        message: &'a str
    } = 1,
    Translation {
        message: &'a str,
        parameters: Vec<&'a str>
    } = 2,
    Popup {
        message: &'a str,
        parameters: Vec<&'a str>
    } = 3,
    JukeboxPopup {
        message: &'a str,
        parameters: Vec<&'a str>
    } = 4,
    Tip {
        message: &'a str
    } = 5,
    System {
        message: &'a str
    } = 6,
    Whisper {
        source: &'a str,
        message: &'a str
    } = 7,
    Announcement {
        source: &'a str,
        message: &'a str
    } = 8,
    ObjectWhisper {
        message: &'a str
    } = 9,
    Object {
        message: &'a str
    } = 10,
    ObjectAnnouncement {
        message: &'a str
    } = 11
}

impl<'a> TextData<'a> {
    #[inline]
    pub fn discriminant(&self) -> u8 {
        // SAFETY: This is safe due to the `repr(u8)` attribute on the enum.
        // This means the enum is prefixed with a u8 tag.
        unsafe { *(self as *const Self as *const u8) }
    }
}

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
    type Error = anyhow::Error;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextMessage<'a> {
    pub data: TextData<'a>,
    pub needs_translation: bool,
    pub xuid: &'a str,
    pub platform_chat_id: &'a str
}

impl<'a> ConnectedPacket for TextMessage<'a> {
    const ID: u32 = 0x09;

    fn serialized_size(&self) -> usize {
        todo!();
    }
}

impl<'a> Serialize for TextMessage<'a> {
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_u8(self.data.discriminant())?;
        buffer.write_bool(self.needs_translation)?;

        match &self.data {
            TextData::Chat { source, message } |
            TextData::Whisper { source, message } |
            TextData::Announcement { source, message } => {
                buffer.write_str(source)?;
                buffer.write_str(message)?;
            },
            TextData::Raw { message } |
            TextData::Tip { message } |
            TextData::System { message }  |
            TextData::Object { message } |
            TextData::ObjectWhisper { message } |
            TextData::ObjectAnnouncement { message } => {
                buffer.write_str(message)?;
            },
            TextData::Translation { message, parameters } |
            TextData::Popup { message, parameters } |
            TextData::JukeboxPopup { message, parameters } => {
                buffer.write_str(message)?;
                buffer.write_var_u32(parameters.len() as u32)?;

                for param in parameters {
                    buffer.write_str(param)?;
                }
            }
        }

        buffer.write_str(self.xuid)?;
        buffer.write_str(self.platform_chat_id)?;

        Ok(())
    }
}

impl<'a> Deserialize<'a> for TextMessage<'a> {
    fn deserialize(mut buffer: SharedBuffer<'a>) -> anyhow::Result<Self> {
        todo!();
    }
}
