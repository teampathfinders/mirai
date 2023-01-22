pub use client_to_server_handshake::*;
pub use disconnect::*;
pub use login::*;
pub use network_settings::*;
pub use play_status::*;
pub use request_network_settings::*;
pub use resource_pack_client_response::*;
pub use resource_pack_stack::*;
pub use resource_packs_info::*;
pub use server_to_client_handshake::*;
pub use start_game::*;

mod client_to_server_handshake;
mod disconnect;
mod login;
mod network_settings;
mod play_status;
mod request_network_settings;
mod resource_pack_client_response;
mod resource_pack_stack;
mod resource_packs_info;
mod server_to_client_handshake;
mod start_game;

pub const GAME_PACKET_ID: u8 = 0xfe;
pub const CLIENT_VERSION_STRING: &str = "1.19";
pub const NETWORK_VERSION: u16 = 560;