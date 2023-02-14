use bytes::BytesMut;
use common::{Encodable, VResult, WriteExtensions};

use super::GamePacket;

/// Connects the client to a Websocket server.
#[derive(Debug, Clone)]
pub struct ConnectAutomationClient<'a> {
    /// URI of the server.
    pub server_uri: &'a str,
}

impl GamePacket for ConnectAutomationClient<'_> {
    const ID: u32 = 0x5f;
}

impl Encodable for ConnectAutomationClient<'_> {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(1 + self.server_uri.len());

        buffer.put_string(self.server_uri);

        Ok(buffer)
    }
}
