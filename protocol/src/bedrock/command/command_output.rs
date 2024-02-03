use uuid::Uuid;

use util::{Serialize};
use util::{BinaryWrite};

use crate::bedrock::CommandOriginType;
use crate::bedrock::ConnectedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CommandOutputType {
    None,
    LastOutput,
    Silent,
    AllOutput,
    DataSet,
}

#[derive(Debug, Clone)]
pub struct CommandOutputMessage<'a> {
    /// Whether the execution was a success. This determines whether the message
    /// is white or red.
    pub is_success: bool,
    /// Message to display in the output.
    pub message: &'a str,
    /// Parameters to use in the outputted message.
    pub parameters: &'a [String],
}

/// Returns the output of a command back to the user.
#[derive(Debug, Clone)]
pub struct CommandOutput<'a> {
    /// Origin of the executed command.
    pub origin: CommandOriginType,
    pub request_id: &'a str,
    /// Type of output.
    pub output_type: CommandOutputType,
    /// How many of the executions were successful.
    pub success_count: u32,
    /// Output(s)
    pub output: &'a [CommandOutputMessage<'a>],
}

impl ConnectedPacket for CommandOutput<'_> {
    const ID: u32 = 0x4f;
}

impl Serialize for CommandOutput<'_> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_var_u32(self.origin as u32)?;
        writer.write_uuid_le(&Uuid::nil())?;
        writer.write_str(self.request_id)?;

        match self.origin {
            CommandOriginType::Test | CommandOriginType::DevConsole => {
                writer.write_var_i64(0)?;
            }
            _ => ()
        }

        writer.write_u8(self.output_type as u8)?;
        writer.write_var_u32(self.success_count)?;

        writer.write_var_u32(self.output.len() as u32)?;
        for output in self.output {
            writer.write_bool(output.is_success)?;
            writer.write_str(output.message)?;

            writer.write_var_u32(output.parameters.len() as u32)?;
            for param in output.parameters {
                writer.write_str(param)?;
            }
        }

        if self.output_type == CommandOutputType::DataSet {
            unimplemented!();
        }

        Ok(())
    }
}