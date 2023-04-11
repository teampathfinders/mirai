use std::io::Write;

use util::bytes::{BinaryWrite, MutableBuffer};
use util::Result;
use util::Serialize;

use crate::raknet::OFFLINE_MESSAGE_DATA;

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
    fn serialize<W>(&self, writer: W) -> anyhow::Result<()>
    where
        W: BinaryWrite,
    {
        writer.write_u8(Self::ID)?;
        writer.write_u64_be(self.time)?;
        writer.write_u64_be(self.server_guid)?;
        writer.write_all(OFFLINE_MESSAGE_DATA)?;

        writer.write_u16_be(self.metadata.len() as u16)?;
        writer.write_all(self.metadata.as_bytes())?;

        Ok(())
    }
}
