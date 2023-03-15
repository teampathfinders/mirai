
use util::Result;

use util::{Deserialize, Serialize};

use super::ConnectedPacket;

/// Synchronises the current tick.
///
/// This packet is first sent by the client and should be responded to with the same request timestamp and a new response timestamp.
#[derive(Debug, Clone)]
pub struct TickSync {
    /// Timestamp of when the client sent the packet.
    pub request: u64,
    /// Timestamp of when the server sent the packet.
    pub response: u64,
}

impl ConnectedPacket for TickSync {
    const ID: u32 = 0x17;

    fn serialized_size(&self) -> usize {
        8
    }
}

impl Deserialize for TickSync {
    fn deserialize(mut buffer: SharedBuffer) -> Result<Self> {
        let request = buffer.get_u64_le();
        let response = buffer.get_u64_le();

        Ok(Self { request, response })
    }
}

impl Serialize for TickSync {
    fn serialize(&self, buffer: &mut OwnedBuffer) {
        buffer.put_u64_le(self.request);
        buffer.put_u64_le(self.response);
    }
}
