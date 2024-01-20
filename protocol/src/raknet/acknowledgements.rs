use std::ops::Range;

use util::{BinaryRead, BinaryWrite, MutableBuffer, SharedBuffer};
use util::iassert;
use util::Result;
use util::{Deserialize, Serialize};

/// Record containing IDs of confirmed raknet.
#[derive(Debug, Clone)]
pub enum AckRecord {
    /// A single ID
    Single(u32),
    /// Range of IDs
    Range(Range<u32>),
}

/// Encodes a list of acknowledgement records.
fn serialize_records<W: BinaryWrite>(writer: &mut W, records: &[AckRecord]) -> anyhow::Result<()> {
    writer.write_i16_be(records.len() as i16)?;
    for record in records {
        match record {
            AckRecord::Single(id) => {
                writer.write_u8(1)?; // Is single
                writer.write_u24_le((*id).try_into()?)?;
            }
            AckRecord::Range(range) => {
                writer.write_u8(0)?; // Is range
                writer.write_u24_le(range.start.try_into()?)?;
                writer.write_u24_le(range.end.try_into()?)?;
            }
        }
    }

    Ok(())
}

/// Decodes a list of acknowledgement records.
fn deserialize_records<'a, R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Vec<AckRecord>> {
    let record_count = reader.read_u16_be()?;
    let mut records = Vec::with_capacity(record_count as usize);

    for _ in 0..record_count {
        let is_range = reader.read_u8()? == 0;
        if is_range {
            records.push(AckRecord::Range(reader.read_u24_le()?.into()..reader.read_u24_le()?.into()));
        } else {
            records.push(AckRecord::Single(reader.read_u24_le()?.into()));
        }
    }

    Ok(records)
}

/// Confirms that raknet have been received.
#[derive(Debug)]
pub struct Ack {
    /// Records containing IDs of received raknet.
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
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_u8(Self::ID)?;

        serialize_records(writer, &self.records)
    }
}

impl<'a> Deserialize<'a> for Nak {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let records = deserialize_records(reader)?;

        Ok(Self { records })
    }
}
