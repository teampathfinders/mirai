
use uuid::Uuid;
use util::{Deserialize, Serialize, Result};
use crate::network::packets::{ConnectedPacket};

use super::CommandOriginType;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CommandOutputType {
    None,
    LastOutput,
    Silent,
    AllOutput,
    DataSet
}

#[derive(Debug, Clone)]
pub struct CommandOutputMessage<'a> {
    pub is_success: bool,
    pub message: &'a str,
    pub parameters: &'a [String]
}

#[derive(Debug, Clone)]
pub struct CommandOutput<'a> {
    pub origin: CommandOriginType,
    pub request_id: &'a str,
    pub output_type: CommandOutputType,
    pub success_count: u32,
    pub output: &'a [CommandOutputMessage<'a>]
}

impl ConnectedPacket for CommandOutput<'_> {
    const ID: u32 = 0x4f;
}

impl Serialize for CommandOutput<'_> {
    fn serialize(&self, buffer: &mut OwnedBuffer) {
        buffer.write_var_u32(self.origin as u32);
        buffer.put_uuid(&Uuid::nil());
        buffer.write_str(self.request_id);

        match self.origin {
            CommandOriginType::Test | CommandOriginType::DevConsole => {
                buffer.write_var_i64(0);
            },
            _ => ()
        }

        buffer.write_u8(self.output_type as u8);
        buffer.write_var_u32(self.success_count);

        buffer.write_var_u32(self.output.len() as u32);
        for output in self.output {
            buffer.write_bool(output.is_success);
            buffer.write_str(output.message);

            buffer.write_var_u32(output.parameters.len() as u32);
            for param in output.parameters {
                buffer.write_str(param);
            }
        }

        if self.output_type == CommandOutputType::DataSet {
            unimplemented!();
        }
    }
}