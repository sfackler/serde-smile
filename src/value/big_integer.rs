use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_bytes::{ByteBuf, Bytes};
use std::fmt;

/// A parsed Smile `BigInteger` value.
///
/// This is a "magic" type which corresponds to the `BigInteger` type defined in Smile. It is intended to be used only
/// for serialization and deserialization; it intentionally does *not* implement any kind of traditional big integer
/// math API.
///
/// It should only be used with the `serde-smile` serializers and deserializers; it will produce a nonsensical encoding
/// when used with other `serde` libraries.
#[derive(Clone, PartialEq, Debug)]
pub struct BigInteger(Vec<u8>);

impl BigInteger {
    pub(crate) const STRUCT_NAME: &'static str = "\0SmileBigInteger";
    pub(crate) const FIELD_NAME: &'static str = "\0SmileBigIntegerField";

    /// Creates a `BigInteger` from its representation as a byte buffer in two's complement big-endian.
    #[inline]
    pub fn from_be_bytes(buf: Vec<u8>) -> Self {
        BigInteger(buf)
    }

    /// Returns a slice containing the two's complement big-endian representation of the `BigInteger`.
    #[inline]
    pub fn as_be_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Consumes the `BigInteger`, returning a byte buffer containing its two's complement big-endian representation.
    #[inline]
    pub fn into_be_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl Serialize for BigInteger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct(Self::STRUCT_NAME, 1)?;
        s.serialize_field(Self::FIELD_NAME, &Bytes::new(&self.0))?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for BigInteger {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BigIntegerVisitor;

        impl<'de> Visitor<'de> for BigIntegerVisitor {
            type Value = BigInteger;

            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("a big integer")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let value = map.next_key::<BigIntegerKey>()?;
                if value.is_none() {
                    return Err(de::Error::custom("big integer key not found"));
                }
                map.next_value::<ByteBuf>()
                    .map(|b| BigInteger(b.into_vec()))
            }
        }

        deserializer.deserialize_struct(Self::STRUCT_NAME, &[Self::FIELD_NAME], BigIntegerVisitor)
    }
}

struct BigIntegerKey;

impl<'de> Deserialize<'de> for BigIntegerKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KeyVisitor;

        impl<'de> Visitor<'de> for KeyVisitor {
            type Value = BigIntegerKey;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid big integer field")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if v == BigInteger::FIELD_NAME {
                    Ok(BigIntegerKey)
                } else {
                    Err(de::Error::custom("expected field with custom name"))
                }
            }
        }

        deserializer.deserialize_identifier(KeyVisitor)
    }
}
