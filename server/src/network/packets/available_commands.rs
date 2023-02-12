use std::collections::{HashSet, HashMap};

use bytes::{BytesMut, BufMut};
use common::{VResult, WriteExtensions, bail};

use crate::network::Encodable;

use super::GamePacket;

pub const COMMAND_PARAMETER_VALID: u32 = 0x100000;
pub const COMMAND_PARAMETER_ENUM: u32 = 0x200000;
pub const COMMAND_PARAMETER_SUFFIXED: u32 = 0x1000000;
pub const COMMAND_PARAMETER_SOFT_ENUM: u32 = 0x4000000;

#[derive(Debug, Clone)]
pub struct CommandEnum {
    pub name: String,
    pub options: Vec<String>,
    pub dynamic: bool,
}

#[derive(Debug, Copy, Clone)]
pub enum CommandArgumentType {
    Int = 1,
    Float = 3,
    Value = 4,
    WildcarInt = 5,
    Operator = 6,
    CompareOperator = 7,
    Target = 8,
    WildcardTarget = 10,
    Filepath = 17,
    IntegerRange = 23,
    EquipmentSlots = 38,
    String = 39,
    BlockPosition = 47,
    Position = 48,
    Message = 51,
    RawText = 53,
    Json = 57,
    BlockStates = 67,
    Command = 70
}

#[derive(Debug)]
pub struct CommandParameter {
    pub name: String,
    pub argument_type: CommandArgumentType,
    pub optional: bool,
    pub options: u8,
    pub command_enum: CommandEnum,
    pub suffix: String,
}

#[derive(Debug)]
pub struct CommandOverload {
    pub parameters: Vec<CommandParameter>,
}

#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub flags: u16,
    pub permission_level: u8,
    pub aliases: Vec<String>,
    pub overloads: Vec<CommandOverload>,
}

#[derive(Debug)]
pub struct CommandConstraint {
    pub enum_option: String,
    pub enum_name: String,
    pub constraints: Vec<u8>,
}

#[derive(Debug)]
pub struct AvailableCommands {
    pub commands: Vec<Command>,
    pub constraints: Vec<CommandConstraint>,
}

impl GamePacket for AvailableCommands {
    const ID: u32 = 0x4c;
}

impl Encodable for AvailableCommands {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();
        
        let mut value_indices = HashMap::new();
        let mut values = Vec::new();
        for command in &self.commands {
            for alias in &command.aliases {
                if !value_indices.contains_key(alias) {
                    value_indices.insert(alias, values.len() as u32);
                    values.push(alias);
                }
            }

            for overload in &command.overloads {
                for parameter in &overload.parameters {
                    for option in &parameter.command_enum.options {
                        if !value_indices.contains_key(option) {
                            value_indices.insert(option, values.len() as u32);
                            values.push(option);
                        }
                    }
                }
            }
        }

        let mut suffix_indices = HashMap::new();
        let mut suffixes = Vec::new();
        for command in &self.commands {
            for overload in &command.overloads {
                for parameter in &overload.parameters {
                    if !parameter.suffix.is_empty() {
                        if !suffix_indices.contains_key(&parameter.suffix) {
                            suffix_indices.insert(&parameter.suffix, suffixes.len() as u32);
                            suffixes.push(&parameter.suffix);
                        }
                    }
                }
            }
        }

        let mut enum_indices = HashMap::new();
        let mut enums = Vec::new();
        for command in &self.commands {
            if !command.aliases.is_empty() {
                let alias_enum = CommandEnum {
                    name: command.name.clone() + "Aliases",
                    options: command.aliases.clone(),
                    dynamic: false
                };
                enum_indices.insert(command.name.clone() + "Aliases", enums.len() as u32);
                enums.push(alias_enum);
            }

            for overload in &command.overloads {
                for parameter in &overload.parameters {
                    if !parameter.command_enum.dynamic && !parameter.command_enum.options.is_empty() {
                        if !enum_indices.contains_key(&parameter.command_enum.name) {
                            enum_indices.insert(parameter.command_enum.name.clone(), enums.len() as u32);
                            enums.push(parameter.command_enum.clone());
                        }
                    }
                }
            }
        }

        let mut dynamic_indices = HashMap::new();
        let mut dynamic_enums = Vec::new();
        for command in &self.commands {
            for overload in &command.overloads {
                for parameter in &overload.parameters {
                    if parameter.command_enum.dynamic {
                        if !dynamic_indices.contains_key(&parameter.command_enum.name) {
                            dynamic_indices.insert(&parameter.command_enum.name, dynamic_enums.len() as u32);
                            dynamic_enums.push(&parameter.command_enum);
                        }
                    }
                }
            }
        }

        tracing::info!("{values:?} {suffixes:?} {enums:?} {dynamic_enums:?}");

        buffer.put_var_u32(values.len() as u32);
        for value in values {
            buffer.put_string(value);
        }

        buffer.put_var_u32(suffixes.len() as u32);
        for suffix in suffixes {
            buffer.put_string(suffix);
        }

        buffer.put_var_u32(enums.len() as u32);
        for command_enum in &enums {
            buffer.put_string(&command_enum.name);
            buffer.put_var_u32(command_enum.options.len() as u32);

            let index_count = value_indices.len() as u32;
            for option in &command_enum.options {
                if index_count <= u8::MAX as u32 {
                    buffer.put_u8(value_indices[option] as u8);
                } else if index_count <= u16::MAX as u32 {
                    buffer.put_u16(value_indices[option] as u16);
                } else {
                    buffer.put_u32(value_indices[option] as u32);
                }
            }
        }

        buffer.put_var_u32(self.commands.len() as u32);
        for command in &self.commands {
            let mut alias = -1i32;
            if !command.aliases.is_empty() {
                alias = enum_indices[&(command.name.clone() + "Aliases")] as i32;
            }

            buffer.put_string(&command.name);
            buffer.put_string(&command.description);
            buffer.put_u16(command.flags);
            buffer.put_u8(command.permission_level);
            buffer.put_i32_le(alias);
            
            buffer.put_var_u32(command.overloads.len() as u32);
            for overload in &command.overloads {
                
                buffer.put_var_u32(overload.parameters.len() as u32);
                for parameter in &overload.parameters {
                    let mut command_type = parameter.argument_type as u32; 

                    if parameter.command_enum.dynamic {
                        command_type = COMMAND_PARAMETER_SOFT_ENUM | COMMAND_PARAMETER_VALID | dynamic_indices[&parameter.command_enum.name];
                    } else if !parameter.command_enum.options.is_empty() {
                        command_type = COMMAND_PARAMETER_ENUM | COMMAND_PARAMETER_VALID | enum_indices[&parameter.command_enum.name];
                    } else if !parameter.suffix.is_empty() {
                        command_type = COMMAND_PARAMETER_SUFFIXED | suffix_indices[&parameter.suffix];
                    }

                    buffer.put_string(&parameter.name);
                    buffer.put_u32_le(command_type);
                    buffer.put_bool(parameter.optional);
                    buffer.put_u8(parameter.options);
                }
            }
        }

        buffer.put_var_u32(dynamic_enums.len() as u32);
        for dynamic_enum in &dynamic_enums {
            buffer.put_string(&dynamic_enum.name);
            buffer.put_var_u32(dynamic_enum.options.len() as u32);

            for option in &dynamic_enum.options {
                buffer.put_string(option);
            }
        }

        buffer.put_var_u32(0);

        Ok(buffer)
    }
}