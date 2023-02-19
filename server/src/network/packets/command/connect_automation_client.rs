use bytes::{BytesMut, Bytes};
use common::{Serialize, VResult, WriteExtensions, size_of_var};

use crate::network::packets::ConnectedPacket;

/// Connects the client to a Websocket server.
#[derive(Debug, Clone)]
pub struct ConnectAutomationClient<'a> {
    /// URI of the server.
    pub server_uri: &'a str,
}

impl ConnectedPacket for ConnectAutomationClient<'_> {
    const ID: u32 = 0x5f;
}

impl Serialize for ConnectAutomationClient<'_> {
    fn serialize(&self) -> VResult<Bytes> {
        let packet_size =
            size_of_var(self.server_uri.len() as u32) +
            self.server_uri.len();

        let mut buffer = BytesMut::with_capacity(packet_size);

        buffer.put_string(self.server_uri);

        Ok(buffer.freeze())
    }
}
