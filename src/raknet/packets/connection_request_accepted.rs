use crate::encodable;
use std::net::SocketAddr;



    pub struct ConnectionRequestAccepted{
        client_address: SocketAddr,
        request_time: i64,
        time: i64
    }


impl ConnectionRequestAccepted {
    const ID: u8 = 0x10;
}