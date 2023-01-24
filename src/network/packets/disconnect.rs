use bytes::{BufMut, BytesMut};

use crate::error::VexResult;
use crate::network::packets::GamePacket;
use crate::network::traits::Encodable;
use crate::util::WriteExtensions;

/// Sent by the server to disconnect a client.
#[derive(Debug)]
pub struct Disconnect {
    /// Whether to immediately send the client to the main menu.
    pub hide_disconnect_screen: bool,
    /// Message to display to the client
    pub kick_message: String,
}

impl GamePacket for Disconnect {
    const ID: u32 = 0x05;
}

impl Encodable for Disconnect {
    fn encode(&self) -> VexResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(1 + 4 + self.kick_message.len());

        buffer.put_bool(self.hide_disconnect_screen);
        buffer.put_string(&self.kick_message);

        Ok(buffer)
    }
}
