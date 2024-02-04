use util::BinaryWrite;
use util::Serialize;

use crate::raknet::OFFLINE_MESSAGE_DATA;

/// Sent in response to [`OpenConnectionRequest1`](crate::raknet::OpenConnectionRequest1).
#[derive(Debug)]
pub struct OpenConnectionReply1 {
    /// GUID of the server.
    /// Corresponds to the random GUID generated on startup.
    pub server_guid: u64,
    /// MTU of the connection.
    /// This should be given the same value as [`OpenConnectionRequest1::mtu`](crate::raknet::OpenConnectionRequest1::mtu).
    pub mtu: u16,
}

impl OpenConnectionReply1 {
    /// Unique identifier of this packet.
    pub const ID: u8 = 0x06;

    /// Estimates the size of the packet when serialized.
    pub const fn size_hint(&self) -> usize {
        1 + 16 + 8 + 1 + 2
    }
}

impl Serialize for OpenConnectionReply1 {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u8(Self::ID)?;
        writer.write_all(OFFLINE_MESSAGE_DATA)?;
        writer.write_u64_be(self.server_guid)?;
        // Disable security, required for login sequence.
        // Encryption will be enabled later on.
        writer.write_u8(0)?;
        writer.write_u16_be(self.mtu)
    }
}
