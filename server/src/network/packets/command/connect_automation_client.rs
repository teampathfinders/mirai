use bytes::{BytesMut, Bytes};
use common::{Serialize, VResult, WriteExtensions, size_of_varint};

use crate::network::packets::ConnectedPacket;

/// Connects the client to a Websocket server.
#[derive(Debug, Clone)]
pub struct ConnectAutomationClient<'a> {
    /// URI of the server.
    pub server_uri: &'a str,
}

impl ConnectedPacket for ConnectAutomationClient<'_> {
    const ID: u32 = 0x5f;

    fn serialized_size(&self) -> usize {
        size_of_varint(self.server_uri.len() as u32) + self.server_uri.len()
    }
}

impl Serialize for ConnectAutomationClient<'_> {
    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_string(self.server_uri);
    }
}
