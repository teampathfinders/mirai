use bytes::{Buf, BytesMut, Bytes};
use uuid::Uuid;
use common::{bail, ReadExtensions, VError, VResult, WriteExtensions};

use common::Deserialize;

use crate::network::packets::ConnectedPacket;

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
    type Error = VError;

    fn try_from(value: u32) -> VResult<Self> {
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
            _ => bail!(BadPacket, "Invalid command origin {value}"),
        })
    }
}

/// Requests execution of a command.
/// Even if the command isn't listed by the [`AvailableCommands`](super::AvailableCommands) packet,
/// the client will still send a request.
#[derive(Debug, Clone)]
pub struct CommandRequest {
    /// The actual command.
    /// This is a raw string (i.e. "/kill @e[type=cow]")
    pub command: String,
    /// Command origin.
    pub origin: CommandOriginType,
    /// Request ID.
    /// If a command is requested by a websocket server, 
    /// then this ID is used to forward the result to the server instead of the client.
    pub request_id: String,
}

impl ConnectedPacket for CommandRequest {
    const ID: u32 = 0x4d;
}

impl Deserialize for CommandRequest {
    fn deserialize(mut buffer: Bytes) -> VResult<Self> {
        let command = buffer.get_string()?;
        let origin = CommandOriginType::try_from(buffer.get_var_u32()?)?;
        buffer.advance(16);
        let request_id = buffer.get_string()?;

        Ok(Self { command, origin, request_id })
    }
}
