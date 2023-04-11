use util::{Deserialize, Serialize};
use util::bytes::{BinaryRead, BinaryWrite, MutableBuffer, SharedBuffer};
use util::Result;

use crate::network::ConnectedPacket;

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

impl<'a> Deserialize<'a> for TickSync {
    fn deserialize<R>(reader: R) -> anyhow::Result<Self>
    where
        R: BinaryRead<'a> + 'a
    {
        let request = reader.read_u64_le()?;
        let response = reader.read_u64_le()?;

        Ok(Self { request, response })
    }
}

impl Serialize for TickSync {
    fn serialize<W>(&self, writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite
    {
        writer.write_u64_le(self.request)?;
        writer.write_u64_le(self.response)
    }
}
