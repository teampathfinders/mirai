use std::net::SocketAddr;
use util::bytes::{MutableBuffer, SharedBuffer};


use crate::network::raknet::CONNECTED_PEER_BIT_FLAG;

/// Raw byte data received directly from the UDP socket.
#[derive(Debug)]
pub struct SharedBufPacket<'a> {
    /// Data contained in the packet
    pub buf: SharedBuffer<'a>,
    /// IP address of the sender or recipient
    pub addr: SocketAddr,
}

impl<'a> SharedBufPacket<'a> {
    /// Checks whether this frame is encapsulated in a [`Frame`](super::frame::Frame).
    #[inline]
    pub fn is_unconnected(&self) -> bool {
        self.buf
            .first()
            .map_or(false, |f| f & CONNECTED_PEER_BIT_FLAG == 0)
    }

    /// Returns the ID of this packet.
    ///
    /// If the packet is encapsulated, this will always return a frame ID in the range 0x80 to 0x8d.
    /// When the packet is not encapsulated, the actual packet ID will be used.
    ///
    /// So this should generally only be used for packets that are not encapsulated.
    #[inline]
    pub fn packet_id(&self) -> Option<u8> {
        self.buf.first().copied()
    }
}


#[derive(Debug)]
pub struct OwnedBufPacket {
    pub buf: MutableBuffer,
    pub addr: SocketAddr
}

impl OwnedBufPacket {
    /// Checks whether this frame is encapsulated in a [`Frame`](super::frame::Frame).
    #[inline]
    pub fn is_unconnected(&self) -> bool {
        self.buf
            .first()
            .map_or(false, |f| f & CONNECTED_PEER_BIT_FLAG == 0)
    }

    /// Returns the ID of this packet.
    ///
    /// If the packet is encapsulated, this will always return a frame ID in the range 0x80 to 0x8d.
    /// When the packet is not encapsulated, the actual packet ID will be used.
    ///
    /// So this should generally only be used for packets that are not encapsulated.
    #[inline]
    pub fn packet_id(&self) -> Option<u8> {
        self.buf.first().copied()
    }
}
