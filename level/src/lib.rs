pub use key::*;
pub use level::*;
pub use sub_chunk::*;
use util::bytes::BinaryRead;

#[cfg(target_endian = "big")]
compile_error!("Big endian architectures are not supported");

/// Performs ceiling division on two u32s.
#[inline(always)]
const fn ceil_div(lhs: u32, rhs: u32) -> u32 {
    (lhs + rhs - 1) / rhs
}

#[derive(Debug, PartialEq, Eq)]
enum PackedArrayReturn {
    Empty,
    ReferBack,
    Data(Box<[u16; 4096]>),
}

#[inline(always)]
fn deserialize_packed_array<'a, R>(reader: &mut R) -> anyhow::Result<PackedArrayReturn>
where
    R: BinaryRead<'a>,
{
    let index_size = reader.read_u8()? >> 1;
    if index_size == 0 {
        return Ok(PackedArrayReturn::Empty);
    } else if index_size == 0x7f {
        return Ok(PackedArrayReturn::ReferBack);
    } else if ![1, 2, 3, 4, 5, 6, 8, 16].contains(&index_size) {
        anyhow::bail!(format!("Invalid index size: {index_size}"));
    }

    let per_word = u32::BITS / index_size as u32;
    let word_count = ceil_div(4096, per_word as u32);
    let mask = !(!0u32 << index_size);

    let mut indices = Box::new([0u16; 4096]);
    let mut offset = 0;

    for _ in 0..word_count {
        let mut word = reader.read_u32_le()?;

        for _ in 0..per_word {
            if offset == 4096 {
                break;
            }

            indices[offset] = (word & mask) as u16;
            word >>= index_size;

            offset += 1;
        }
    }

    Ok(PackedArrayReturn::Data(indices))
}

#[cfg(test)]
mod test;

mod biome;
pub mod database;
mod ffi;
mod key;
mod level;
mod level_dat;
mod sub_chunk;
