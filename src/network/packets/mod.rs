pub use cache_status::*;
pub use client_to_server_handshake::*;
pub use disconnect::*;
pub use login::*;
pub use network_settings::*;
pub use online_ping::*;
pub use online_pong::*;
pub use packet::*;
pub use play_status::*;
pub use request_network_settings::*;
pub use resource_pack_client_response::*;
pub use resource_pack_stack::*;
pub use resource_packs_info::*;
pub use server_to_client_handshake::*;
pub use start_game::*;
pub use traits::*;

mod cache_status;
mod client_to_server_handshake;
mod disconnect;
mod login;
mod network_settings;
mod online_ping;
mod online_pong;
mod packet;
mod play_status;
mod request_network_settings;
mod resource_pack_client_response;
mod resource_pack_stack;
mod resource_packs_info;
mod server_to_client_handshake;
mod start_game;
mod traits;

/// ID of Minecraft game packets.
pub const GAME_PACKET_ID: u8 = 0xfe;
/// Semver version that this server supports.
pub const CLIENT_VERSION_STRING: &str = "1.19.60";
/// Protocol version that this server supports.
pub const NETWORK_VERSION: u32 = 567;
