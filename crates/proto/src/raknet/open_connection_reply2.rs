use std::net::SocketAddr;

use util::{BinaryWrite, IPV4_MEM_SIZE, IPV6_MEM_SIZE};

use util::Serialize;

use crate::raknet::OFFLINE_MESSAGE_DATA;

/// Sent in response to [`OpenConnectionRequest2`](crate::raknet::OpenConnectionRequest2).
#[derive(Debug)]
pub struct OpenConnectionReply2 {
    /// GUID of the server.
    /// Corresponds to the random server GUID generated on startup.
    pub server_guid: u64,
    /// IP address of the client.
    pub client_address: SocketAddr,
    /// MTU of the connection.
    /// This value should be the same as [`OpenConnectionRequest2::mtu`](crate::raknet::OpenConnectionRequest2::mtu).
    pub mtu: u16,
}

impl OpenConnectionReply2 {
    /// Unique identifier of the packet.
    pub const ID: u8 = 0x08;

    /// Estimates the size of the packet when serialized.
    pub const fn size_hint(&self) -> usize {
        1 + 16 + if self.client_address.is_ipv4() { IPV4_MEM_SIZE } else { IPV6_MEM_SIZE } + 2 + 1
    }
}

impl Serialize for OpenConnectionReply2 {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u8(Self::ID)?;
        writer.write_all(OFFLINE_MESSAGE_DATA)?;
        writer.write_u64_be(self.server_guid)?;
        writer.write_addr(&self.client_address)?;
        writer.write_u16_be(self.mtu)?;
        // Encryption not enabled, must be false to continue login sequence.
        // Actual encryption will be enabled later on, using `ServerToClientHandshake`.
        writer.write_bool(false)
    }
}
