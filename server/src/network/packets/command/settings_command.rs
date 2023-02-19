use bytes::BytesMut;
use common::{Deserialize, ReadExtensions, VResult};
use crate::network::packets::ConnectedPacket;

/// Sent by the client when changing settings that require the execution of commands.
/// For instance, when the showcoordinates game rule is changed.
#[derive(Debug, Clone)]
pub struct SettingsCommand {
    /// Command the client requested to execute.
    pub command: String,
    /// Whether to suppress the output of the command that was executed.
    pub suppress_output: bool
}

impl ConnectedPacket for SettingsCommand {
    const ID: u32 = 0x8c;
}

impl Deserialize for SettingsCommand {
    fn deserialize(mut buffer: BytesMut) -> VResult<Self> {
        let command = buffer.get_string()?;
        let suppress_output = buffer.get_bool();

        Ok(Self {
            command, suppress_output
        })
    }
}