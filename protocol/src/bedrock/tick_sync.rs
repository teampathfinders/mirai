use util::{Deserialize, Serialize};
use util::{BinaryRead, BinaryWrite, MutableBuffer, SharedBuffer};
use util::Result;

use crate::bedrock::ConnectedPacket;

/// Synchronises the current tick.
///
/// This packet is first sent by the client and should be responded to with the same request timestamp and a new response timestamp.
#[derive(Debug, Clone)]
pub struct TickSync {
    /// Timestamp of when the client sent the packet.
    pub request_tick: u64,
    /// Timestamp of when the server sent the packet.
    pub response_tick: u64,
}

impl ConnectedPacket for TickSync {
    const ID: u32 = 0x17;

    fn serialized_size(&self) -> usize {
        8
    }
}

impl<'a> Deserialize<'a> for TickSync {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let request = reader.read_u64_le()?;
        let response = reader.read_u64_le()?;

        Ok(Self { request_tick: request, response_tick: response })
    }
}

impl Serialize for TickSync {
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_u64_le(self.request_tick)?;
        buffer.write_u64_le(self.response_tick)
    }
}
