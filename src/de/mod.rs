use crate::de::key_deserializer::KeyDeserializer;
use crate::de::read::{Buf, MutBuf};
pub use crate::de::read::{IoRead, MutSliceRead, Read, SliceRead};
use crate::de::string_cache::StringCache;
use crate::Error;
use serde::de::{self, DeserializeOwned, Visitor};
use serde::{Deserialize, Deserializer as _};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::io::BufRead;
use std::str;

mod key_deserializer;
mod read;
mod string_cache;

pub fn from_slice<'de, T>(slice: &'de [u8]) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    let mut de = Deserializer::from_slice(slice)?;
    let value = T::deserialize(&mut de)?;
    de.end()?;
    Ok(value)
}

pub fn from_mut_slice<'de, T>(slice: &'de mut [u8]) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    let mut de = Deserializer::from_mut_slice(slice)?;
    let value = T::deserialize(&mut de)?;
    de.end()?;
    Ok(value)
}

pub fn from_reader<T, R>(reader: R) -> Result<T, Error>
where
    T: DeserializeOwned,
    R: BufRead,
{
    let mut de = Deserializer::from_reader(reader)?;
    let value = T::deserialize(&mut de)?;
    de.end()?;
    Ok(value)
}

pub struct Deserializer<'de, R> {
    reader: R,
    remaining_depth: u8,
    shared_strings: Option<StringCache<'de>>,
    shared_properties: Option<StringCache<'de>>,
}

impl<'de> Deserializer<'de, SliceRead<'de>> {
    pub fn from_slice(slice: &'de [u8]) -> Result<Self, Error> {
        Deserializer::new(SliceRead::new(slice))
    }
}

impl<'de> Deserializer<'de, MutSliceRead<'de>> {
    pub fn from_mut_slice(slice: &'de mut [u8]) -> Result<Self, Error> {
        Deserializer::new(MutSliceRead::new(slice))
    }
}

impl<'de, R> Deserializer<'de, IoRead<R>>
where
    R: BufRead,
{
    pub fn from_reader(reader: R) -> Result<Self, Error> {
        Deserializer::new(IoRead::new(reader))
    }
}

impl<'de, R> Deserializer<'de, R>
where
    R: Read<'de>,
{
    pub fn new(mut reader: R) -> Result<Self, Error> {
        let header = reader
            .read(4)?
            .ok_or_else(Error::eof_while_parsing_header)?;
        if !header.starts_with(b":)\n") {
            return Err(Error::invalid_header());
        }

        let info = header[3];
        if info & 0xf0 != 0 {
            return Err(Error::unsupported_version());
        }

        Ok(Deserializer {
            reader,
            remaining_depth: 128,
            shared_strings: if info & 0x02 != 0 {
                Some(StringCache::new())
            } else {
                None
            },
            shared_properties: if info & 0x01 != 0 {
                Some(StringCache::new())
            } else {
                None
            },
        })
    }

    pub fn end(&mut self) -> Result<(), Error> {
        match self.reader.next()? {
            Some(0xff) => Ok(()),
            Some(_) => Err(Error::trailing_data()),
            None => Ok(()),
        }
    }

    fn recursion_checked<F, T>(&mut self, f: F) -> Result<T, Error>
    where
        F: FnOnce(&mut Deserializer<'de, R>) -> Result<T, Error>,
    {
        self.remaining_depth -= 1;
        if self.remaining_depth == 0 {
            return Err(Error::recursion_limit_exceeded());
        }
        let r = f(self);
        self.remaining_depth += 1;
        r
    }

    fn parse_u8(&mut self) -> Result<u8, Error> {
        self.reader
            .next()?
            .ok_or_else(Error::eof_while_parsing_value)
    }

    fn parse_shared_string<V>(&mut self, reference: u16, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let s = self
            .shared_strings
            .as_ref()
            .and_then(|c| c.get(reference))
            .ok_or_else(Error::invalid_string_reference)?;
        match s {
            Cow::Borrowed(s) => visitor.visit_borrowed_str(*s),
            Cow::Owned(s) => visitor.visit_str(s),
        }
    }

    fn parse_vint(&mut self, byte_limit: usize) -> Result<u64, Error> {
        let mut value = 0;
        for _ in 0..byte_limit {
            let byte = self.parse_u8()?;
            let end = byte & 0x80 != 0;

            let shift = if end { 6 } else { 7 };
            value = value << shift | byte as u64 & 0x7f;

            if end {
                return Ok(value);
            }
        }

        Err(Error::unterminated_vint())
    }

    fn parse_i32<V>(&mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let vint = self.parse_vint(5)? as u32;
        let decoded = zigzag_i32(vint);
        visitor.visit_i32(decoded)
    }

    fn parse_i64<V>(&mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let vint = self.parse_vint(10)? as u64;
        let decoded = zigzag_i64(vint);
        visitor.visit_i64(decoded)
    }

    fn parse_7_bit_binary<'a>(&'a mut self) -> Result<Buf<'a, 'de>, Error> {
        let raw_len = self.parse_vint(10)?;
        let chunks = raw_len / 7;
        let remainder = raw_len % 7;
        let encoded_remainder = if remainder == 0 { 0 } else { remainder + 1 };

        let encoded_len = chunks
            .checked_mul(8)
            .and_then(|v| v.checked_add(encoded_remainder))
            .and_then(|v| usize::try_from(v).ok())
            .ok_or_else(Error::buffer_length_overflow)?;

        let mut buf = self
            .reader
            .read_mut(encoded_len)?
            .ok_or_else(Error::eof_while_parsing_value)?;

        let mut in_base = 0;
        let mut out_base = 0;
        for _ in 0..chunks {
            buf[out_base] = buf[in_base] << 1 | buf[in_base + 1] >> 6;
            buf[out_base + 1] = buf[in_base + 1] << 2 | buf[in_base + 2] >> 5;
            buf[out_base + 2] = buf[in_base + 2] << 3 | buf[in_base + 3] >> 4;
            buf[out_base + 3] = buf[in_base + 3] << 4 | buf[in_base + 4] >> 3;
            buf[out_base + 4] = buf[in_base + 4] << 5 | buf[in_base + 5] >> 2;
            buf[out_base + 5] = buf[in_base + 5] << 6 | buf[in_base + 6] >> 1;
            buf[out_base + 6] = buf[in_base + 6] << 7 | buf[in_base + 7];

            in_base += 8;
            out_base += 7;
        }

        if remainder > 0 {
            // the last byte is annoyingly right-aligned
            buf[in_base + remainder as usize] <<= 7 - remainder as usize;

            for i in 0..(remainder as usize) {
                buf[out_base + i] = buf[in_base + i] << i + 1 | buf[in_base + i + 1] >> 6 - i;
            }
        }

        let out = match buf {
            MutBuf::Short(buf) => Buf::Short(&buf[..raw_len as usize]),
            MutBuf::Long(buf) => Buf::Long(&buf[..raw_len as usize]),
        };
        Ok(out)
    }

    fn parse_big_integer<V>(&mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let buf = self.parse_7_bit_binary()?;
        if buf.len() <= 8 {
            let mut out = [0; 8];
            out[8 - buf.len()..].copy_from_slice(&buf);
            let v = u64::from_be_bytes(out);
            visitor.visit_u64(v)
        } else {
            // FIXME support via BigInteger "magic" type
            Err(Error::unsupported_big_integer())
        }
    }

    fn parse_f32<V>(&mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let buf = self
            .reader
            .read(5)?
            .ok_or_else(Error::eof_while_parsing_value)?;
        let raw = (buf[0] as u32) << 28
            | (buf[1] as u32) << 21
            | (buf[2] as u32) << 14
            | (buf[3] as u32) << 7
            | (buf[4] as u32);
        let value = f32::from_bits(raw);
        visitor.visit_f32(value)
    }

    fn parse_f64<V>(&mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let buf = self
            .reader
            .read(10)?
            .ok_or_else(Error::eof_while_parsing_value)?;
        let raw = (buf[0] as u64) << 63
            | (buf[1] as u64) << 56
            | (buf[2] as u64) << 49
            | (buf[3] as u64) << 42
            | (buf[4] as u64) << 35
            | (buf[5] as u64) << 28
            | (buf[6] as u64) << 21
            | (buf[7] as u64) << 14
            | (buf[8] as u64) << 7
            | (buf[9] as u64);
        let value = f64::from_bits(raw);
        visitor.visit_f64(value)
    }

    fn parse_big_decimal<V>(&mut self, _visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        // FIXME support via BigDecimal "magic" type
        Err(Error::unsupported_big_decimal())
    }

    fn parse_short_string<V>(&mut self, len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let buf = self
            .reader
            .read(len)?
            .ok_or_else(Error::eof_while_parsing_value)?;
        match buf {
            Buf::Short(buf) => {
                let s = str::from_utf8(buf).map_err(|_| Error::invalid_utf8())?;
                if let Some(shared_strings) = &mut self.shared_strings {
                    if s.len() <= 64 {
                        shared_strings.intern(Cow::Owned(s.to_string()));
                    }
                }

                visitor.visit_str(s)
            }
            Buf::Long(buf) => {
                let s = str::from_utf8(buf).map_err(|_| Error::invalid_utf8())?;
                if let Some(shared_strings) = &mut self.shared_strings {
                    if s.len() <= 64 {
                        shared_strings.intern(Cow::Borrowed(s));
                    }
                }

                visitor.visit_borrowed_str(s)
            }
        }
    }

    fn parse_long_string<V>(&mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let buf = self
            .reader
            .read_until(0xfc)?
            .ok_or_else(Error::eof_while_parsing_value)?;
        match buf {
            Buf::Short(buf) => {
                let s = str::from_utf8(buf).map_err(|_| Error::invalid_utf8())?;
                visitor.visit_str(s)
            }
            Buf::Long(buf) => {
                let s = str::from_utf8(buf).map_err(|_| Error::invalid_utf8())?;
                visitor.visit_borrowed_str(s)
            }
        }
    }

    fn parse_binary<V>(&mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let buf = self.parse_7_bit_binary()?;
        match buf {
            Buf::Short(buf) => visitor.visit_bytes(buf),
            Buf::Long(buf) => visitor.visit_borrowed_bytes(buf),
        }
    }

    fn parse_long_shared_string<V>(
        &mut self,
        reference_hi: u8,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let reference_lo = self.parse_u8()?;
        let reference = (reference_hi as u16) << 8 | reference_lo as u16;
        self.parse_shared_string(reference, visitor)
    }

    fn parse_array<V>(&mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.recursion_checked(|de| {
            let value = visitor.visit_seq(SeqAccess { de })?;
            match de.reader.next()? {
                Some(0xf9) => Ok(value),
                Some(_) => Err(Error::trailing_data()),
                None => Err(Error::eof_while_parsing_array()),
            }
        })
    }

    fn parse_map<V>(&mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.recursion_checked(|de| {
            let value = visitor.visit_map(MapAccess { de })?;
            match de.reader.next()? {
                Some(0xfb) => Ok(value),
                Some(_) => Err(Error::trailing_data()),
                None => Err(Error::eof_while_parsing_map()),
            }
        })
    }

    fn parse_raw_binary<V>(&mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let len = self.parse_vint(10)?;
        let len = usize::try_from(len).map_err(|_| Error::buffer_length_overflow())?;
        let buf = self
            .reader
            .read(len)?
            .ok_or_else(Error::eof_while_parsing_value)?;

        match buf {
            Buf::Short(buf) => visitor.visit_bytes(buf),
            Buf::Long(buf) => visitor.visit_borrowed_bytes(buf),
        }
    }

    fn parse_value<V>(&mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.parse_u8()? {
            0x00 => Err(Error::reserved_token()),
            token @ 0x01..=0x1f => self.parse_shared_string(token as u16 - 1, visitor),
            0x20 => visitor.visit_borrowed_str(""),
            0x21 => visitor.visit_unit(),
            0x22 => visitor.visit_bool(false),
            0x23 => visitor.visit_bool(true),
            0x24 => self.parse_i32(visitor),
            0x25 => self.parse_i64(visitor),
            0x26 => self.parse_big_integer(visitor),
            0x27 => Err(Error::reserved_token()),
            0x28 => self.parse_f32(visitor),
            0x29 => self.parse_f64(visitor),
            0x2a => self.parse_big_decimal(visitor),
            0x2b => Err(Error::reserved_token()),
            0x2c..=0x3f => Err(Error::reserved_token()),
            token @ 0x40..=0x5f => self.parse_short_string(token as usize - (0x40 - 1), visitor),
            token @ 0x60..=0x7f => self.parse_short_string(token as usize - (0x60 - 33), visitor),
            token @ 0x80..=0x9f => self.parse_short_string(token as usize - (0x80 - 2), visitor),
            token @ 0xa0..=0xbf => self.parse_short_string(token as usize - (0xa0 - 34), visitor),
            token @ 0xc0..=0xdf => visitor.visit_i32(zigzag_i32(token as u32 - 0xc0)),
            0xe0 => self.parse_long_string(visitor),
            0xe1..=0xe3 => Err(Error::reserved_token()),
            0xe4 => self.parse_long_string(visitor),
            0xe5..=0xe7 => Err(Error::reserved_token()),
            0xe8 => self.parse_binary(visitor),
            0xe9..=0xeb => Err(Error::reserved_token()),
            token @ 0xec..=0xef => self.parse_long_shared_string(token - 0xec, visitor),
            0xf0..=0xf7 => Err(Error::reserved_token()),
            0xf8 => self.parse_array(visitor),
            0xf9 => Err(Error::unexpected_token()),
            0xfa => self.parse_map(visitor),
            0xfb => Err(Error::unexpected_token()),
            0xfc => Err(Error::unexpected_token()),
            0xfd => self.parse_raw_binary(visitor),
            0xfe => Err(Error::reserved_token()),
            0xff => Err(Error::eof_while_parsing_value()),
        }
    }
}

impl<'de, 'a, R> serde::Deserializer<'de> for &'a mut Deserializer<'de, R>
where
    R: Read<'de>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_value(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.reader.peek()? {
            Some(0x21) => {
                self.reader.consume();
                visitor.visit_none()
            }
            _ => visitor.visit_some(self),
        }
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
        match self.reader.peek()? {
            Some(0xfa) => {
                self.reader.consume();
                self.recursion_checked(|de| {
                    let value = visitor.visit_enum(VariantAccess { de })?;
                    match de.reader.next()? {
                        Some(0xfb) => Ok(value),
                        Some(_) => Err(Error::trailing_data()),
                        None => Err(Error::eof_while_parsing_map()),
                    }
                })
            }
            Some(_) => visitor.visit_enum(UnitVariantAccess { de: self }),
            None => Err(Error::eof_while_parsing_value()),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string unit
        unit_struct seq tuple tuple_struct map struct identifier ignored_any
        bytes byte_buf
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

#[inline]
fn zigzag_i32(v: u32) -> i32 {
    ((v >> 1) as i32) ^ (-((v & 1) as i32))
}

#[inline]
fn zigzag_i64(v: u64) -> i64 {
    ((v >> 1) as i64) ^ (-((v & 1) as i64))
}

struct SeqAccess<'a, 'de, R> {
    de: &'a mut Deserializer<'de, R>,
}

impl<'de, R> de::SeqAccess<'de> for SeqAccess<'_, 'de, R>
where
    R: Read<'de>,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.de.reader.peek()? {
            Some(0xf9) => return Ok(None),
            Some(_) => {}
            None => return Err(Error::eof_while_parsing_array()),
        }

        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct MapAccess<'a, 'de, R> {
    de: &'a mut Deserializer<'de, R>,
}

impl<'de, R> de::MapAccess<'de> for MapAccess<'_, 'de, R>
where
    R: Read<'de>,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.de.reader.peek()? {
            Some(0xfb) => return Ok(None),
            Some(_) => {}
            None => return Err(Error::eof_while_parsing_map()),
        }

        seed.deserialize(KeyDeserializer { de: &mut *self.de })
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct VariantAccess<'a, 'de, R> {
    de: &'a mut Deserializer<'de, R>,
}

impl<'de, R> de::EnumAccess<'de> for VariantAccess<'_, 'de, R>
where
    R: Read<'de>,
{
    type Error = Error;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(KeyDeserializer { de: &mut *self.de })?;
        Ok((variant, self))
    }
}

impl<'de, R> de::VariantAccess<'de> for VariantAccess<'_, 'de, R>
where
    R: Read<'de>,
{
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Deserialize::deserialize(self.de)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.deserialize_seq(visitor)
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.deserialize_struct("", fields, visitor)
    }
}

struct UnitVariantAccess<'a, 'de, R> {
    de: &'a mut Deserializer<'de, R>,
}

impl<'de, R> de::EnumAccess<'de> for UnitVariantAccess<'_, 'de, R>
where
    R: Read<'de>,
{
    type Error = Error;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(&mut *self.de)?;
        Ok((variant, self))
    }
}

impl<'de, R> de::VariantAccess<'de> for UnitVariantAccess<'_, 'de, R>
where
    R: Read<'de>,
{
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
