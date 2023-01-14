use crate::raknet::packets::RaknetPacket;

pub struct ConnectionRequest {
    pub guid: i64,
    pub time: i64
}

impl ConnectionRequest {
    const ID: u8 = 0x09;
}