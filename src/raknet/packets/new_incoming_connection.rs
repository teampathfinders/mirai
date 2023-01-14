use crate::decodable;
use std::net::SocketAddr;

decodable!(
    0x13,
    pub struct NewIncomingConnection;
);

