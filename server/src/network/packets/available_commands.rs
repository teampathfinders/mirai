
#[derive(Debug)]
pub struct CommandEnum {
    pub command_type: String,
    pub options: Vec<String>,
    pub dynamic: bool
}

#[derive(Debug)]
pub struct CommandParameter {
    pub name: String,
    pub command_type: u32,
    pub optional: bool,
    pub options: u8,
    pub command_enum: CommandEnum,
    pub suffix: String
}

#[derive(Debug)]
pub struct CommandOverload {
    pub parameters: Vec<CommandParameter>
}

#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub flags: u16,
    pub permission_level: u8,
    pub aliases: Vec<String>,
    pub overloads: Vec<CommandOverload>
}

#[derive(Debug)]
pub struct CommandConstraint {
    pub enum_option: String,
    pub enum_name: String,
    pub constraints: Vec<u8>
}

#[derive(Debug)]
pub struct AvailableCommands {
    pub commands: Vec<Command>,
    pub constraints: Vec<CommandConstraint>
}