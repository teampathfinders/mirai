
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CommandPermissionLevel {
    Normal,
    GameDirectors,
    Admin,
    Host,
    Owner,
    Internal,
}

/// Used for autocompletion.
///
/// This object contains the list of available options.
#[derive(Debug, Clone)]
pub struct CommandEnum {
    /// ID of the autocompleted type.
    /// If the enum is dynamic, this ID can be used in the [`UpdateDynamicEnum`](super::UpdateDynamicEnum)
    /// packet to update the autocompletion options.
    pub enum_id: String,
    /// Available options.
    pub options: Vec<String>,
    /// Whether the server can update this enum on the fly.
    pub dynamic: bool,
}

/// Type of a parameter.
#[derive(Debug, Copy, Clone)]
pub enum CommandParameterType {
    Int = 1,
    Float = 3,
    Value = 4,
    WildcardInt = 5,
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
    Command = 70,
}

/// Describes a single command parameter.
#[derive(Debug, Clone)]
pub struct CommandParameter {
    /// Name of the parameter.
    pub name: String,
    /// Type of the argument.
    pub argument_type: CommandParameterType,
    /// Whether the argument is optional.
    pub optional: bool,
    /// Additional options for the parameter.
    pub options: u8,
    /// Used for autocompletion.
    pub command_enum: Option<CommandEnum>,
    /// Suffix.
    pub suffix: String,
}

/// Describes a command argument combination.
#[derive(Debug, Clone)]
pub struct CommandOverload {
    /// Command parameters.
    pub parameters: Vec<CommandParameter>,
}

/// Describes a Minecraft command.
#[derive(Debug, Clone)]
pub struct Command {
    /// Name of the command.
    pub name: String,
    /// Description of the command.
    pub description: String,
    /// Who is allowed to use this command.
    pub permission_level: CommandPermissionLevel,
    /// Aliases.
    pub aliases: Vec<String>,
    /// All different argument combinations of the command.
    pub overloads: Vec<CommandOverload>,
}

// #[derive(Debug, Clone)]
// pub struct CommandEnum {
//     pub name: String,
//     pub options: Vec<String>
// }

// #[derive(Debug, Clone)]
// pub enum CommandParameterType {

// }

// #[derive(Debug, Clone)]
// pub struct CommandParameter {
//     pub name: String,
//     pub param_type: CommandParameterType,
//     pub optional: bool,
//     pub options: u8,
//     pub enum_data: Option<CommandEnum>,
//     pub suffix: Option<String>
// }

// #[derive(Debug, Clone)]
// pub struct CommandOverload {
//     pub parameters: Vec<CommandParameter>
// }

// #[derive(Debug, Clone)]
// pub struct Command {
//     pub name: String,
//     pub aliases: Option<CommandEnum>,
//     pub description: String,
//     pub overloads: Vec<CommandOverload>,
//     pub permission_level: CommandPermissionLevel
// }

// #[derive(Debug, Clone)]
// pub enum CommandPermissionLevel {

// }