use std::io::Write;

use util::bytes::BinaryWrite;
use util::bytes::MutableBuffer;
use util::Result;
use util::Serialize;

use crate::network::ConnectedPacket;

const DEFINITIONS: &[u8] = include_bytes!("../../../included/biomes.nbt");

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
    fn serialize<W>(&self, writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite
    {
        writer.write_all(DEFINITIONS)?;

        Ok(())
    }
}
