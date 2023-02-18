use common::{nvassert, VResult};

use crate::{command::ParsedCommand, level_manager::LevelManager};

impl LevelManager {
    pub fn handle_gamerule(command: ParsedCommand) -> VResult<String> {
        nvassert!(command.name == "gamerule");
    
        todo!();
    
        Ok("Success".to_owned())
    }
}