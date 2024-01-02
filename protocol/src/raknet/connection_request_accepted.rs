use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use util::{BinaryWrite, MutableBuffer, IPV4_MEM_SIZE, IPV6_MEM_SIZE};
use util::Result;
use util::Serialize;

/// Sent in response to [`ConnectionRequest`](crate::raknet::ConnectionRequest).
#[derive(Debug)]
pub struct ConnectionRequestAccepted {
    /// IP address of the client.
    pub client_address: SocketAddr,
    /// Corresponds to [`ConnectionRequest::time`](crate::raknet::ConnectionRequest::time).
    pub request_time: i64,
}

impl ConnectionRequestAccepted {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x10;

    pub fn serialized_size(&self) -> usize {
        1 + IPV6_MEM_SIZE + 2 + 10 * IPV4_MEM_SIZE + 8 + 8
    }
}

impl Serialize for ConnectionRequestAccepted {
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_u8(Self::ID)?;
        buffer.write_addr(&self.client_address)?;
        buffer.write_u16_be(0)?; // System index

        let null_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 19132));
        for _ in 0..20 {
            // 20 internal IDs
            buffer.write_addr(&null_addr)?;
        }
        buffer.write_i64_be(self.request_time)?;
        buffer.write_i64_be(self.request_time) // Response time
    }
}
