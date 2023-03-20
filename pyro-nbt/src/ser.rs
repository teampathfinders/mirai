use paste::paste;
use serde::de::SeqAccess;
use serde::ser::{
    Impossible, SerializeMap, SerializeSeq, SerializeStruct, SerializeTuple,
};
use serde::{ser, Serialize};
use std::io::Write;
use std::marker::PhantomData;
use util::bytes::{BinaryWrite, MutableBuffer};
use util::{bail, Error, Result};

use crate::{
    de, BigEndian, FieldType, LittleEndian, Variable, Variant, VariantImpl,
};

/// Serializes the given data in big endian format.
///
/// This is the format used by Minecraft: Java Edition.
///
/// See [`to_be_bytes_in`] for an alternative that serializes into the given writer, instead
/// of producing a new one.
#[inline]
pub fn to_be_bytes<T>(v: &T) -> Result<MutableBuffer>
where
    T: ?Sized + Serialize,
{
    let mut ser = Serializer::<_, BigEndian>::new(MutableBuffer::new());

    v.serialize(&mut ser)?;
    Ok(ser.into_inner())
}

/// Serializes the given data in little endian format.
///
/// This is the format used by disk formats in Minecraft: Bedrock Edition.
///
/// See [`to_le_bytes_in`] for an alternative that serializes into the given writer, instead
/// of producing a new one.
#[inline]
pub fn to_le_bytes<T>(v: &T) -> Result<MutableBuffer>
where
    T: ?Sized + Serialize,
{
    let mut ser = Serializer::<_, LittleEndian>::new(MutableBuffer::new());

    v.serialize(&mut ser)?;
    Ok(ser.into_inner())
}

/// Serializes the given data in variable format.
///
/// This is the format used by network formats in Minecraft: Bedrock Edition.
///
/// See [`to_var_bytes_in`] for an alternative that serializes into the given writer, instead
/// of producing a new one.
#[inline]
pub fn to_var_bytes<T>(v: &T) -> Result<MutableBuffer>
where
    T: ?Sized + Serialize,
{
    let mut ser = Serializer::<_, Variable>::new(MutableBuffer::new());

    v.serialize(&mut ser)?;
    Ok(ser.into_inner())
}

/// Serializes the given data, into the given writer, in big endian format.
///
/// This is the format used by Minecraft: Java Edition.
#[inline]
pub fn to_be_bytes_in<W, T>(w: W, v: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
        W: Write
{
    let mut ser = Serializer::<W, BigEndian>::new(w);
    v.serialize(&mut ser)
}

/// Serializes the given data, into the given writer, in little endian format.
///
/// This is the format used by disk formats in Minecraft: Bedrock Edition.
#[inline]
pub fn to_le_bytes_in<W, T>(w: W, v: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
        W: Write,
{
    let mut ser = Serializer::<W, LittleEndian>::new(w);
    v.serialize(&mut ser)
}

/// Serializes the given data, into the given writer, in variable format.
///
/// This is the format used by network formats in Minecraft: Bedrock Edition.
#[inline]
pub fn to_var_bytes_in<W, T>(w: W, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
        W: Write
{
    let mut ser = Serializer::<W, Variable>::new(w);
    value.serialize(&mut ser)
}

/// NBT data serialiser.
#[derive(Debug)]
pub struct Serializer<W, F>
where
    W: Write,
    F: VariantImpl,
{
    writer: W,
    /// Whether this is the first data to be written.
    /// This makes sure that the name and type of the root compound are written.
    is_initial: bool,
    /// Stores the length of the list that is currently being serialised.
    len: usize,
    _marker: PhantomData<F>,
}

impl<W, M> Serializer<W, M>
where
    W: Write,
    M: VariantImpl,
{
    /// Creates a new and empty serialiser.
    #[inline]
    pub fn new(w: W) -> Serializer<W, M> {
        Serializer {
            writer: w,
            is_initial: true,
            len: 0,
            _marker: PhantomData,
        }
    }

    /// Consumes the serialiser and returns the inner writer.
    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }
}

/// Returns a `not supported` error.
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

impl<'a, W, M> ser::Serializer for &'a mut Serializer<W, M>
where
    M: VariantImpl,
    W: Write,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Impossible<(), Error>;

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
        match M::AS_ENUM {
            Variant::BigEndian => self.writer.write_i16_be(v)?;,
            Variant::LittleEndian | Variant::Variable => {
                self.writer.write_i16_le(v)?;
            }
        }
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<()> {
        match M::AS_ENUM {
            Variant::BigEndian => self.writer.write_i32_be(v)?;,
            Variant::LittleEndian => self.writer.write_i32_le(v)?;,
            Variant::Variable => self.writer.write_var_i32(v),
        }
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<()> {
        match M::AS_ENUM {
            Variant::BigEndian => self.writer.write_i64_be(v)?;,
            Variant::LittleEndian => self.writer.write_i64_le(v)?;,
            Variant::Variable => self.writer.write_var_i64(v),
        }
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<()> {
        match M::AS_ENUM {
            Variant::BigEndian => self.writer.write_f32_be(v)?;,
            Variant::LittleEndian | Variant::Variable => {
                self.writer.write_f32_le(v)?;
            }
        }
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<()> {
        match M::AS_ENUM {
            Variant::BigEndian => self.writer.write_f64_be(v)?;,
            Variant::LittleEndian | Variant::Variable => {
                self.writer.write_f64_le(v)?;
            }
        }
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<()> {
        match M::AS_ENUM {
            Variant::BigEndian => self.writer.write_u16_be(v.len() as u16),
            Variant::LittleEndian => self.writer.write_u16_le(v.len() as u16),
            Variant::Variable => self.writer.write_var_u32(v.len() as u32),
        }?;

        self.writer.write_all(v.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        match M::AS_ENUM {
            Variant::BigEndian => self.writer.write_i32_be(v.len() as i32),
            Variant::LittleEndian => self.writer.write_i32_le(v.len() as i32),
            Variant::Variable => self.writer.write_var_u32(v.len() as u32),
        }?;

        self.writer.write_all(v)?;
        Ok(())
    }

    #[inline]
    fn serialize_none(self) -> Result<()> {
        unreachable!("None fields cannot exist, this should have been stopped by the key serializer");
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<()> {
        unreachable!("Unit fields cannot exist, this should have been stopped by the key serializer");
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
        Ok(())
        // unreachable!("Unit struct fields cannot exist, this should have been stopped by the key serializer");
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

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self> {
        if let Some(len) = len {
            self.len = len;
            Ok(self)
        } else {
            bail!(
                Unsupported,
                "Sequences with a size not known upfront are not supported"
            );
        }
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self> {
        self.len = len;
        Ok(self)
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
        // nbt::Value does not distinguish between maps and structs.
        // Therefore this is also needed here
        if self.is_initial {
            self.writer.write_u8(FieldType::Compound as u8)?;
            self.serialize_str("")?;
            self.is_initial = false;
        }

        Ok(self)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
        if self.is_initial {
            self.writer.write_u8(FieldType::Compound as u8)?;
            self.serialize_str(name)?;
            self.is_initial = false;
        }

        Ok(self)
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

impl<'a, W, F> SerializeSeq for &'a mut Serializer<W, F>
where
    W: Write,
    F: VariantImpl,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, element: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.len != 0 {
            let ty_serializer = FieldTypeSerializer::new(self);
            element.serialize(ty_serializer)?;

            match F::AS_ENUM {
                Variant::BigEndian => self.writer.write_i32_be(self.len as i32),
                Variant::LittleEndian => {
                    self.writer.write_i32_le(self.len as i32)
                }
                Variant::Variable => self.writer.write_var_u32(self.len as u32),
            }?;
            self.len = 0;
        }

        element.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, M> SerializeTuple for &'a mut Serializer<W, M>
where
    W: Write,
    M: VariantImpl,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, element: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.len != 0 {
            let ty_serializer = FieldTypeSerializer::new(self);
            element.serialize(ty_serializer)?;

            match M::AS_ENUM {
                Variant::BigEndian => self.writer.write_i32_be(self.len as i32),
                Variant::LittleEndian => {
                    self.writer.write_i32_le(self.len as i32)
                }
                Variant::Variable => self.writer.write_var_u32(self.len as u32),
            }?;
            self.len = 0;
        }

        element.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, M> SerializeMap for &'a mut Serializer<W, M>
where
    W: Write,
    M: VariantImpl,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<K>(&mut self, _key: &K) -> Result<()>
    where
        K: ?Sized + Serialize,
    {
        unimplemented!("Use MapSerializer::serialize_entry instead");
    }

    fn serialize_value<V>(&mut self, _value: &V) -> Result<()>
    where
        V: ?Sized + Serialize,
    {
        unimplemented!("Use MapSerializer::serialize_entry instead");
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        let ty_serializer = FieldTypeSerializer::new(self);
        value.serialize(ty_serializer)?;

        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        self.writer.write_u8(FieldType::End as u8)
    }
}

impl<'a, W, M> SerializeStruct for &'a mut Serializer<W, M>
where
    W: Write,
    M: VariantImpl,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<V>(&mut self, key: &'static str, value: &V) -> Result<()>
    where
        V: ?Sized + Serialize,
    {
        let ty_serializer = FieldTypeSerializer::new(self);
        value.serialize(ty_serializer)?;

        match M::AS_ENUM {
            Variant::LittleEndian => self.writer.write_u16_le(key.len() as u16),
            Variant::BigEndian => self.writer.write_u16_be(key.len() as u16),
            Variant::Variable => self.writer.write_var_u32(key.len() as u32),
        }?;

        self.writer.write_all(key.as_bytes())?;
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        self.writer.write_u8(FieldType::End as u8)
    }
}

/// Separate serialiser that writes data types to the writer.
///
/// Serde does not provide any type information, hence this exists.
///
/// This serialiser writes the data type of the given value and does not consume it.
struct FieldTypeSerializer<'a, W, F>
where
    W: Write,
    F: VariantImpl,
{
    ser: &'a mut Serializer<W, F>,
}

impl<'a, W, F> FieldTypeSerializer<'a, W, F>
where
    W: Write,
    F: VariantImpl,
{
    pub fn new(ser: &'a mut Serializer<W, F>) -> Self {
        Self { ser }
    }
}

impl<'a, W, F> ser::Serializer for FieldTypeSerializer<'a, W, F>
where
    W: Write,
    F: VariantImpl,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Impossible<(), Error>;

    forward_unsupported!(char, u8, u16, u32, u64, i128);

    #[inline]
    fn serialize_bool(self, _v: bool) -> Result<()> {
        self.ser.writer.write_u8(FieldType::Byte as u8)
    }

    #[inline]
    fn serialize_i8(
        self,
        _v: i8,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        self.ser.writer.write_u8(FieldType::Byte as u8)
    }

    #[inline]
    fn serialize_i16(
        self,
        v: i16,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        self.ser.writer.write_u8(FieldType::Short as u8)
    }

    fn serialize_i32(
        self,
        v: i32,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        self.ser.writer.write_u8(FieldType::Int as u8)
    }

    fn serialize_i64(
        self,
        v: i64,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        self.ser.writer.write_u8(FieldType::Long as u8)
    }

    fn serialize_f32(
        self,
        v: f32,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        self.ser.writer.write_u8(FieldType::Float as u8)
    }

    fn serialize_f64(
        self,
        v: f64,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        self.ser.writer.write_u8(FieldType::Double as u8)
    }

    fn serialize_str(
        self,
        v: &str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        self.ser.writer.write_u8(FieldType::String as u8)
    }

    fn serialize_bytes(
        self,
        v: &[u8],
    ) -> std::result::Result<Self::Ok, Self::Error> {
        self.ser.writer.write_u8(FieldType::ByteArray as u8)
    }

    fn serialize_none(self) -> std::result::Result<Self::Ok, Self::Error> {
        todo!();
    }

    fn serialize_some<T: ?Sized>(
        self,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_struct(
        self,
        name: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
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
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeSeq, Self::Error> {
        self.ser.writer.write_u8(FieldType::List as u8)?;
        Ok(self)
    }

    fn serialize_tuple(
        self,
        len: usize,
    ) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        self.ser.writer.write_u8(FieldType::List as u8)?;
        Ok(self)
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

    #[inline]
    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeMap, Self::Error> {
        self.ser.writer.write_u8(FieldType::Compound as u8)?;
        Ok(self)
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
        self.ser.writer.write_u8(FieldType::Compound as u8)?;
        Ok(self)
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

impl<'a, W, F> SerializeSeq for FieldTypeSerializer<'a, W, F>
where
    W: Write,
    F: VariantImpl,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, _element: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, F> SerializeTuple for FieldTypeSerializer<'a, W, F>
where
    W: Write,
    F: VariantImpl,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, _element: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, F> SerializeMap for FieldTypeSerializer<'a, W, F>
where
    W: Write,
    F: VariantImpl,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<K>(&mut self, _key: &K) -> Result<()>
    where
        K: ?Sized + Serialize,
    {
        Ok(())
    }

    #[inline]
    fn serialize_value<V>(&mut self, _value: &V) -> Result<()>
    where
        V: ?Sized + Serialize,
    {
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, F> SerializeStruct for FieldTypeSerializer<'a, W, F>
where
    W: Write,
    F: VariantImpl,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<V>(
        &mut self,
        _key: &'static str,
        value: &V,
    ) -> Result<()>
    where
        V: ?Sized + Serialize,
    {
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}
