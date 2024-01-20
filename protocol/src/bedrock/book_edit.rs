use util::{bail, Deserialize, Result};
use util::{BinaryRead, SharedBuffer};

use crate::bedrock::ConnectedPacket;

/// Sent when the client makes changes to a book.
/// The client sends this packet every time the client briefly stops typing,
/// not when the book is closed.
#[derive(Debug, Clone)]
pub struct BookEdit<'a> {
    /// Action to perform on the book.
    pub action: BookEditAction<'a>,
    /// Inventory slot that the book was in.
    pub inventory_slot: u8,
}

/// An action performed on a book.
#[derive(Debug, Clone)]
pub enum BookEditAction<'a> {
    ReplacePage {
        /// Page to be modified.
        page_number: u8,
        /// New text for the page.
        text: &'a str,
    },
    AddPage {
        /// Page to add.
        page_number: u8,
        /// Text to add to the new page.
        text: &'a str,
    },
    DeletePage {
        /// Page to delete.
        page_number: u8
    },
    SwapPages {
        /// First page.
        first_page: u8,
        /// Second page.
        second_page: u8,
    },
    Sign {
        /// Title of the book.
        title: &'a str,
        /// Author of the book.
        /// This isn't necessarily the client's username, it can be freely modified.
        author: &'a str,
        /// XUID of the client.
        xuid: &'a str,
    },
}

impl<'a> ConnectedPacket for BookEdit<'a> {
    const ID: u32 = 0x61;
}

impl<'a> Deserialize<'a> for BookEdit<'a> {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let action = reader.read_u8()?;
        let inventory_slot = reader.read_u8()?;

        Ok(Self {
            inventory_slot,
            action: match action {
                0 => BookEditAction::ReplacePage {
                    page_number: reader.read_u8()?,
                    text: reader.read_str()?,
                },
                1 => BookEditAction::AddPage {
                    page_number: reader.read_u8()?,
                    text: reader.read_str()?,
                },
                2 => BookEditAction::DeletePage {
                    page_number: reader.read_u8()?
                },
                3 => BookEditAction::SwapPages {
                    first_page: reader.read_u8()?,
                    second_page: reader.read_u8()?,
                },
                4 => BookEditAction::Sign {
                    title: reader.read_str()?,
                    author: reader.read_str()?,
                    xuid: reader.read_str()?,
                },
                _ => bail!(Malformed, "Invalid book edit action {action}")
            },
        })
    }
}