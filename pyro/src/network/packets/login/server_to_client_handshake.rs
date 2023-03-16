

use util::bytes::{BinaryWriter, MutableBuffer, VarString};

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

impl<'a> ConnectedPacket for ServerToClientHandshake<'a> {
    const ID: u32 = 0x03;

    fn serialized_size(&self) -> usize {
        self.jwt.var_len()
    }
}

impl<'a> Serialize for ServerToClientHandshake<'a> {
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_str(self.jwt);
        Ok(())
    }
}
