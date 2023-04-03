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
