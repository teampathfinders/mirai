use crate::encodable;
use std::net::SocketAddr;

pub struct OpenConnectionReply2 {
    server_guid: i64,
    client_address: SocketAddr,
    mtu: u16,
    encryption_enabled: bool,
}

impl OpenConnectionReply2 {
    const ID: u8 = 0x08;
}
