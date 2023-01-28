#[derive(Debug)]
pub struct ResourcePackClientResponse {
    pub status: u8,
    pub pack_ids: String, // ResourcePackIds
}

impl ResourcePackClientResponse {
    /// Unique ID of this packet.
    pub const ID: u8 = 0x08;
}
