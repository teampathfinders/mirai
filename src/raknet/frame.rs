use std::io::Read;

use bytes::{Buf, BufMut, BytesMut};

use crate::error::VexResult;
use crate::raknet::packets::{Decodable, Encodable};
use crate::raknet::Reliability;
use crate::util::ReadExtensions;
use crate::vex_assert;

/// Bit flag indicating that the packet is encapsulated in a frame.
pub const FRAME_BIT_FLAG: u8 = 0x80;
/// Bit flag indicating that the packet is fragmented.
pub const COMPOUND_BIT_FLAG: u8 = 0b0001;

/// Contains a set of frames.
#[derive(Debug)]
pub struct FrameBatch {
    pub sequence_number: u32,
    /// Individual frames
    pub frames: Vec<Frame>,
}

impl Decodable for FrameBatch {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        vex_assert!(buffer.get_u8() & 0x80 != 0);

        let sequence_number = buffer.get_u24_le();
        let mut frames = Vec::new();

        while buffer.has_remaining() {
            frames.push(Frame::decode(&mut buffer)?);
        }

        Ok(Self {
            sequence_number,
            frames,
        })
    }
}

impl Encodable for FrameBatch {
    fn encode(&self) -> VexResult<BytesMut> {
        todo!("FrameSet encoder");
    }
}

/// Encapsulates game packets.
///
/// A frame provides extra metadata for the Raknet reliability layer.
#[derive(Debug, Default, Clone)]
pub struct Frame {
    /// Reliability of this packet.
    pub reliability: Reliability,

    pub reliable_index: u32,
    pub sequence_index: u32,

    // Fragments =====================
    /// Whether the frame is fragmented/compounded
    pub is_compound: bool,
    /// Unique ID of the the compound
    pub compound_id: u16,
    /// Amount of fragments in the compound
    pub compound_size: u32,
    /// Index of the fragment in this compound
    pub compound_index: u32,

    // Ordering ======================
    /// Index in the order channel
    pub order_index: u32,
    /// Channel to perform ordering in
    pub order_channel: u8,

    /// Raw bytes of the body.
    pub body: BytesMut,
}

impl Frame {
    fn decode(buffer: &mut BytesMut) -> VexResult<Self>
    where
        Self: Sized,
    {
        let flags = buffer.get_u8();

        let reliability = Reliability::try_from(flags >> 5)?;
        let is_compound = flags & COMPOUND_BIT_FLAG != 0;
        let length = buffer.get_u16() / 8;

        let mut reliable_index = 0;
        if reliability.is_reliable() {
            reliable_index = buffer.get_u24_le();
        }

        let mut sequence_index = 0;
        if reliability.is_sequenced() {
            sequence_index = buffer.get_u24_le();
        }

        let mut order_index = 0;
        let mut order_channel = 0;
        if reliability.is_ordered() {
            order_index = buffer.get_u24_le();
            order_channel = buffer.get_u8();
        }

        let mut compound_size = 0;
        let mut compound_id = 0;
        let mut compound_index = 0;
        if is_compound {
            compound_size = buffer.get_u32();
            compound_id = buffer.get_u16();
            compound_index = buffer.get_u32();
        }

        let mut chunks = buffer.chunks(length as usize);
        let body = BytesMut::from(chunks.next().unwrap());

        buffer.advance(length as usize);

        Ok(Self {
            reliability,
            reliable_index,
            sequence_index,
            is_compound,
            compound_id,
            compound_size,
            compound_index,
            order_index,
            order_channel,
            body,
        })
    }
}

impl Encodable for Frame {
    fn encode(&self) -> VexResult<BytesMut> {
        todo!("Frame encoder implementation")
    }
}
