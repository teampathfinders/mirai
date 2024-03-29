use util::BinaryWrite;

use util::Serialize;

use crate::raknet::OFFLINE_MESSAGE_DATA;

/// Response to [`UnconnectedPing`](crate::raknet::UnconnectedPing).
#[derive(Debug)]
pub struct UnconnectedPong<'a> {
    /// Timestamp of when the ping was sent.
    /// This should be given the same value as [`UnconnectedPing::time`](crate::raknet::UnconnectedPing::time).
    pub time: u64,
    /// Randomised GUID of the server.
    /// Corresponds to the random GUID generated on startup.
    pub server_guid: u64,
    /// Contains the info to be displayed in the server banner in the server tab.
    /// Corresponds to the metadata generated by [`refresh_metadata`](mirai::instance::ServerInstance::refresh_metadata).
    pub metadata: &'a str,
}

impl UnconnectedPong<'_> {
    /// Unique identifier of this packet.
    pub const ID: u8 = 0x1c;

    /// Estimates the size of the packet when serialized.
    pub const fn size_hint(&self) -> usize {
        1 + 8 + 8 + 16 + 2 + self.metadata.len()
    }
}

impl Serialize for UnconnectedPong<'_> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u8(Self::ID)?;
        writer.write_u64_be(self.time)?;
        writer.write_u64_be(self.server_guid)?;
        writer.write_all(OFFLINE_MESSAGE_DATA)?;

        writer.write_u16_be(self.metadata.len() as u16)?;
        writer.write_all(self.metadata.as_bytes())?;

        Ok(())
    }
}
