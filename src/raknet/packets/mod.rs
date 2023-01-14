mod traits;
mod unconnected_ping;
mod unconnected_pong;
mod packet;
mod open_connection_request1;
mod open_connection_reply1;
mod open_connection_reply2;
mod open_connection_request2;
mod connection_request;
mod connection_request_accepted;
mod new_incoming_connection;
mod login;

pub use traits::*;
pub use unconnected_ping::*;
pub use unconnected_pong::*;
pub use packet::*;

pub const OFFLINE_MESSAGE_DATA: &[u8] = &[
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];