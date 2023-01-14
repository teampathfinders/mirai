use crate::decodable;
use std::net::SocketAddr;

decodable!(
     0x09,
    pub struct ConnectionRequest{
        guid: i64,
        time: i64
    }
);

