use crate::de::read::Buf;
use crate::de::{zigzag_i32, Read};
use crate::value::BigDecimal;
use crate::{Deserializer, Error};
use serde::de::value::BorrowedStrDeserializer;
use serde::de::{DeserializeSeed, MapAccess, Visitor};
use serde::forward_to_deserialize_any;

#[derive(Copy, Clone)]
pub(crate) enum Stage {
    Scale,
    Buf,
}

pub(crate) struct BigDecimalDeserializer<'a, 'de, R> {
    pub(crate) de: &'a mut Deserializer<'de, R>,
    pub(crate) stage: Option<Stage>,
}

impl<'de, R> MapAccess<'de> for BigDecimalDeserializer<'_, 'de, R>
where
    R: Read<'de>,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.stage {
            Some(Stage::Scale) => seed
                .deserialize(BorrowedStrDeserializer::new(BigDecimal::SCALE_FIELD_NAME))
                .map(Some),
            Some(Stage::Buf) => seed
                .deserialize(BorrowedStrDeserializer::new(BigDecimal::VALUE_FIELD_NAME))
                .map(Some),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.stage {
            Some(Stage::Scale) => {
                self.stage = Some(Stage::Buf);
                seed.deserialize(BigDecimalValueDeserializer {
                    de: &mut *self.de,
                    stage: Stage::Scale,
                })
            }
            Some(Stage::Buf) => {
                self.stage = None;
                seed.deserialize(BigDecimalValueDeserializer {
                    de: &mut *self.de,
                    stage: Stage::Buf,
                })
            }
            None => panic!("next_value_seed called after end"),
        }
    }
}

struct BigDecimalValueDeserializer<'a, 'de, R> {
    de: &'a mut Deserializer<'de, R>,
    stage: Stage,
}

impl<'de, R> serde::Deserializer<'de> for BigDecimalValueDeserializer<'_, 'de, R>
where
    R: Read<'de>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.stage {
            Stage::Scale => {
                let scale = self.de.parse_vint(5)?;
                let scale = zigzag_i32(scale as u32);
                visitor.visit_i32(scale)
            }
            Stage::Buf => {
                let buf = self.de.parse_7_bit_binary()?;
                match buf {
                    Buf::Short(buf) => visitor.visit_bytes(buf),
                    Buf::Long(buf) => visitor.visit_borrowed_bytes(buf),
                }
            }
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
