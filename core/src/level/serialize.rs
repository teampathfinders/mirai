use level::{BiomeEncoding, Biomes};
use util::BinaryWrite;

#[inline]
fn serialize_biome_palette<W: BinaryWrite>(writer: &mut W, palette: &[u32]) -> anyhow::Result<()> {
    writer.write_var_i32(palette.len() as i32)?;
    for v in palette {
        writer.write_var_i32(*v as i32)?;
    }

    Ok(())
}

#[inline]
pub fn serialize_biomes<W: BinaryWrite>(writer: &mut W, biomes: &Biomes) -> anyhow::Result<()> {
    for fragment in biomes.fragments() {
        match fragment {
            BiomeEncoding::Paletted(paletted) => {
                let indices = paletted.indices();
                let max_index = paletted.palette().len() - 1;

                level::serialize_packed_array(writer, indices, max_index, true)?;
                serialize_biome_palette(writer, paletted.palette())?;
            }
            BiomeEncoding::Single(_id) => {}
            _ => {
                // TODO: other encoding types
                todo!()
            }
        }
    }

    Ok(())
}

//
// #[inline]
// pub fn encode_biome(biome: &Biome, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
//     Ok(())
// }
