use std::net::SocketAddr;

use bytes::{BufMut, BytesMut};
use common::{VResult, WriteExtensions};

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
        let mut buffer = BytesMut::new();

        buffer.put_string(&self.address.ip().to_string());
        buffer.put_u16_le(self.address.port());

        Ok(buffer)
    }
}
