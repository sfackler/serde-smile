use crate::de::read::Buf;
use crate::de::Read;
use crate::value::BigInteger;
use crate::{Deserializer, Error};
use serde::de::{DeserializeSeed, MapAccess, Visitor};
use serde::forward_to_deserialize_any;

pub(crate) struct BigIntegerDeserializer<'a, 'de, R> {
    pub(crate) de: &'a mut Deserializer<'de, R>,
}

impl<'de, R> MapAccess<'de> for BigIntegerDeserializer<'_, 'de, R>
where
    R: Read<'de>,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        seed.deserialize(BigIntegerFieldDeserializer).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(BigIntegerValueDeserializer { de: &mut *self.de })
    }
}

struct BigIntegerFieldDeserializer;

impl<'de> serde::Deserializer<'de> for BigIntegerFieldDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(BigInteger::FIELD_NAME)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string seq
        bytes byte_buf map struct option unit newtype_struct
        ignored_any unit_struct tuple_struct tuple enum identifier
    }
}

struct BigIntegerValueDeserializer<'a, 'de, R> {
    de: &'a mut Deserializer<'de, R>,
}

impl<'de, R> serde::Deserializer<'de> for BigIntegerValueDeserializer<'_, 'de, R>
where
    R: Read<'de>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let buf = self.de.parse_7_bit_binary()?;
        match buf {
            Buf::Short(buf) => visitor.visit_bytes(buf),
            Buf::Long(buf) => visitor.visit_borrowed_bytes(buf),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
