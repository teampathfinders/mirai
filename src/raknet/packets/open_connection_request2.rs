use crate::decodable;
use std::net::SocketAddr;

decodable!(
    0x07,
    pub struct OpenConnectionRequest2 {
        server_address: SocketAddr,
        mtu: u16,
        client_guid: i64
    }
);

