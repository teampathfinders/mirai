use bytes::{BufMut, BytesMut};
use crate::error::VexResult;
use crate::raknet::packets::{Encodable, OFFLINE_MESSAGE_DATA, RAKNET_VERSION};

pub struct IncompatibleProtocol {
    pub server_guid: i64
}

impl IncompatibleProtocol {
    pub const ID: u8 = 0x19;
}

impl Encodable for IncompatibleProtocol {
    fn encode(&self) -> VexResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(1 + 1 + 16 + 8);

        buffer.put_u8(Self::ID);
        buffer.put_u8(RAKNET_VERSION);
        buffer.put(OFFLINE_MESSAGE_DATA);
        buffer.put_i64(self.server_guid);

        Ok(buffer)
    }
}