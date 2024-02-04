use util::{BinaryWrite, Serialize};

use crate::raknet::{OFFLINE_MESSAGE_DATA, RAKNET_VERSION};

/// Notifies the client that they're using a version of the Raknet protocol that is incompatible
/// with the version used by the server ([`RAKNET_VERSION`]).
///
/// This packet should be sent in response to [`OpenConnectionRequest1`](crate::raknet::OpenConnectionRequest1)
/// if the [`protocol_version`](crate::raknet::OpenConnectionRequest1::protocol_version) field does not match the server's version.
#[derive(Debug)]
pub struct IncompatibleProtocol {
    /// Randomly generated GUID of the server.
    /// Corresponds to the random GUID generated on startup.
    pub server_guid: u64,
}

impl IncompatibleProtocol {
    /// Unique idrentifier of this packet.
    pub const ID: u8 = 0x19;

    pub const fn size_hint(&self) -> usize {
        1 + 1 + 16 + 8
    }
}

impl Serialize for IncompatibleProtocol {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u8(Self::ID)?;
        writer.write_u8(RAKNET_VERSION)?;
        writer.write_all(OFFLINE_MESSAGE_DATA)?;
        writer.write_u64_be(self.server_guid)
    }
}
