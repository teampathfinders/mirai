use level::{Biome, SubChunk};
use util::bytes::MutableBuffer;

#[inline]
pub fn encode_subchunk(subchunk: &SubChunk, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
    todo!();

    Ok(())
}

#[inline]
pub fn encode_biome(biome: &Biome, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
    Ok(())
}
