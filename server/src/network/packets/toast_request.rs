use bytes::{BufMut, BytesMut};
use common::{VResult, WriteExtensions, size_of_var};

use common::Serialize;

use super::ConnectedPacket;

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
}

impl Serialize for ToastRequest<'_> {
    fn serialize(&self) -> VResult<BytesMut> {
        let packet_size = 
            size_of_var(self.title.len() as u32) + self.title.len() +
            size_of_var(self.message.len() as u32) + self.message.len();

        let mut buffer = BytesMut::with_capacity(packet_size);

        buffer.put_string(self.title);
        buffer.put_string(self.message);

        Ok(buffer)
    }
}
