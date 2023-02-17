use std::collections::HashMap;

use common::{VResult, bail};
use dashmap::DashMap;

use super::Command;

#[derive(Debug)]
pub enum ParsedParameterValue {
    Int(i32),
    Float(f32),
    String(String)
}

#[derive(Debug)]
pub struct ParsedCommand {
    pub name: String,
    pub parameters: HashMap<String, ParsedParameterValue>
}

impl ParsedCommand {
    /// Parses the command and verifies the arguments.
    pub fn parse(command_list: &DashMap<String, Command>, raw: &str) -> VResult<Self> {
        let mut parts = raw.split(' ');
        
        // Make sure the string is not empty.
        let name_raw = if let Some(name) = parts.next() {
            name
        } else {
            bail!(InvalidCommand, "Command line cannot be empty")
        };

        let mut chars = name_raw.chars();
        chars.next();
        let name = chars.as_str();

        // Verify the command exists.
        if !command_list.contains_key(name) {
            bail!(InvalidCommand, "Unknown command: {name}. Please check that the command exists and you have permission to use it.");
        }

        tracing::debug!("{name}");

        todo!()
    }
}