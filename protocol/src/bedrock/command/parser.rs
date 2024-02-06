use std::{borrow::Cow, cmp::Ordering};
use std::collections::HashMap;
use std::str::Split;

use crate::bedrock::{Command, CommandDataType, CommandOverload};

/// A type of error that occurred while parsing a command.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// The command issued does not exist.
    UnknownCommand,
    /// The command is missing a required argument.
    MissingArgument,
    /// An invalid option was used in an argument.
    InvalidOption,
    /// Some syntax in the command was incorrect.
    InvalidSyntax
}

/// Error that occurred while parsing a command.
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Type of error that occurred.
    pub kind: ParseErrorKind,
    /// Information about the error.
    pub description: Cow<'static, str>,
}

pub type ParseResult = Result<ParsedCommand, ParseError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandTarget {
    AllPlayers,
    AllEntities,
    ClosestPlayer,
    RandomPlayer,
    Yourself,
    SpecificPlayer(String)
}

impl From<&str> for CommandTarget {
    #[inline]
    fn from(value: &str) -> CommandTarget {
        match value {
            "@a" => Self::AllPlayers,
            "@e" => Self::AllEntities,
            "@p" => Self::ClosestPlayer,
            "@r" => Self::RandomPlayer,
            "@s" => Self::Yourself,
            username => Self::SpecificPlayer(username.to_owned())
        }
    }
}

/// Represents a command argument that has successfully been parsed.
#[derive(Debug)]
pub enum ParsedArgument {
    /// An integer argument.
    Int(i32),
    /// A floating point argument.
    Float(f32),
    /// A string argument.
    String(String),
    /// A selector or target argument. These are the `@s`, `@p`, etc. targets that you often see in commands.
    Target(CommandTarget)
}

impl ParsedArgument {
    /// Converts the argument to a string if it is a string type.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None
        }
    }

    /// Converts the argument to a float if it is a float type.
    pub const fn as_float(&self) -> Option<f32> {
        match self {
            Self::Float(f) => Some(*f),
            _ => None
        }
    }

    /// Converts the argument to a target if it is a target type.
    pub const fn as_target(&self) -> Option<&CommandTarget> {
        match self {
            Self::Target(t) => Some(t),
            _ => None
        }
    }

    /// Converts the argument to an integer if it is an integer type.
    pub const fn as_int(&self) -> Option<i32> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None
        }
    }
}

/// A command that has successfully been parsed.
/// Receiving this struct means that the syntax of the command was completely valid.
#[derive(Debug)]
pub struct ParsedCommand {
    /// The name of the command that is scheduled for execution.
    pub name: String,
    /// Parameters given with the command.
    pub parameters: HashMap<String, ParsedArgument>,
}

impl ParsedCommand {
    /// Parses the command and verifies the arguments.
    pub fn default_parser(syntax: &Command, input: &str) -> ParseResult {
        let mut parts = input.split(' ');

        // Make sure the string is not empty.
        let name = if let Some(name) = parts.next() {
            let mut chars = name.chars();
            chars.next();
            chars.as_str().to_owned()
        } else {
            return Err(ParseError {
                kind: ParseErrorKind::InvalidSyntax,
                description: Cow::Borrowed("Command cannot be empty")
            })
        };

        let mut latest_error = String::new();
        let mut furthest_param = -1i32;

        for overload in &syntax.overloads {
            let parse_result = parse_overload(overload, parts.clone());
            match parse_result {
                Ok(parsed) => {
                    return Ok(Self {
                        name,
                        parameters: parsed,
                    });
                },
                Err((msg, index)) => {
                    // Only log the overload that was most "successful". (i.e. most arguments parsed correctly)
                    match furthest_param.cmp(&(index as i32)) {
                        Ordering::Less => {
                            latest_error = msg;
                            furthest_param = index as i32
                        }
                        // If two overloads are equally successful, use the newest one only.
                        Ordering::Equal => {
                            latest_error = msg;
                        }
                        Ordering::Greater => ()
                    }
                }
            }
        }

        Err(ParseError {
            kind: ParseErrorKind::InvalidSyntax,
            description: Cow::Owned(format!("Syntax error: {latest_error}"))
        })
    }
}

/// Parses a specific overload from the command.
fn parse_overload(overload: &CommandOverload, mut parts: Split<char>)
    -> Result<HashMap<String, ParsedArgument>, (String, usize)>
{
    let mut parsed = HashMap::new();
    for (i, parameter) in overload.parameters.iter().enumerate() {
        let Some(part) = parts.next() else {
            if parameter.optional {
                return Ok(parsed);
            } else {
                return Err((format!("Expected {} arguments, got {}", overload.parameters.len(), i), i));
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
                let mut options_tip = cmd_enum.options.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
                if cmd_enum.options.len() > 3 {
                    options_tip += "..";
                }

                return Err((format!("Option '{part}' is invalid. Help: use one of the predefined options: {options_tip}."), i));
            }
        }

        // Parse the value into the correct type.
        let value = match parameter.data_type {
            CommandDataType::String => ParsedArgument::String(part.into()),
            CommandDataType::Target => ParsedArgument::Target(part.into()),
            CommandDataType::Int => {
                let result = part.parse();
                if let Ok(value) = result {
                    ParsedArgument::Int(value)
                } else {
                    return Err((format!("Failed to parse argument '{}'. Expected a valid integer.", part), i));
                }
            }
            _ => todo!()
        };

        parsed.insert(parameter.name.clone(), value);
    }

    Ok(parsed)
}
