//! Rust implementation of the RakNet protocol.

pub use packets::*;
use util::glob_export;

mod packets;

glob_export!(ack);
glob_export!(broadcast);
glob_export!(compound_collector);
glob_export!(frame);
glob_export!(login);
glob_export!(order_channel);
glob_export!(raw);
glob_export!(receive);
glob_export!(recovery_queue);
glob_export!(reliability);
glob_export!(send_queue);
glob_export!(send);
glob_export!(session);
glob_export!(ticker);

/// Version of Raknet that this server uses.
pub const RAKNET_VERSION: u8 = 11;
/// Special sequence of bytes that is contained in every unframed Raknet packet.
pub const OFFLINE_MESSAGE_DATA: &[u8] = &[
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];
