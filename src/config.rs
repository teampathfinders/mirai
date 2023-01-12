pub const CLIENT_VERSION_STRING: &str = "1.19";
pub const NETWORK_VERSION: u16 = 560;
pub const RAKNET_VERSION: u8 = 10;

pub struct ServerConfig {
    pub max_players: usize,
    pub ipv4_port: u16,
}
