pub use acknowledgements::*;
pub use connection_request::*;
pub use connection_request_accepted::*;
pub use disconnect::*;
pub use incompatible_protocol::*;
pub use new_incoming_connection::*;
pub use open_connection_reply1::*;
pub use open_connection_reply2::*;
pub use open_connection_request1::*;
pub use open_connection_request2::*;
pub use traits::*;
pub use unconnected_ping::*;
pub use unconnected_pong::*;

pub use crate::raknet::packet::*;

mod acknowledgements;
mod client_to_server_handshake;
mod connection_request;
mod connection_request_accepted;
mod disconnect;
mod incompatible_protocol;
mod login;
mod new_incoming_connection;
mod open_connection_reply1;
mod open_connection_reply2;
mod open_connection_request1;
mod open_connection_request2;
mod play_status;
mod resource_pack_client_response;
mod resource_pack_stack;
mod resource_packs_info;
mod server_to_client_handshake;
mod start_game;
mod traits;
mod unconnected_ping;
mod unconnected_pong;

pub const RAKNET_VERSION: u8 = 10;
pub const OFFLINE_MESSAGE_DATA: &[u8] = &[
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];
