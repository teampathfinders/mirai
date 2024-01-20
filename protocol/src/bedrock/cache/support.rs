use util::{BinaryRead, SharedBuffer};
use util::Deserialize;
use util::Result;

use crate::bedrock::ConnectedPacket;

/// Sent during login to let the server know whether the client supports caching.
#[derive(Debug, Clone)]
pub struct CacheStatus {
    /// Whether the client supports the client-side blob cache.
    pub supports_cache: bool,
}

impl ConnectedPacket for CacheStatus {
    const ID: u32 = 0x81;
}

impl<'a> Deserialize<'a> for CacheStatus {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let support = reader.read_bool()?;

        Ok(Self { supports_cache: support })
    }
}
