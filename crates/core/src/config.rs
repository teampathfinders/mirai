//! Server configuration

use std::{
    net::{SocketAddr, SocketAddrV4, SocketAddrV6},
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use parking_lot::RwLock;
use proto::bedrock::{CompressionAlgorithm, ThrottleSettings};
use util::CowString;

use crate::{
    command::Context,
    instance::{Instance, IPV4_LOCAL_ADDR},
};

/// Compression related settings.
pub struct Compression {
    /// Which algorithm to use for compression.
    pub algorithm: CompressionAlgorithm,
    /// Packets above this size threshold will be compressed.
    pub threshold: u16,
}

/// Configuration for the database connection.
pub struct DatabaseConfig {
    /// Host address of the database server.
    ///
    /// Default: localhost.
    ///
    /// When running the server and database in Docker containers, this
    /// should be set to the Docker network name.
    ///
    /// See [Docker networks](`https://docs.docker.com/network/`) for more information.
    pub host: String,
    /// Port of the database server.
    ///
    /// This should usually be set to 6379 when using a Redis server.
    ///
    /// Default: 6379.
    pub port: u16,
}

/// Configuration of the level
pub struct LevelConfig {
    /// The path to the level.
    pub path: String
}

/// Server configuration options.
pub struct Config {
    /// The port that the IPv4 socket is listening on.
    pub(super) ipv4_addr: SocketAddrV4,
    /// The port that the (optional) IPv6 socket is listening on.
    pub(super) ipv6_addr: Option<SocketAddrV6>,
    /// Name of the server.
    ///
    /// This appears at the top of the player list and as the title for LAN broadcasted games.
    pub(super) name: CowString<'static>,
    /// Compression-related settings
    pub(super) compression: Compression,
    /// The client throttling behaviour.
    ///
    /// See [`ThrottleSettings`] for more info,
    pub(super) throttling: ThrottleSettings,
    /// Maximum amount of players the server allows concurrently.
    pub(super) max_connections: AtomicUsize,
    /// The maximum render distance that clients are allowed to use.
    ///
    /// Any client that requests a higher render distance will be capped to this value.
    pub(super) max_render_distance: AtomicUsize,
    /// Database configuration
    pub(super) database: DatabaseConfig,
    /// Level configuration
    pub(super) level: LevelConfig,
    /// Callback that generates a new message of the day.
    pub(super) motd_callback: Box<dyn Fn(&Arc<Instance>) -> CowString<'static> + Send + Sync>,
}

impl Config {
    pub(super) fn new() -> Config {
        Config {
            ipv4_addr: SocketAddrV4::new(IPV4_LOCAL_ADDR, 19132),
            ipv6_addr: None,
            name: CowString::Borrowed("mirai server"),
            compression: Compression {
                algorithm: CompressionAlgorithm::Flate,
                threshold: 1,
            },
            throttling: ThrottleSettings {
                enabled: false,
                scalar: 0.0,
                threshold: 0,
            },
            database: DatabaseConfig {
                host: String::from("localhost"),
                port: 6379,
            },
            level: LevelConfig { path: String::from("resources\\level") },
            max_connections: AtomicUsize::new(10),
            max_render_distance: AtomicUsize::new(12),
            motd_callback: Box::new(|_| "Powered by Mirai".into()),
        }
    }

    /// Returns the IPv4 port that the server is listening on.
    #[inline]
    pub const fn ipv4_addr(&self) -> SocketAddrV4 {
        self.ipv4_addr
    }

    /// Returns the (optional) IPv6 port that the server is listening on.
    #[inline]
    pub const fn ipv6_addr(&self) -> Option<SocketAddrV6> {
        self.ipv6_addr
    }

    /// Returns the server name.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the compression settings.
    #[inline]
    pub const fn compression(&self) -> &Compression {
        &self.compression
    }

    /// Returns the current client throttling settings.
    #[inline]
    pub const fn throttling(&self) -> &ThrottleSettings {
        &self.throttling
    }

    /// Returns the current maximum player count.
    #[inline]
    pub fn max_connections(&self) -> usize {
        self.max_connections.load(Ordering::Relaxed)
    }

    /// Sets the maximum amount of players that can be connected at the same time.
    ///
    /// If the maximum is set below the current player count, no players will be disconnected.
    /// Instead any new connections will be refused and the player count will slowly adjust to
    /// the new maximum as players leave.
    ///
    /// This mechanism can also be used to gracefully shutdown the server. Set the maximum player count to 0
    /// so that no players are allowed to join and wait for connected players to slowly leave.
    #[inline]
    pub fn set_max_connections(&self, max: usize) {
        self.max_connections.store(max, Ordering::Relaxed);
    }

    /// Returns the maximum render distance.
    #[inline]
    pub fn max_render_distance(&self) -> usize {
        self.max_render_distance.load(Ordering::Relaxed)
    }

    /// Sets the maximum render distance.
    #[inline]
    pub fn set_max_render_distance(&self, max: usize) {
        self.max_render_distance.store(max, Ordering::Relaxed);
    }

    /// Returns the database configuration.
    #[inline]
    pub fn database(&self) -> &DatabaseConfig {
        &self.database
    }

    /// Returns the level configuration.
    #[inline]
    pub fn level(&self) -> &LevelConfig {
        &self.level
    }
}
