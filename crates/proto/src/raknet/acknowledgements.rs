use std::ops::Range;

use util::{BinaryRead, BinaryWrite};
use util::iassert;

use util::{Deserialize, Serialize};

/// Record containing IDs of confirmed raknet.
#[derive(Debug, Clone)]
pub enum AckEntry {
    /// A single ID
    Single(u32),
    /// Range of IDs
    Range(Range<u32>),
}

/// Encodes a list of acknowledgement records.
fn serialize_records<W: BinaryWrite>(writer: &mut W, records: &[AckEntry]) -> anyhow::Result<()> {
    writer.write_i16_be(records.len() as i16)?;
    for record in records {
        match record {
            AckEntry::Single(id) => {
                writer.write_u8(1)?; // Is single
                writer.write_u24_le(*id)?;
            }
            AckEntry::Range(range) => {
                writer.write_u8(0)?; // Is range
                writer.write_u24_le(range.start)?;
                writer.write_u24_le(range.end)?;
            }
        }
    }

    Ok(())
}

/// Decodes a list of acknowledgement records.
fn deserialize_records<'a, R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Vec<AckEntry>> {
    let record_count = reader.read_u16_be()?;
    let mut records = Vec::with_capacity(record_count as usize);

    for _ in 0..record_count {
        let is_range = reader.read_u8()? == 0;
        if is_range {
            let min = reader.read_u24_le()?;
            let max = reader.read_u24_le()?;

            records.push(AckEntry::Range(min..max));
        } else {
            records.push(AckEntry::Single(reader.read_u24_le()?));
        }
    }

    Ok(records)
}

/// Confirms that raknet have been received.
#[derive(Debug)]
pub struct Ack {
    /// Records containing IDs of received raknet.
    pub records: Vec<AckEntry>,
}

impl Ack {
    /// Unique identifier for this packet.
    pub const ID: u8 = 0xc0;

    pub fn serialized_size(&self) -> usize {
        1 + self.records.iter().fold(0, |acc, r| {
            acc + match r {
                AckEntry::Single(_) => 1 + 3,
                AckEntry::Range(_) => 1 + 3 + 3,
            }
        })
    }
}

impl Serialize for Ack {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u8(Self::ID)?;

        serialize_records(writer, &self.records)
    }
}

impl<'a> Deserialize<'a> for Ack {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        iassert!(reader.read_u8()? == Self::ID);

        let records = deserialize_records(reader)?;

        Ok(Self { records })
    }
}

/// Notifies the recipient of possibly lost raknet.
#[derive(Debug)]
pub struct Nak {
    /// Records containing the missing IDs
    pub records: Vec<AckEntry>,
}

impl Nak {
    /// Unique identifier of this packet.
    pub const ID: u8 = 0xa0;

    /// Estimates the size of the packet when serialized.
    pub fn serialized_size(&self) -> usize {
        1 + self.records.iter().fold(0, |acc, r| {
            acc + match r {
                AckEntry::Single(_) => 1 + 3,
                AckEntry::Range(_) => 1 + 3 + 3,
            }
        })
    }
}

impl Serialize for Nak {
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u8(Self::ID)?;

        serialize_records(writer, &self.records)
    }
}

impl<'a> Deserialize<'a> for Nak {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        iassert!(reader.read_u8()? == Self::ID);

        let records = deserialize_records(reader)?;

        Ok(Self { records })
    }
}
