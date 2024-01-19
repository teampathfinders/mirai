//! An interface that can interact with the Minecraft Bedrock world format.

#![allow(dead_code)]
#![forbid(missing_docs)]
#![deny(clippy::undocumented_unsafe_blocks)]

#[cfg(target_endian = "big")]
compile_error!("Big endian architectures are not supported");

use util::{BinaryRead, BinaryWrite};

/// Performs ceiling division on two u32s.
const fn ceil_div(lhs: u32, rhs: u32) -> u32 {
    (lhs + rhs - 1) / rhs
}

/// Return value from packed array deserialisation.
#[derive(Debug, PartialEq, Eq)]
pub enum PackedArrayReturn {
    /// The packed array was empty.
    Empty,
    /// This array inherits from the previously processed array.
    Inherit,
    /// New data for the array.
    Data(Box<[u16; 4096]>),
}

/// Serializes a packed array into binary format.
///
/// # Arguments
/// * `writer` - Write to serialize into.
/// * `array` - Array to serialize into packed form.
/// * `max_index` - Amount of unique elements of the array.
/// * `is_network` - Serialize into network format.
pub fn serialize_packed_array<W>(writer: &mut W, array: &[u16; 4096], max_index: usize, is_network: bool) -> anyhow::Result<()>
where
    W: BinaryWrite,
{
    // Determine the required bits per index
    let index_size = {
        let mut bits_per_block = 0;
        // Loop over allowed values.
        for b in [1, 2, 3, 4, 5, 6, 8, 16] {
            if 2usize.pow(b) >= max_index {
                bits_per_block = b;
                break;
            }
        }

        bits_per_block as u8
    };

    writer.write_u8(index_size << 1 | is_network as u8)?;

    // Amount of indices that fit in a single 32-bit integer.
    let per_word = u32::BITS / index_size as u32;

    let mut offset = 0;
    while offset < 4096 {
        let mut word = 0;
        for w in 0..per_word {
            if offset == 4096 {
                break;
            }

            let index = array[offset] as u32;
            word |= index << (w * index_size as u32);
            //            println!("word {word:#033b}, index {index:#05b}, is {index_size}");

            offset += 1;
        }

        writer.write_u32_le(word)?;
    }

    Ok(())
}

/// Deserializes a packed array into an array that can be used by other code.
///
/// # Arguments
/// * `reader` - Reader to deserialize from.
///
/// # Returns
/// See [`PackedArrayReturn`].
pub fn deserialize_packed_array<'a, R>(reader: &mut R) -> anyhow::Result<PackedArrayReturn>
where
    R: BinaryRead<'a>,
{
    let index_size = reader.read_u8()? >> 1;
    if index_size == 0 {
        return Ok(PackedArrayReturn::Empty);
    } else if index_size == 0x7f {
        return Ok(PackedArrayReturn::Inherit);
    } else if ![1, 2, 3, 4, 5, 6, 8, 16].contains(&index_size) {
        anyhow::bail!(format!("Invalid index size: {index_size}"));
    }

    let per_word = u32::BITS / index_size as u32;
    let word_count = ceil_div(4096, per_word);
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
mod ffi;
mod key;
mod settings;
mod subchunk;

/// Direct access to the LevelDB database.
pub mod database;
/// Implements serialization and deserialization for important types.
pub mod provider;

pub use biome::*;
pub use key::*;
pub use subchunk::*;
