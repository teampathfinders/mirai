use crate::encodable;
use std::net::SocketAddr;

encodable!(
    0x10,
    pub struct ConnectionRequestAccepted{
        client_address: SocketAddr,
        request_time: i64,
        time: i64
    }
);

