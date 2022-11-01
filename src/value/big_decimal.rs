use crate::value::BigInteger;
use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_bytes::{ByteBuf, Bytes};
use std::fmt;

/// A parsed Smile `BigDecimal` value.
///
/// This is a "magic" type which corresponds to the `BigDecimal` type defined in Smile. It is intended to be used only
/// for serialization and deserialization, and it intentionally does *not* implement any kind of traditional big decimal
/// math API.
///
/// It should only be used with the `serde-smile` serializers and deserializers; it will produce a nonsensical encoding
/// when used with other `serde` libraries.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BigDecimal {
    value: BigInteger,
    scale: i32,
}

impl BigDecimal {
    pub(crate) const STRUCT_NAME: &'static str = "\0SmileBigDecimal";
    pub(crate) const SCALE_FIELD_NAME: &'static str = "\0SmileBigDecimalScale";
    pub(crate) const VALUE_FIELD_NAME: &'static str = "\0SmileBigDecimalValue";

    /// Creates a `BigDecimal` from an unscaled arbitrary precision integer and a scale.
    ///
    /// The value of the decimal is `value * 10^-scale`.
    #[inline]
    pub fn new(value: BigInteger, scale: i32) -> Self {
        BigDecimal { scale, value }
    }

    /// Returns the `BigDecimal`'s unscaled value.
    #[inline]
    pub fn unscaled_value(&self) -> &BigInteger {
        &self.value
    }

    /// Consumes the `BigDecimal`, returning its unscaled value.
    #[inline]
    pub fn into_unscaled_value(self) -> BigInteger {
        self.value
    }

    /// Returns the `BigDecimal`'s scale.
    #[inline]
    pub fn scale(&self) -> i32 {
        self.scale
    }
}

impl Serialize for BigDecimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct(Self::STRUCT_NAME, 2)?;
        s.serialize_field(Self::SCALE_FIELD_NAME, &self.scale)?;
        s.serialize_field(
            Self::VALUE_FIELD_NAME,
            &Bytes::new(self.value.as_be_bytes()),
        )?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for BigDecimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            Self::STRUCT_NAME,
            &[Self::SCALE_FIELD_NAME, Self::VALUE_FIELD_NAME],
            BigDecimalVisitor,
        )
    }
}

pub(crate) struct BigDecimalVisitor;

impl BigDecimalVisitor {
    pub(crate) fn finish_map<'de, A>(self, mut map: A) -> Result<BigDecimal, A::Error>
    where
        A: MapAccess<'de>,
    {
        let scale = map.next_value()?;

        match map.next_key::<BigDecimalKey>()? {
            Some(BigDecimalKey::Value) => {}
            Some(_) | None => return Err(de::Error::custom("expected big decimal value field")),
        }
        let value = map
            .next_value::<ByteBuf>()
            .map(|b| BigInteger::from_be_bytes(b.into_vec()))?;

        Ok(BigDecimal { scale, value })
    }
}

impl<'de> Visitor<'de> for BigDecimalVisitor {
    type Value = BigDecimal;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a big decimal")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        match map.next_key::<BigDecimalKey>()? {
            Some(BigDecimalKey::Scale) => {}
            Some(_) | None => return Err(de::Error::custom("expected big decimal scale field")),
        }
        self.finish_map(map)
    }
}

enum BigDecimalKey {
    Scale,
    Value,
}

impl<'de> Deserialize<'de> for BigDecimalKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KeyVisitor;

        impl<'de> Visitor<'de> for KeyVisitor {
            type Value = BigDecimalKey;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid big decimal field")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if v == BigDecimal::SCALE_FIELD_NAME {
                    Ok(BigDecimalKey::Scale)
                } else if v == BigDecimal::VALUE_FIELD_NAME {
                    Ok(BigDecimalKey::Value)
                } else {
                    Err(de::Error::custom("expected field with custom name"))
                }
            }
        }

        deserializer.deserialize_identifier(KeyVisitor)
    }
}
