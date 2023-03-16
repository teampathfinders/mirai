

use util::{pyassert, Result, error};

use crate::{command::ParsedCommand, level_manager::LevelManager, network::GameRule};

impl LevelManager {
    pub fn handle_gamerule_command(&self, command: ParsedCommand) -> Result<String> {
        pyassert!(command.name == "gamerule");
        
        let rule_name = command.parameters.get("rule")
            // Rule parameter should exist, but this is here just to be sure.
            .ok_or_else(|| error!(Malformed, "Missing game rule name."))?
            .read_str()?;

        // Command has value parameter, store the game rule value.
        if let Some(value) = command.parameters.get("value") {
            let new_value = GameRule::from_parsed(rule_name, value)?;
            let old_value = self.set_game_rule(new_value)?;

            if let Some(old_value) = old_value {
                Ok(format!("Set game rule '{rule_name}' to {new_value} (was {old_value})."))
            } else {
                Ok(format!("Set game rule '{rule_name}' to {new_value} (was not set)."))
            }
        } else {
            // Command has no value parameter, load the game rule value.
            if let Some(value) = self.get_game_rule(rule_name) {
                Ok(format!("Game rule '{rule_name}' is set to {value}"))
            } else {
                Ok(format!("Game rule '{rule_name}' is not set"))
            }
        }
    }
}