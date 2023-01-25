use bytes::{BufMut, BytesMut};

use crate::error::VexResult;
use crate::network::packets::GamePacket;
use crate::network::traits::Encodable;
use crate::util::WriteExtensions;

#[derive(Debug)]
pub struct ServerToClientHandshake {

}

impl GamePacket for ServerToClientHandshake {
    const ID: u32 = 0x03;
}

impl Encodable for ServerToClientHandshake {
    fn encode(&self) -> VexResult<BytesMut> {
        todo!();
    }
}
