
use std::net::SocketAddr;
#[derive(Debug)]
pub struct OpenConnectionReply2 {
    pub server_guid: i64,
    pub client_address: SocketAddr,
    pub mtu: u16,
    pub encryption_enabled: bool,
}

impl OpenConnectionReply2 {
    pub const ID: u8 = 0x08;
}
