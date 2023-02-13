use bytes::BytesMut;
use common::{Encodable, VResult, WriteExtensions};

use super::GamePacket;

/// Connects the client to a Websocket server.
#[derive(Debug)]
pub struct ConnectAutomationClient {
    /// URI of the server.
    pub server_uri: String,
}

impl GamePacket for ConnectAutomationClient {
    const ID: u32 = 0x5f;
}

impl Encodable for ConnectAutomationClient {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(1 + self.server_uri.len());

        buffer.put_string(&self.server_uri);

        Ok(buffer)
    }
}
