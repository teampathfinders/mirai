use bytes::{BufMut, BytesMut};

use vex_common::error::VResult;
use vex_common::traits::Encodable;

use crate::network::packets::GamePacket;
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
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(2 + self.jwt.len());

        buffer.put_string(self.jwt);

        Ok(buffer)
    }
}
