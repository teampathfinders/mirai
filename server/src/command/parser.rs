use std::collections::HashMap;

use std::str::Split;
use common::{VResult, bail, error};
use dashmap::DashMap;

use super::{Command, CommandDataType, CommandOverload};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CommandParseErrorKind {
    NonExistentCommand,
    MissingArgument,
    InvalidOption
}

#[derive(Debug, Clone)]
pub struct CommandParseError {
    kind: CommandParseErrorKind,
    description: String
}

#[derive(Debug)]
pub enum ParsedArgument {
    Int(i32),
    Float(f32),
    String(String)
}

#[derive(Debug)]
pub struct ParsedCommand {
    pub name: String,
    pub parameters: HashMap<String, ParsedArgument>
}

impl ParsedCommand {
    /// Parses the command and verifies the arguments.
    pub fn parse(command_list: &DashMap<String, Command>, raw: &str)
        -> Result<ParsedCommand, String>
    {
        let mut parts = raw.split(' ');
        
        // Make sure the string is not empty.
        let name = if let Some(name) = parts.next() {
            let mut chars = name.chars();
            chars.next();
            chars.as_str().to_owned()
        } else {
            return Err("Command line cannot be empty".to_owned())
        };

        // Verify the command exists and find the command's parameters.
        if let Some(command) = command_list.get(&name) {
            let mut errors = Vec::new();
            for overload in &command.overloads {
                let parse_result = parse_overload(overload, parts.clone());
                if let Ok(parsed) = parse_result {
                    return Ok(ParsedCommand {
                        name, parameters: parsed
                    })
                } else {
                    errors.push(parse_result.unwrap_err());
                }
            }

            return Err(format!(
                "Given arguments did not match any command overload\n{}", 
                errors
                    .iter()
                    .enumerate()
                    .map(|(i, e)| format!("\tOverload {i} - {e}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ))
        } else {
            return Err(format!("Unknown command: {name}. Please check that the command exists and you have permission to use it."))
        }
    }
}

fn parse_overload(overload: &CommandOverload, mut parts: Split<char>) 
    -> Result<HashMap<String, ParsedArgument>, String> 
{
    let mut parsed = HashMap::new();
    for (i, parameter) in overload.parameters.iter().enumerate() {
        let part = if let Some(part) = parts.next() {
            part
        } else {
            if parameter.optional {
                return Ok(parsed)
            } else {
                return Err(format!("Expected {} arguments, got {}", overload.parameters.len(), i))
            }
        };

        // Verify that the argument matches one of the predefined options.
        if let Some(ref cmd_enum) = parameter.command_enum {
            let valid = cmd_enum
                .options
                .iter()
                .any(|o| o == part);

            if !valid {
                // Invalid option.
                return Err(format!("Argument '{part}' is invalid. You must choose one of the available options"))
            }
        }

        let value = match parameter.data_type {
            CommandDataType::String => ParsedArgument::String(part.to_owned()),
            _ => todo!()
        };

        parsed.insert(parameter.name.clone(), value);
    }

    Ok(parsed)
}