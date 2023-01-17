use crate::raknet::Reliability;
use bytes::BytesMut;
use crate::error::VexResult;
use crate::raknet::packets::{Decodable, Encodable};

pub struct FragmentInfo {
    /// Unique ID of the compound
    pub compound_id: i16,
    /// Amount of fragments in this compound
    pub compound_size: i32,
    /// Index of the fragment in the compound
    pub index: i32,
}

pub struct OrderInfo {
    /// Index in the channel
    pub index: u32,
    /// Channel to put the packet in
    pub channel: u8,
}

#[derive(Debug)]
pub struct Frame {
    pub sequence_number: u32,
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

impl Decodable for Frame {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> where Self: Sized {
        todo!("Frame decoder implementation")
    }
}

impl Encodable for Frame {
    fn encode(&self) -> VexResult<BytesMut> {
        todo!("Frame encoder implementation")
    }
}