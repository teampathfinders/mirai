use bytes::BytesMut;
use common::VResult;

use crate::network::Encodable;

use super::GamePacket;

#[derive(Debug)]
pub struct BiomeDefinitionList;

impl GamePacket for BiomeDefinitionList {
    const ID: u32 = 0x7a;
}

impl Encodable for BiomeDefinitionList {
    fn encode(&self) -> VResult<BytesMut> {
        todo!();
    }
}
