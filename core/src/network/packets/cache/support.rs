use util::bytes::{BinaryRead, SharedBuffer};
use util::Deserialize;
use util::Result;

use crate::network::ConnectedPacket;

/// Sent during login to let the server know whether the client supports caching.
#[derive(Debug, Clone)]
pub struct CacheStatus {
    /// Whether the client supports the client-side blob cache.
    pub support: bool,
}

impl ConnectedPacket for CacheStatus {
    const ID: u32 = 0x81;
}

impl Deserialize<'_> for CacheStatus {
    fn deserialize(mut buffer: SharedBuffer) -> anyhow::Result<Self> {
        let support = buffer.read_bool()?;

        Ok(Self { support: support })
    }
}
