use std::{
    any::{Any, TypeId},
    fmt::format,
    io::Write,
};

use serde::{ser, Serialize};
use util::{bail, Error, Result};
use util::bytes::WriteBuffer;

use crate::{
    bail, de::Flavor, error::Result, Error, TAG_BYTE, TAG_COMPOUND, TAG_DOUBLE,
    TAG_END, TAG_FLOAT, TAG_INT, TAG_LONG, TAG_SHORT, TAG_STRING,
};

pub struct Serializer {
    flavor: Flavor,
    output: WriteBuffer,
}

pub fn to_bytes<T>(value: &T, flavor: Flavor) -> Result<WriteBuffer>
where
    T: Serialize,
{
    let mut serializer = Serializer { flavor, output: WriteBuffer::new() };

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
        self.output.write_be::<bool>(v);
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.output.write_be::<i8>(v);
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        match self.flavor {
            Flavor::LittleEndian | Flavor::Network => {
                self.output.write_i16_le(v);
            }
            Flavor::BigEndian => {
                self.output.write_be::<i16>(v);
            }
        }

        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        match self.flavor {
            Flavor::LittleEndian | Flavor::Network => {
                self.output.write_le::<i32>(v);
            }
            Flavor::BigEndian => {
                self.output.write_be::<i32>(v);
            }
        }

        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        match self.flavor {
            Flavor::LittleEndian | Flavor::Network => {
                self.output.write_le::<i64>(v);
            }
            Flavor::BigEndian => {
                self.output.write_i64_be(v);
            }
        }

        Ok(())
    }

    // NBT does not support unsigned types.
    fn serialize_u8(self, _v: u8) -> Result<()> {
        bail!(Unsupported)
    }

    // NBT does not support unsigned types.
    fn serialize_u16(self, _v: u16) -> Result<()> {
        bail!(Unsupported)
    }

    // NBT does not support unsigned types.
    fn serialize_u32(self, _v: u32) -> Result<()> {
        bail!(Unsupported)
    }

    // NBT does not support unsigned types.
    fn serialize_u64(self, _v: u64) -> Result<()> {
        bail!(Unsupported)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        match self.flavor {
            Flavor::LittleEndian | Flavor::Network => {
                self.output.write_le::<f32>(v);
            }
            Flavor::BigEndian => {
                self.output.write_be::<f32>(v);
            }
        }

        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        match self.flavor {
            Flavor::LittleEndian | Flavor::Network => {
                self.output.write_le::<f64>(v);
            }
            Flavor::BigEndian => {
                self.output.write_be::<f64>(v);
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
        match self.flavor {
            Flavor::LittleEndian => {
                self.output.write_u16_le(v.len() as u16);
            }
            Flavor::BigEndian => {
                self.output.write_u16_be(v.len() as u16);
            }
            Flavor::Network => {
                todo!();
            }
        }
        self.output.write_all(v.as_bytes())?;

        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        todo!()
    }

    fn serialize_none(self) -> Result<()> {
        todo!()
    }

    fn serialize_some<T: ?Sized>(self, _v: &T) -> Result<()>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_unit(self) -> Result<()> {
        todo!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> std::result::std::result::Result<Self, Error> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> std::result::std::result::Result<Self, Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::std::result::Result<Self, Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> std::result::std::result::Result<Self, Error> {
        todo!()
    }

    fn serialize_map(
        self,
        len: Option<usize>,
    ) -> std::result::std::result::Result<Self, Error> {
        Ok(self)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> std::result::std::result::Result<Self, Error> {
        self.output.write_be::<u8>(TAG_COMPOUND);
        // self.output.write_u16_le(name.len() as u16);
        // self.output.write_all(name.as_bytes())?;
        Ok(self)
        // self.serialize_map(Some(len))
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
        T: ?Sized + Serialize,
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
        // Encode the header.
        value.serialize(HeaderSerializer::new(&mut self.output, Some(key)))?;
        // Encode the name
        // key.serialize(&mut **self)?;
        serde::Serializer::serialize_str(&mut **self, key)?;
        // Encode the value
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

// enum DataType {
//     I8,
//     I16,
//     I32,
//     I64,
//     F32,
//     F64,
//     String
// }

// trait ValidTag {
//     // fn ty() -> Option<DataType>;
//     fn id() -> Result<u8>;
//     fn is_primitive() -> bool;
// }

// impl<T: ?Sized + Serialize> ValidTag for T {
//     // default fn ty() -> Option<DataType> { None }
//     default fn id() -> Result<u8> {
//         Err(Error::Unsupported(format!("{} serialization is not supported", std::any::type_name::<T>())))
//     }
//     default fn is_primitive() -> bool { false }
// }

// macro_rules! valid_tag {
//     ($t: ty, $i: ident /*, $d: ident*/) => {
//         impl ValidTag for $t {
//             // fn ty() -> Option<DataType> {
//             //     Some(DataType::$d)
//             // }

//             fn id() -> Result<u8> {
//                 Ok($crate::$i)
//             }

//             fn is_primitive() -> bool {
//                 true
//             }
//         }
//     }
// }

// // valid_tag!(i8, I8);
// // valid_tag!(i16, I16);
// // valid_tag!(i32, I32);
// // valid_tag!(i64, I64);
// // valid_tag!(f32, F32);
// // valid_tag!(f64, F64);
// // valid_tag!(&str, String);
// // valid_tag!(String, String);

// valid_tag!(i8, TAG_BYTE);
// valid_tag!(i16, TAG_SHORT);
// valid_tag!(i32, TAG_INT);
// valid_tag!(i64, TAG_LONG);
// valid_tag!(f32, TAG_FLOAT);
// valid_tag!(f64, TAG_DOUBLE);
// valid_tag!(&str, TAG_STRING);
// valid_tag!(String, TAG_STRING);

struct HeaderSerializer<'a> {
    key: Option<&'static str>,
    // serializer: &'a mut Serializer
    buffer: &'a mut WriteBuffer,
}

impl<'a> HeaderSerializer<'a> {
    pub fn new(buffer: &'a mut WriteBuffer, key: Option<&'static str>) -> Self {
        Self { key, buffer }
    }
}

impl<'a> ser::Serializer for HeaderSerializer<'a> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, _v: bool) -> Result<()> {
        self.buffer.write_u8(TAG_BYTE);
        Ok(())
    }

    fn serialize_i8(self, _v: i8) -> Result<()> {
        self.buffer.write_u8(TAG_BYTE);
        Ok(())
    }

    fn serialize_i16(self, _v: i16) -> Result<()> {
        self.buffer.write_u8(TAG_SHORT);
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        todo!()
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        todo!()
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        todo!()
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        todo!()
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        todo!()
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        todo!()
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        todo!()
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.buffer.write_u8(TAG_DOUBLE);
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        todo!()
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.buffer.write_u8(TAG_STRING);
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
        T: Serialize,
    {
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
        T: Serialize,
    {
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
        T: Serialize,
    {
        todo!()
    }

    fn serialize_seq(
        self,
        len: Option<usize>,
    ) -> std::result::Result<Self::SerializeSeq, Self::Error> {
        todo!()
    }

    fn serialize_tuple(
        self,
        len: usize,
    ) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> std::result::Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(
        self,
        len: Option<usize>,
    ) -> std::result::Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
        todo!()
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

impl<'a> ser::SerializeSeq for HeaderSerializer<'a> {
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

impl<'a> ser::SerializeTuple for HeaderSerializer<'a> {
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

impl<'a> ser::SerializeTupleStruct for HeaderSerializer<'a> {
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

impl<'a> ser::SerializeTupleVariant for HeaderSerializer<'a> {
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

impl<'a> ser::SerializeMap for HeaderSerializer<'a> {
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

impl<'a> ser::SerializeStruct for HeaderSerializer<'a> {
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

impl<'a> ser::SerializeStructVariant for HeaderSerializer<'a> {
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

// struct ValueSerializer<'a> {
//     key: Option<&'static str>,
//     // serializer: &'a mut Serializer
//     buffer: &'a mut WriteBuffer
// }

// impl<'a> HeaderSerializer<'a> {
//     pub fn new(buffer: &'a mut WriteBuffer, key: Option<&'static str>) -> Self {
//         Self {
//             key, buffer
//         }
//     }
// }

// impl<'a> ser::Serializer for HeaderSerializer<'a> {
//     type Ok = ();
//     type Error = Error;

//     type SerializeSeq = Self;
//     type SerializeTuple = Self;
//     type SerializeTupleStruct = Self;
//     type SerializeTupleVariant = Self;
//     type SerializeMap = Self;
//     type SerializeStruct = Self;
//     type SerializeStructVariant = Self;

//     fn serialize_bool(self, _v: bool) -> Result<()> {
//         self.buffer.write_u8(TAG_BYTE);
//         Ok(())
//     }

//     fn serialize_i8(self, _v: i8) -> Result<()> {
//         self.buffer.write_u8(TAG_BYTE);
//         Ok(())
//     }

//     fn serialize_i16(self, _v: i16) -> Result<()> {
//         self.buffer.write_u8(TAG_SHORT);
//         Ok(())
//     }

//     fn serialize_i32(self, v: i32) -> Result<()> {
//         todo!()
//     }

//     fn serialize_i64(self, v: i64) -> Result<()> {
//         todo!()
//     }

//     fn serialize_u8(self, v: u8) -> Result<()> {
//         todo!()
//     }

//     fn serialize_u16(self, v: u16) -> Result<()> {
//         todo!()
//     }

//     fn serialize_u32(self, v: u32) -> Result<()> {
//         todo!()
//     }

//     fn serialize_u64(self, v: u64) -> Result<()> {
//         todo!()
//     }

//     fn serialize_f32(self, v: f32) -> Result<()> {
//         todo!()
//     }

//     fn serialize_f64(self, v: f64) -> Result<()> {
//         self.buffer.write_u8(TAG_DOUBLE);
//         Ok(())
//     }

//     fn serialize_char(self, v: char) -> Result<()> {
//         todo!()
//     }

//     fn serialize_str(self, v: &str) -> Result<()> {
//         self.buffer.write_u8(TAG_STRING);
//         Ok(())
//     }

//     fn serialize_bytes(self, v: &[u8]) -> Result<()> {
//         todo!()
//     }

//     fn serialize_none(self) -> Result<()> {
//         todo!()
//     }

//     fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
//     where
//         T: Serialize {
//         todo!()
//     }

//     fn serialize_unit(self) -> Result<()> {
//         todo!()
//     }

//     fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
//         todo!()
//     }

//     fn serialize_unit_variant(
//         self,
//         name: &'static str,
//         variant_index: u32,
//         variant: &'static str,
//     ) -> Result<()> {
//         todo!()
//     }

//     fn serialize_newtype_struct<T: ?Sized>(
//         self,
//         name: &'static str,
//         value: &T,
//     ) -> Result<()>
//     where
//         T: Serialize {
//         todo!()
//     }

//     fn serialize_newtype_variant<T: ?Sized>(
//         self,
//         name: &'static str,
//         variant_index: u32,
//         variant: &'static str,
//         value: &T,
//     ) -> Result<()>
//     where
//         T: Serialize {
//         todo!()
//     }

//     fn serialize_seq(self, len: Option<usize>) -> std::result::Result<Self::SerializeSeq, Self::Error> {
//         todo!()
//     }

//     fn serialize_tuple(self, len: usize) -> std::result::Result<Self::SerializeTuple, Self::Error> {
//         todo!()
//     }

//     fn serialize_tuple_struct(
//         self,
//         name: &'static str,
//         len: usize,
//     ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
//         todo!()
//     }

//     fn serialize_tuple_variant(
//         self,
//         name: &'static str,
//         variant_index: u32,
//         variant: &'static str,
//         len: usize,
//     ) -> std::result::Result<Self::SerializeTupleVariant, Self::Error> {
//         todo!()
//     }

//     fn serialize_map(self, len: Option<usize>) -> std::result::Result<Self::SerializeMap, Self::Error> {
//         todo!()
//     }

//     fn serialize_struct(
//         self,
//         name: &'static str,
//         len: usize,
//     ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
//         todo!()
//     }

//     fn serialize_struct_variant(
//         self,
//         name: &'static str,
//         variant_index: u32,
//         variant: &'static str,
//         len: usize,
//     ) -> std::result::Result<Self::SerializeStructVariant, Self::Error> {
//         todo!()
//     }
// }

// impl<'a> ser::SerializeSeq for HeaderSerializer<'a> {
//     type Ok = ();
//     type Error = Error;

//     fn serialize_element<T>(&mut self, value: &T) -> Result<()>
//     where
//         T: ?Sized + Serialize
//     {
//         todo!();
//     }

//     fn end(self) -> Result<()> {
//         todo!();
//     }
// }

// impl<'a> ser::SerializeTuple for HeaderSerializer<'a> {
//     type Ok = ();
//     type Error = Error;

//     fn serialize_element<T>(&mut self, value: &T) -> Result<()>
//     where
//         T: ?Sized + Serialize,
//     {
//         todo!();
//     }

//     fn end(self) -> Result<()> {
//         todo!();
//     }
// }

// impl<'a> ser::SerializeTupleStruct for HeaderSerializer<'a> {
//     type Ok = ();
//     type Error = Error;

//     fn serialize_field<T>(&mut self, value: &T) -> Result<()>
//     where
//         T: ?Sized + Serialize,
//     {
//         todo!();
//     }

//     fn end(self) -> Result<()> {
//         todo!();
//     }
// }

// impl<'a> ser::SerializeTupleVariant for HeaderSerializer<'a> {
//     type Ok = ();
//     type Error = Error;

//     fn serialize_field<T>(&mut self, value: &T) -> Result<()>
//     where
//         T: ?Sized + Serialize,
//     {
//         todo!();
//     }

//     fn end(self) -> Result<()> {
//         todo!();
//     }
// }

// impl<'a> ser::SerializeMap for HeaderSerializer<'a> {
//     type Ok = ();
//     type Error = Error;

//     fn serialize_key<T>(&mut self, key: &T) -> Result<()>
//     where
//         T: ?Sized + Serialize,
//     {
//         todo!();
//     }

//     fn serialize_value<T>(&mut self, value: &T) -> Result<()>
//     where
//         T: ?Sized + Serialize,
//     {
//         todo!();
//     }

//     fn end(self) -> Result<()> {
//         todo!();
//     }
// }

// impl<'a> ser::SerializeStruct for HeaderSerializer<'a> {
//     type Ok = ();
//     type Error = Error;

//     fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
//     where
//         T: ?Sized + Serialize,
//     {
//         todo!();
//     }

//     fn end(self) -> Result<()> {
//         todo!();
//     }
// }

// impl<'a> ser::SerializeStructVariant for HeaderSerializer<'a> {
//     type Ok = ();
//     type Error = Error;

//     fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
//     where
//         T: ?Sized + Serialize,
//     {
//         todo!();
//     }

//     fn end(self) -> Result<()> {
//         todo!();
//     }
// }

#[cfg(test)]
mod test {
    use serde::Serialize;

    use crate::{de::Flavor, ser::to_bytes};

    #[test]
    fn write_compound() {
        #[derive(Serialize)]
        pub struct Inner {
            string: &'static str,
        }

        #[derive(Serialize)]
        pub struct OuterStruct {
            byte: i8,
            short: i16,
            float: f64,
            inner: &'static str,
        }

        let compound = OuterStruct {
            byte: 5,
            short: 3974,
            float: 2.5,
            inner: "Hello, World",
        };

        let buffer = to_bytes(&compound, Flavor::LittleEndian).unwrap();
        println!("{buffer:?} {}", String::from_utf8_lossy(buffer.as_ref()));
    }
}
