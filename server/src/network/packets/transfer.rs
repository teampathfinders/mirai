use std::net::SocketAddr;

use bytes::{BufMut, BytesMut};
use common::{VResult, WriteExtensions};

use crate::network::Encodable;

use super::GamePacket;

#[derive(Debug)]
pub struct Transfer {
    pub address: SocketAddr,
}

impl GamePacket for Transfer {
    const ID: u32 = 0x55;
}

impl Encodable for Transfer {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_string(&self.address.ip().to_string());
        buffer.put_u16(self.address.port());

        Ok(buffer)
    }
}
