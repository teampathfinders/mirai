use bytes::BytesMut;
use common::{VResult, WriteExtensions};

use crate::network::Encodable;

use super::GamePacket;

#[derive(Debug)]
pub struct SetCommandsEnabled {
    pub enabled: bool
}

impl GamePacket for SetCommandsEnabled {
    const ID: u32 = 0x3b;
}

impl Encodable for SetCommandsEnabled {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_bool(self.enabled);

        Ok(buffer)
    }
}