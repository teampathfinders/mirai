use bytes::{BytesMut, Bytes};
use common::{VResult, WriteExtensions};

use common::Serialize;

use super::ConnectedPacket;

/// Enables or disables the usage of commands.
///
/// If commands are disabled, the client will prevent itself from even sending any.
#[derive(Debug, Clone)]
pub struct SetCommandsEnabled {
    /// Whether commands are enabled.
    pub enabled: bool,
}

impl ConnectedPacket for SetCommandsEnabled {
    const ID: u32 = 0x3b;
}

impl Serialize for SetCommandsEnabled {
    fn serialize(&self) -> VResult<Bytes> {
        let mut buffer = BytesMut::with_capacity(1);

        buffer.put_bool(self.enabled);

        Ok(buffer.freeze())
    }
}
