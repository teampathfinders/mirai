

use tokio::io::ReadBuf;
use util::bytes::{BinaryReader, SharedBuffer};

use crate::network::packets::ConnectedPacket;
use util::Deserialize;
use util::Result;

/// Sent during login to let the server know whether the client supports caching.
#[derive(Debug, Clone)]
pub struct CacheStatus {
    /// Whether the client supports the client-side blob cache.
    pub supports_cache: bool,
}

impl ConnectedPacket for CacheStatus {
    const ID: u32 = 0x81;
}

impl Deserialize for CacheStatus {
    fn deserialize(mut buffer: SharedBuffer) -> Result<Self> {
        let support = buffer.read_bool()?;

        Ok(Self { supports_cache: support })
    }
}
