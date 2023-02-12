use bytes::{BytesMut, BufMut};
use common::{VResult, WriteExtensions};

use crate::network::Encodable;

use super::GamePacket;

/// Displays a notification at the top of the screen.
#[derive(Debug)]
pub struct ToastRequest {
    /// Title of the notification.
    pub title: String,
    /// Message displayed in the notification.
    pub message: String
}

impl GamePacket for ToastRequest {
    const ID: u32 = 0xba;
}

impl Encodable for ToastRequest {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_string(&self.title);
        buffer.put_string(&self.message);

        Ok(buffer)
    }
}