/// A permission level within the command system.
/// Commands use permission levels separate from the standard permission levels.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum CommandPermissionLevel {
    Normal,
    GameDirectors,
    Admin,
    Host,
    Owner,
    Internal,
}

impl TryFrom<u8> for CommandPermissionLevel {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> anyhow::Result<Self> {
        if value <= 5 {
            Ok(unsafe {
                std::mem::transmute::<u8, CommandPermissionLevel>(value)
            })
        } else {
            anyhow::bail!("Command permission level out of range, expect <=5, got {value}")
        }
    }
}

/// Used for autocompletion.
///
/// This object contains the list of available options.
#[derive(Debug, Clone)]
pub struct CommandEnum {
    /// ID of the autocompleted type.
    /// If the enum is dynamic, this ID can be used in the [`UpdateDynamicEnum`](crate::network::UpdateDynamicEnum)
    /// packet to update the autocompletion options.
    pub enum_id: String,
    /// Available options.
    pub options: Vec<String>,
    /// Whether the server can update this enum after the command has been registered.
    pub dynamic: bool,
}

/// Type of a parameter.
#[derive(Debug, Copy, Clone)]
pub enum CommandDataType {
    /// An integer.
    Int = 1,
    /// A float.
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
    pub data_type: CommandDataType,
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