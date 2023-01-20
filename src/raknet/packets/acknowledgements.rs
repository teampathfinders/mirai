use bytes::{BufMut, BytesMut};
use crate::error::VexResult;
use crate::raknet::packets::Encodable;

#[derive(Debug)]
pub enum AckRecord {
    Single(u32),
    Range(u32, u32),
}

#[derive(Debug)]
pub struct Ack {
    pub records: Vec<AckRecord>,
}

impl Ack {
    pub const ID: u8 = 0xc0;
}

impl Encodable for Ack {
    fn encode(&self) -> VexResult<BytesMut> {
        let mut buffer = BytesMut::with_capacity(10);

        buffer.put_u8(Self::ID);
        buffer.put_u16(self.records.len() as u16);
        for record in &self.records {
            match record {
                AckRecord::Single(id) => {
                    buffer.put_u8(1); // Is single
                    buffer.put_u24_le(id);
                },
                AckRecord::Range(start, end) => {
                    buffer.put_u8(0); // Is range
                    buffer.put_u24_le(start);
                    buffer.put_u24_le(end);
                }
            }
        }

        Ok(buffer)
    }
}

#[derive(Debug)]
pub struct Nack {
    pub records: Vec<AckRecord>,
}

impl Nack {
    pub const ID: u8 = 0xa0;
}
