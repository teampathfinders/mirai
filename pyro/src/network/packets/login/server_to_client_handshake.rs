use std::fmt::Write;


use util::bytes::{MutableBuffer, VarString};

use crate::network::packets::ConnectedPacket;
use util::Serialize;
use util::Result;

/// Sent by the server to initiate encryption.
/// The client responds with a [`ClientToServerHandshake`](super::ClientToServerHandshake) to
/// indicate encryption has successfully been initiated.
#[derive(Debug, Clone)]
pub struct ServerToClientHandshake<'a> {
    /// Token containing the salt and public key.
    pub jwt: &'a str,
}

impl ConnectedPacket for ServerToClientHandshake<'_> {
    const ID: u32 = 0x03;

    fn serialized_size(&self) -> usize {
        self.jwt.var_len()
    }
}

impl Serialize for ServerToClientHandshake<'_> {
    fn serialize(&self, buffer: &mut MutableBuffer) {
        buffer.write_str(self.jwt);
    }
}
