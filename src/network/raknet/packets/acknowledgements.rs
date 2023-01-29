use std::ops::Range;

use bytes::{Buf, BufMut, BytesMut};

use crate::network::traits::{Decodable, Encodable};
use crate::util::{ReadExtensions, WriteExtensions};
use crate::vex_assert;

/// Record containing IDs of confirmed packets.
#[derive(Debug)]
pub enum AcknowledgementRecord {
    /// A single ID
    Single(u32),
    /// Range of IDs
    Range(Range<u32>),
}

/// Confirms that packets have been received.
#[derive(Debug)]
pub struct Acknowledgement {
    /// Records containing IDs of received packets.
    pub records: Vec<AcknowledgementRecord>,
}

impl Acknowledgement {
    /// Unique identifier for this packet.
    pub const ID: u8 = 0xc0;
}

impl Encodable for Acknowledgement {
    fn encode(&self) -> anyhow::Result<BytesMut> {
        let mut buffer = BytesMut::with_capacity(10);

        buffer.put_u8(Self::ID);
        buffer.put_i16(self.records.len() as i16);
        for record in &self.records {
            match record {
                AcknowledgementRecord::Single(id) => {
                    buffer.put_u8(1); // Is single
                    buffer.put_u24_le(*id);
                }
                AcknowledgementRecord::Range(range) => {
                    buffer.put_u8(0); // Is range
                    buffer.put_u24_le(range.start);
                    buffer.put_u24_le(range.end);
                }
            }
        }

        Ok(buffer)
    }
}

impl Decodable for Acknowledgement {
    fn decode(mut buffer: BytesMut) -> anyhow::Result<Self> {
        vex_assert!(buffer.get_u8() == Self::ID);

        let record_count = buffer.get_u16();
        let mut records = Vec::with_capacity(record_count as usize);

        for _ in 0..record_count {
            let is_range = buffer.get_u8() == 0;
            if is_range {
                records.push(AcknowledgementRecord::Range(buffer.get_u24_le()..buffer.get_u24_le()));
            } else {
                records.push(AcknowledgementRecord::Single(buffer.get_u24_le()));
            }
        }

        Ok(Self { records })
    }
}

/// Notifiers the recipient of possibly lost packets.
#[derive(Debug)]
pub struct NegativeAcknowledgement {
    /// Records containing the missing IDs
    pub records: Vec<AcknowledgementRecord>,
}

impl NegativeAcknowledgement {
    /// Unique identifier of this packet.
    pub const ID: u8 = 0xa0;
}

impl Encodable for NegativeAcknowledgement {
    fn encode(&self) -> anyhow::Result<BytesMut> {
        let mut buffer = BytesMut::with_capacity(10);

        buffer.put_u8(Self::ID);
        buffer.put_i16(self.records.len() as i16);
        for record in &self.records {
            match record {
                AcknowledgementRecord::Single(id) => {
                    buffer.put_u8(1); // Is single
                    buffer.put_u24_le(*id);
                }
                AcknowledgementRecord::Range(range) => {
                    buffer.put_u8(0); // Is range
                    buffer.put_u24_le(range.start);
                    buffer.put_u24_le(range.end);
                }
            }
        }

        Ok(buffer)
    }
}

impl Decodable for NegativeAcknowledgement {
    fn decode(mut buffer: BytesMut) -> anyhow::Result<Self> {
        vex_assert!(buffer.get_u8() == Self::ID);

        let record_count = buffer.get_u16();
        let mut records = Vec::with_capacity(record_count as usize);

        for _ in 0..record_count {
            let is_range = buffer.get_u8() == 0;
            if is_range {
                records.push(AcknowledgementRecord::Single(buffer.get_u24_le()));
            } else {
                records.push(AcknowledgementRecord::Range(buffer.get_u24_le()..buffer.get_u24_le()));
            }
        }

        Ok(Self { records })
    }
}
