use std::io::Write;

use util::bytes::{BinaryWrite, MutableBuffer};
use util::Result;
use util::Serialize;

use crate::network::OFFLINE_MESSAGE_DATA;

/// Response to [`OfflinePing`](crate::offline_ping::OfflinePing).
#[derive(Debug)]
pub struct UnconnectedPong<'a> {
    /// Timestamp of when the ping was sent.
    /// This should be given the same value as [`OfflinePing::time`](crate::offline_ping::OfflinePing::time).
    pub time: u64,
    /// Randomised GUID of the server.
    /// Corresponds to [`ServerInstance::guid`](crate::ServerInstance::guid)
    pub server_guid: u64,
    /// Contains the info to be displayed in the server banner in the server tab.
    /// Corresponds to [`ServerInstance::metadata`](crate::ServerInstance::metadata)
    pub metadata: &'a str,
}

impl UnconnectedPong<'_> {
    /// Unique identifier of this packet.
    pub const ID: u8 = 0x1c;

    pub fn serialized_size(&self) -> usize {
        1 + 8 + 8 + 16 + 2 + self.metadata.len()
    }
}

impl Serialize for UnconnectedPong<'_> {
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_u8(Self::ID)?;
        buffer.write_u64_be(self.time)?;
        buffer.write_u64_be(self.server_guid)?;
        buffer.write_all(OFFLINE_MESSAGE_DATA)?;

        buffer.write_u16_be(self.metadata.len() as u16)?;
        buffer.write_all(self.metadata.as_bytes())?;

        Ok(())
    }
}
