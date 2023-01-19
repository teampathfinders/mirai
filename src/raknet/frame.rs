use crate::error::VexResult;
use crate::raknet::packets::{Decodable, Encodable};
use crate::raknet::Reliability;
use bytes::{Buf, BytesMut};
use crate::util::{ReadExtensions};
use crate::vex_assert;

pub const FRAME_BIT_FLAG: u8 = 0x80;

#[derive(Debug)]
pub struct FrameSet {
    pub sequence_number: u32,
    pub frames: Vec<Frame>
}

impl Decodable for FrameSet {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        vex_assert!(buffer.get_u8() & 0x80 != 0);

        let sequence_number = buffer.get_u24_le();
        tracing::info!("sn: {sequence_number}");

        let mut frames = Vec::new();

        // while buffer.has_remaining() {
        //     frames.push(Frame::decode(&mut buffer)?);
        // }

        Ok(Self {
            sequence_number, frames
        })
    }
}

impl Encodable for FrameSet {
    fn encode(&self) -> VexResult<BytesMut> {
        todo!("FrameSet encoder");
    }
}

#[derive(Debug)]
pub struct Frame {
    /// Reliability of this packet.
    pub reliability: Reliability,

    // Fragments =====================
    /// Unique ID of the the compound
    pub compound_id: u16,
    /// Amount of fragments in the compound
    pub compound_size: i32,
    /// Index of the fragment in this compound
    pub compound_index: i32,

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


        todo!("Frame decoder implementation")
    }
}

impl Encodable for Frame {
    fn encode(&self) -> VexResult<BytesMut> {
        todo!("Frame encoder implementation")
    }
}
