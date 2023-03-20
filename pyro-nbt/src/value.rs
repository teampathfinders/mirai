use std::collections::HashMap;
use std::fmt;
use serde::{de, Deserialize, Deserializer};
use serde::de::{MapAccess, SeqAccess, Visitor};

#[derive(Debug, Clone)]
pub enum Value {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<u8>),
    String(String),
    List(Vec<Value>),
    Compound(HashMap<String, Value>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>)
}

impl<'de> Deserialize<'de> for Value {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            #[inline]
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid NBT value")
            }

            #[inline]
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
                where
                    E: de::Error
            {
                Ok(Value::Byte(v as i8))
            }

            #[inline]
            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
                where
                    E: de::Error
            {
                Ok(Value::Byte(v))
            }

            #[inline]
            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
                where
                    E: de::Error,
            {
                Ok(Value::Short(v))
            }

            #[inline]
            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
                where
                    E: de::Error,
            {
                Ok(Value::Int(v))
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
                where
                    E: de::Error
            {
                Ok(Value::Long(v))
            }

            #[inline]
            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
                where
                    E: de::Error,
            {
                Ok(Value::Float(v))
            }

            #[inline]
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
                where
                    E: de::Error,
            {
                Ok(Value::Double(v))
            }

            #[inline]
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                where
                    E: de::Error,
            {
                Ok(Value::String(v))
            }

            #[inline]
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
                where
                    E: de::Error,
            {
                Ok(Value::ByteArray(v))
            }

            #[inline]
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>
            {
                let mut out = Vec::new();
                if let Some(hint) = seq.size_hint() {
                    out.reserve(hint);
                }

                while let Some(element) = seq.next_element()? {
                    out.push(element);
                }

                Ok(Value::List(out))
            }

            #[inline]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: MapAccess<'de>,
            {
                let mut out: HashMap<String, Value> = HashMap::new();
                if let Some(hint) = map.size_hint() {
                    out.reserve(hint);
                }

                while let Some((key, value)) = map.next_entry()? {
                    out.insert(key, value);
                }

                Ok(Value::Compound(out))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}