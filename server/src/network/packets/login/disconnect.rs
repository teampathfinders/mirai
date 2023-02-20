use bytes::Bytes;
use bytes::{BufMut, BytesMut};

use crate::network::packets::ConnectedPacket;
use common::{Serialize, size_of_var};
use common::VResult;
use common::WriteExtensions;

pub const DISCONNECTED_NOT_AUTHENTICATED: &str =
    "disconnectionScreen.notAuthenticated";
pub const DISCONNECTED_NO_REASON: &str = "disconnectionScreen.noReason";
pub const DISCONNECTED_TIMEOUT: &str = "disconnectionScreen.timeout";
pub const DISCONNECTED_LOGIN_FAILED: &str = "disconnect.loginFailed";
pub const DISCONNECTED_ENCRYPTION_FAIL: &str =
    "Encryption checksums do not match.";
pub const DISCONNECTED_BAD_PACKET: &str = "Client sent bad packet.";

/// Sent by the server to disconnect a client.
#[derive(Debug, Clone)]
pub struct Disconnect<'a> {
    /// Whether to immediately send the client to the main menu.
    pub hide_disconnect_screen: bool,
    /// Message to display to the client
    pub kick_message: &'a str,
}

impl ConnectedPacket for Disconnect<'_> {
    const ID: u32 = 0x05;

    fn serialized_size(&self) -> usize {
        1 + size_of_var(self.kick_message.len() as u32) + self.kick_message.len()
    }
}

impl Serialize for Disconnect<'_> {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_bool(self.hide_disconnect_screen);
        buffer.put_string(self.kick_message);
    }
}
