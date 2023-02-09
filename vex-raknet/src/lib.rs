pub use listener::*;
pub use session::*;

mod frame;
mod raw;
mod reliability;
mod packets;
mod session;
mod listener;
mod async_queue;

/// ID of Minecraft game packets.
pub const GAME_PACKET_ID: u8 = 0xfe;
/// Protocol version that this server supports.
pub const NETWORK_VERSION: u32 = 567;
pub const CLIENT_VERSION_STRING: &str = "1.19.60";
/// Version of Raknet that this server uses.
pub const RAKNET_VERSION: u8 = 11;
/// Special sequence of bytes that is contained in every unframed Raknet packet.
pub const OFFLINE_MESSAGE_DATA: &[u8] = &[
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

