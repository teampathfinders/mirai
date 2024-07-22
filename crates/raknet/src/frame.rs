use util::{RVec, BinaryRead, BinaryWrite, Deserialize, Serialize};

use crate::Reliability;

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
#[derive(Debug)]
pub struct FrameBatch {
    /// Unique ID of this frame batch.
    pub sequence_number: u32,
    /// Individual frames
    pub frames: Vec<Frame>,
}

impl FrameBatch {
    /// Whether the batch is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}

impl Serialize for FrameBatch {
    #[allow(clippy::unwrap_used)] // Frame size_hint always returns `Some`.
    fn size_hint(&self) -> Option<usize> {
        let hint = self.frames
            .iter()
            .fold(4, |i, f| i + f.size_hint().unwrap());

        Some(hint)
    }

    /// Serializes a batch of frames to a buffer.
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u8(CONNECTED_PEER_BIT_FLAG)?;
        writer.write_u24_le(self.sequence_number)?;

        for frame in &self.frames {
            frame.serialize_into(writer)?;
        }

        Ok(())
    }
}

impl<'a> Deserialize<'a> for FrameBatch {
    /// Deserializes a batch of frames from a buffer.
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        reader.advance(1)?; // Skip batch ID

        let batch_number = reader.read_u24_le()?;
        let mut frames = Vec::new();

        while !reader.eof() {
            frames.push(Frame::deserialize_from(reader)?);
        }

        #[cfg(debug_assertions)]
        if reader.remaining() != 0 {
            tracing::warn!("Not all bytes have been read from the packet buffer");
        }

        Ok(Self { sequence_number: batch_number, frames })
    }
}

/// Encapsulates game raknet.
///
/// A frame provides extra metadata for the Raknet reliability layer.
#[derive(Debug, Clone)]
pub struct Frame {
    /// Reliability of this packet.
    pub reliability: Reliability,
    /// Reliability index.
    pub reliable_index: u32,
    /// Sequence index.
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
    pub body: RVec,
}

impl Frame {
    /// Creates a new frame.
    pub fn new(reliability: Reliability, body: RVec) -> Self {
        Self { 
            reliability, 
            body, 
            reliable_index: 0,
            sequence_index: 0,
            is_compound: false,
            compound_id: 0,
            compound_size: 0,
            compound_index: 0,
            order_channel: 0,
            order_index: 0
        }
    }
}

impl Serialize for Frame {
    fn size_hint(&self) -> Option<usize> {
        // This is a very rough estimate and will always overestimate.
        let hint = std::mem::size_of::<Self>() + self.body.len();
        Some(hint)
    }
    
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        let mut flags = (self.reliability as u8) << 5;
        if self.is_compound {
            flags |= COMPOUND_BIT_FLAG;
        }

        writer.write_u8(flags)?;
        writer.write_u16_be(self.body.len() as u16 * 8)?;
        if self.reliability.is_reliable() {
            writer.write_u24_le(self.reliable_index)?;
        }
        if self.reliability.is_sequenced() {
            writer.write_u24_le(self.sequence_index)?;
        }
        if self.reliability.is_ordered() {
            writer.write_u24_le(self.order_index)?;
            writer.write_u8(self.order_channel)?;
        }
        if self.is_compound {
            writer.write_u32_be(self.compound_size)?;
            writer.write_u16_be(self.compound_id)?;
            writer.write_u32_be(self.compound_index)?;
        }

        writer.write_all(self.body.as_ref())?;
        Ok(())
    }
}

impl<'a> Deserialize<'a> for Frame {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let flags = reader.read_u8()?;

        let reliability = Reliability::try_from(flags >> 5)?;
        let is_compound = flags & COMPOUND_BIT_FLAG != 0;
        let length = reader.read_u16_be()? / 8;

        let reliable_index = if reliability.is_reliable() { reader.read_u24_le()? } else { 0 };
        let sequence_index = if reliability.is_sequenced() { reader.read_u24_le()? } else { 0 };
    
        let order_index = if reliability.is_ordered() { reader.read_u24_le()? } else { 0 };
        let order_channel = if reliability.is_ordered() { reader.read_u8()? } else  { 0 };

        let compound_size = if is_compound { reader.read_u32_be()? } else { 0 };
        let compound_id = if is_compound { reader.read_u16_be()? } else { 0 };
        let compound_index = if is_compound { reader.read_u32_be()? } else { 0 };

        let body = RVec::alloc_from_slice(reader.take_n(length as usize)?);
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
}

impl Default for Frame {
    fn default() -> Frame {
        Frame { 
            reliability: Reliability::Unreliable, 
            body: RVec::alloc(),
            reliable_index: 0,
            sequence_index: 0,
            is_compound: false,
            compound_id: 0,
            compound_size: 0,
            compound_index: 0,
            order_channel: 0,
            order_index: 0
        }
    }
}