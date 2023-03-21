use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Duration;

use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use parking_lot::RwLock;
use tokio::sync::oneshot::{Receiver, Sender};
use tokio_util::sync::CancellationToken;

use level::{Level};
use util::Result;

use crate::{
    {GameRule, GameRulesChanged}, SessionManager,
};
use crate::Command;
use crate::SERVER_CONFIG;

/// Interval between standard Minecraft ticks.
const LEVEL_TICK_INTERVAL: Duration = Duration::from_millis(1000 / 20);

pub struct LevelManager {
    /// Used to load world data from disk.
    level: RwLock<Level>,
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
    pub fn new(
        session_manager: Arc<SessionManager>,
        token: CancellationToken,
    ) -> Result<Arc<Self>> {
        let (level_path, autosave_interval) = {
            let config = SERVER_CONFIG.read();
            (config.level_path, config.autosave_interval)
        };

        let level = RwLock::new(Level::open(level_path)?);
        dbg!(&level.read().dat);

        let manager = Arc::new(Self {
            level,
            commands: DashMap::new(),
            game_rules: DashMap::from_iter([
                (
                    "showcoordinates".to_owned(),
                    GameRule::ShowCoordinates(false),
                ),
                (
                    "naturalregeneration".to_owned(),
                    GameRule::NaturalRegeneration(false),
                ),
            ]),
            session_manager,
            tick: AtomicU64::new(0),
            token,
        });

        Ok(manager)
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
    pub fn set_game_rule(
        &self,
        game_rule: GameRule,
    ) -> Result<Option<GameRule>> {
        let name = game_rule.name();

        self.session_manager
            .broadcast(GameRulesChanged { game_rules: &[game_rule] })?;
        Ok(self.game_rules.insert(name.to_owned(), game_rule))
    }

    /// Modifies multiple game rules at the same time.
    /// This function also notifies all the clients of the change.
    #[inline]
    pub fn set_game_rules(&self, game_rules: &[GameRule]) -> Result<()> {
        for game_rule in game_rules {
            let name = game_rule.name();
            self.game_rules.insert(name.to_owned(), *game_rule);
        }

        self.session_manager
            .broadcast(GameRulesChanged { game_rules })
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
