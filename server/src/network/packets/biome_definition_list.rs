use bytes::BytesMut;
use common::VResult;

use common::Encodable;

use super::GamePacket;

const DEFINITIONS: &[u8] = include_bytes!("../../../included/biomes.nbt");

/// Sends a list of available biomes to the client.
#[derive(Debug)]
pub struct BiomeDefinitionList;

impl GamePacket for BiomeDefinitionList {
    const ID: u32 = 0x7a;
}

impl Encodable for BiomeDefinitionList {
    fn encode(&self) -> VResult<BytesMut> {
        Ok(BytesMut::from(DEFINITIONS))
    }
}
