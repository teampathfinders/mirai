use std::sync::Arc;

use common::VResult;
use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use parking_lot::{RwLock, RwLockReadGuard};

use crate::command::Command;
use crate::network::{
    packets::{GameRule, GameRulesChanged},
    session::SessionManager,
};

#[derive(Debug)]
pub struct LevelManager {
    commands: DashMap<String, Command>,
    /// Currently set game rules.
    game_rules: DashMap<String, GameRule>,
    /// Used to broadcast level events to the sessions.
    session_manager: Arc<SessionManager>,
}

impl LevelManager {
    pub fn new(session_manager: Arc<SessionManager>) -> Self {
        Self {
            commands: DashMap::new(),
            game_rules: DashMap::new(),
            session_manager,
        }
    }

    /// Returns the requested command
    #[inline]
    pub fn get_command(&self, name: &str) -> Option<Ref<String, Command>> {
        self.commands.get(name)
    }

    /// Returns a list of available commands.
    #[inline]
    pub fn get_commands(&self) -> &DashMap<String, Command> {
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
