use bytes::{BufMut, BytesMut};

use crate::network::packets::GamePacket;
use crate::network::traits::Encodable;
use crate::util::WriteExtensions;

#[derive(Debug, Copy, Clone)]
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
    /// Displays "Unable to connect to world."
    FailedInvalidTenant,
    FailedVanillaEdu,
    FailedIncompatible,
    FailedServerFull,
    FailedEditorToVanillaMismatch,
    FailedVanillaToEditorMismatch,
}

/// Sends a status update to the client.
#[derive(Debug)]
pub struct PlayStatus {
    /// Status to send to the client.
    pub status: Status,
}

impl GamePacket for PlayStatus {
    const ID: u32 = 0x02;
}

impl Encodable for PlayStatus {
    fn encode(&self) -> anyhow::Result<BytesMut> {
        let mut buffer = BytesMut::with_capacity(4);

        buffer.put_i32(self.status as i32);

        Ok(buffer)
    }
}
