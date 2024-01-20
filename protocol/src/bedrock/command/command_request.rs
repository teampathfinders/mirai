use anyhow::anyhow;
use util::{bail, Error, Result};
use util::{BinaryRead, SharedBuffer};
use util::Deserialize;

use crate::bedrock::ConnectedPacket;

/// Command origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandOriginType {
    Player,
    Block,
    MinecartBlock,
    DevConsole,
    Test,
    AutomationPlayer,
    ClientAutomation,
    DedicatedServer,
    Entity,
    Virtual,
    GameArgument,
    EntityServer,
    Precompiled,
    GameDirectorEntityServer,
    Script,
    Executor,
}

impl TryFrom<u32> for CommandOriginType {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> anyhow::Result<Self> {
        Ok(match value {
            0 => Self::Player,
            1 => Self::Block,
            2 => Self::MinecartBlock,
            3 => Self::DevConsole,
            4 => Self::Test,
            5 => Self::AutomationPlayer,
            6 => Self::ClientAutomation,
            7 => Self::DedicatedServer,
            8 => Self::Entity,
            9 => Self::Virtual,
            10 => Self::GameArgument,
            11 => Self::EntityServer,
            12 => Self::Precompiled,
            13 => Self::GameDirectorEntityServer,
            14 => Self::Script,
            15 => Self::Executor,
            _ => return Err(anyhow!("Invalid command origin {value}")),
        })
    }
}

/// Requests execution of a command.
/// Even if the command isn't listed by the [`AvailableCommands`](crate::bedrock::AvailableCommands) packet,
/// the client will still send a request.
#[derive(Debug, Clone)]
pub struct CommandRequest<'a> {
    /// The actual command.
    /// This is a raw string (i.e. "/kill @e[type=cow]")
    pub command: &'a str,
    /// Command origin.
    pub origin: CommandOriginType,
    /// Request ID.
    /// If a command is requested by a websocket server, 
    /// then this ID is used to forward the result to the server instead of the client.
    pub request_id: &'a str,
}

impl<'a> ConnectedPacket for CommandRequest<'a> {
    const ID: u32 = 0x4d;
}

impl<'a> Deserialize<'a> for CommandRequest<'a> {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let command = reader.read_str()?;
        let origin = CommandOriginType::try_from(reader.read_var_u32()?)?;
        reader.advance(16);
        let request_id = reader.read_str()?;

        Ok(Self { command, origin, request_id })
    }
}
