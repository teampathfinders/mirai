use level::Dimension;
use util::{BinaryRead, SharedBuffer};
use util::{Deserialize, Vector};
use crate::bedrock::ConnectedPacket;

pub struct SubChunkRequest {
    pub dimension: Dimension,
    pub position: Vector<i32, 3>,
    pub offsets: Vec<Vector<i8, 3>>
}

impl ConnectedPacket for SubChunkRequest {
    const ID: u32 = 0xaf;
}

impl<'a> Deserialize<'a> for SubChunkRequest {
    fn deserialize(mut buffer: SharedBuffer<'a>) -> anyhow::Result<Self> {
        let dimension = Dimension::try_from(buffer.read_var_u32()?)?;
        let position = buffer.read_veci()?;

        let count = buffer.read_u32_le()?;
        let mut offsets = Vec::with_capacity(count as usize);
        for _ in 0..count {
            offsets.push(buffer.read_vecb()?);
        }

        Ok(Self {
            dimension, position, offsets
        })
    }
}