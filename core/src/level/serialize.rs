use level::{BiomeEncoding, Biomes, SubChunk};
use util::bytes::{BinaryWrite, MutableBuffer};

#[inline]
fn serialize_biome_palette(
    buffer: &mut MutableBuffer,
    palette: &[u32],
) -> anyhow::Result<()> {
    buffer.write_var_i32(palette.len() as i32)?;
    for v in palette {
        buffer.write_var_i32(*v as i32)?;
    }

    Ok(())
}

#[inline]
pub fn serialize_biomes(
    buffer: &mut MutableBuffer,
    biomes: &Biomes,
) -> anyhow::Result<()> {
    for fragment in biomes.fragments() {
        match fragment {
            BiomeEncoding::Paletted(paletted) => {
                let indices = paletted.indices();
                let max_index = paletted.max_index();

                level::serialize_packed_array(
                    buffer, indices, max_index, true,
                )?;
                serialize_biome_palette(buffer, paletted.palette())?;
            }
            _ => {
                // TODO: other encoding types
            }
        }
    }

    Ok(())
}

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
