use util::{BinaryRead, Deserialize, Vector};

use crate::bedrock::ConnectedPacket;

use super::Rgba;

/// An individual pixel on a map.
#[derive(Debug, Clone)]
pub struct MapPixelRequest {
    /// RGBA colour of the pixel.
    pub color: Rgba,
    /// Index of the requested pixel in the map.
    pub index: u16
}

/// Sent by the client to request data about a map it does not
/// know about yet.
#[derive(Debug, Clone)]
pub struct MapInfoRequest {
    /// ID of the map the data is requested for.
    pub map_id: i64,
    /// Pixels contained in this map.
    pub pixels: Vec<MapPixelRequest>
}

impl ConnectedPacket for MapInfoRequest {
    const ID: u32 = 0x44;
}

impl<'a> Deserialize<'a> for MapInfoRequest {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let map_id = reader.read_var_i64()?;
        let pixel_count = reader.read_u32_le()?;

        let mut pixels = Vec::with_capacity(pixel_count as usize);
        for _ in 0..pixel_count {
            let color = reader.read_vecub()?;
            let index = reader.read_u16_le()?;

            pixels.push(MapPixelRequest {
                color, index
            });
        }

        Ok(MapInfoRequest {
            map_id, pixels
        })
    }
}