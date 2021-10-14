//! Definition of a Smile value.

pub use crate::value::big_decimal::BigDecimal;
use crate::value::big_decimal::BigDecimalVisitor;
pub use crate::value::big_integer::BigInteger;
use crate::value::big_integer::BigIntegerVisitor;
use indexmap::IndexMap;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

mod big_decimal;
mod big_integer;

/// A representation of a Smile value.
#[derive(PartialEq, Debug)]
pub enum Value {
    /// A null value.
    Null,
    /// A boolean value.
    Boolean(bool),
    /// An integer value.
    Integer(i32),
    /// A long value.
    Long(i64),
    /// A big integer value.
    BigInteger(BigInteger),
    /// A float value.
    Float(f32),
    /// A double value.
    Double(f64),
    /// A big decimal value.
    BigDecimal(BigDecimal),
    /// A string value.
    String(String),
    /// A binary value.
    Binary(Vec<u8>),
    /// An array value.
    Array(Vec<Value>),
    /// An object value.
    Object(IndexMap<String, Value>),
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Null => serializer.serialize_unit(),
            Value::Boolean(v) => serializer.serialize_bool(*v),
            Value::Integer(v) => serializer.serialize_i32(*v),
            Value::Long(v) => serializer.serialize_i64(*v),
            Value::BigInteger(v) => v.serialize(serializer),
            Value::Float(v) => serializer.serialize_f32(*v),
            Value::Double(v) => serializer.serialize_f64(*v),
            Value::BigDecimal(v) => v.serialize(serializer),
            Value::String(v) => serializer.serialize_str(v),
            Value::Binary(v) => serializer.serialize_bytes(v),
            Value::Array(v) => v.serialize(serializer),
            Value::Object(v) => v.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // We want to avoid the normal coercion of BigInteger values to normal Rust integers. Rather than adding another
        // special case in the deserialize for Value, we instead just hint that we are trying to deserialize a
        // BigInteger. The Deserializer logic will deserialize BigIntegers without conversion but still handle other
        // values normally.
        deserializer.deserialize_struct(
            BigInteger::STRUCT_NAME,
            &[BigInteger::FIELD_NAME],
            ValueVisitor,
        )
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("any Smile value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Boolean(v))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Integer(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Long(v))
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Float(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Double(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::String(v.to_string()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::String(v))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Binary(v.to_vec()))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Binary(v))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Null)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut vec = vec![];
        while let Some(value) = seq.next_element()? {
            vec.push(value);
        }
        Ok(Value::Array(vec))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut v = IndexMap::new();

        let mut key = String::new();
        match map.next_key_seed(FirstKeySeed { key: &mut key })? {
            Some(FirstKey::BigInteger) => {
                return BigIntegerVisitor.finish_map(map).map(Value::BigInteger)
            }
            Some(FirstKey::BigDecimal) => {
                return BigDecimalVisitor.finish_map(map).map(Value::BigDecimal)
            }
            Some(FirstKey::Other) => {}
            None => return Ok(Value::Object(v)),
        }

        v.insert(key, map.next_value()?);
        while let Some((key, value)) = map.next_entry()? {
            v.insert(key, value);
        }

        Ok(Value::Object(v))
    }
}

enum FirstKey {
    BigInteger,
    BigDecimal,
    Other,
}

struct FirstKeySeed<'a> {
    key: &'a mut String,
}

impl<'de> DeserializeSeed<'de> for FirstKeySeed<'_> {
    type Value = FirstKey;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

impl<'de> Visitor<'de> for FirstKeySeed<'_> {
    type Value = FirstKey;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v {
            BigInteger::FIELD_NAME => Ok(FirstKey::BigInteger),
            BigDecimal::SCALE_FIELD_NAME => Ok(FirstKey::BigDecimal),
            _ => {
                self.key.push_str(v);
                Ok(FirstKey::Other)
            }
        }
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match &*v {
            BigInteger::FIELD_NAME => Ok(FirstKey::BigInteger),
            BigDecimal::SCALE_FIELD_NAME => Ok(FirstKey::BigDecimal),
            _ => {
                *self.key = v;
                Ok(FirstKey::Other)
            }
        }
    }
}
