use util::{bail, Error, Result};
use util::{Deserialize, Serialize};
use util::bytes::{BinaryRead, BinaryWrite, MutableBuffer, SharedBuffer, VarInt, VarString};

use crate::network::ConnectedPacket;

/// Text message data.
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextData<'a> {
    /// A simple message without any extra information.
    Raw {
        /// Message to display.
        message: &'a str
    } = 0,
    /// A player chat message.
    Chat {
        /// Player who sent the message.
        source: &'a str,
        /// Message to display.
        message: &'a str
    } = 1,
    /// A message that should be translated.
    Translation {
        /// Message to display.
        message: &'a str,
        /// Extra information used in translations.
        parameters: Vec<&'a str>
    } = 2,
    /// A popup.
    Popup {
        /// Message to display.
        message: &'a str,
        /// Extra information used in the popup.
        parameters: Vec<&'a str>
    } = 3,
    /// Popup shown when a disc is played.
    JukeboxPopup {
        /// Message to display.
        message: &'a str,
        /// Extra information used in the popup.
        parameters: Vec<&'a str>
    } = 4,
    /// A tip.
    Tip {
        /// Message to display.
        message: &'a str
    } = 5,
    /// Used for server messages such as a changed skin or joining/leaving player.
    System {
        /// Message to display.
        message: &'a str
    } = 6,
    /// Player to player whispering.
    Whisper {
        /// Who sent the whisper (usually a player).
        source: &'a str,
        /// Message to display.
        message: &'a str
    } = 7,
    /// An announcement.
    Announcement {
        /// Source of this message (usually a player).
        source: &'a str,
        /// Message to display.
        message: &'a str
    } = 8,
    /// Whispers from an object such as a command block.
    ObjectWhisper {
        /// Message to display.
        message: &'a str
    } = 9,
    /// Message from an object such as a command block.
    Object {
        /// Message to display.
        message: &'a str
    } = 10,
    /// Announcement from an object such as a command block.
    ObjectAnnouncement {
        /// Message to display.
        message: &'a str
    } = 11
}

impl<'a> TextData<'a> {
    /// Returns the enum discriminant of `self`.
    #[inline]
    pub fn discriminant(&self) -> u8 {
        // SAFETY: This is safe due to the `repr(u8)` attribute on the enum.
        // This means the enum is prefixed with a u8 tag.
        unsafe { *(self as *const Self as *const u8) }
    }
}

/// Displays messages in chat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextMessage<'a> {
    /// Data contained in the message.
    pub data: TextData<'a>,
    /// Whether strings containing `%` should be translated by the client.
    pub needs_translation: bool,
    /// XUID of the source.
    pub xuid: &'a str,
    /// Platform chat ID of the source.
    pub platform_chat_id: &'a str
}

impl<'a> ConnectedPacket for TextMessage<'a> {
    const ID: u32 = 0x09;

    fn serialized_size(&self) -> usize {
        1 + 1 + self.xuid.var_len() + self.platform_chat_id.var_len() + match &self.data {
            TextData::Chat { source, message } |
            TextData::Whisper { source, message } |
            TextData::Announcement { source, message } => {
                source.var_len() + message.var_len()
            },
            TextData::Raw { message } |
            TextData::Tip { message } |
            TextData::System { message }  |
            TextData::Object { message } |
            TextData::ObjectWhisper { message } |
            TextData::ObjectAnnouncement { message } => {
                message.var_len()
            },
            TextData::Translation { message, parameters } |
            TextData::Popup { message, parameters } |
            TextData::JukeboxPopup { message, parameters } => {
                message.var_len()
                    + (parameters.len() as u32).var_len()
                    + parameters.iter().fold(0, |acc, p| acc + p.var_len())
            }
        }
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
        let message_type = buffer.read_u8()?;
        let needs_translation = buffer.read_bool()?;

        let data = match message_type {
            0 => TextData::Raw {
                message: buffer.read_str()?
            },
            1 => TextData::Chat {
                source: buffer.read_str()?,
                message: buffer.read_str()?
            },
            2 => TextData::Translation {
                message: buffer.read_str()?,
                parameters: {
                    let count = buffer.read_var_u32()?;
                    let mut params = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        params.push(buffer.read_str()?);
                    }

                    params
                }
            },
            3 => TextData::Popup {
                message: buffer.read_str()?,
                parameters: {
                    let count = buffer.read_var_u32()?;
                    let mut params = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        params.push(buffer.read_str()?);
                    }

                    params
                }
            },
            4 => TextData::JukeboxPopup {
                message: buffer.read_str()?,
                parameters: {
                    let count = buffer.read_var_u32()?;
                    let mut params = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        params.push(buffer.read_str()?);
                    }

                    params
                }
            },
            5 => TextData::Tip {
                message: buffer.read_str()?
            },
            6 => TextData::System {
                message: buffer.read_str()?
            },
            7 => TextData::Whisper {
                source: buffer.read_str()?,
                message: buffer.read_str()?
            },
            8 => TextData::Announcement {
                source: buffer.read_str()?,
                message: buffer.read_str()?
            },
            9 => TextData::ObjectWhisper {
                message: buffer.read_str()?
            },
            10 => TextData::Object {
                message: buffer.read_str()?
            },
            11 => TextData::ObjectAnnouncement {
                message: buffer.read_str()?
            },
            _ => anyhow::bail!("Invalid message type")
        };

        let xuid = buffer.read_str()?;
        let platform_chat_id = buffer.read_str()?;

        Ok(Self {
            data,
            needs_translation,
            xuid,
            platform_chat_id
        })
    }
}
