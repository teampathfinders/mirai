use bytes::{BufMut, BytesMut};

use common::VResult;
use crate::network::packets::GamePacket;
use crate::network::traits::Encodable;
use common::WriteExtensions;

pub const DISCONNECTED_NOT_AUTHENTICATED: &str = "disconnectionScreen.notAuthenticated";
pub const DISCONNECTED_NO_REASON: &str = "disconnectionScreen.noReason";
pub const DISCONNECTED_TIMEOUT: &str = "disconnectionScreen.timeout";
pub const DISCONNECTED_LOGIN_FAILED: &str = "disconnect.loginFailed";
pub const DISCONNECTED_ENCRYPTION_FAIL: &str = "Encryption checksums do not match.";
pub const DISCONNECTED_BAD_PACKET: &str = "Client sent bad packet.";

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
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(1 + 4 + self.kick_message.len());

        buffer.put_bool(self.hide_disconnect_screen);
        buffer.put_string(&self.kick_message);

        Ok(buffer)
    }
}
