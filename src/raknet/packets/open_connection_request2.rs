
use std::net::SocketAddr;
#[derive(Debug)]
pub struct OpenConnectionRequest2 {
    pub server_address: SocketAddr,
    pub mtu: u16,
    pub client_guid: i64,
}

impl OpenConnectionRequest2 {
    pub const ID: u8 = 0x07;
}
