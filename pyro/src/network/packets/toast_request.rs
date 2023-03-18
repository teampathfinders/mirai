
use util::{Result};
use util::bytes::{BinaryWrite, MutableBuffer, size_of_varint};

use util::Serialize;

use crate::ConnectedPacket;

/// Displays a notification at the top of the screen.
#[derive(Debug, Clone)]
pub struct ToastRequest<'a> {
    /// Title of the notification.
    pub title: &'a str,
    /// Message displayed in the notification.
    pub message: &'a str,
}

impl ConnectedPacket for ToastRequest<'_> {
    const ID: u32 = 0xba;
    
    fn serialized_size(&self) -> usize {
        size_of_varint(self.title.len() as u32) + self.title.len() +
        size_of_varint(self.message.len() as u32) + self.message.len()       
    }
}

impl Serialize for ToastRequest<'_> {
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_str(self.title);
        buffer.write_str(self.message);

        Ok(())
    }
}
