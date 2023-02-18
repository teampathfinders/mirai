use bytes::{BufMut, BytesMut};
use common::{VResult, WriteExtensions, size_of_var};

use common::Encodable;

use super::GamePacket;

/// Displays a notification at the top of the screen.
#[derive(Debug, Clone)]
pub struct ToastRequest<'a> {
    /// Title of the notification.
    pub title: &'a str,
    /// Message displayed in the notification.
    pub message: &'a str,
}

impl GamePacket for ToastRequest<'_> {
    const ID: u32 = 0xba;
}

impl Encodable for ToastRequest<'_> {
    fn encode(&self) -> VResult<BytesMut> {
        let packet_size = 
            size_of_var(self.title.len() as u32) + self.title.len() +
            size_of_var(self.message.len() as u32) + self.message.len();

        let mut buffer = BytesMut::with_capacity(packet_size);

        buffer.put_string(self.title);
        buffer.put_string(self.message);

        Ok(buffer)
    }
}
