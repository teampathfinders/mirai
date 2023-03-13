use std::fmt;

use crate::{
    TAG_BYTE, TAG_BYTE_ARRAY, TAG_COMPOUND, TAG_DOUBLE, TAG_END, TAG_FLOAT,
    TAG_INT, TAG_INT_ARRAY, TAG_LIST, TAG_LONG, TAG_LONG_ARRAY, TAG_SHORT,
    TAG_STRING,
};
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{de, Deserialize};
use util::bytes::ReadBuffer;
use util::{bail, Error, Result};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Flavor {
    Network,
    LittleEndian,
    BigEndian,
}

#[derive(Debug)]
pub struct Deserializer<'de> {
    flavor: Flavor,
    input: ReadBuffer<'de>,

    // State
    next_ty: u8,
    is_key: bool,
}

impl<'de> Deserializer<'de> {
    #[inline]
    pub(crate) fn from_bytes(input: &'de [u8], flavor: Flavor) -> Self {
        let mut input = ReadBuffer::from(input);

        assert_eq!(input.read_be::<u8>().unwrap(), TAG_COMPOUND);

        let mut de = Deserializer {
            input,
            flavor,
            next_ty: TAG_COMPOUND,
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

    fn deserialize_raw_str(&mut self) -> Result<&str> {
        let len = match self.flavor {
            Flavor::BigEndian => self.input.read_be::<u16>(),
            Flavor::LittleEndian => self.input.read_le::<u16>(),
            Flavor::Network => todo!(),
        }?;

        let data = self.input.take(len as usize)?;
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
                TAG_BYTE => self.deserialize_i8(visitor),
                TAG_SHORT => self.deserialize_i16(visitor),
                TAG_INT => self.deserialize_i32(visitor),
                TAG_LONG => self.deserialize_i64(visitor),
                TAG_FLOAT => self.deserialize_f32(visitor),
                TAG_DOUBLE => self.deserialize_f64(visitor),
                TAG_BYTE_ARRAY => self.deserialize_byte_buf(visitor),
                TAG_STRING => self.deserialize_string(visitor),
                TAG_LIST => self.deserialize_seq(visitor),
                TAG_COMPOUND => {
                    let m = self.deserialize_map(visitor);
                    m
                }
                TAG_INT_ARRAY => self.deserialize_seq(visitor),
                TAG_LONG_ARRAY => self.deserialize_seq(visitor),
                _ => {
                    dbg!(&self);
                    bail!(
                        Malformed,
                        "deserializer encountered an invalid NBT tag type: {}",
                        self.next_ty
                    )
                },
            }
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let n = self.input.read_be::<bool>()?;
        visitor.visit_bool(n)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let n = self.input.read_be::<i8>()?;
        visitor.visit_i8(n)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let n = match self.flavor {
            Flavor::BigEndian => self.input.read_be::<i16>(),
            Flavor::LittleEndian | Flavor::Network => {
                self.input.read_le::<i16>()
            }
        }?;

        visitor.visit_i16(n)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let n = match self.flavor {
            Flavor::BigEndian => self.input.read_be::<i32>(),
            Flavor::LittleEndian | Flavor::Network => {
                self.input.read_le::<i32>()
            }
        }?;

        visitor.visit_i32(n)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let n = match self.flavor {
            Flavor::BigEndian => self.input.read_be::<i64>(),
            Flavor::LittleEndian | Flavor::Network => {
                self.input.read_le::<i64>()
            }
        }?;

        visitor.visit_i64(n)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(Unsupported)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(Unsupported)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(Unsupported)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(Unsupported)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let n = match self.flavor {
            Flavor::BigEndian => self.input.read_be::<f32>(),
            _ => self.input.read_le::<f32>(),
        }?;

        visitor.visit_f32(n)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let n = match self.flavor {
            Flavor::BigEndian => self.input.read_be::<f64>(),
            _ => self.input.read_le::<f64>(),
        }?;

        visitor.visit_f64(n)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(Unsupported)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = match self.flavor {
            Flavor::BigEndian => self.input.read_be::<u16>()?,
            Flavor::LittleEndian => self.input.read_le::<u16>()?,
            Flavor::Network => {
                todo!();
            }
        };

        let data = self.input.take(len as usize)?;
        let str = std::str::from_utf8(data)?;
        if self.is_key {
            println!("key = {str}, ty = {}", self.next_ty);
        }

        visitor.visit_str(str)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = match self.flavor {
            Flavor::BigEndian => self.input.read_be::<u16>()?,
            Flavor::LittleEndian => self.input.read_le::<u16>()?,
            Flavor::Network => {
                todo!();
            }
        };

        let data = self.input.take(len as usize)?;
        let string = String::from_utf8(data.to_vec())?;

        visitor.visit_string(string)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(Unsupported)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(Unsupported)
    }

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
        bail!(Unsupported)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bail!(Unsupported)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let ty = if self.next_ty == TAG_BYTE_ARRAY || self.next_ty == TAG_INT_ARRAY || self.next_ty == TAG_LONG_ARRAY {
            self.next_ty
        } else {
            self.input.read_le()?
        };

        dbg!(ty, self.next_ty);
        let de = SeqDeserializer::new(self, ty)?;
        visitor.visit_seq(de)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
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
        bail!(Unsupported)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let de = MapDeserializer::from(self);
        visitor.visit_map(de)
    }

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
        bail!(Unsupported)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

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
    ty: u8,
    remaining: i32
}

impl<'de, 'a> SeqDeserializer<'a, 'de> {
    pub fn new(de: &'a mut Deserializer<'de>, ty: u8) -> Result<Self> {
        debug_assert_ne!(ty, TAG_END);

        // ty is not read in here because the x_array types don't have a type prefix.

        de.next_ty = ty;
        let remaining = match de.flavor {
            Flavor::BigEndian => de.input.read_be::<i32>()?,
            Flavor::LittleEndian => de.input.read_le::<i32>()?,
            Flavor::Network => todo!(),
        };

        println!("remaining {remaining}\tlist type {ty}");
        Ok(Self {
            de, ty, remaining
        })
    }
}

impl<'de, 'a> SeqAccess<'de> for SeqDeserializer<'a, 'de> {
    type Error = Error;

    fn next_element_seed<E>(&mut self, seed: E) -> Result<Option<E::Value>>
    where
        E: DeserializeSeed<'de>,
    {
        // FIXME: remaining might have to be moved to SeqDeserializer to support nested sequences.
        if self.remaining > 0 {
            dbg!(self.remaining);
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
    fn from(v: &'a mut Deserializer<'de>) -> Self {
        Self { de: v }
    }
}

impl<'de, 'a> MapAccess<'de> for MapDeserializer<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        self.de.is_key = true;
        self.de.next_ty = self.de.input.read_be::<u8>()?;

        let r = if self.de.next_ty == TAG_END {
            println!("compound end");
            Ok(None)
        } else {
            // Reads 0 tag instead of string, because string length starts with 0
            seed.deserialize(&mut *self.de).map(Some)
        };

        self.de.is_key = false;
        r
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        debug_assert_ne!(self.de.next_ty, TAG_END);
        seed.deserialize(&mut *self.de)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use serde::Deserialize;

    use super::{from_be_bytes, Deserializer};

    const BIGTEST_NBT: &[u8] = include_bytes!("../test/bigtest.nbt");
    const HELLO_WORLD_NBT: &[u8] = include_bytes!("../test/hello_world.nbt");
    const PLAYER_NAN_VALUE_NBT: &[u8] =
        include_bytes!("../test/player_nan_value.nbt");

    #[test]
    fn read_bigtest_nbt() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Food {
            name: String,
            value: f32
        }

        #[derive(Deserialize, Debug, PartialEq)]
        struct Nested {
            egg: Food,
            ham: Food
        }

        #[derive(Deserialize, Debug, PartialEq)]
        struct ListCompound {
            #[serde(rename = "created-on")]
            created_on: i64,
            name: String
        }

        #[derive(Deserialize, Debug, PartialEq)]
        #[allow(non_snake_case)]
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
            #[serde(rename = "byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with n=0 (0, 62, 34, 16, 8, ...))")]
            byteArrayTest: Vec<i8>,
            #[serde(rename = "shortTest")]
            short_test: i16
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
            pos: (f64, f64, f64),
            #[serde(rename = "Motion")]
            motion: (f64, f64, f64),
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
            rotation: (f32, f32),
        }

        let decoded: (Player, usize) =
            from_be_bytes(PLAYER_NAN_VALUE_NBT).unwrap();

        dbg!(decoded);
    }
}
