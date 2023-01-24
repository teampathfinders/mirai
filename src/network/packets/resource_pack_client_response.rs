#[derive(Debug)]
pub struct ResourcePackClientResponse {
    pub status: u8,
    pub pack_ids: String, // ResourcePackIds
}

impl ResourcePackClientResponse {
    pub const ID: u8 = 0x08;
}
