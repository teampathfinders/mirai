#[derive(Debug)]
pub struct ResourcePackStack {
    pub forced_to_accept: bool,
    pub resource_pack_entry: Vec<ResourcePackEntry>,
    pub behavior_pack_entry: Vec<BehaviorPackEntry>,
    pub experimental: bool,
    pub game_version: String


}
#[derive(Debug)]
pub struct ResourcePackEntry {
    pub pack_id: String,
    pub pack_version: String,
    pub subpack_name: String


}


#[derive(Debug)]
pub struct BehaviorPackEntry {
    pub pack_id: String,
    pub pack_version: String,
    pub subpack_name: String


}


impl ResourcePackStack {
    pub const ID: u8 = 0x07;
}
