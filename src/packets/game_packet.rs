use bytes::{Buf, BufMut, BytesMut};

use crate::error::VexResult;
use crate::packets::{GAME_PACKET_ID, GameDecodable, GameEncodable, GamePacket, RequestNetworkSettings};
use crate::raknet::Header;
use crate::raknet::packets::{Decodable, Encodable};
use crate::vex_assert;

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
            internal_packet: internal_packet,
        }
    }

    pub fn subclients(mut self, sender: u8, target: u8) -> Self {
        self.header.target_subclient = target;
        self.header.sender_subclient = sender;
        self
    }
}

impl<T: GamePacket + GameEncodable> Encodable for Packet<T> {
    fn encode(&self) -> VexResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_u8(Self::ID);
        self.header.encode(&mut buffer);
        self.internal_packet.encode(&mut buffer)?;

        Ok(buffer)
    }
}