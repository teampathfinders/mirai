pub mod acknowledgements;
pub mod connection_request;
pub mod connection_request_accepted;
pub mod disconnect;
pub mod incompatible_protocol;
pub mod new_incoming_connection;
pub mod offline_ping;
pub mod offline_pong;
pub mod open_connection_reply1;
pub mod open_connection_reply2;
pub mod open_connection_request1;
pub mod open_connection_request2;

pub const RAKNET_VERSION: u8 = 11;
pub const OFFLINE_MESSAGE_DATA: &[u8] = &[
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];
