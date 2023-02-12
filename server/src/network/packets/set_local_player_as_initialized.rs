use bytes::BytesMut;
use common::{VResult, ReadExtensions};

use crate::network::Decodable;

use super::GamePacket;

#[derive(Debug)]
pub struct SetLocalPlayerAsInitialized {
    pub runtime_id: u64
}

impl GamePacket for SetLocalPlayerAsInitialized {
    const ID: u32 = 0x71;
}

impl Decodable for SetLocalPlayerAsInitialized {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        Ok(Self {
            runtime_id: buffer.get_var_u64()?
        })
    }
}