use bytes::{BufMut, BytesMut};

use crate::error::VexResult;
use crate::network::packets::GamePacket;
use crate::network::traits::Encodable;
use crate::util::WriteExtensions;

/// Sent by the server to initiate encryption.
/// The client responds with a [`ClientToServerHandshake`](super::ClientToServerHandshake) to
/// indicate encryption has successfully been initiated.
#[derive(Debug)]
pub struct ServerToClientHandshake<'a> {
    /// Token containing the salt and public key.
    pub jwt: &'a str,
}

impl GamePacket for ServerToClientHandshake<'_> {
    const ID: u32 = 0x03;
}

impl Encodable for ServerToClientHandshake<'_> {
    fn encode(&self) -> VexResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(2 + self.jwt.len());

        buffer.put_string(self.jwt);

        Ok(buffer)
    }
}
