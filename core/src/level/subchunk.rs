// use crate::data::BLOCK_STATE_DATA;
use level::{SubChunk, SubChunkVersion, SubLayer};
use util::BinaryWrite;

#[derive(Debug)]
pub struct NetSubLayer {
    indices: Box<[u16; 4096]>,
    palette: Vec<u32>,
}

impl NetSubLayer {
    pub fn serialize<W>(&self, writer: &mut W) -> anyhow::Result<()>
    where
        W: BinaryWrite,
    {
        level::serialize_packed_array(writer, &self.indices, self.palette.len(), true)?;

        if !self.palette.is_empty() {
            writer.write_var_i32(self.palette.len() as i32)?;
        }

        for entry in &self.palette {
            writer.write_var_u32(*entry)?;
        }

        Ok(())
    }
}

impl From<SubLayer> for NetSubLayer {
    fn from(value: SubLayer) -> Self {
        todo!()
        // let palette = value.palette().iter().flat_map(|entry| BLOCK_STATE_DATA.get(entry)).collect::<Vec<u32>>();

        // Self { palette, indices: value.take_indices() }
    }
}

#[derive(Debug)]
pub struct NetSubChunk {
    version: SubChunkVersion,
    index: i8,
    layers: Vec<NetSubLayer>,
}

impl NetSubChunk {
    pub fn serialize_in<W>(&self, mut writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite,
    {
        writer.write_u8(self.version as u8)?;
        if self.version != SubChunkVersion::Legacy {
            writer.write_u8(self.layers.len() as u8)?;
        }
        writer.write_i8(self.index)?;

        for layer in &self.layers {
            layer.serialize(&mut writer)?;
        }

        Ok(())
    }
}

impl From<SubChunk> for NetSubChunk {
    fn from(value: SubChunk) -> Self {
        Self {
            version: value.version(),
            index: value.index(),
            layers: value.take_layers().into_iter().map(NetSubLayer::from).collect(),
        }
    }
}
