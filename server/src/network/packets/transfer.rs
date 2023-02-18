use std::net::SocketAddr;

use bytes::{BufMut, BytesMut};
use common::{VResult, WriteExtensions, size_of_var};

use common::Encodable;

use super::GamePacket;

/// Transfers the client to another server.
/// The client does this by first returning to the main menu and then connecting to the selected server.
#[derive(Debug, Clone)]
pub struct Transfer {
    /// Address of the server to transfer to.
    pub address: SocketAddr,
}

impl GamePacket for Transfer {
    const ID: u32 = 0x55;
}

impl Encodable for Transfer {
    fn encode(&self) -> VResult<BytesMut> {
        let addr_string = self.address.ip().to_string();
        let packet_size = size_of_var(addr_string.len() as u32) + addr_string.len() + 2;

        let mut buffer = BytesMut::with_capacity(packet_size);

        buffer.put_string(&addr_string);
        buffer.put_u16_le(self.address.port());

        Ok(buffer)
    }
}
