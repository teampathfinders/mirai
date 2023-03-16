
use util::{Deserialize, Result};
use util::bytes::{BinaryReader, SharedBuffer};
use crate::ConnectedPacket;

/// Sent by the client when changing settings that require the execution of commands.
/// For instance, when the showcoordinates game rule is changed.
#[derive(Debug, Clone)]
pub struct SettingsCommand<'a> {
    /// Command the client requested to execute.
    pub command: &'a str,
    /// Whether to suppress the output of the command that was executed.
    pub suppress_output: bool
}

impl<'a> ConnectedPacket for SettingsCommand<'a> {
    const ID: u32 = 0x8c;
}

impl<'a> Deserialize<'a> for SettingsCommand<'a> {
    fn deserialize(mut buffer: SharedBuffer<'a>) -> Result<Self> {
        let command = buffer.read_str()?;
        let suppress_output = buffer.read_bool()?;

        Ok(Self {
            command, suppress_output
        })
    }
}