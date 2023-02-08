use lazy_static::lazy_static;
use parking_lot::RwLock;

/// Supported compression algorithms.
///
/// Snappy is fast, but has produces lower compression ratios.
/// Flate is slow, but produces high compression ratios.
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum CompressionAlgorithm {
    /// The Deflate/Zlib compression algorithm.
    Deflate = 0,
    /// The Snappy compression algorithm.
    /// Available since Minecraft 1.19.30.
    Snappy = 1,
}

/// Settings for client throttling.
///
/// If client throttling is enabled, the client will tick fewer players,
/// improving performance on low-end devices.
#[derive(Debug, Copy, Clone)]
pub struct ClientThrottleSettings {
    /// Regulates whether the client should throttle players.
    pub enabled: bool,
    /// Threshold for client throttling.
    /// If the number of players in the game exceeds this value, players will be throttled.
    pub threshold: u8,
    /// Amount of players that are ticked when throttling is enabled.
    pub scalar: f32,
}

/// Global service that contains all configuration settings
pub struct ServerConfig {
    /// Port to bind the IPv4 socket to.
    pub ipv4_port: u16,
    /// Port to bind the IPv6 socket to.
    pub ipv6_port: u16,
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
        ipv4_port: 19132,
        ipv6_port: 19133,
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
