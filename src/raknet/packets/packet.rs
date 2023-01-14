use std::net::SocketAddr;
use bytes::BytesMut;

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
            .map_or(false, |f| f & 0x80 == 0)
    }

    #[inline]
    pub fn packet_id(&self) -> Option<u8> {
        self.buffer
            .first()
            .map(|i| *i)
    }
}