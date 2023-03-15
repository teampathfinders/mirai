use std::net::SocketAddr;


use util::{Result};
use util::bytes::{BinaryWriter, MutableBuffer, size_of_varint};

use util::Serialize;

use super::ConnectedPacket;

/// Transfers the client to another server.
/// The client does this by first returning to the main menu and then connecting to the selected server.
#[derive(Debug, Clone)]
pub struct Transfer {
    /// Address of the server to transfer to.
    pub addr: SocketAddr,
}

impl ConnectedPacket for Transfer {
    const ID: u32 = 0x55;

    fn serialized_size(&self) -> usize {
        let addr_string = self.addr.to_string();

        size_of_varint(addr_string.len() as u32) + addr_string.len() + 2
    }
}

impl Serialize for Transfer {
    fn serialize(&self, buffer: &mut MutableBuffer) {
        buffer.write_str(&self.addr.to_string());
        buffer.write_u16_le(self.addr.port());
    }
}
