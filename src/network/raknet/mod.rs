pub use frame::*;
pub use raw::*;
pub use reliability::*;

pub use crate::network::header::*;

pub mod packets;

mod frame;
mod raw;
mod reliability;

pub const RAKNET_VERSION: u8 = 11;
pub const OFFLINE_MESSAGE_DATA: &[u8] = &[
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];
