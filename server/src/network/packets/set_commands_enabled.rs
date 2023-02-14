use bytes::BytesMut;
use common::{VResult, WriteExtensions};

use common::Encodable;

use super::GamePacket;

/// Enables or disables the usage of commands.
///
/// If commands are disabled, the client will prevent itself from even sending any.
#[derive(Debug, Clone)]
pub struct SetCommandsEnabled {
    /// Whether commands are enabled.
    pub enabled: bool,
}

impl GamePacket for SetCommandsEnabled {
    const ID: u32 = 0x3b;
}

impl Encodable for SetCommandsEnabled {
    fn encode(&self) -> VResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_bool(self.enabled);

        Ok(buffer)
    }
}
