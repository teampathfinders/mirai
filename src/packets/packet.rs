use bytes::{Buf, BufMut, BytesMut};

use crate::error::VexResult;
use crate::packets::{GAME_PACKET_ID, GamePacket, RequestNetworkSettings};
use crate::raknet::Header;
use crate::raknet::packets::{Decodable, Encodable};
use crate::util::{ReadExtensions, WriteExtensions};
use crate::vex_assert;

#[derive(Debug)]
pub struct Packet<T: GamePacket> {
    header: Header,
    internal_packet: T,
}

impl<T: GamePacket> Packet<T> {
    pub const ID: u8 = 0xfe;

    pub fn new(internal_packet: T) -> Self {
        Self {
            header: Header {
                id: T::ID,
                target_subclient: 0,
                sender_subclient: 0,
            },
            internal_packet,
        }
    }

    pub fn subclients(mut self, sender: u8, target: u8) -> Self {
        self.header.target_subclient = target;
        self.header.sender_subclient = sender;
        self
    }
}

// 0xfe ID
// Batch byte size
// List of header + packet

impl<T: GamePacket + Encodable> Encodable for Packet<T> {
    fn encode(&self) -> VexResult<BytesMut> {
        let mut buffer = BytesMut::new();
        let header = self.header.encode();
        let body = self.internal_packet.encode()?;

        buffer.put_var_u32(header.len() as u32 + body.len() as u32);
        buffer.put(header);
        buffer.put(body);

        Ok(buffer)
    }
}