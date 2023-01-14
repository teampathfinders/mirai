use crate::decodable;
use std::net::SocketAddr;

pub struct NewIncomingConnection;

impl NewIncomingConnection {
    const ID: u8 = 0x13;
}
