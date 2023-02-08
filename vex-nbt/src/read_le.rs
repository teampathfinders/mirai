use std::collections::HashMap;
use bytes::{Buf, Bytes};
use crate::{Error, OwnedTag, RefTag, TAG_BYTE, TAG_BYTE_ARRAY, TAG_COMPOUND, TAG_DOUBLE, TAG_END, TAG_FLOAT, TAG_INT, TAG_INT_ARRAY, TAG_LIST, TAG_LONG, TAG_LONG_ARRAY, TAG_SHORT, TAG_STRING, Value};

impl OwnedTag {
    /// Decodes an NBT structure from a vector of bytes (little endian).
    pub fn decode_le<V: Into<Vec<u8>>>(stream: V) -> Result<Self, Error> {
        let stream = Bytes::from(stream.into());
        Self::decode_with_le(stream)
    }

    /// Decodes an NBT structure from a given byte stream (little endian).
    pub fn decode_with_le(mut stream: Bytes) -> Result<Self, Error> {
        let (name, value) = Value::decode_tag_le(&mut stream)?;

        Ok(Self {
            name, value
        })
    }
}

impl Value {
    fn decode_tag_le(stream: &mut Bytes) -> Result<(String, Self), Error> {
        let tag_type = stream.get_u8();
        if tag_type == TAG_END {
            return Ok((String::new(), Self::End))
        }

        let name = Self::decode_tag_name_le(stream);
        let value = Self::decode_tag_value_le(stream, tag_type)?;

        Ok((name, value))
    }

    fn decode_tag_name_le(stream: &mut Bytes) -> String {
        let length = stream.get_u16_le();
        let cursor = stream.len() - stream.remaining();

        let name = String::from_utf8_lossy(
            &stream.as_ref()[cursor..cursor + length as usize]
        ).to_string();

        stream.advance(length as usize);

        name
    }

    fn decode_tag_value_le(stream: &mut Bytes, tag_type: u8) -> Result<Self, Error> {
        Ok(match tag_type {
            TAG_END => Self::End,
            TAG_BYTE => {
                let value = stream.get_i8();
                Self::Byte(value)
            },
            TAG_SHORT => {
                let value = stream.get_i16_le();
                Self::Short(value)
            },
            TAG_INT => {
                let value = stream.get_i32_le();
                Self::Int(value)
            },
            TAG_LONG => {
                let value = stream.get_i64_le();
                Self::Long(value)
            },
            TAG_FLOAT => {
                let value = stream.get_f32_le();
                Self::Float(value)
            },
            TAG_DOUBLE => {
                let value = stream.get_f64_le();
                Self::Double(value)
            },
            TAG_STRING => {
                let length = stream.get_u16_le();
                let cursor = stream.len() - stream.remaining();

                let string = String::from_utf8_lossy(
                    &stream.as_ref()[cursor..cursor + length as usize]
                ).to_string();

                stream.advance(length as usize);

                Self::String(string)
            },
            TAG_LIST => {
                let list_type = stream.get_u8();
                let length = stream.get_i32_le();

                let mut list = Vec::with_capacity(length as usize);
                for _ in 0..length {
                    list.push(Self::decode_tag_value_le(stream, list_type)?);
                }

                Self::List(list)
            },
            TAG_COMPOUND => {
                let mut map = HashMap::new();
                let mut tag;

                loop {
                    tag = Self::decode_tag_le(stream)?;
                    if tag.1 == Self::End {
                        break
                    }

                    map.insert(tag.0, tag.1);
                }

                Self::Compound(map)
            },
            TAG_BYTE_ARRAY => {
                let length = stream.get_i32_le();
                let mut list = Vec::with_capacity(length as usize);

                for _ in 0..length {
                    list.push(stream.get_i8());
                }

                Self::ByteArray(list)
            },
            TAG_INT_ARRAY => {
                let length = stream.get_i32_le();
                let mut list = Vec::with_capacity(length as usize);

                for _ in 0..length {
                    list.push(stream.get_i32());
                }

                Self::IntArray(list)
            },
            TAG_LONG_ARRAY => {
                let length = stream.get_i32_le();
                let mut list = Vec::with_capacity(length as usize);

                for _ in 0..length {
                    list.push(stream.get_i64());
                }

                Self::LongArray(list)
            }
            _ => return Err(Error::InvalidTag(tag_type))
        })
    }
}