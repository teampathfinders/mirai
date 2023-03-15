use bytes::Bytes;
use bytes::{BufMut, BytesMut};
use util::bytes::MutableBuffer;

use crate::network::packets::ConnectedPacket;
use util::Serialize;
use util::Result;
use util::WriteExtensions;

/// Status of the [`PlayStatus`] packet.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Status {
    /// Sent by the server after receiving the [`ClientToServerHandshake`](super::ClientToServerHandshake) packet.
    /// This indicates the client has successfully logged in.
    LoginSuccess,
    /// Displays "Could not connect: Outdated client!"
    FailedClient,
    /// Displays "Could not connect: Outdated server!"
    FailedServer,
    /// Sent after world data to spawn the player.
    PlayerSpawn,
    /// Displays "Unable to connect. You do not have access to this world."
    FailedInvalidTenant,
    /// Displays "The server is not running Minecraft: Education Edition. Failed to connect."
    FailedVanillaEdu,
    /// Displays "The server is running an incompatible edition of Minecraft. Failed to connect."
    FailedIncompatible,
    /// Displays "Wow this server is popular! Check back later to see if space opens up. Server Full."
    FailedServerFull,
    /// Displays "The server is not in Editor Mode. Failed to connect."
    FailedEditorToVanillaMismatch,
}

/// Sends a status update to the client.
#[derive(Debug)]
pub struct PlayStatus {
    /// Status to send to the client.
    pub status: Status,
}

impl ConnectedPacket for PlayStatus {
    const ID: u32 = 0x02;

    fn serialized_size(&self) -> usize {
        4
    }
}

impl Serialize for PlayStatus {
    fn serialize(&self, buffer: &mut MutableBuffer) {
        buffer.write_be(self.status as u32);
    }
}
