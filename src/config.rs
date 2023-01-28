use lazy_static::lazy_static;
use parking_lot::RwLock;

use crate::network::packets::{ClientThrottleSettings, CompressionAlgorithm};

/// Global service that contains all configuration settings
pub struct ServerConfig {
    /// Max player count.
    pub max_players: usize,
    /// Compression algorithm to use (either Snappy or Deflate).
    pub compression_algorithm: CompressionAlgorithm,
    /// When a packet's size surpasses this threshold, it will be compressed.
    /// Set the threshold to 0 to disable compression.
    pub compression_threshold: u16,
    /// Client throttling settings.
    pub client_throttle: ClientThrottleSettings,
    /// Enable encryption.
    pub enable_encryption: bool,
    /// Name of the server.
    /// This is only visible in LAN games.
    pub server_name: &'static str,
}

lazy_static! {
    /// Current server configuration
    pub static ref SERVER_CONFIG: RwLock<ServerConfig> = RwLock::new(ServerConfig {
        max_players: 10,
        compression_algorithm: CompressionAlgorithm::Deflate,
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
