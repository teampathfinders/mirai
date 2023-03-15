use std::cmp::Ordering;
use std::collections::HashMap;

use std::str::Split;
use util::{Result, bail, error};
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

impl ParsedArgument {
    pub fn read_str(&self) -> Result<&str> {
        if let Self::String(ref value) = self {
            Ok(value)
        } else {
            bail!(InvalidCommand, "Expected string, found {:?}", self)
        }
    }
}

#[derive(Debug)]
pub struct ParsedCommand {
    pub name: String,
    pub parameters: HashMap<String, ParsedArgument>
}

impl ParsedCommand {
    /// Parses the command and verifies the arguments.
    pub fn parse(command_list: &DashMap<String, Command>, raw: &str)
        -> std::result::Result<Self, String>
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
            let mut latest_error = String::new();
            let mut furthest_param = -1i32;

            for overload in &command.overloads {
                let parse_result = parse_overload(overload, parts.clone());
                if let Ok(parsed) = parse_result {
                    return Ok(Self {
                        name, parameters: parsed
                    })
                } else {
                    let err = parse_result.unwrap_err();

                    // Only log the overload that was most "successful". (i.e. most arguments parsed correctly)
                    match furthest_param.cmp(&(err.1 as i32)) {
                        Ordering::Less => {
                            latest_error = err.0;
                            furthest_param = err.1 as i32
                        },
                        // If two overloads are equally successful, use the newest one only.
                        Ordering::Equal => {
                            latest_error = err.0;
                        },
                        Ordering::Greater => ()
                    }
                }
            }

            return Err(format!(
                "Syntax error: {latest_error}"
            ))
        } else {
            return Err(format!("Unknown command: {name}. Please check that the command exists and you have permission to use it."))
        }
    }
}

fn parse_overload(overload: &CommandOverload, mut parts: Split<char>) 
    -> std::result::Result<HashMap<String, ParsedArgument>, (String, usize)>
{
    let mut parsed = HashMap::new();
    for (i, parameter) in overload.parameters.iter().enumerate() {
        let part = if let Some(part) = parts.next() {
            part
        } else if parameter.optional {
            return Ok(parsed)
        } else {
            return Err((format!("Expected {} arguments, got {}", overload.parameters.len(), i), i))
        };

        // Verify that the argument matches one of the predefined options.
        if let Some(ref cmd_enum) = parameter.command_enum {
            let valid = cmd_enum
                .options
                .iter()
                .any(|o| o == part);

            if !valid {
                // Invalid option.
                let mut options_tip = cmd_enum.options.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
                if cmd_enum.options.len() > 3 {
                    options_tip += "..";
                }

                return Err((format!("Option '{part}' is invalid. Help: use one of the predefined options: {options_tip}."), i))
            }
        }

        // Parse the value into the correct type.
        let value = match parameter.data_type {
            CommandDataType::String => ParsedArgument::String(part.to_owned()),
            CommandDataType::Int => {
                let result = part.parse();
                if let Ok(value) = result {
                    ParsedArgument::Int(value)
                } else {
                    return Err((format!("Failed to parse argument '{}'. Expected a valid integer.", part), i))
                }
            }
            _ => todo!()
        };

        parsed.insert(parameter.name.clone(), value);
    }

    Ok(parsed)
}