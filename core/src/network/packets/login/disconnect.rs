use util::bytes::{BinaryWrite, MutableBuffer, VarString};
use util::Result;
use util::Serialize;

use crate::network::ConnectedPacket;

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
    pub hide_reason: bool,
    /// Message to display to the client
    pub reason: &'a str,
}

impl ConnectedPacket for Disconnect<'_> {
    const ID: u32 = 0x05;

    fn serialized_size(&self) -> usize {
        1 + self.reason.var_len()
    }
}

impl Serialize for Disconnect<'_> {
    fn serialize<W>(&self, writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite
    {
        writer.write_bool(self.hide_reason)?;
        writer.write_str(self.reason)
    }
}
