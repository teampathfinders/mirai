use std::io::Write;

use serde::{Serialize, ser};

use crate::{de::Mode, bytes_mut::WriteBuffer, error::Result, Error, buf_mut::BufMut, TAG_COMPOUND, TAG_END};

pub struct Serializer {
    mode: Mode,
    output: WriteBuffer
}

pub fn to_bytes<T>(value: &T, mode: Mode) -> Result<WriteBuffer> 
where
    T: Serialize
{
    let mut serializer = Serializer {
        mode, output: WriteBuffer::new()
    };
    value.serialize(&mut serializer)?;

    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output.write_bool(v);
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.output.write_i8(v);
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        match self.mode {
            Mode::LittleEndian | Mode::Network => {
                self.output.write_i16_le(v);
            },
            Mode::BigEndian => {
                self.output.write_i16(v);
            }
        }

        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        match self.mode {
            Mode::LittleEndian | Mode::Network => {
                self.output.write_i32_le(v);
            },
            Mode::BigEndian => {
                self.output.write_i32(v);
            }
        }

        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        match self.mode {
            Mode::LittleEndian | Mode::Network => {
                self.output.write_i64_le(v);
            },
            Mode::BigEndian => {
                self.output.write_i64(v);
            }
        }

        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.output.write_u8(v);
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        match self.mode {
            Mode::LittleEndian | Mode::Network => {
                self.output.write_u16_le(v);
            },
            Mode::BigEndian => {
                self.output.write_u16(v);
            }
        }

        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        match self.mode {
            Mode::LittleEndian | Mode::Network => {
                self.output.write_u32_le(v);
            },
            Mode::BigEndian => {
                self.output.write_u32(v);
            }
        }

        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        match self.mode {
            Mode::LittleEndian | Mode::Network => {
                self.output.write_u64_le(v);
            },
            Mode::BigEndian => {
                self.output.write_u64(v);
            }
        }

        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        match self.mode {
            Mode::LittleEndian | Mode::Network => {
                self.output.write_f32_le(v);
            },
            Mode::BigEndian => {
                self.output.write_f32(v);
            }
        }

        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        match self.mode {
            Mode::LittleEndian | Mode::Network => {
                self.output.write_f64_le(v);
            },
            Mode::BigEndian => {
                self.output.write_f64(v);
            }
        }

        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        let mut utf8_buf = [0u8; 4];
        v.encode_utf8(&mut utf8_buf);
        
        self.output.write_all(&utf8_buf[..v.len_utf8()])?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        match self.mode {
            Mode::LittleEndian => {
                self.output.write_u16_le(v.len() as u16);
            },
            Mode::BigEndian => {
                self.output.write_u16(v.len() as u16);
            },
            Mode::Network => {
                todo!();
            }
        }
        self.output.write_all(v.as_bytes())?;

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        todo!()
    }

    fn serialize_none(self) -> Result<()> {
        todo!()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
    where
        T: Serialize {
        todo!()
    }

    fn serialize_unit(self) -> Result<()> {
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize {
        todo!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> std::result::Result<Self, Error> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> std::result::Result<Self, Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self, name: &'static str, len: usize,
    ) -> std::result::Result<Self, Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> std::result::Result<Self, Error> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> std::result::Result<Self, Error> {
        self.output.write_u8(TAG_COMPOUND);
        Ok(self)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> std::result::Result<Self, Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> std::result::Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize 
    {
        todo!();
    }

    fn end(self) -> Result<()> {
        todo!();
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!();
    }

    fn end(self) -> Result<()> {
        todo!();
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!();
    }

    fn end(self) -> Result<()> {
        todo!();
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!();
    }

    fn end(self) -> Result<()> {
        todo!();
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!();
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!();
    }

    fn end(self) -> Result<()> {
        todo!();
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output.write_u8(TAG_END);
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!();
    }

    fn end(self) -> Result<()> {
        todo!();
    }
}

#[cfg(test)]
mod test {
    use serde::Serialize;

    use crate::{ser::to_bytes, de::Mode};

    #[test]
    fn write_compound() {
        #[derive(Serialize)]
        pub struct Compound {
            byte: u8,
            short: u16,
            float: f64
        }

        let compound = Compound {
            byte: 5,
            short: 35974,
            float: 2.5
        };

        let buffer = to_bytes(&compound, Mode::LittleEndian).unwrap();
        println!("{buffer:?}");
    }
}