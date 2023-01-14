use crate::encodable;
use std::net::SocketAddr;

encodable!(
    0x08,
    pub struct OpenConnectionReply2 {
        server_guid: i64,
        client_address: SocketAddr,
        mtu: u16,
        encryption_enabled: bool
    }
);

