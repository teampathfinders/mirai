use util::{Serialize};
use util::{BinaryWrite, size_of_varint};

use crate::bedrock::ConnectedPacket;

/// Connects the client to a Websocket server.
#[derive(Debug, Clone)]
pub struct ConnectAutomationClient<'a> {
    /// URI of the server.
    pub server_uri: &'a str,
}

impl<'a> ConnectedPacket for ConnectAutomationClient<'a> {
    const ID: u32 = 0x5f;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.server_uri.len() as u32) + self.server_uri.len()
    }
}

impl<'a> Serialize for ConnectAutomationClient<'a> {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_str(self.server_uri)
    }
}
