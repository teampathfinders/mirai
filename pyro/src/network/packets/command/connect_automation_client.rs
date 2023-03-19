
use util::{Serialize, Result};
use util::bytes::{BinaryWrite, MutableBuffer, size_of_varint};

use crate::ConnectedPacket;

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
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_str(self.server_uri);
        Ok(())
    }
}
