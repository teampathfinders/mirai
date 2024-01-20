use std::io::Write;

use util::BinaryWrite;
use util::MutableBuffer;
use util::Result;
use util::Serialize;

use crate::bedrock::ConnectedPacket;

const DEFINITIONS: &[u8] = include_bytes!("../../../core/include/biomes.nbt");

/// Sends a list of available biomes to the client.
#[derive(Debug, Clone)]
pub struct BiomeDefinitionList;

impl ConnectedPacket for BiomeDefinitionList {
    const ID: u32 = 0x7a;

    fn serialized_size(&self) -> usize {
        DEFINITIONS.len()
    }
}

impl Serialize for BiomeDefinitionList {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_all(DEFINITIONS)?;

        Ok(())
    }
}
