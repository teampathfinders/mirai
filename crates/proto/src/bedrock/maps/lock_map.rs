use util::{BinaryRead, Deserialize};

use crate::bedrock::ConnectedPacket;

/// Sent by the client to instruct the server to create a new map
/// that is a locked copy of the original map. 
/// This is used for the cartography table.
#[derive(Debug, Clone)]
pub struct MapCreateLockedCopy {
    /// ID of the original map that is being copied.
    pub original_map: i64,
    /// ID of the new map to create. This map will be locked
    /// and therefore cannot be changed.
    pub new_map: i64
}

impl ConnectedPacket for MapCreateLockedCopy {
    const ID: u32 = 0x83;
}

impl<'a> Deserialize<'a> for MapCreateLockedCopy {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let original_map = reader.read_var_i64()?;
        let new_map = reader.read_var_i64()?;

        Ok(MapCreateLockedCopy { 
            original_map, new_map
        })
    }
}