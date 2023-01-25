use bytes::{BufMut, BytesMut};

use crate::error::VexResult;
use crate::network::traits::Encodable;
use crate::util::WriteExtensions;

#[derive(Debug)]
pub struct ServerToClientHandshake {
    /// Base64-X509-encoded ECC public key
    pub public_key: String,
    pub token: String,
}

impl ServerToClientHandshake {
    pub const ID: u8 = 0x03;
}

impl Encodable for ServerToClientHandshake {
    fn encode(&self) -> VexResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(2 + self.token.len());

        buffer.put_var_u32(self.token.len() as u32);
        buffer.put(self.token.as_bytes());

        Ok(BytesMut::new())
    }
}