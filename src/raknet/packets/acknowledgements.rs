use bytes::{Buf, BufMut, BytesMut};

use crate::error::VexResult;
use crate::raknet::packets::{Decodable, Encodable};
use crate::util::{ReadExtensions, WriteExtensions};
use crate::vex_assert;

/// Record containing IDs of confirmed packets.
#[derive(Debug)]
pub enum AckRecord {
    /// A single ID
    Single(u32),
    /// Range of IDs
    Range(u32, u32),
}

/// Confirms that packets have been received.
#[derive(Debug)]
pub struct Ack {
    /// Records containing IDs of received packets.
    pub records: Vec<AckRecord>,
}

impl Ack {
    /// Unique identifier for this packet.
    pub const ID: u8 = 0xc0;
}

impl Encodable for Ack {
    fn encode(&self) -> VexResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(10);

        buffer.put_u8(Self::ID);
        buffer.put_i16(self.records.len() as i16);
        for record in &self.records {
            match record {
                AckRecord::Single(id) => {
                    buffer.put_u8(1); // Is single
                    buffer.put_u24_le(*id);
                }
                AckRecord::Range(start, end) => {
                    buffer.put_u8(0); // Is range
                    buffer.put_u24_le(*start);
                    buffer.put_u24_le(*end);
                }
            }
        }

        Ok(buffer)
    }
}

impl Decodable for Ack {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        vex_assert!(buffer.get_u8() == Self::ID);

        let record_count = buffer.get_u16();
        let mut records = Vec::with_capacity(record_count as usize);

        for _ in 0..record_count {
            let is_range = buffer.get_u8() == 0;
            if is_range {
                records.push(AckRecord::Single(buffer.get_u24_le()));
            } else {
                records.push(AckRecord::Range(buffer.get_u24_le(), buffer.get_u24_le()));
            }
        }

        Ok(Ack { records })
    }
}

/// Notifiers the recipient of possibly lost packets.
#[derive(Debug)]
pub struct Nack {
    /// Records containing the missing IDs
    pub records: Vec<AckRecord>,
}

impl Nack {
    /// Unique identifier of this packet.
    pub const ID: u8 = 0xa0;
}

impl Encodable for Nack {
    fn encode(&self) -> VexResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(10);

        buffer.put_u8(Self::ID);
        buffer.put_i16(self.records.len() as i16);
        for record in &self.records {
            match record {
                AckRecord::Single(id) => {
                    buffer.put_u8(1); // Is single
                    buffer.put_u24_le(*id);
                }
                AckRecord::Range(start, end) => {
                    buffer.put_u8(0); // Is range
                    buffer.put_u24_le(*start);
                    buffer.put_u24_le(*end);
                }
            }
        }

        Ok(buffer)
    }
}

impl Decodable for Nack {
    fn decode(mut buffer: BytesMut) -> VexResult<Self> {
        vex_assert!(buffer.get_u8() == Self::ID);

        let record_count = buffer.get_u16();
        let mut records = Vec::with_capacity(record_count as usize);

        for _ in 0..record_count {
            let is_range = buffer.get_u8() == 0;
            if is_range {
                records.push(AckRecord::Single(buffer.get_u24_le()));
            } else {
                records.push(AckRecord::Range(buffer.get_u24_le(), buffer.get_u24_le()));
            }
        }

        Ok(Nack { records })
    }
}
