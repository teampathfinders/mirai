use crate::bedrock::ConnectedPacket;
use crate::types::Dimension;
use util::BinaryRead;
use util::{Deserialize, Vector};

#[derive(Debug, Clone)]
pub struct SubChunkRequest {
    pub dimension: Dimension,
    pub position: Vector<i32, 3>,
    pub offsets: Vec<Vector<i8, 3>>,
}

impl ConnectedPacket for SubChunkRequest {
    const ID: u32 = 0xaf;
}

impl<'a> Deserialize<'a> for SubChunkRequest {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let dimension = Dimension::try_from(reader.read_var_u32()?)?;
        let position = reader.read_veci()?;

        let count = reader.read_u32_le()?;
        let mut offsets = Vec::with_capacity(count as usize);
        for _ in 0..count {
            offsets.push(reader.read_vecb()?);
        }

        Ok(Self { dimension, position, offsets })
    }
}
