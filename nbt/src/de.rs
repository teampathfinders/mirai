use crate::{ReadBuffer, Buf, bail, TAG_COMPOUND, TAG_END};
use crate::error::{Error, Result};
use serde::de::{Visitor, MapAccess, DeserializeSeed};
use serde::{de, Deserialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Flavor {
    Network,
    LittleEndian,
    BigEndian
}

pub struct Deserializer<'de> {
    flavor: Flavor,
    input: ReadBuffer<'de>,
}

impl<'de> Deserializer<'de> {
    #[inline]
    pub fn from_bytes(input: &'de [u8], flavor: Flavor) -> Self {
        Deserializer { input: ReadBuffer::from(input), flavor: flavor }
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
            Flavor::BigEndian => self.input.read_u16(),
            Flavor::LittleEndian => self.input.read_u16_le(),
            Flavor::Network => todo!()
        }?;

        let data = self.input.take_n(len as usize)?;
        let str = std::str::from_utf8(data)?;

        Ok(str)
    }
}

pub fn from_bytes<'a, T>(b: &'a [u8], flavor: Flavor) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(b, flavor);
    let t = T::deserialize(&mut deserializer)?;

    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingBytes)
    }
}

#[inline]
pub fn from_le_bytes<'a, T>(b: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    from_bytes(b, Flavor::LittleEndian)
}

#[inline]
pub fn from_be_bytes<'a, T>(b: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    from_bytes(b, Flavor::BigEndian)
}

#[inline]
pub fn from_network_bytes<'a, T>(b: &'a [u8]) -> Result<T>
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
        todo!();
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Ok(x) = self.input.read_bool() {
            visitor.visit_bool(x)
        } else {
            bail!(UnexpectedEof)
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Ok(x) = self.input.read_i8() {
            visitor.visit_i8(x)
        } else {
            bail!(UnexpectedEof)
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.flavor {
            Flavor::BigEndian => {
                if let Ok(x) = self.input.read_i16() {
                    visitor.visit_i16(x)
                } else {
                    bail!(UnexpectedEof)
                }
            },
            _ => {
                if let Ok(x) = self.input.read_i16_le() {
                    visitor.visit_i16(x)
                } else {
                    bail!(UnexpectedEof)
                }       
            }
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.flavor {
            Flavor::BigEndian => {
                if let Ok(x) = self.input.read_i32() {
                    visitor.visit_i32(x)
                } else {
                    bail!(UnexpectedEof)
                }
            },
            _ => {
                if let Ok(x) = self.input.read_i32_le() {
                    visitor.visit_i32(x)
                } else {
                    bail!(UnexpectedEof)
                }       
            }
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.flavor {
            Flavor::BigEndian => {
                if let Ok(x) = self.input.read_i64() {
                    visitor.visit_i64(x)
                } else {
                    bail!(UnexpectedEof)
                }
            },
            _ => {
                if let Ok(x) = self.input.read_i64_le() {
                    visitor.visit_i64(x)
                } else {
                    bail!(UnexpectedEof)
                }       
            }
        }
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
                if let Ok(x) = self.input.read_f32() {
                    visitor.visit_f32(x)
                } else {
                    bail!(UnexpectedEof)
                }
            },
            _ => {
                if let Ok(x) = self.input.read_f32_le() {
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
                if let Ok(x) = self.input.read_f64() {
                    visitor.visit_f64(x)
                } else {
                    bail!(UnexpectedEof)
                }
            },
            _ => {
                if let Ok(x) = self.input.read_f64_le() {
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
        let opt_len = match self.flavor {
            Flavor::BigEndian => {
                self.input.read_u16()
            },
            Flavor::LittleEndian => {
                self.input.read_u16_le()
            },
            Flavor::Network => {
                todo!();
            }
        };

        if let Ok(len) = opt_len {
            if let Ok(data) = self.input.take_n(len as usize) {
                let str = std::str::from_utf8(data)?;
                return visitor.visit_str(str)
            }
        }
        
        Err(Error::UnexpectedEof)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let opt_len = match self.flavor {
            Flavor::BigEndian => {
                self.input.read_u16()
            },
            Flavor::LittleEndian => {
                self.input.read_u16_le()
            },
            Flavor::Network => {
                todo!();
            }
        };

        if let Ok(len) = opt_len {
            if let Ok(data) = self.input.take_n(len as usize) {
                let str = String::from_utf8(data.to_vec())?;
                return visitor.visit_string(str)
            }
        }
        
        Err(Error::UnexpectedEof)
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
        todo!();
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!();
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
        visitor.visit_map(self)
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
        todo!();
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!();
    }
}

impl<'de, 'a> MapAccess<'de> for Deserializer<'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        todo!();
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        if self.input.peek::<u8>()? == TAG_END {
            
        }

        todo!();
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use serde::Deserialize;

    use super::{Deserializer, from_be_bytes};

    const BIGTEST_NBT: &[u8] = include_bytes!("../test/bigtest.nbt");
    const HELLO_WORLD_NBT: &[u8] = include_bytes!("../test/hello_world.nbt");
    const PLAYER_NAN_VALUE_NBT: &[u8] =
        include_bytes!("../test/player_nan_value.nbt");

    #[test]
    fn read_bigtest_nbt() {
        // let decoded = from_be_bytes(BIGTEST_NBT).unwrap();

    }

    
    #[test]
    fn read_hello_world_nbt() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct HelloWorld {
            name: &'static str
        }

        let decoded: HelloWorld = from_be_bytes(HELLO_WORLD_NBT).unwrap();
        assert_eq!(decoded, HelloWorld {
            name: "Bananrama"
        });
    }

    
    #[test]
    fn read_player_nan_value_nbt() {

    }
}