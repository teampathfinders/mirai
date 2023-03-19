use paste::paste;
use std::any::TypeId;
use std::fmt;
use std::io::Read;
use std::marker::PhantomData;

use crate::{
    BigEndian, FieldType, LittleEndian, Variable, Variant, VariantImpl,
};
use serde::de::Unexpected::Seq;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{de, Deserialize};
use util::bytes::{BinaryReader, MutableBuffer, SharedBuffer};
use util::{bail, Error, Result};

/// Verifies that the deserialised type is equal to the expected type.
macro_rules! is_ty {
    ($expected: ident, $actual: expr) => {
        if $actual != FieldType::$expected {
            bail!(
                Malformed,
                "Expected type {:?}, but found {:?}",
                FieldType::$expected,
                $actual
            )
        }
    };
}

/// Returns a `not supported` error.
macro_rules! forward_unsupported {
    ($($ty: ident),+) => {
        paste! {$(
            #[inline]
            fn [<deserialize_ $ty>]<V>(self, visitor: V) -> util::Result<V::Value>
            where
                V: Visitor<'de>
            {
                bail!(Unsupported, concat!("Deserialisation of `", stringify!($ty), "` is not supported"));
            }
        )+}
    }
}

/// NBT deserialiser.
#[derive(Debug)]
pub struct Deserializer<'de, F>
where
    F: VariantImpl,
{
    input: SharedBuffer<'de>,
    next_ty: FieldType,
    is_key: bool,
    _marker: PhantomData<F>,
}

impl<'de, F> Deserializer<'de, F>
where
    F: VariantImpl,
{
    #[inline]
    pub fn new(input: &'de [u8]) -> Self {
        let mut input = SharedBuffer::from(input);
        assert_eq!(input.read_u8().unwrap(), FieldType::Compound as u8);

        let mut de = Deserializer {
            input,
            next_ty: FieldType::Compound,
            is_key: false,
            _marker: PhantomData,
        };

        let _ = de.deserialize_raw_str().unwrap();
        de
    }

    #[inline]
    fn deserialize_raw_str(&mut self) -> Result<&str> {
        let len = match F::AS_ENUM {
            Variant::BigEndian => self.input.read_u16_be()? as u32,
            Variant::LittleEndian => self.input.read_u16_le()? as u32,
            Variant::Variable => self.input.read_var_u32()?,
        };

        let data = self.input.take_n(len as usize)?;
        let str = std::str::from_utf8(data)?;

        Ok(str)
    }
}

#[inline]
fn from_bytes<'a, T, F>(b: &'a [u8]) -> Result<(T, usize)>
where
    T: Deserialize<'a>,
    F: VariantImpl,
{
    let start = b.len();
    let mut deserializer = Deserializer::<F>::new(b);
    let output = T::deserialize(&mut deserializer)?;
    let end = deserializer.input.len();

    Ok((output, start - end))
}

#[inline]
pub fn from_le_bytes<'a, T>(b: &'a [u8]) -> Result<(T, usize)>
where
    T: Deserialize<'a>,
{
    from_bytes::<T, LittleEndian>(b)
}

#[inline]
pub fn from_be_bytes<'a, T>(b: &'a [u8]) -> Result<(T, usize)>
where
    T: Deserialize<'a>,
{
    from_bytes::<T, BigEndian>(b)
}

#[inline]
pub fn from_var_bytes<'a, T>(b: &'a [u8]) -> Result<(T, usize)>
where
    T: Deserialize<'a>,
{
    from_bytes::<T, Variable>(b)
}

impl<'de, 'a, F> de::Deserializer<'de> for &'a mut Deserializer<'de, F>
where
    F: VariantImpl,
{
    type Error = Error;

    forward_unsupported!(char, u8, u16, u32, u64, u128);

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.is_key {
            self.deserialize_str(visitor)
        } else {
            match self.next_ty {
                FieldType::End => bail!(Malformed, "found unexpected end tag"),
                FieldType::Byte => self.deserialize_i8(visitor),
                FieldType::Short => self.deserialize_i16(visitor),
                FieldType::Int => self.deserialize_i32(visitor),
                FieldType::Long => self.deserialize_i64(visitor),
                FieldType::Float => self.deserialize_f32(visitor),
                FieldType::Double => self.deserialize_f64(visitor),
                FieldType::ByteArray => self.deserialize_byte_buf(visitor),
                FieldType::String => self.deserialize_string(visitor),
                FieldType::List => self.deserialize_seq(visitor),
                FieldType::Compound => {
                    let m = self.deserialize_map(visitor);
                    m
                }
                FieldType::IntArray => self.deserialize_seq(visitor),
                FieldType::LongArray => self.deserialize_seq(visitor),
            }
        }
    }

    #[inline]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        is_ty!(Byte, self.next_ty);

        let n = self.input.read_bool()?;
        visitor.visit_bool(n)
    }

    #[inline]
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        is_ty!(Byte, self.next_ty);

        let n = self.input.read_i8()?;
        visitor.visit_i8(n)
    }

    #[inline]
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        is_ty!(Short, self.next_ty);

        let n = match F::AS_ENUM {
            Variant::BigEndian => self.input.read_i16_be(),
            Variant::LittleEndian | Variant::Variable => {
                self.input.read_i16_le()
            }
        }?;

        visitor.visit_i16(n)
    }

    #[inline]
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        is_ty!(Int, self.next_ty);

        let n = match F::AS_ENUM {
            Variant::BigEndian => self.input.read_i32_be(),
            Variant::LittleEndian => self.input.read_i32_le(),
            Variant::Variable => self.input.read_var_i32(),
        }?;

        visitor.visit_i32(n)
    }

    #[inline]
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        is_ty!(Long, self.next_ty);

        let n = match F::AS_ENUM {
            Variant::BigEndian => self.input.read_i64_be(),
            Variant::LittleEndian => self.input.read_i64_le(),
            Variant::Variable => self.input.read_var_i64(),
        }?;

        visitor.visit_i64(n)
    }

    #[inline]
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        is_ty!(Float, self.next_ty);

        let n = match F::AS_ENUM {
            Variant::BigEndian => self.input.read_f32_be(),
            _ => self.input.read_f32_le(),
        }?;

        visitor.visit_f32(n)
    }

    #[inline]
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        is_ty!(Double, self.next_ty);

        let n = match F::AS_ENUM {
            Variant::BigEndian => self.input.read_f64_be(),
            _ => self.input.read_f64_le(),
        }?;

        visitor.visit_f64(n)
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = match F::AS_ENUM {
            Variant::BigEndian => self.input.read_u16_be()? as u32,
            Variant::LittleEndian => self.input.read_u16_le()? as u32,
            Variant::Variable => self.input.read_var_u32()?,
        };

        let data = self.input.take_n(len as usize)?;
        let str = std::str::from_utf8(data)?;

        visitor.visit_str(str)
    }

    #[inline]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        is_ty!(String, self.next_ty);

        let len = match F::AS_ENUM {
            Variant::BigEndian => self.input.read_u16_be()? as u32,
            Variant::LittleEndian => self.input.read_u16_le()? as u32,
            Variant::Variable => self.input.read_var_u32()?,
        };

        let data = self.input.take_n(len as usize)?;
        let string = String::from_utf8(data.to_vec())?;

        visitor.visit_string(string)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(Unsupported, "NBT does not support borrowed byte arrays")
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(Unsupported, "NBT does not support byte buffers")
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // This is only used to represent possibly missing fields.
        // If this code is reached, it means the key was found and the field exists.
        // Therefore this is always some.
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(Unsupported)
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(Unsupported, "NBT does not support unit structs")
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(Unsupported, "NBT does not support newtype structs")
    }

    #[inline]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(0, visitor)
    }

    #[inline]
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let ty = match self.next_ty {
            FieldType::ByteArray => FieldType::Byte,
            FieldType::IntArray => FieldType::Int,
            FieldType::LongArray => FieldType::Long,
            _ => FieldType::try_from(self.input.read_u8()?)?,
        };

        let de = SeqDeserializer::new(self, ty, len as u32)?;
        visitor.visit_seq(de)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(Unsupported, "NBT does not support tuple structs")
    }

    #[inline]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        is_ty!(Compound, self.next_ty);

        let de = MapDeserializer::from(self);
        visitor.visit_map(de)
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(Unsupported, "NBT does not support enums")
    }

    #[inline]
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

#[derive(Debug)]
struct SeqDeserializer<'a, 'de: 'a, F>
where
    F: VariantImpl,
{
    de: &'a mut Deserializer<'de, F>,
    ty: FieldType,
    remaining: u32,
    _marker: PhantomData<F>,
}

impl<'de, 'a, F> SeqDeserializer<'a, 'de, F>
where
    F: VariantImpl,
{
    #[inline]
    pub fn new(
        de: &'a mut Deserializer<'de, F>,
        ty: FieldType,
        expected_len: u32,
    ) -> Result<Self> {
        debug_assert_ne!(ty, FieldType::End);

        // ty is not read in here because the x_array types don't have a type prefix.

        de.next_ty = ty;
        let remaining = match F::AS_ENUM {
            Variant::BigEndian => de.input.read_i32_be()? as u32,
            Variant::LittleEndian => de.input.read_i32_le()? as u32,
            Variant::Variable => de.input.read_var_u32()?,
        };

        if expected_len != 0 && expected_len != remaining {
            bail!(Malformed, "expected sequence of length {expected_len}, got length {remaining}");
        }

        Ok(Self { de, ty, remaining, _marker: PhantomData })
    }
}

impl<'de, 'a, F> SeqAccess<'de> for SeqDeserializer<'a, 'de, F>
where
    F: VariantImpl,
{
    type Error = Error;

    #[inline]
    fn next_element_seed<E>(&mut self, seed: E) -> Result<Option<E::Value>>
    where
        E: DeserializeSeed<'de>,
    {
        if self.remaining > 0 {
            self.remaining -= 1;
            let output = seed.deserialize(&mut *self.de).map(Some);
            self.de.next_ty = self.ty;
            output
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug)]
struct MapDeserializer<'a, 'de: 'a, F>
where
    F: VariantImpl,
{
    de: &'a mut Deserializer<'de, F>,
    _marker: PhantomData<F>,
}

impl<'de, 'a, F> From<&'a mut Deserializer<'de, F>>
    for MapDeserializer<'a, 'de, F>
where
    F: VariantImpl,
{
    #[inline]
    fn from(v: &'a mut Deserializer<'de, F>) -> Self {
        Self { de: v, _marker: PhantomData }
    }
}

impl<'de, 'a, F> MapAccess<'de> for MapDeserializer<'a, 'de, F>
where
    F: VariantImpl,
{
    type Error = Error;

    #[inline]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        self.de.is_key = true;
        self.de.next_ty = FieldType::try_from(self.de.input.read_u8()?)?;

        let r = if self.de.next_ty == FieldType::End {
            Ok(None)
        } else {
            // Reads 0 tag instead of string, because string length starts with 0
            seed.deserialize(&mut *self.de).map(Some)
        };

        self.de.is_key = false;
        r
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        debug_assert_ne!(self.de.next_ty, FieldType::End);
        seed.deserialize(&mut *self.de)
    }
}