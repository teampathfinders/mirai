#[derive(Debug)]
pub struct ResourcePacksInfo {
    pub forcedtoaccept: bool,
    pub scripting_enabled: bool,
    pub behaviorpackinfos: String,
    pub resourcepackinfos: String
    // ResourcePackInfo for both behavior and resourcepackinfos


}


impl ResourcePacksInfo {
    pub const ID: u8 = 0x06;
}
