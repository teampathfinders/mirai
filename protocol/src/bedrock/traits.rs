/// Implemented by all game raknet.
pub trait ConnectedPacket {
    /// Unique ID of the packet.
    const ID: u32;
    /// Returns the size of this packet in bytes after it has been serialized.
    /// This is used to preallocate the required amount of memory to serialize
    /// a complete packet in one allocation.
    fn serialized_size(&self) -> usize {
        0
    }
}
