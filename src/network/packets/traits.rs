use bytes::BytesMut;

use crate::error::VexResult;

/// Implemented by all game packets.
pub trait GamePacket {
    /// Unique ID of the packet.
    const ID: u32;
}
