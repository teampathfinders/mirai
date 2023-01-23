use bytes::{BufMut, BytesMut};

use crate::error::VexResult;
use crate::packets::GamePacket;
use crate::raknet::packets::Encodable;
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

        buffer.put_u8(self.hide_disconnect_screen as u8);
        buffer.put_var_u32(self.kick_message.len() as u32);
        buffer.put(self.kick_message.as_bytes());

        Ok(buffer)
    }
}