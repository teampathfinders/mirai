use std::ops::Range;

use util::bytes::{BinaryReader, BinaryWrite, MutableBuffer, SharedBuffer};
use util::pyassert;
use util::Result;
use util::{Deserialize, Serialize};

/// Record containing IDs of confirmed packets.
#[derive(Debug, Clone)]
pub enum AckRecord {
    /// A single ID
    Single(u32),
    /// Range of IDs
    Range(Range<u32>),
}

/// Encodes a list of acknowledgement records.
fn encode_records(
    buffer: &mut MutableBuffer,
    records: &[AckRecord],
) -> Result<()> {
    buffer.write_i16_be(records.len() as i16);
    for record in records {
        match record {
            AckRecord::Single(id) => {
                buffer.write_u8(1); // Is single
                buffer.write_u24_le((*id).try_into()?);
            }
            AckRecord::Range(range) => {
                buffer.write_u8(0); // Is range
                buffer.write_u24_le(range.start.try_into()?);
                buffer.write_u24_le(range.end.try_into()?);
            }
        }
    }

    Ok(())
}

/// Decodes a list of acknowledgement records.
fn decode_records(mut buffer: SharedBuffer) -> Result<Vec<AckRecord>> {
    let record_count = buffer.read_u16_be()?;
    let mut records = Vec::with_capacity(record_count as usize);

    for _ in 0..record_count {
        let is_range = buffer.read_u8()? == 0;
        if is_range {
            records.push(AckRecord::Range(
                buffer.read_u24_le()?.into()..buffer.read_u24_le()?.into(),
            ));
        } else {
            records.push(AckRecord::Single(buffer.read_u24_le()?.into()));
        }
    }

    Ok(records)
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

    pub fn serialized_size(&self) -> usize {
        1 + self.records.iter().fold(0, |acc, r| {
            acc + match r {
                AckRecord::Single(_) => 1 + 3,
                AckRecord::Range(_) => 1 + 3 + 3,
            }
        })
    }
}

impl Serialize for Ack {
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_u8(Self::ID);

        encode_records(buffer, &self.records)
    }
}

impl Deserialize<'_> for Ack {
    fn deserialize(mut buffer: SharedBuffer) -> Result<Self> {
        pyassert!(buffer.read_u8()? == Self::ID);

        let records = decode_records(buffer)?;

        Ok(Self { records })
    }
}

/// Notifies the recipient of possibly lost packets.
#[derive(Debug)]
pub struct Nak {
    /// Records containing the missing IDs
    pub records: Vec<AckRecord>,
}

impl Nak {
    /// Unique identifier of this packet.
    pub const ID: u8 = 0xa0;

    pub fn serialized_size(&self) -> usize {
        1 + self.records.iter().fold(0, |acc, r| {
            acc + match r {
                AckRecord::Single(_) => 1 + 3,
                AckRecord::Range(_) => 1 + 3 + 3,
            }
        })
    }
}

impl Serialize for Nak {
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_u8(Self::ID);

        encode_records(buffer, &self.records)
    }
}

impl Deserialize<'_> for Nak {
    fn deserialize(mut buffer: SharedBuffer) -> Result<Self> {
        pyassert!(buffer.read_u8()? == Self::ID);

        let records = decode_records(buffer)?;

        Ok(Self { records })
    }
}
