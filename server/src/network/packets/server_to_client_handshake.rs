use bytes::{BufMut, BytesMut};

use crate::network::packets::GamePacket;
use common::Encodable;
use common::VResult;
use common::WriteExtensions;

/// Sent by the server to initiate encryption.
/// The client responds with a [`ClientToServerHandshake`](super::ClientToServerHandshake) to
/// indicate encryption has successfully been initiated.
#[derive(Debug, Clone)]
pub struct ServerToClientHandshake<'a> {
    /// Token containing the salt and public key.
    pub jwt: &'a str,
}

impl GamePacket for ServerToClientHandshake<'_> {
    const ID: u32 = 0x03;
}

impl Encodable for ServerToClientHandshake<'_> {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(2 + self.jwt.len());

        buffer.put_string(self.jwt);

        Ok(buffer)
    }
}
