use bytes::BytesMut;
use common::{Serialize, VResult, WriteExtensions, size_of_var};

use crate::network::packets::GamePacket;

/// Connects the client to a Websocket server.
#[derive(Debug, Clone)]
pub struct ConnectAutomationClient<'a> {
    /// URI of the server.
    pub server_uri: &'a str,
}

impl GamePacket for ConnectAutomationClient<'_> {
    const ID: u32 = 0x5f;
}

impl Serialize for ConnectAutomationClient<'_> {
    fn serialize(&self) -> VResult<BytesMut> {
        let packet_size =
            size_of_var(self.server_uri.len() as u32) +
            self.server_uri.len();

        let mut buffer = BytesMut::with_capacity(packet_size);

        buffer.put_string(self.server_uri);

        Ok(buffer)
    }
}
