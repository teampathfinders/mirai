use bytes::{Buf, BytesMut};

use vex_common::{Decodable, VResult};

use crate::network::Decodable;
use crate::network::packets::GamePacket;
use crate::vassert;

/// Sent by the client in response to a [`ServerToClientHandshake`](super::ServerToClientHandshake)
/// to confirm that encryption is working.
///
/// It has no data.
#[derive(Debug)]
pub struct ClientToServerHandshake;

impl GamePacket for ClientToServerHandshake {
    /// Unique ID of this packet.
    const ID: u32 = 0x04;
}

impl Decodable for ClientToServerHandshake {
    fn decode(mut buffer: BytesMut) -> VResult<Self> {
        Ok(Self)
    }
}
