use std::num::NonZeroUsize;
use std::ops::Range;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use parking_lot::RwLock;
use tokio::sync::oneshot::{Receiver, Sender};
use tokio_util::sync::CancellationToken;

use level::provider::Provider;
use level::{Biomes, Dimension, SubChunk};
use util::{Result, Vector};

use crate::command::Command;
use crate::config::SERVER_CONFIG;
use crate::network::{LevelChunk, SubChunkRequestMode, SubChunkResponse};
use crate::network::{
    SessionManager, {GameRule, GameRulesChanged},
};

use crate::level::serialize::serialize_biomes;
use lru::LruCache;
use util::bytes::MutableBuffer;

/// Interval between standard Minecraft ticks.
const TICK_INTERVAL: Duration = Duration::from_millis(1000 / 20);

#[derive(Debug)]
pub struct CombinedChunk {
    biome: Biomes,
    sub_chunks: Vec<SubChunk>,
}

#[derive(Debug)]
pub struct LevelCache {
    cache: LruCache<Vector<i32, 2>, CombinedChunk>,
}

impl LevelCache {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for LevelCache {
    fn default() -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(25).unwrap()),
        }
    }
}

pub struct LevelManager {
    cache: RwLock<LevelCache>,
    /// Used to load world data from disk.
    provider: Provider,
    /// List of commands available in this level.
    commands: DashMap<String, Command>,
    /// Currently set game rules.
    game_rules: DashMap<String, GameRule>,
    /// Used to broadcast level events to the sessions.
    session_manager: Arc<SessionManager>,
    /// Current world tick.
    /// This is the standard Minecraft tick.
    /// The level is ticked 20 times every second.
    tick: AtomicU64,
    token: CancellationToken,
}

impl LevelManager {
    pub fn new(session_manager: Arc<SessionManager>, token: CancellationToken) -> anyhow::Result<Arc<Self>> {
        let (level_path, autosave_interval) = {
            let config = SERVER_CONFIG.read();
            (config.level_path, config.autosave_interval)
        };

        let provider = Provider::open(level_path)?;
        let cache = RwLock::new(LevelCache::new());

        let manager = Arc::new(Self {
            provider,
            cache,
            commands: DashMap::new(),
            game_rules: DashMap::from_iter([
                ("showcoordinates".to_owned(), GameRule::ShowCoordinates(false)),
                ("naturalregeneration".to_owned(), GameRule::NaturalRegeneration(false)),
            ]),
            session_manager,
            tick: AtomicU64::new(0),
            token,
        });

        let clone = manager.clone();
        tokio::spawn(async move {
            clone.ticker_job().await
        });

        Ok(manager)
    }

    #[inline]
    pub fn get_current_tick(&self) -> u64 {
        self.tick.load(Ordering::SeqCst)
    }

    /// Returns the requested command
    #[inline]
    pub fn get_command(&self, name: &str) -> Option<Ref<String, Command>> {
        self.commands.get(name)
    }

    /// Returns a list of available commands.
    #[inline]
    pub const fn get_commands(&self) -> &DashMap<String, Command> {
        &self.commands
    }

    /// Adds a command to the list of available commands.
    #[inline]
    pub fn add_command(&self, command: Command) {
        self.commands.insert(command.name.clone(), command);
    }

    #[inline]
    pub fn add_commands(&self, commands: &[Command]) {
        commands.iter().for_each(|cmd| {
            self.commands.insert(cmd.name.clone(), cmd.clone());
        });
    }

    /// Returns the specified game rule
    #[inline]
    pub fn get_game_rule(&self, name: &str) -> Option<GameRule> {
        self.game_rules.get(name).map(|kv| *kv.value())
    }

    /// Returns a list of currently applied game rules.
    #[inline]
    pub fn get_game_rules(&self) -> Vec<GameRule> {
        self.game_rules.iter().map(|kv| *kv.value()).collect::<Vec<_>>()
    }

    /// Sets the value of a game rule, returning the old value if there was one.
    #[inline]
    pub fn set_game_rule(&self, game_rule: GameRule) -> anyhow::Result<Option<GameRule>> {
        let name = game_rule.name();

        self.session_manager.broadcast(GameRulesChanged { game_rules: &[game_rule] })?;
        Ok(self.game_rules.insert(name.to_owned(), game_rule))
    }

    /// Modifies multiple game rules at the same time.
    /// This function also notifies all the clients of the change.
    #[inline]
    pub fn set_game_rules(&self, game_rules: &[GameRule]) -> anyhow::Result<()> {
        for game_rule in game_rules {
            let name = game_rule.name();
            self.game_rules.insert(name.to_owned(), *game_rule);
        }

        self.session_manager.broadcast(GameRulesChanged { game_rules })
    }

    /// Loads all chunks in a radius around a specified center.
    pub fn request_subchunks(&self, center: Vector<i32, 3>, offsets: &[Vector<i8, 3>]) -> anyhow::Result<SubChunkResponse> {
        todo!();
        // https://github.com/df-mc/dragonfly/blob/5f8833e69a933fdbb15625217ebf3b6d8e28fbf5/server/session/chunk.go
    }

    pub fn request_biomes(&self, coordinates: Vector<i32, 2>, dimension: Dimension) -> anyhow::Result<LevelChunk> {
        let biomes = self.provider.get_biomes(coordinates.clone(), dimension)?;
        if biomes.is_none() {
            todo!();
        }
        let biomes = biomes.unwrap();

        let sub_chunks = (-4..20)
            .into_iter()
            .filter_map(|cy| {
                let sub = match self.provider.get_subchunk(coordinates.clone(), cy, dimension) {
                    Ok(sub) => Some(sub),
                    Err(err) => {
                        tracing::error!("Failed to load sub chunk [{},{},{}]: {err:?}", coordinates.x, cy, coordinates.y);
                        return None;
                    }
                };

                sub
            })
            .collect::<Vec<_>>();

        let count = sub_chunks.iter().filter(|o| o.is_some()).count();

        let mut raw_payload = MutableBuffer::new();
        serialize_biomes(&mut raw_payload, &biomes)?;

        let packet = LevelChunk {
            coordinates,
            request_mode: SubChunkRequestMode::Limited,
            highest_sub_chunk: count as u16,
            sub_chunk_count: count as u32,
            blob_hashes: None,
            raw_payload,
        };

        Ok(packet)
    }

    fn tick(&self) {
        self.tick.fetch_add(1, Ordering::SeqCst);
    }

    async fn ticker_job(&self) {
        let mut interval = tokio::time::interval(TICK_INTERVAL);
        loop {
            tokio::select! {
                _ = self.token.cancelled() => break,
                _ = interval.tick() => ()
            };

            self.tick();
            // match self.tick() {
            //     Ok(_) => (),
            //     Err(e) => tracing::error!("{e:?}")
            // }
        }
    }

    // /// Simple job that runs [`flush`](Self::flush) on a specified interval.
    // async fn autosave_job(&self, sender: Sender<()>, interval: Duration) {
    //     let mut interval = tokio::time::interval(interval);
    //
    //     // First tick completes immediately, prevent running autosave immediately after world has
    //     // been opened.
    //     interval.tick().await;
    //
    //     // Run until there are no more references to the chunk manager.
    //     // (other than this job).
    //     //
    //     // This prevents a memory leak in case someone drops the chunk manager.
    //     loop {
    //         match self.flush() {
    //             Ok(_) => (),
    //             Err(e) => {
    //                 tracing::error!("Failed to save level: {e}");
    //             }
    //         }
    //
    //         tokio::select! {
    //             _ = interval.tick() => (),
    //             _ = self.token.cancelled() => break
    //         }
    //     }
    //
    //     // Save before closing.
    //     match self.flush() {
    //         Ok(_) => (),
    //         Err(e) => {
    //             tracing::error!("Failed to save level: {e}");
    //         }
    //     }
    //
    //     // Send the signal that the level has been closed.
    //     let _ = sender.send(());
    //     tracing::info!("Closed level");
    // }
}
