use std::io::Write;
use crate::Reliability;
use util::pyassert;
use util::Result;
use util::{Deserialize, Serialize};
use util::bytes::{BinaryReader, BinaryWrite, MutableBuffer, SharedBuffer};

/// Bit flag indicating that the packet is encapsulated in a frame.
pub const CONNECTED_PEER_BIT_FLAG: u8 = 0x80;
/// Set if the packet is an acknowledgement.
pub const ACK_BIT_FLAG: u8 = 0x40;
/// Set if the packet is a negative acknowledgement.
pub const NACK_BIT_FLAG: u8 = 0x20;
/// Set when the packet is a compound.
pub const COMPOUND_BIT_FLAG: u8 = 0x10;
/// Unknown what this is.
/// Possibly used for Raknet congestion control.
pub const CONTINUOUS_SEND_BIT_FLAG: u8 = 0x08;
/// Unknown what this is.
/// Possibly used for Raknet congestion control.
pub const NEEDS_B_AND_AS_BIT_FLAG: u8 = 0x04;

/// Contains a set of frames.
#[derive(Debug, Default)]
pub struct FrameBatch {
    /// Unique ID of this frame batch.
    pub sequence_number: u32,
    /// Individual frames
    pub frames: Vec<Frame>,
}

impl FrameBatch {
    /// Gives a rough estimate of the size of this batch in bytes.
    /// This estimate will always be greater than the actual size of the batch.
    #[inline]
    pub fn estimate_size(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.frames.iter().fold(0, |acc, f| {
                acc + std::mem::size_of::<Frame>() + f.body.len()
            })
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn deserialize(mut buffer: SharedBuffer) -> Result<Self> {
        debug_assert_ne!(buffer.read_u8()? & 0x80, 0);

        let batch_number = buffer.read_u24_le()?;
        let mut frames = Vec::new();

        while !buffer.is_empty() {
            frames.push(Frame::deserialize(&mut buffer)?);
        }
        debug_assert_eq!(buffer.len(), 0);

        Ok(Self { sequence_number: batch_number.into(), frames })
    }

    pub fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_u8(CONNECTED_PEER_BIT_FLAG)?;
        buffer.write_u24_le(self.sequence_number.try_into()?)?;

        for frame in &self.frames {
            frame.serialize(buffer)?;
        }

        Ok(())
    }
}

/// Encapsulates game packets.
///
/// A frame provides extra metadata for the Raknet reliability layer.
#[derive(Debug, Default)]
pub struct Frame {
    /// Reliability of this packet.
    pub reliability: Reliability,

    pub reliable_index: u32,
    pub sequence_index: u32,
    /// Whether the frame is fragmented/compounded
    pub is_compound: bool,
    /// Unique ID of the the compound
    pub compound_id: u16,
    /// Amount of fragments in the compound
    pub compound_size: u32,
    /// Index of the fragment in this compound
    pub compound_index: u32,
    /// Index in the order channel
    pub order_index: u32,
    /// Channel to perform ordering in
    pub order_channel: u8,
    /// Raw bytes of the body.
    pub body: MutableBuffer,
}

impl Frame {
    /// Creates a new frame.
    pub fn new(reliability: Reliability, body: MutableBuffer) -> Self {
        Self { reliability, body, ..Default::default() }
    }

    /// Decodes the frame.
    #[allow(clippy::useless_let_if_seq)]
    fn deserialize(buffer: &mut SharedBuffer) -> Result<Self> {
        let flags = buffer.read_u8()?;

        let reliability = Reliability::try_from(flags >> 5)?;
        let is_compound = flags & COMPOUND_BIT_FLAG != 0;
        let length = buffer.read_u16_be()? / 8;

        let mut reliable_index = 0;
        if reliability.is_reliable() {
            reliable_index = buffer.read_u24_le()?.into();
        }

        let mut sequence_index = 0;
        if reliability.is_sequenced() {
            sequence_index = buffer.read_u24_le()?.into();
        }

        let mut order_index = 0;
        let mut order_channel = 0;
        if reliability.is_ordered() {
            order_index = buffer.read_u24_le()?.into();
            order_channel = buffer.read_u8()?;
        }

        let mut compound_size = 0;
        let mut compound_id = 0;
        let mut compound_index = 0;
        if is_compound {
            compound_size = buffer.read_u32_be()?;
            compound_id = buffer.read_u16_be()?;
            compound_index = buffer.read_u32_be()?;
        }

        let body = MutableBuffer::from(buffer.take_n(length as usize)?.to_vec());
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

    /// Encodes the frame.
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        let mut flags = (self.reliability as u8) << 5;
        if self.is_compound {
            flags |= COMPOUND_BIT_FLAG;
        }

        buffer.write_u8(flags)?;
        buffer.write_u16_be(self.body.len() as u16 * 8)?;
        if self.reliability.is_reliable() {
            buffer.write_u24_le(self.reliable_index.try_into()?)?;
        }
        if self.reliability.is_sequenced() {
            buffer.write_u24_le(self.sequence_index.try_into()?)?;
        }
        if self.reliability.is_ordered() {
            buffer.write_u24_le(self.order_index.try_into()?)?;
            buffer.write_u8(self.order_channel)?;
        }
        if self.is_compound {
            buffer.write_u32_be(self.compound_size)?;
            buffer.write_u16_be(self.compound_id)?;
            buffer.write_u32_be(self.compound_index)?;
        }

        buffer.write_all(self.body.as_ref())?;
        Ok(())
    }
}
