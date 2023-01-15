
use std::net::SocketAddr;

pub struct ConnectionRequestAccepted {
    pub client_address: SocketAddr,
    pub request_time: i64,
    pub time: i64,
}

impl ConnectionRequestAccepted {
    pub const ID: u8 = 0x10;
}
