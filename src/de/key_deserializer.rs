use crate::de::read::Buf;
use crate::de::{Deserializer, Read};
use crate::Error;
use serde::de::{self, Visitor};
use serde::{forward_to_deserialize_any, serde_if_integer128};
use std::borrow::Cow;
use std::ops::Deref;
use std::str;

pub(crate) struct KeyDeserializer<'a, 'de, R> {
    pub(crate) de: &'a mut Deserializer<'de, R>,
}

impl<'de, R> KeyDeserializer<'_, 'de, R>
where
    R: Read<'de>,
{
    fn parse_shared_str<'a>(&'a mut self, reference: u16) -> Result<Str<'a, 'de>, Error> {
        let cow = self
            .de
            .shared_properties
            .as_ref()
            .and_then(|c| c.get(reference))
            .ok_or_else(Error::invalid_string_reference)?;

        let s = match cow {
            Cow::Borrowed(s) => Str::Long(s),
            Cow::Owned(s) => Str::Short(s),
        };

        Ok(s)
    }

    fn parse_long_shared_str<'a>(&'a mut self, reference_hi: u8) -> Result<Str<'a, 'de>, Error> {
        let reference_lo = self.de.parse_u8()?;
        let reference = (reference_hi as u16) << 8 | reference_lo as u16;
        self.parse_shared_str(reference)
    }

    fn parse_str_inner<'a, F>(&'a mut self, f: F) -> Result<Str<'a, 'de>, Error>
    where
        F: FnOnce(&'a mut R) -> Result<Option<Buf<'a, 'de>>, Error>,
    {
        let buf = f(&mut self.de.reader)?.ok_or_else(Error::eof_while_parsing_value)?;

        match buf {
            Buf::Short(buf) => {
                let s = str::from_utf8(buf).map_err(|_| Error::invalid_utf8())?;
                if s.len() <= 64 {
                    if let Some(shared_properties) = &mut self.de.shared_properties {
                        shared_properties.intern(Cow::Owned(s.to_string()));
                    }
                }

                Ok(Str::Short(s))
            }
            Buf::Long(buf) => {
                let s = str::from_utf8(buf).map_err(|_| Error::invalid_utf8())?;
                if s.len() <= 64 {
                    if let Some(shared_properties) = &mut self.de.shared_properties {
                        shared_properties.intern(Cow::Borrowed(s));
                    }
                }

                Ok(Str::Long(s))
            }
        }
    }

    fn parse_long_str<'a>(&'a mut self) -> Result<Str<'a, 'de>, Error> {
        self.parse_str_inner(|r| r.read_until(0xfc))
    }

    fn parse_short_str<'a>(&'a mut self, len: usize) -> Result<Str<'a, 'de>, Error> {
        self.parse_str_inner(|r| r.read(len))
    }

    fn parse_str<'a>(&'a mut self) -> Result<Str<'a, 'de>, Error> {
        match self.de.parse_u8()? {
            0x00..=0x1f => Err(Error::reserved_token()),
            0x20 => Ok(Str::Long("")),
            0x21..=0x2f => Err(Error::reserved_token()),
            token @ 0x30..=0x33 => self.parse_long_shared_str(token - 0x30),
            0x34 => self.parse_long_str(),
            0x35..=0x39 => Err(Error::reserved_token()),
            0x3a => Err(Error::unexpected_token()),
            0x3b..=0x3f => Err(Error::reserved_token()),
            token @ 0x40..=0x7f => self.parse_shared_str(token as u16 - 0x40),
            token @ 0x80..=0xbf => self.parse_short_str(token as usize - (0x80 - 1)),
            token @ 0xc0..=0xf7 => self.parse_short_str(token as usize - (0xc0 - 2)),
            0xf8..=0xfa => Err(Error::reserved_token()),
            0xfb => Err(Error::unexpected_token()),
            0xfc..=0xff => Err(Error::reserved_token()),
        }
    }
}

macro_rules! deserialize_integer_key {
    ($method:ident => $visit:ident) => {
        fn $method<V>(mut self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            let s = self.parse_str()?;
            match (s.parse(), s) {
                (Ok(integer), _) => visitor.$visit(integer),
                (Err(_), Str::Short(s)) => visitor.visit_str(s),
                (Err(_), Str::Long(s)) => visitor.visit_borrowed_str(s),
            }
        }
    };
}

impl<'de, R> de::Deserializer<'de> for KeyDeserializer<'_, 'de, R>
where
    R: Read<'de>,
{
    type Error = Error;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.parse_str()? {
            Str::Short(s) => visitor.visit_str(s),
            Str::Long(s) => visitor.visit_borrowed_str(s),
        }
    }

    deserialize_integer_key!(deserialize_i8 => visit_i8);
    deserialize_integer_key!(deserialize_i16 => visit_i16);
    deserialize_integer_key!(deserialize_i32 => visit_i32);
    deserialize_integer_key!(deserialize_i64 => visit_i64);

    serde_if_integer128! {
        deserialize_integer_key!(deserialize_i128 => visit_i128);
    }

    deserialize_integer_key!(deserialize_u8 => visit_u8);
    deserialize_integer_key!(deserialize_u16 => visit_u16);
    deserialize_integer_key!(deserialize_u32 => visit_u32);
    deserialize_integer_key!(deserialize_u64 => visit_u64);

    serde_if_integer128! {
        deserialize_integer_key!(deserialize_u128 => visit_u128);
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(EnumAccess { de: self })
    }

    forward_to_deserialize_any! {
        bool f32 f64 char str string bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

struct EnumAccess<'a, 'de, R> {
    de: KeyDeserializer<'a, 'de, R>,
}

impl<'de, R> de::EnumAccess<'de> for EnumAccess<'_, 'de, R>
where
    R: Read<'de>,
{
    type Error = Error;

    type Variant = UnitVariantAccess;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(self.de)?;
        Ok((variant, UnitVariantAccess))
    }
}

struct UnitVariantAccess;

impl<'de> de::VariantAccess<'de> for UnitVariantAccess {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        Err(de::Error::invalid_type(
            de::Unexpected::UnitVariant,
            &"newtype variant",
        ))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            de::Unexpected::UnitVariant,
            &"tuple variant",
        ))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            de::Unexpected::UnitVariant,
            &"struct variant",
        ))
    }
}

enum Str<'a, 'de> {
    Short(&'a str),
    Long(&'de str),
}

impl Deref for Str<'_, '_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Str::Short(s) => *s,
            Str::Long(s) => *s,
        }
    }
}
