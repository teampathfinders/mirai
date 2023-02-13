use bytes::{Buf, BytesMut};
use common::{bail, ReadExtensions, VError, VResult};

use common::Decodable;

use super::GamePacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandOrigin {
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

impl TryFrom<u32> for CommandOrigin {
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

#[derive(Debug)]
pub struct CommandRequest {
    pub command: String,
    pub origin: CommandOrigin,
    pub request_id: String,
}

impl GamePacket for CommandRequest {
    const ID: u32 = 0x4d;
}

impl Decodable for CommandRequest {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        let command = buffer.get_string()?;
        let origin = CommandOrigin::try_from(buffer.get_var_u32()?)?;
        let uuid = buffer.advance(16);
        let request_id = buffer.get_string()?;

        Ok(Self { command, origin, request_id })
    }
}
