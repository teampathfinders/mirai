use lazy_static::lazy_static;
use parking_lot::RwLock;

use crate::packets::{ClientThrottleSettings, CompressionAlgorithm};

/// Global service that contains all configuration settings
pub struct ServerConfig {
    pub max_players: usize,
    pub compression_algorithm: CompressionAlgorithm,
    pub compression_threshold: u16,
    pub client_throttle: ClientThrottleSettings,
    pub enable_encryption: bool,
    pub server_name: &'static str,
}

lazy_static! {
    pub static ref SERVER_CONFIG: RwLock<ServerConfig> = RwLock::new(ServerConfig {
        max_players: 10,
        compression_algorithm: CompressionAlgorithm::Snappy,
        compression_threshold: 1, // Compress all packets
        client_throttle: ClientThrottleSettings { // Disable client throttling
            enabled: false,
            threshold: 0,
            scalar: 0.0
        },
        enable_encryption: true,
        server_name: "Pathfinders"
    });
}