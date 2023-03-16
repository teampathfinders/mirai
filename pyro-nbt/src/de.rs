use std::fmt;

use crate::FieldType;
use serde::de::Unexpected::Seq;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{de, Deserialize};
use util::bytes::{BinaryReader, MutableBuffer, SharedBuffer};
use util::{bail, Error, Result};

macro_rules! is_ty {
    ($expected: ident, $actual: expr) => {
        if $actual != FieldType::$expected {
            bail!(
                Malformed,
                "expected type {:?}, but found {:?}",
                FieldType::$expected,
                $actual
            )
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Flavor {
    Network,
    LittleEndian,
    BigEndian,
}

#[derive(Debug)]
pub struct Deserializer<'de> {
    flavor: Flavor,
    input: SharedBuffer<'de>,
    next_ty: FieldType,
    is_key: bool,
}

impl<'de> Deserializer<'de> {
    #[inline]
    pub(crate) fn from_bytes(input: &'de [u8], flavor: Flavor) -> Self {
        let mut input = SharedBuffer::from(input);
        assert_eq!(input.read_u8().unwrap(), FieldType::Compound as u8);

        let mut de = Deserializer {
            input,
            flavor,
            next_ty: FieldType::Compound,
            is_key: false,
        };

        let _ = de.deserialize_raw_str().unwrap();
        de
    }

    #[inline]
    pub(crate) fn from_le_bytes(input: &'de [u8]) -> Self {
        Self::from_bytes(input, Flavor::LittleEndian)
    }

    #[inline]
    pub(crate) fn from_be_bytes(input: &'de [u8]) -> Self {
        Self::from_bytes(input, Flavor::BigEndian)
    }

    #[inline]
    pub(crate) fn from_network_bytes(input: &'de [u8]) -> Self {
        Self::from_bytes(input, Flavor::Network)
    }

    #[inline]
    fn deserialize_raw_str(&mut self) -> Result<&str> {
        let len = match self.flavor {
            Flavor::BigEndian => self.input.read_u16_be()? as u32,
            Flavor::LittleEndian => self.input.read_u16_le()? as u32,
            Flavor::Network => self.input.read_var_u32()?,
        };

        let data = self.input.take_n(len as usize)?;
        let str = std::str::from_utf8(data)?;

        Ok(str)
    }
}

#[inline]
pub fn from_bytes<'a, T>(b: &'a [u8], flavor: Flavor) -> Result<(T, usize)>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(b, flavor);
    let start = deserializer.input.len();
    let output = T::deserialize(&mut deserializer)?;
    let end = deserializer.input.len();

    Ok((output, start - end))
}

#[inline]
pub fn from_le_bytes<'a, T>(b: &'a [u8]) -> Result<(T, usize)>
where
    T: Deserialize<'a>,
{
    from_bytes(b, Flavor::LittleEndian)
}

#[inline]
pub fn from_be_bytes<'a, T>(b: &'a [u8]) -> Result<(T, usize)>
where
    T: Deserialize<'a>,
{
    from_bytes(b, Flavor::BigEndian)
}

#[inline]
pub fn from_net_bytes<'a, T>(b: &'a [u8]) -> Result<(T, usize)>
where
    T: Deserialize<'a>,
{
    from_bytes(b, Flavor::Network)
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

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

        let n = match self.flavor {
            Flavor::BigEndian => self.input.read_i16_be(),
            Flavor::LittleEndian | Flavor::Network => self.input.read_i16_le(),
        }?;

        visitor.visit_i16(n)
    }

    #[inline]
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        is_ty!(Int, self.next_ty);

        let n = match self.flavor {
            Flavor::BigEndian => self.input.read_i32_be(),
            Flavor::LittleEndian | Flavor::Network => self.input.read_i32_le(),
        }?;

        visitor.visit_i32(n)
    }

    #[inline]
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        is_ty!(Long, self.next_ty);

        let n = match self.flavor {
            Flavor::BigEndian => self.input.read_i64_be(),
            Flavor::LittleEndian | Flavor::Network => self.input.read_i64_le(),
        }?;

        visitor.visit_i64(n)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(Unsupported, "NBT does not support `u8`, use `i8` instead")
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(Unsupported, "NBT does not support `u16`, use `i16` instead")
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(Unsupported, "NBT does not support `u32`, use `i32` instead")
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(Unsupported, "NBT does not support `u64`, use `i64` instead")
    }

    #[inline]
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        is_ty!(Float, self.next_ty);

        let n = match self.flavor {
            Flavor::BigEndian => self.input.read_f32_be(),
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

        let n = match self.flavor {
            Flavor::BigEndian => self.input.read_f64_be(),
            _ => self.input.read_f64_le(),
        }?;

        visitor.visit_f64(n)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(
            Unsupported,
            "NBT does not support unicode characters, use String instead"
        )
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = match self.flavor {
            Flavor::BigEndian => self.input.read_u16_be()? as u32,
            Flavor::LittleEndian => self.input.read_u16_le()? as u32,
            Flavor::Network => self.input.read_var_u32()?,
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

        let len = match self.flavor {
            Flavor::BigEndian => self.input.read_u16_be()? as u32,
            Flavor::LittleEndian => self.input.read_u16_le()? as u32,
            Flavor::Network => self.input.read_var_u32()?,
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
        name: &'static str,
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
        visitor: V,
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
}

#[derive(Debug)]
struct SeqDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    ty: FieldType,
    remaining: u32,
}

impl<'de, 'a> SeqDeserializer<'a, 'de> {
    #[inline]
    pub fn new(
        de: &'a mut Deserializer<'de>,
        ty: FieldType,
        expected_len: u32,
    ) -> Result<Self> {
        debug_assert_ne!(ty, FieldType::End);

        // ty is not read in here because the x_array types don't have a type prefix.

        de.next_ty = ty;
        let remaining = match de.flavor {
            Flavor::BigEndian => de.input.read_i32_be()? as u32,
            Flavor::LittleEndian => de.input.read_i32_le()? as u32,
            Flavor::Network => de.input.read_var_u32()?,
        };

        if expected_len != 0 && expected_len != remaining {
            bail!(Malformed, "expected sequence of length {expected_len}, got length {remaining}");
        }

        Ok(Self { de, ty, remaining })
    }
}

impl<'de, 'a> SeqAccess<'de> for SeqDeserializer<'a, 'de> {
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
struct MapDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> From<&'a mut Deserializer<'de>> for MapDeserializer<'a, 'de> {
    #[inline]
    fn from(v: &'a mut Deserializer<'de>) -> Self {
        Self { de: v }
    }
}

impl<'de, 'a> MapAccess<'de> for MapDeserializer<'a, 'de> {
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

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use serde::Deserialize;

    use crate::{de::Deserializer, from_be_bytes};

    const BIGTEST_NBT: &[u8] = include_bytes!("../test/bigtest.nbt");
    const HELLO_WORLD_NBT: &[u8] = include_bytes!("../test/hello_world.nbt");
    const PLAYER_NAN_VALUE_NBT: &[u8] =
        include_bytes!("../test/player_nan_value.nbt");

    #[test]
    fn read_bigtest_nbt() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Food {
            name: String,
            value: f32,
        }

        #[derive(Deserialize, Debug, PartialEq)]
        struct Nested {
            egg: Food,
            ham: Food,
        }

        #[derive(Deserialize, Debug, PartialEq)]
        struct ListCompound {
            #[serde(rename = "created-on")]
            created_on: i64,
            name: String,
        }

        #[derive(Deserialize, Debug, PartialEq)]
        struct AllTypes {
            #[serde(rename = "nested compound test")]
            nested: Nested,
            #[serde(rename = "intTest")]
            int_test: i32,
            #[serde(rename = "byteTest")]
            byte_test: i8,
            #[serde(rename = "stringTest")]
            string_test: String,
            #[serde(rename = "listTest (long)")]
            long_list_test: [i64; 5],
            #[serde(rename = "doubleTest")]
            double_test: f64,
            #[serde(rename = "floatTest")]
            float_test: f32,
            #[serde(rename = "longTest")]
            long_test: i64,
            #[serde(rename = "listTest (compound)")]
            compound_list_test: (ListCompound, ListCompound),
            #[serde(
                rename = "byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with n=0 (0, 62, 34, 16, 8, ...))"
            )]
            byte_array_test: Vec<i8>,
            #[serde(rename = "shortTest")]
            short_test: i16,
        }

        let decoded: AllTypes = from_be_bytes(BIGTEST_NBT).unwrap().0;
        println!("{decoded:?}");
    }

    #[test]
    fn read_hello_world_nbt() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct HelloWorld {
            name: String,
        }

        let decoded: HelloWorld = from_be_bytes(HELLO_WORLD_NBT).unwrap().0;
        assert_eq!(decoded, HelloWorld { name: String::from("Bananrama") });

        dbg!(decoded);
    }

    #[test]
    fn read_player_nan_value_nbt() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Player {
            #[serde(rename = "Pos")]
            pos: [f64; 3],
            #[serde(rename = "Motion")]
            motion: [f64; 3],
            #[serde(rename = "OnGround")]
            on_ground: bool,
            #[serde(rename = "DeathTime")]
            death_time: i16,
            #[serde(rename = "Air")]
            air: i16,
            #[serde(rename = "Health")]
            health: i16,
            #[serde(rename = "FallDistance")]
            fall_distance: f32,
            #[serde(rename = "AttackTime")]
            attack_time: i16,
            #[serde(rename = "HurtTime")]
            hurt_time: i16,
            #[serde(rename = "Fire")]
            fire: i16,
            #[serde(rename = "Rotation")]
            rotation: [f32; 2],
        }

        let decoded: (Player, usize) =
            from_be_bytes(PLAYER_NAN_VALUE_NBT).unwrap();

        dbg!(decoded);
    }
}
