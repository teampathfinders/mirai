use level::{Biome, SubChunk};
use util::bytes::{MutableBuffer, BinaryWrite};



// #[inline]
// pub fn encode_subchunk(subchunk: &SubChunk, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
//     subchunk.serialize_network(buffer)?;
//
//     Ok(())
// }
//
// #[inline]
// pub fn encode_biome(biome: &Biome, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
//     Ok(())
// }
