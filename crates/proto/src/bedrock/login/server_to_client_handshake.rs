use util::{BinaryWrite, VarString};
use util::Serialize;

use crate::bedrock::ConnectedPacket;

/// Sent by the server to initiate encryption.
/// The client responds with a [`ClientToServerHandshake`](crate::bedrock::ClientToServerHandshake) to
/// indicate encryption has successfully been initiated.
#[derive(Debug, Clone)]
pub struct ServerToClientHandshake<'a> {
    /// Token containing the salt and public key.
    pub jwt: &'a str,
}

impl<'a> ConnectedPacket for ServerToClientHandshake<'a> {
    const ID: u32 = 0x03;
}

impl<'a> Serialize for ServerToClientHandshake<'a> {
    fn size_hint(&self) -> Option<usize> {
        Some(self.jwt.var_len())
    }

    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_str(self.jwt)
    }
}
