pub use listener::*;

mod frame;
mod raw;
mod reliability;
mod packets;
mod session;
mod listener;
mod async_queue;

/// Version of Raknet that this server uses.
pub const RAKNET_VERSION: u8 = 11;
/// Special sequence of bytes that is contained in every unframed Raknet packet.
pub const OFFLINE_MESSAGE_DATA: &[u8] = &[
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

