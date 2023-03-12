use bytes::{Buf, BufMut, BytesMut, Bytes};
use util::{bail, Deserialize, Serialize, ReadExtensions, Error, Result};
use crate::network::packets::ConnectedPacket;

/// Sent when the client makes changes to a book.
/// The client sends this packet every time the client briefly stops typing,
/// not when the book is closed.
#[derive(Debug, Clone)]
pub struct BookEdit {
    /// Action to perform on the book.
    pub action: BookEditAction,
    /// Inventory slot that the book was in.
    pub inventory_slot: u8,
}

/// An action performed on a book.
#[derive(Debug, Clone)]
pub enum BookEditAction {
    ReplacePage {
        /// Page to be modified.
        page_number: u8,
        /// New text for the page.
        text: String
    },
    AddPage {
        /// Page to add.
        page_number: u8,
        /// Text to add to the new page.
        text: String
    },
    DeletePage {
        /// Page to delete.
        page_number: u8
    },
    SwapPages {
        /// First page.
        first_page: u8,
        /// Second page.
        second_page: u8
    },
    Sign {
        /// Title of the book.
        title: String,
        /// Author of the book.
        /// This isn't necessarily the client's username, it can be freely modified.
        author: String,
        /// XUID of the client.
        xuid: String
    }
}

impl ConnectedPacket for BookEdit {
    const ID: u32 = 0x61;
}

impl Deserialize for BookEdit {
    fn deserialize(mut buffer: Bytes) -> Result<Self>{
        let action = buffer.get_u8();;
        let inventory_slot = buffer.get_u8();

        Ok(Self {
            inventory_slot,
            action: match action {
                0 => BookEditAction::ReplacePage {
                    page_number: buffer.get_u8(),
                    text: buffer.get_string()?
                },
                1 => BookEditAction::AddPage {
                    page_number: buffer.get_u8(),
                    text: buffer.get_string()?
                },
                2 => BookEditAction::DeletePage {
                    page_number: buffer.get_u8()
                },
                3 => BookEditAction::SwapPages {
                    first_page: buffer.get_u8(),
                    second_page: buffer.get_u8()
                },
                4 => BookEditAction::Sign {
                    title: buffer.get_string()?,
                    author: buffer.get_string()?,
                    xuid: buffer.get_string()?
                },
                _ => bail!(Malformed, "Invalid book edit action {action}")
            }
        })
    }
}