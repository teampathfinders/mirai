use bytes::Bytes;
use bytes::{BufMut, BytesMut};

use crate::network::packets::ConnectedPacket;
use util::{Serialize, size_of_varint, VarString};
use util::Result;
use util::WriteExtensions;

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
    pub hide_message: bool,
    /// Message to display to the client
    pub message: &'a str,
}

impl ConnectedPacket for Disconnect<'_> {
    const ID: u32 = 0x05;

    fn serialized_size(&self) -> usize {
        1 + self.message.var_len()
    }
}

impl Serialize for Disconnect<'_> {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.write_bool(self.hide_message);
        buffer.put_string(self.message);
    }
}
