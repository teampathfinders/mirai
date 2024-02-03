use util::{bail, Error, Result};
use util::{Deserialize, Serialize};
use util::{BinaryRead, BinaryWrite, VarInt, VarString};

use crate::bedrock::ConnectedPacket;

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
    pub const fn discriminant(&self) -> u8 {
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
    pub xuid: u64,
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
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u8(self.data.discriminant())?;
        writer.write_bool(self.needs_translation)?;

        match &self.data {
            TextData::Chat { source, message } |
            TextData::Whisper { source, message } |
            TextData::Announcement { source, message } => {
                writer.write_str(source)?;
                writer.write_str(message)?;
            },
            TextData::Raw { message } |
            TextData::Tip { message } |
            TextData::System { message }  |
            TextData::Object { message } |
            TextData::ObjectWhisper { message } |
            TextData::ObjectAnnouncement { message } => {
                writer.write_str(message)?;
            },
            TextData::Translation { message, parameters } |
            TextData::Popup { message, parameters } |
            TextData::JukeboxPopup { message, parameters } => {
                writer.write_str(message)?;
                writer.write_var_u32(parameters.len() as u32)?;

                for param in parameters {
                    writer.write_str(param)?;
                }
            }
        }

        writer.write_str(&self.xuid.to_string())?;
        writer.write_str(self.platform_chat_id)?;

        Ok(())
    }
}

impl<'a> Deserialize<'a> for TextMessage<'a> {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let message_type = reader.read_u8()?;
        let needs_translation = reader.read_bool()?;

        let data = match message_type {
            0 => TextData::Raw {
                message: reader.read_str()?
            },
            1 => TextData::Chat {
                source: reader.read_str()?,
                message: reader.read_str()?
            },
            2 => TextData::Translation {
                message: reader.read_str()?,
                parameters: {
                    let count = reader.read_var_u32()?;
                    let mut params = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        params.push(reader.read_str()?);
                    }

                    params
                }
            },
            3 => TextData::Popup {
                message: reader.read_str()?,
                parameters: {
                    let count = reader.read_var_u32()?;
                    let mut params = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        params.push(reader.read_str()?);
                    }

                    params
                }
            },
            4 => TextData::JukeboxPopup {
                message: reader.read_str()?,
                parameters: {
                    let count = reader.read_var_u32()?;
                    let mut params = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        params.push(reader.read_str()?);
                    }

                    params
                }
            },
            5 => TextData::Tip {
                message: reader.read_str()?
            },
            6 => TextData::System {
                message: reader.read_str()?
            },
            7 => TextData::Whisper {
                source: reader.read_str()?,
                message: reader.read_str()?
            },
            8 => TextData::Announcement {
                source: reader.read_str()?,
                message: reader.read_str()?
            },
            9 => TextData::ObjectWhisper {
                message: reader.read_str()?
            },
            10 => TextData::Object {
                message: reader.read_str()?
            },
            11 => TextData::ObjectAnnouncement {
                message: reader.read_str()?
            },
            _ => anyhow::bail!("Invalid message type")
        };

        let xuid = reader.read_str()?.parse()?;
        let platform_chat_id = reader.read_str()?;

        Ok(Self {
            data,
            needs_translation,
            xuid,
            platform_chat_id
        })
    }
}
