use crate::de::read::Buf;
use crate::value::BigInteger;
use crate::Error;
use serde::de::value::BorrowedStrDeserializer;
use serde::de::{DeserializeSeed, MapAccess, Visitor};
use serde::forward_to_deserialize_any;

pub(crate) struct BigIntegerDeserializer<'a, 'de> {
    pub(crate) buf: Option<Buf<'a, 'de>>,
}

impl<'de> MapAccess<'de> for BigIntegerDeserializer<'_, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.buf.is_none() {
            return Ok(None);
        }

        seed.deserialize(BorrowedStrDeserializer::new(BigInteger::FIELD_NAME))
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(BigIntegerValueDeserializer {
            buf: self.buf.take().expect("next_value_seed called after end"),
        })
    }
}

struct BigIntegerValueDeserializer<'a, 'de> {
    buf: Buf<'a, 'de>,
}

impl<'de> serde::Deserializer<'de> for BigIntegerValueDeserializer<'_, 'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.buf {
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
