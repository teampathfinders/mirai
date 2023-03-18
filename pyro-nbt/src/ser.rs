use paste::paste;
use serde::{ser, Serialize};
use std::io::Write;
use std::marker::PhantomData;
use util::bytes::{BinaryWrite, MutableBuffer};
use util::{Error, Result};

use crate::{de, BigEndian, LittleEndian, Variant, VariantImpl};

#[inline]
pub fn to_bytes<T, F>(t: &T) -> Result<MutableBuffer>
where
    T: Serialize,
    F: VariantImpl,
{
    let mut ser = Serializer::<MutableBuffer, F>::new(MutableBuffer::new());
    // t.serialize(&mut ser)?;
    Ok(ser.into_inner())
}

#[derive(Debug)]
pub struct Serializer<W, V>
where
    W: Write,
    V: VariantImpl,
{
    writer: W,
    _marker: PhantomData<V>,
}

impl<W, V> Serializer<W, V>
where
    W: Write,
    V: VariantImpl,
{
    #[inline]
    pub fn new(w: W) -> Serializer<W, V> {
        Serializer { writer: w, _marker: PhantomData }
    }

    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }
}

macro_rules! forward_unsupported {
    ($($ty: ident),+) => {
        paste! {$(
           #[inline]
            fn [<serialize_ $ty>](self, v: $ty) -> util::Result<()> {
                util::bail!(Unsupported, concat!("Serialisation of `", stringify!($ty), "` is not supported"));
            }
        )+}
    }
}

impl<'a, W, V> ser::Serializer for &'a mut Serializer<W, V>
where
    V: VariantImpl,
    W: Write,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    forward_unsupported!(char, u8, u16, u32, u64, i128);

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.writer.write_bool(v)?;
        Ok(())
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<()> {
        self.writer.write_i8(v)?;
        Ok(())
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<()> {
        match V::AS_ENUM {
            Variant::BigEndian => self.writer.write_i16_be(v),
            Variant::LittleEndian | Variant::Variable => {
                self.writer.write_i16_le(v)
            }
        }
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<()> {
        match V::AS_ENUM {
            Variant::BigEndian => self.writer.write_i32_be(v),
            Variant::LittleEndian => self.writer.write_i32_le(v),
            Variant::Variable => self.writer.write_var_i32(v),
        }
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<()> {
        match V::AS_ENUM {
            Variant::BigEndian => self.writer.write_i64_be(v),
            Variant::LittleEndian => self.writer.write_i64_le(v),
            Variant::Variable => self.writer.write_var_i64(v),
        }
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<()> {
        match V::AS_ENUM {
            Variant::BigEndian => self.writer.write_f32_be(v),
            Variant::LittleEndian | Variant::Variable => {
                self.writer.write_f32_le(v)
            }
        }
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<()> {
        match V::AS_ENUM {
            Variant::BigEndian => self.writer.write_f64_be(v),
            Variant::LittleEndian | Variant::Variable => {
                self.writer.write_f64_le(v)
            }
        }
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<()> {
        match V::AS_ENUM {
            Variant::BigEndian => self.writer.write_u16_be(v.len() as u16),
            Variant::LittleEndian => self.writer.write_u16_le(v.len() as u16),
            Variant::Variable => self.writer.write_var_u32(v.len() as u32),
        }?;

        self.writer.write_all(v.as_bytes())?;
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
