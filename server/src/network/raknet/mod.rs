pub use broadcast::*;
pub use frame::*;
pub use raw::*;
pub use reliability::*;
pub use ticker::*;

pub use crate::network::header::*;

pub mod packets;

mod broadcast;
mod frame;
mod raw;
mod reliability;
mod ticker;

/// Version of Raknet that this server uses.
pub const RAKNET_VERSION: u8 = 11;
/// Special sequence of bytes that is contained in every unframed Raknet packet.
pub const OFFLINE_MESSAGE_DATA: &[u8] = &[
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd,
    0x12, 0x34, 0x56, 0x78,
];
