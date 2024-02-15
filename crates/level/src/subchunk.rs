use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::iter::FusedIterator;
use std::ops::{Index, IndexMut};

use serde::{Deserialize, Serialize};

use util::{BinaryRead, BinaryWrite};
use util::{RVec, Vector};

use crate::PackedArrayReturn;

/// Version of the subchunk.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SubChunkVersion {
    /// Legacy sub chunks are from before the Aquatic update.
    /// These sub chunks only contain a single layer.
    Legacy = 1,
    /// Limited sub chunks are from before the Caves and Cliffs update.
    Limited = 8,
    /// Limitless are post Caves and Cliffs. The only difference between `Limitless` and `Limited` is the fact that limitless
    /// contains a sub chunk index.
    Limitless = 9,
}

impl TryFrom<u8> for SubChunkVersion {
    type Error = anyhow::Error;

    fn try_from(v: u8) -> anyhow::Result<Self> {
        Ok(match v {
            1 => Self::Legacy,
            8 => Self::Limited,
            9 => Self::Limitless,
            _ => anyhow::bail!(format!("Invalid chunk version: {v}")),
        })
    }
}

mod block_version {
    use serde::{Deserialize, Deserializer, Serializer};

    /// Deserializes a block version.
    #[inline]
    pub fn deserialize<'de, D>(de: D) -> anyhow::Result<Option<[u8; 4]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let word = Option::<i32>::deserialize(de)?;
        Ok(word.map(i32::to_be_bytes))
    }

    /// Serializes a block version.
    #[inline]
    #[allow(clippy::trivially_copy_pass_by_ref)] // Serde requirement.
    pub fn serialize<S>(v: &Option<[u8; 4]>, ser: S) -> anyhow::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(b) = v {
            ser.serialize_i32(i32::from_be_bytes(*b))
        } else {
            ser.serialize_none()
        }
    }
}

/// Definition of block in the sub chunk block palette.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename = "")]
pub struct PaletteEntry {
    /// Name of the block.
    pub name: String,
    /// Version of the block.
    #[serde(with = "block_version")]
    pub version: Option<[u8; 4]>,
    /// Block-specific properties.
    pub states: HashMap<String, nbt::Value>,
}

impl PaletteEntry {
    /// Hashes this block.
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        hasher.write(self.name.as_bytes());
        for (k, v) in &self.states {
            hasher.write(k.as_bytes());
            v.hash(&mut hasher);
        }

        hasher.finish()
    }
}

/// A layer in a sub chunk.
///
/// Sub chunks can have multiple layers.
/// The first layer contains plain old block data,
/// while the second layer (if it exists) generally contains water logging data.
///
/// The layer is prefixed with a byte indicating the size in bits of the block indices.
/// This is followed by `4096 / (32 / bits)` 32-bit integers containing the actual indices.
/// In case the size is 3, 5 or 6, there is one more integer appended to the end to fit all data.
///
/// Immediately following the indices, the palette starts.
/// This is prefixed with a 32-bit little endian integer specifying the size of the palette.
/// The rest of the palette then consists of `n` concatenated NBT compounds.
#[doc(alias = "storage record")]
#[derive(Debug, PartialEq, Eq)]
pub struct SubLayer {
    /// List of indices into the palette.
    ///
    /// Coordinates can be converted to an offset into the array using [`to_offset`].
    pub(crate) indices: Box<[u16; 4096]>,
    /// List of all different block types in this sub chunk layer.
    pub(crate) palette: Vec<PaletteEntry>,
}

impl SubLayer {
    /// Creates an iterator over the blocks in this layer.
    ///
    /// This iterates over every indices
    #[inline]
    pub fn iter(&self) -> LayerIter {
        LayerIter::from(self)
    }

    /// Gets a reference to a block inside the subchunk.
    pub fn get(&self, pos: Vector<u8, 3>) -> Option<&PaletteEntry> {
        if pos.x > 16 || pos.y > 16 || pos.z > 16 {
            return None;
        }

        let offset = to_offset(pos);
        debug_assert!(offset < 4096, "Array offset out of range");

        let index = self.indices[offset] as usize;
        Some(&self.palette[index])
    }

    // FIXME: Using this method will modify every block with the same index
    // instead of only the block at the specified position.
    // pub fn get_mut(&mut self, pos: Vector<u8, 3>) -> Option<&mut PaletteEntry> {
    //     if pos.x > 16 || pos.y > 16 || pos.z > 16 {
    //         return None;
    //     }

    //     let offset = to_offset(pos);
    //     debug_assert!(offset < 4096);

    //     let index = self.indices[offset] as usize;
    //     Some(&mut self.palette[index])
    // }

    /// Returns a reference to the block palette.
    pub fn palette(&self) -> &[PaletteEntry] {
        &self.palette
    }

    /// Returns a mutable reference to the block palette.
    pub fn palette_mut(&mut self) -> &mut [PaletteEntry] {
        &mut self.palette
    }

    /// Returns a reference to the block indices.
    pub const fn indices(&self) -> &[u16; 4096] {
        &self.indices
    }

    /// Returns a mutable reference to the block indices.
    pub fn indices_mut(&mut self) -> &mut [u16; 4096] {
        &mut self.indices
    }

    /// Takes ownership of the layer and returns the indices.
    pub fn take_indices(self) -> Box<[u16; 4096]> {
        self.indices
    }
}

impl SubLayer {
    /// Deserializes a single layer from the given buffer.
    fn deserialize_disk<'a, R>(mut reader: R) -> anyhow::Result<Self>
    where
        R: BinaryRead<'a> + Copy + 'a,
    {
        let indices = match crate::deserialize_packed_array(&mut reader)? {
            PackedArrayReturn::Data(data) => data,
            PackedArrayReturn::Empty => anyhow::bail!("Sub layer packed array index size cannot be 0"),
            PackedArrayReturn::Inherit => anyhow::bail!("Sub layer packed array does not support biome referral"),
        };

        let len = reader.read_u32_le()? as usize;
        let mut palette = Vec::with_capacity(len);

        for _ in 0..len {
            let (entry, n) = nbt::from_le_bytes(reader)?;

            palette.push(entry);
            reader.advance(n)?;
        }

        Ok(Self { indices, palette })
    }

    fn deserialize_network<'a, R>(mut reader: R) -> anyhow::Result<Self>
    where
        R: BinaryRead<'a> + Copy + 'a,
    {
        let _indices = match crate::deserialize_packed_array(&mut reader)? {
            PackedArrayReturn::Data(data) => data,
            PackedArrayReturn::Empty => anyhow::bail!("Sub layer packed array index size cannot be 0"),
            PackedArrayReturn::Inherit => anyhow::bail!("Sub layer packed array does not support biome referral"),
        };

        todo!();
    }

    fn serialize_disk<W>(&self, mut writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite,
    {
        crate::serialize_packed_array(&mut writer, &self.indices, self.palette.len(), false)?;

        writer.write_u32_le(self.palette.len() as u32)?;
        for entry in &self.palette {
            nbt::to_le_bytes_in(&mut writer, entry)?;
        }

        Ok(())
    }

    fn serialize_network<W>(&self, mut writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite,
    {
        crate::serialize_packed_array(&mut writer, &self.indices, self.palette.len(), true)?;

        if !self.palette.is_empty() {
            writer.write_var_i32(self.palette.len() as i32)?;
        }

        for _entry in &self.palette {

            // https://github.com/df-mc/dragonfly/blob/master/server/world/chunk/paletted_storage.go#L35
            // Requires new palette storage that only stores runtime IDs.
        }

        todo!()
    }
}

impl<I> Index<I> for SubLayer
where
    I: Into<Vector<u8, 3>>,
{
    type Output = PaletteEntry;

    fn index(&self, position: I) -> &Self::Output {
        let position = position.into();
        assert!(
            position.x <= 16 && position.y <= 16 && position.z <= 16,
            "Block position out of sub chunk bounds"
        );

        let offset = to_offset(position);
        let index = self.indices[offset] as usize;
        &self.palette[index]
    }
}

impl<I> IndexMut<I> for SubLayer
where
    I: Into<Vector<u8, 3>>,
{
    fn index_mut(&mut self, position: I) -> &mut Self::Output {
        let position = position.into();
        assert!(
            position.x <= 16 && position.y <= 16 && position.z <= 16,
            "Block position out of sub chunk bounds"
        );

        let offset = to_offset(position);
        let index = self.indices[offset] as usize;
        &mut self.palette[index]
    }
}

impl Default for SubLayer {
    // Std using const generics for arrays would be really nice...
    fn default() -> Self {
        Self {
            indices: Box::new([0; 4096]),
            palette: Vec::new(),
        }
    }
}

/// Converts coordinates to offsets into the block palette indices.
///
/// These coordinates should be in the range [0, 16) for each component.
#[inline]
pub fn to_offset(position: Vector<u8, 3>) -> usize {
    16 * 16 * position.x as usize + 16 * position.z as usize + position.y as usize
}

/// Converts an offset back to coordinates.
///
/// This offset should be in the range [0, 4096).
#[inline]
pub fn from_offset(offset: usize) -> Vector<u8, 3> {
    Vector::from([(offset >> 8) as u8 & 0xf, offset as u8 & 0xf, (offset >> 4) as u8 & 0xf])
}

/// A Minecraft sub chunk.
///
/// Every world contains
#[derive(Debug, PartialEq, Eq)]
pub struct SubChunk {
    /// Version of the sub chunk.
    ///
    /// See [`SubChunkVersion`] for more info.
    pub(crate) version: SubChunkVersion,
    /// Index of the sub chunk.
    ///
    /// This specifies the vertical position of the sub chunk.
    /// It is only used if `version` is set to [`Limitless`](SubChunkVersion::Limitless)
    /// and set to 0 otherwise.
    pub(crate) index: i8,
    /// Layers the sub chunk consists of.
    ///
    /// See [`SubLayer`] for more info.
    pub(crate) layers: Vec<SubLayer>,
}

impl SubChunk {
    /// Version of this subchunk.
    /// See [`SubChunkVersion`] for more information.
    pub const fn version(&self) -> SubChunkVersion {
        self.version
    }

    /// Vertical index of this subchunk
    pub const fn index(&self) -> i8 {
        self.index
    }

    /// The layers (storage records) contained in this subchunk.
    pub fn layers(&self) -> &[SubLayer] {
        &self.layers
    }

    /// Get an immutable reference to the layer at the specified index.
    pub fn layer(&self, index: usize) -> Option<&SubLayer> {
        self.layers.get(index)
    }

    /// Get a mutable reference to the layer at the specified index.
    pub fn layer_mut(&mut self, index: usize) -> Option<&mut SubLayer> {
        self.layers.get_mut(index)
    }

    /// Takes ownership of the subchunk and returns an owned list of its layers.
    #[inline]
    pub fn take_layers(self) -> Vec<SubLayer> {
        self.layers
    }
}

impl SubChunk {
    /// Deserialize a full sub chunk from the given buffer.
    pub fn deserialize_disk<'a, R>(mut reader: R) -> anyhow::Result<Self>
    where
        R: BinaryRead<'a> + Copy + 'a,
    {
        let version = SubChunkVersion::try_from(reader.read_u8()?)?;
        let layer_count = match version {
            SubChunkVersion::Legacy => 1,
            _ => reader.read_u8()?,
        };

        if layer_count == 0 || layer_count > 2 {
            anyhow::bail!("Sub chunk must have 1 or 2 layers");
        }

        let index = if version == SubChunkVersion::Limitless { reader.read_i8()? } else { 0 };

        // let mut layers = SmallVec::with_capacity(layer_count as usize);
        let mut layers = Vec::with_capacity(layer_count as usize);
        for _ in 0..layer_count {
            layers.push(SubLayer::deserialize_disk(reader)?);
        }

        Ok(Self { version, index, layers })
    }

    /// Serialises the sub chunk into a new buffer and returns the buffer.
    pub fn serialize_disk(&self) -> anyhow::Result<RVec> {
        let mut buffer = RVec::alloc();
        self.serialize_disk_in(&mut buffer)?;
        Ok(buffer)
    }

    /// Serialises the sub chunk into the given writer.
    pub fn serialize_disk_in<W>(&self, mut writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite,
    {
        writer.write_u8(self.version as u8)?;
        writer.write_u8(self.layers.len() as u8)?;

        if self.version == SubChunkVersion::Limitless {
            writer.write_i8(self.index)?;
        }

        for layer in &self.layers {
            layer.serialize_disk(&mut writer)?;
        }

        Ok(())
    }
}

impl Index<usize> for SubChunk {
    type Output = SubLayer;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.layers[index]
    }
}

impl IndexMut<usize> for SubChunk {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.layers[index]
    }
}

/// Iterator over blocks in a layer.
pub struct LayerIter<'a> {
    /// Indices in the sub chunk.
    /// While iterating, this is slowly consumed by `std::slice::split_at`.
    indices: &'a [u16],
    /// All possible block states in the current chunk.
    palette: &'a [PaletteEntry],
}

impl<'a> From<&'a SubLayer> for LayerIter<'a> {
    #[inline]
    fn from(value: &'a SubLayer) -> Self {
        Self {
            indices: value.indices.as_ref(),
            palette: value.palette.as_ref(),
        }
    }
}

impl<'a> Iterator for LayerIter<'a> {
    type Item = &'a PaletteEntry;

    fn next(&mut self) -> Option<Self::Item> {
        // ExactSizeIterator::is_empty is unstable
        if self.len() == 0 {
            return None;
        }

        let (a, b) = self.indices.split_at(1);
        self.indices = b;
        self.palette.get(a[0] as usize)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.len()))
    }
}

impl FusedIterator for LayerIter<'_> {}

impl ExactSizeIterator for LayerIter<'_> {
    fn len(&self) -> usize {
        self.indices.len()
    }
}
