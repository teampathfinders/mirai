use std::io::Write;
use std::net::SocketAddr;
use util::bytes::{BinaryWriter, MutableBuffer};

use crate::network::raknet::OFFLINE_MESSAGE_DATA;
use util::Result;
use util::Serialize;

/// Sent in response to [`OpenConnectionRequest2`](super::open_connection_request2::OpenConnectionRequest2).
#[derive(Debug)]
pub struct OpenConnectionReply2 {
    /// GUID of the server.
    /// Corresponds to [`ServerInstance::guid`](crate::ServerInstance::guid).
    pub server_guid: u64,
    /// IP address of the client.
    pub client_address: SocketAddr,
    /// MTU of the connection.
    /// This value should be the same as [`OpenConnectionRequest2::mtu`](super::open_connection_request2::OpenConnectionRequest2::mtu).
    pub mtu: u16,
}

impl OpenConnectionReply2 {
    /// Unique identifier of the packet.
    pub const ID: u8 = 0x08;

    pub fn serialized_size(&self) -> usize {
        1 + 16
            + if self.client_address.is_ipv4() {
                1 + 4 + 2
            } else {
                1 + 2 + 2 + 4 + 16 + 4
            }
            + 2
            + 1
    }
}

impl Serialize for OpenConnectionReply2 {
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_u8(Self::ID);
        buffer.append(OFFLINE_MESSAGE_DATA);
        buffer.write_u64_be(self.server_guid);
        buffer.write_addr(self.client_address);
        buffer.write_u16_be(self.mtu);
        // Encryption not enabled, must be false to continue login sequence.
        // Actual encryption will be enabled later on, using `ServerToClientHandshake`.
        buffer.write_bool(false);

        Ok(())
    }
}
