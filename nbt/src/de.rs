use std::fmt;

use crate::error::{Error, Result};
use crate::{
    bail, ReadBuffer, TAG_BYTE, TAG_BYTE_ARRAY, TAG_COMPOUND, TAG_DOUBLE,
    TAG_END, TAG_FLOAT, TAG_INT, TAG_INT_ARRAY, TAG_LIST, TAG_LONG,
    TAG_LONG_ARRAY, TAG_SHORT, TAG_STRING,
};
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{de, Deserialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Flavor {
    Network,
    LittleEndian,
    BigEndian,
}

pub struct Deserializer<'de> {
    flavor: Flavor,
    input: ReadBuffer<'de>,

    // State
    latest_tag: u8,
    remaining: u32,
    is_key: bool
}

impl<'de> Deserializer<'de> {
    #[inline]
    pub fn from_bytes(input: &'de [u8], flavor: Flavor) -> Self {
        Deserializer {
            input: ReadBuffer::from(input),
            flavor,
            latest_tag: TAG_END,
            remaining: 0,
            is_key: false
        }
    }

    #[inline]
    pub fn from_le_bytes(input: &'de [u8]) -> Self {
        Self::from_bytes(input, Flavor::LittleEndian)
    }

    #[inline]
    pub fn from_be_bytes(input: &'de [u8]) -> Self {
        Self::from_bytes(input, Flavor::BigEndian)
    }

    #[inline]
    pub fn from_network_bytes(input: &'de [u8]) -> Self {
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
            match self.latest_tag {
                TAG_BYTE => self.deserialize_i8(visitor),
                TAG_SHORT => self.deserialize_i16(visitor),
                TAG_INT => self.deserialize_i32(visitor),
                TAG_LONG => self.deserialize_i64(visitor),
                TAG_FLOAT => self.deserialize_f32(visitor),
                TAG_DOUBLE => self.deserialize_f64(visitor),
                TAG_BYTE_ARRAY => self.deserialize_byte_buf(visitor),
                TAG_STRING => self.deserialize_string(visitor),
                TAG_LIST => self.deserialize_seq(visitor),
                TAG_COMPOUND => self.deserialize_map(visitor),
                TAG_INT_ARRAY => self.deserialize_seq(visitor),
                TAG_LONG_ARRAY => self.deserialize_seq(visitor),
                _ => bail!(
                    InvalidType, 
                    "deserializer encountered an invalid NBT tag type: {}", self.latest_tag
                )
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
            Flavor::BigEndian => {
                self.input.read_be::<i16>()
            }
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
            Flavor::BigEndian => {
                self.input.read_be::<i32>()
            }
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
            Flavor::BigEndian => {
                self.input.read_be::<i64>()
            }
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
        match self.flavor {
            Flavor::BigEndian => {
                if let Ok(x) = self.input.read_be::<f32>() {
                    visitor.visit_f32(x)
                } else {
                    bail!(UnexpectedEof)
                }
            }
            _ => {
                if let Ok(x) = self.input.read_le::<f32>() {
                    visitor.visit_f32(x)
                } else {
                    bail!(UnexpectedEof)
                }
            }
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.flavor {
            Flavor::BigEndian => {
                if let Ok(x) = self.input.read_be::<f64>() {
                    visitor.visit_f64(x)
                } else {
                    bail!(UnexpectedEof)
                }
            }
            _ => {
                if let Ok(x) = self.input.read_le::<f64>() {
                    visitor.visit_f64(x)
                } else {
                    bail!(UnexpectedEof)
                }
            }
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!();
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
        dbg!(str);

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
        dbg!(&string);

        visitor.visit_string(string)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!();
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!();
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!();
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!();
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!();
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!();
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let _list_type = self.input.read_be::<u8>()?;
        self.remaining = match self.flavor {
            Flavor::BigEndian => self.input.read_be::<u32>()?,
            Flavor::LittleEndian => self.input.read_le::<u32>()?,
            Flavor::Network => todo!(),
        };

        let de = SeqDeserializer::from(self);
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
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!();
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(MapDeserializer::from(self))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        debug_assert_eq!(self.input.read_be::<u8>()?, TAG_COMPOUND);
        let name = self.deserialize_raw_str()?;
        dbg!(name);
        self.deserialize_map(visitor)
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
        todo!();
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

struct SeqDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>
}

impl<'de, 'a> From<&'a mut Deserializer<'de>>
    for SeqDeserializer<'a, 'de>
{
    fn from(v: &'a mut Deserializer<'de>) -> Self {
        Self {
            de: v,
        }
    }
}

impl<'de, 'a> SeqAccess<'de> for SeqDeserializer<'a, 'de> {
    type Error = Error;

    fn next_element_seed<E>(&mut self, seed: E) -> Result<Option<E::Value>>
    where
        E: DeserializeSeed<'de>,
    {
        // FIXME: remaining might have to be moved to SeqDeserializer to support nested sequences.
        if self.de.remaining > 0 {
            self.de.remaining -= 1;
            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }
}

struct StringKeySeed<'a>(&'a str);

impl<'de, 'a> DeserializeSeed<'de> for StringKeySeed<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error> 
    where
        D: de::Deserializer<'de>
    {
        struct StringKeyVisitor<'de>(Option<&'de str>);

        impl<'de> Visitor<'de> for StringKeyVisitor<'de> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a u16")
            }

            fn visit_borrowed_str<E>(mut self, v: &'de str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error, 
            {   
                self.0 = Some(v);

                Ok(())    
            }
        }

        deserializer.deserialize_str(StringKeyVisitor(None))
    }
}

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
        self.de.latest_tag = self.de.input.read_be::<u8>()?;

        let r = if self.de.latest_tag == TAG_END {
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
        struct AllTypes {
            #[serde(rename = "stringTest")]
            string_test: String,
            #[serde(rename = "listTest (long)")]
            list_test: [i64; 5],
            #[serde(rename = "byteTest")]
            byte_test: i8,
            #[serde(rename = "shortTest")]
            short_test: i16,
            #[serde(rename = "intTest")]
            int_test: i32,
            #[serde(rename = "longTest")]
            long_test: i64,
            #[serde(rename = "floatTest")]
            float_test: f32,
            #[serde(rename = "doubleTest")]
            double_test: f64
        }

        let decoded: super::Result<AllTypes> = from_be_bytes(BIGTEST_NBT);
        if let Err(err) = decoded {
            println!("{err}");
            panic!("failed")
        }

        let decoded = decoded.unwrap();
    }

    #[test]
    fn read_hello_world_nbt() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct HelloWorld {
            name: String,
        }

        let decoded: super::Result<HelloWorld> = from_be_bytes(HELLO_WORLD_NBT);
        if let Err(err) = decoded {
            println!("{err}");
            panic!("failed")
        }

        let decoded = decoded.unwrap();
        assert_eq!(decoded, HelloWorld { name: String::from("Bananrama") });
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

        println!("{PLAYER_NAN_VALUE_NBT:?}");
        let decoded: Player = from_be_bytes(PLAYER_NAN_VALUE_NBT).unwrap();
        dbg!(decoded);
    }
}
