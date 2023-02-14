use bytes::{BufMut, BytesMut};
use common::{VResult, WriteExtensions};

use common::Encodable;

use super::GamePacket;

/// Displays a notification at the top of the screen.
#[derive(Debug)]
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
        let mut buffer = BytesMut::new();

        buffer.put_string(self.title);
        buffer.put_string(self.message);

        Ok(buffer)
    }
}
