use crate::decodable;
use std::net::SocketAddr;

pub struct OpenConnectionRequest2 {
    server_address: SocketAddr,
    mtu: u16,
    client_guid: i64,
}

impl OpenConnectionRequest2 {
    const ID: u8 = 0x07;
}
