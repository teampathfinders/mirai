use level::{BiomeEncoding, Biomes, SubChunk, SubChunkVersion, SubLayer};
use util::bytes::{BinaryWrite, MutableBuffer};

#[inline]
fn serialize_biome_palette(buffer: &mut MutableBuffer, palette: &[u32]) -> anyhow::Result<()> {
    buffer.write_var_i32(palette.len() as i32)?;
    for v in palette {
        buffer.write_var_i32(*v as i32)?;
    }

    Ok(())
}

#[inline]
pub fn serialize_biomes(buffer: &mut MutableBuffer, biomes: &Biomes) -> anyhow::Result<()> {
    for fragment in biomes.fragments() {
        match fragment {
            BiomeEncoding::Paletted(paletted) => {
                let indices = paletted.indices();
                let max_index = paletted.max_index();

                level::serialize_packed_array(buffer, indices, max_index, true)?;
                serialize_biome_palette(buffer, paletted.palette())?;
            }
            _ => {
                // TODO: other encoding types
                todo!()
            }
        }
    }

    Ok(())
}

pub fn encode_subchunk(subchunk: &SubChunk, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
    buffer.write_u8(subchunk.version() as u8)?;
    if subchunk.version() != SubChunkVersion::Legacy {
        buffer.write_u8(subchunk.layer_len())?;
    }
    buffer.write_i8(subchunk.index())?;

    for layer in subchunk.layers() {
        encode_layer(layer, buffer)?;
    }

    Ok(())
}

#[inline]
fn encode_layer(layer: &SubLayer, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
    buffer.reserve(4096 * 4 + 1);

    Ok(())
}

//
// #[inline]
// pub fn encode_biome(biome: &Biome, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
//     Ok(())
// }
