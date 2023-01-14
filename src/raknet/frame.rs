use bytes::BytesMut;
use crate::raknet::Reliability;

pub struct FragmentInfo {
    /// Unique ID of the compound
    pub compound_id: i16,
    /// Amount of fragments in this compound
    pub compound_size: i32,
    /// Index of the fragment in the compound
    pub index: i32
}

pub struct OrderInfo {
    /// Index in the channel
    pub index: u32,
    /// Channel to put the packet in
    pub channel: u8
}

#[derive(Debug)]
pub struct Frame {
    pub sequence_number: u32,
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

    pub body: BytesMut,
}
