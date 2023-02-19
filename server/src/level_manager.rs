use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Duration;

use common::VResult;
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use level::ChunkManager;
use parking_lot::{RwLock, RwLockReadGuard};
use tokio::sync::oneshot::Receiver;
use tokio::task::JoinHandle;

use crate::command::Command;
use crate::config::SERVER_CONFIG;
use crate::network::{
    packets::{GameRule, GameRulesChanged},
    session::SessionManager,
};

/// Interval between standard Minecraft ticks.
const LEVEL_TICK_INTERVAL: Duration = Duration::from_millis(1000 / 20);

#[derive(Debug)]
pub struct LevelManager {
    /// Used to load world data from disk.
    chunks: Arc<ChunkManager>,
    /// List of commands available in this level.
    commands: DashMap<String, Command>,
    /// Currently set game rules.
    game_rules: DashMap<String, GameRule>,
    /// Used to broadcast level events to the sessions.
    session_manager: Arc<SessionManager>,
    /// Current world tick.
    /// This is the standard Minecraft tick.
    /// The level is ticked 20 times every second.
    tick: AtomicU64
}

impl LevelManager {
    pub fn new(
        session_manager: Arc<SessionManager>
    ) -> VResult<(Arc<Self>, Receiver<()>)> {
        let (world_path, autosave_interval) = {
            let config = SERVER_CONFIG.read();
            (config.level_path.clone(), config.autosave_interval)
        };

        let (chunks, chunk_notifier) = ChunkManager::new(world_path, autosave_interval)?;
        let manager = Arc::new(Self {
            chunks,
            commands: DashMap::new(),
            game_rules: DashMap::from_iter([
                ("showcoordinates".to_owned(), GameRule::ShowCoordinates(false)),
                ("naturalregeneration".to_owned(), GameRule::NaturalRegeneration(false))
            ]),
            session_manager,
            tick: AtomicU64::new(0)
        });

        Ok((manager, chunk_notifier))
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
    pub fn add_many_commands(&self, commands: &[Command]) {
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
        self.game_rules
            .iter()
            .map(|kv| *kv.value())
            .collect::<Vec<_>>()
    }

    /// Sets the value of a game rule, returning the old value if there was one.
    #[inline]
    pub fn set_game_rule(&self, game_rule: GameRule) -> Option<GameRule> {
        let name = game_rule.name();

        self.session_manager
            .broadcast(GameRulesChanged { game_rules: &[game_rule] });
        self.game_rules.insert(name.to_owned(), game_rule)
    }

    /// Modifies multiple game rules at the same time.
    /// This function also notifies all the clients of the change.
    #[inline]
    pub fn set_game_rules(&self, game_rules: &[GameRule]) {
        for game_rule in game_rules {
            let name = game_rule.name();
            self.game_rules.insert(name.to_owned(), *game_rule);
        }

        self.session_manager
            .broadcast(GameRulesChanged { game_rules });
    }
}
