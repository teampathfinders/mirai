use crate::raknet::FRAME_BIT_FLAG;
use bytes::BytesMut;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct RawPacket {
    pub buffer: BytesMut,
    pub address: SocketAddr,
}

impl RawPacket {
    #[inline]
    pub fn is_offline_packet(&self) -> bool {
        self.buffer
            .first()
            .map_or(false, |f| f & FRAME_BIT_FLAG == 0)
    }

    #[inline]
    pub fn packet_id(&self) -> Option<u8> {
        self.buffer.first().map(|i| *i)
    }
}
