pub use acknowledgements::*;
pub use acknowledgements::*;
pub use connection_request::*;
pub use connection_request_accepted::*;
pub use disconnect::*;
pub use incompatible_protocol::*;
pub use new_incoming_connection::*;
pub use offline_ping::*;
pub use offline_pong::*;
pub use online_ping::*;
pub use online_pong::*;
pub use open_connection_reply1::*;
pub use open_connection_reply2::*;
pub use open_connection_request1::*;
pub use open_connection_request2::*;
pub use crate::raknet::traits::*;
pub use crate::raknet::traits::*;

pub use crate::raknet::packet::*;

mod acknowledgements;
mod connection_request;
mod connection_request_accepted;
mod disconnect;
mod incompatible_protocol;
mod new_incoming_connection;
mod offline_ping;
mod offline_pong;
mod online_ping;
mod online_pong;
mod open_connection_reply1;
mod open_connection_reply2;
mod open_connection_request1;
mod open_connection_request2;

pub const RAKNET_VERSION: u8 = 11;
pub const OFFLINE_MESSAGE_DATA: &[u8] = &[
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];
