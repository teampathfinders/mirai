use bytes::BytesMut;

/// Implemented by all game packets.
pub trait ConnectedPacket {
    /// Unique ID of the packet.
    const ID: u32;
}
