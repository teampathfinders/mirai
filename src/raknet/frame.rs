use std::io::Read;

use bytes::{Buf, BufMut, BytesMut};

use crate::error::VexResult;
use crate::raknet::packets::{Decodable, Encodable};
use crate::raknet::Reliability;
use crate::util::{ReadExtensions, WriteExtensions};
use crate::vex_assert;

/// Bit flag indicating that the packet is encapsulated in a frame.
pub const FRAME_BIT_FLAG: u8 = 0x80;
/// Bit flag indicating that the packet is fragmented.
pub const COMPOUND_BIT_FLAG: u8 = 0b0001_0000;

/// Contains a set of frames.
#[derive(Debug, Default, Clone)]
pub struct FrameBatch {
    batch_number: u32,
    /// Individual frames
    frames: Vec<Frame>,
}

impl FrameBatch {
    /// Gives a rough estimate of the size of this batch in bytes.
    /// This estimate will always be greater than the actual size of the batch.
    pub fn estimate_size(&self) -> usize {
        std::mem::size_of::<FrameBatch>()
            + self.frames.iter().fold(0, |acc, f| {
            acc + std::mem::size_of::<Frame>() + f.body.len()
        })
    }

    pub fn batch_number(mut self, batch_number: u32) -> Self {
        self.batch_number = batch_number;
        self
    }

    pub fn get_batch_number(&self) -> u32 {
        self.batch_number
    }

    pub fn push(mut self, frame: Frame) -> Self {
        self.frames.push(frame);
        self
    }

    pub fn get_frames(&self) -> &[Frame] {
        &self.frames
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}

impl Decodable for FrameBatch {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        vex_assert!(buffer.get_u8() & 0x80 != 0);

        let batch_number = buffer.get_u24_le();
        let mut frames = Vec::new();

        while buffer.has_remaining() {
            frames.push(Frame::decode(&mut buffer)?);
        }
        assert_eq!(buffer.remaining(), 0);

        Ok(Self {
            batch_number,
            frames,
        })
    }
}

impl Encodable for FrameBatch {
    fn encode(&self) -> VexResult<BytesMut> {
        let mut buffer = BytesMut::new();

        buffer.put_u8(FRAME_BIT_FLAG);
        buffer.put_u24_le(self.batch_number);

        for frame in &self.frames {
            frame.encode(&mut buffer);
        }

        Ok(buffer)
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
    /// Creates a new frame.
    pub fn new(reliability: Reliability, body: BytesMut) -> Self {
        Self {
            reliability,
            body,
            ..Default::default()
        }
    }

    fn decode(buffer: &mut BytesMut) -> VexResult<Self> {
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

        let mut body = BytesMut::with_capacity(length as usize);
        body.resize(length as usize, 0u8);

        let position = buffer.len() - buffer.remaining();
        body.copy_from_slice(&buffer.as_ref()[position..(position + length as usize)]);
        buffer.advance(length as usize);

        let frame = Self {
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
        };

        Ok(frame)
    }

    fn encode(&self, buffer: &mut BytesMut) {
        let reliability = (self.reliability as u8) << 5;
        let mut flags = reliability;
        if self.is_compound {
            flags |= COMPOUND_BIT_FLAG;
        }

        buffer.put_u8(flags);
        buffer.put_u16(self.body.len() as u16 * 8);
        if self.reliability.is_reliable() {
            buffer.put_u24_le(self.reliable_index);
        }
        if self.reliability.is_sequenced() {
            buffer.put_u24_le(self.sequence_index);
        }
        if self.reliability.is_ordered() {
            buffer.put_u24_le(self.order_index);
            buffer.put_u8(self.order_channel);
        }
        if self.is_compound {
            buffer.put_u32(self.compound_size);
            buffer.put_u16(self.compound_id);
            buffer.put_u32(self.compound_index);
        }

        buffer.put(self.body.as_ref());
    }
}
