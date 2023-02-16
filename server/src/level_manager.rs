use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::{RwLock, RwLockReadGuard};

use crate::network::{packets::GameRule, session::SessionManager};

#[derive(Debug)]
pub struct LevelManager {
    game_rules: DashMap<String, GameRule>,

    /// Used to broadcast level events to the sessions.
    session_manager: Arc<SessionManager>
}

impl LevelManager {
    pub fn new(session_manager: Arc<SessionManager>) -> Self {
        Self {
            game_rules: DashMap::new(),
            session_manager
        }
    }

    /// Sets the value of a game rule, returning the old value if there was one.
    #[inline]
    pub fn set_game_rule(&self, game_rule: GameRule) -> Option<GameRule> {
        let name = game_rule.name();
        self.game_rules.insert(name.to_owned(), game_rule)
    }

    /// Modifies multiple game rules at the same time.
    #[inline]
    pub fn set_game_rules(&self, game_rules: &[GameRule]) {
        for game_rule in game_rules {
            self.set_game_rule(*game_rule);
        }
    }
}

impl Drop for LevelManager {
    fn drop(&mut self) {
        println!("Drop");
    }
}