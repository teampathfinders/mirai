use std::net::SocketAddr;

use bytes::BytesMut;

use crate::raknet::FRAME_BIT_FLAG;

/// Raw byte data received directly from the UDP socket.
#[derive(Debug)]
pub struct RawPacket {
    /// Data contained in the packet
    pub buffer: BytesMut,
    /// IP address of the sender or recipient
    pub address: SocketAddr,
}

impl RawPacket {
    /// Checks whether this frame is encapsulated in a [`Frame`](crate::raknet::Frame).
    #[inline]
    pub fn is_offline_packet(&self) -> bool {
        self.buffer
            .first()
            .map_or(false, |f| f & FRAME_BIT_FLAG == 0)
    }

    /// Returns the ID of this packet.
    ///
    /// If the packet is encapsulated, this will always return a frame ID in the range 0x80 to 0x8d.
    /// When the packet is not encapsulated, the actual packet ID will be used.
    ///
    /// So this should generally only be used for packets that are not encapsulated.
    #[inline]
    pub fn packet_id(&self) -> Option<u8> {
        self.buffer.first().copied()
    }
}
