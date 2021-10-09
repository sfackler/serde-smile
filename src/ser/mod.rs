use crate::ser::key_serializer::{KeySerializer, MaybeStatic};
use crate::ser::string_cache::StringCache;
use crate::Error;
use byteorder::WriteBytesExt;
use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use serde::{serde_if_integer128, Serialize};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::io::Write;

mod key_serializer;
mod string_cache;

pub struct Builder {
    raw_binary: bool,
    shared_strings: bool,
    shared_properties: bool,
}

impl Builder {
    pub fn raw_binary(&mut self, raw_binary: bool) -> &mut Self {
        self.raw_binary = raw_binary;
        self
    }

    pub fn shared_strings(&mut self, shared_strings: bool) -> &mut Self {
        self.shared_strings = shared_strings;
        self
    }

    pub fn shared_properties(&mut self, shared_properties: bool) -> &mut Self {
        self.shared_properties = shared_properties;
        self
    }

    pub fn build<W>(&self, mut writer: W) -> Result<Serializer<W>, Error>
    where
        W: Write,
    {
        let mut flags = 0;
        if self.raw_binary {
            flags |= 0x04;
        }
        if self.shared_strings {
            flags |= 0x02;
        }
        if self.shared_properties {
            flags |= 0x01;
        }
        let buf = [b':', b')', b'\n', flags];
        writer.write_all(&buf).map_err(Error::io)?;

        Ok(Serializer {
            writer,
            raw_binary: self.raw_binary,
            shared_strings: if self.shared_strings {
                Some(StringCache::new())
            } else {
                None
            },
            shared_properties: if self.shared_properties {
                Some(StringCache::new())
            } else {
                None
            },
        })
    }
}

pub struct Serializer<W> {
    writer: W,
    raw_binary: bool,
    shared_strings: Option<StringCache>,
    shared_properties: Option<StringCache>,
}

impl Serializer<()> {
    pub fn builder() -> Builder {
        Builder {
            raw_binary: true,
            shared_strings: false,
            shared_properties: true,
        }
    }
}

impl<W> Serializer<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Result<Self, Error> {
        Serializer::builder().build(writer)
    }

    pub fn into_inner(self) -> W {
        self.writer
    }

    pub fn end(mut self) -> Result<W, Error> {
        self.writer.write_u8(0xff).map_err(Error::io)?;

        Ok(self.writer)
    }

    fn serialize_vint(&mut self, v: u64) -> Result<(), Error> {
        let mut buf = [0; 10];

        let bytes = match v {
            0..=0x3f => {
                buf[0] = v as u8;
                1
            }
            0x40..=0x1f_ff => {
                buf[0] = (v >> 6) as u8;
                buf[1] = (v & 0x3f) as u8;
                2
            }
            0x20_00..=0x0f_ff_ff => {
                buf[0] = (v >> 13) as u8;
                buf[1] = (v >> 6) as u8 & 0x7f;
                buf[2] = v as u8 & 0x3f;
                3
            }
            0x10_00_00..=0x07_ff_ff_ff => {
                buf[0] = (v >> 20) as u8;
                buf[1] = (v >> 13) as u8 & 0x7f;
                buf[2] = (v >> 6) as u8 & 0x7f;
                buf[3] = v as u8 & 0x3f;
                4
            }
            0x08_00_00_00..=0x03_ff_ff_ff_ff => {
                buf[0] = (v >> 27) as u8;
                buf[1] = (v >> 20) as u8 & 0x7f;
                buf[2] = (v >> 13) as u8 & 0x7f;
                buf[3] = (v >> 6) as u8 & 0x7f;
                buf[4] = v as u8 & 0x3f;
                5
            }
            0x04_00_00_00_00..=0x01_ff_ff_ff_ff_ff => {
                buf[0] = (v >> 34) as u8;
                buf[1] = (v >> 27) as u8 & 0x7f;
                buf[2] = (v >> 20) as u8 & 0x7f;
                buf[3] = (v >> 13) as u8 & 0x7f;
                buf[4] = (v >> 6) as u8 & 0x7f;
                buf[5] = v as u8 & 0x3f;
                6
            }
            0x02_00_00_00_00_00..=0xff_ff_ff_ff_ff_ff => {
                buf[0] = (v >> 41) as u8;
                buf[1] = (v >> 34) as u8 & 0x7f;
                buf[2] = (v >> 27) as u8 & 0x7f;
                buf[3] = (v >> 20) as u8 & 0x7f;
                buf[4] = (v >> 13) as u8 & 0x7f;
                buf[5] = (v >> 6) as u8 & 0x7f;
                buf[6] = v as u8 & 0x3f;
                7
            }
            0x01_00_00_00_00_00_00..=0x7f_ff_ff_ff_ff_ff_ff => {
                buf[0] = (v >> 48) as u8;
                buf[1] = (v >> 41) as u8 & 0x7f;
                buf[2] = (v >> 34) as u8 & 0x7f;
                buf[3] = (v >> 27) as u8 & 0x7f;
                buf[4] = (v >> 20) as u8 & 0x7f;
                buf[5] = (v >> 13) as u8 & 0x7f;
                buf[6] = (v >> 6) as u8 & 0x7f;
                buf[7] = v as u8 & 0x3f;
                8
            }
            0x80_00_00_00_00_00_00..=0x3f_ff_ff_ff_ff_ff_ff_ff => {
                buf[0] = (v >> 55) as u8;
                buf[1] = (v >> 48) as u8 & 0x7f;
                buf[2] = (v >> 41) as u8 & 0x7f;
                buf[3] = (v >> 34) as u8 & 0x7f;
                buf[4] = (v >> 27) as u8 & 0x7f;
                buf[5] = (v >> 20) as u8 & 0x7f;
                buf[6] = (v >> 13) as u8 & 0x7f;
                buf[7] = (v >> 6) as u8 & 0x7f;
                buf[8] = v as u8 & 0x3f;
                9
            }
            0x40_00_00_00_00_00_00_00..=0xff_ff_ff_ff_ff_ff_ff_ff => {
                buf[0] = (v >> 62) as u8;
                buf[1] = (v >> 55) as u8 & 0x7f;
                buf[2] = (v >> 48) as u8 & 0x7f;
                buf[3] = (v >> 41) as u8 & 0x7f;
                buf[4] = (v >> 34) as u8 & 0x7f;
                buf[5] = (v >> 27) as u8 & 0x7f;
                buf[6] = (v >> 20) as u8 & 0x7f;
                buf[7] = (v >> 13) as u8 & 0x7f;
                buf[8] = (v >> 6) as u8 & 0x7f;
                buf[9] = v as u8 & 0x3f;
                10
            }
        };

        buf[bytes - 1] |= 0x80;
        self.writer.write_all(&buf[..bytes]).map_err(Error::io)
    }

    fn serialize_shared_str(&mut self, v: &str) -> Result<bool, Error> {
        let shared_strings = match &mut self.shared_strings {
            Some(shared_strings) => shared_strings,
            None => return Ok(false),
        };

        if v.len() > 64 {
            return Ok(false);
        }

        match shared_strings.get(v) {
            Some(backref) => {
                if backref <= 30 {
                    self.writer.write_u8(backref as u8 + 1).map_err(Error::io)?;
                } else {
                    let buf = [0xec | (backref >> 8) as u8, backref as u8];
                    self.writer.write_all(&buf).map_err(Error::io)?;
                }
                Ok(true)
            }
            None => {
                shared_strings.intern(Cow::Owned(v.to_string()));
                Ok(false)
            }
        }
    }

    fn serialize_7_bit_binary(&mut self, v: &[u8]) -> Result<(), Error> {
        self.serialize_vint(v.len() as u64)?;

        let mut it = v.chunks_exact(7);
        while let Some(chunk) = it.next() {
            let buf = [
                chunk[0] >> 1,
                ((chunk[0] << 6) | (chunk[1] >> 2)) & 0x7f,
                ((chunk[1] << 5) | (chunk[2] >> 3)) & 0x7f,
                ((chunk[2] << 4) | (chunk[3] >> 4)) & 0x7f,
                ((chunk[3] << 3) | (chunk[4] >> 5)) & 0x7f,
                ((chunk[4] << 2) | (chunk[5] >> 6)) & 0x7f,
                ((chunk[5] << 1) | (chunk[6] >> 7)) & 0x7f,
                chunk[6] & 0x7f,
            ];
            self.writer.write_all(&buf).map_err(Error::io)?;
        }

        let mut buf = [0; 7];
        for (i, &b) in it.remainder().iter().enumerate() {
            buf[i] |= b >> (i + 1);
            buf[i + 1] = (b << (6 - i)) & 0x7f;
        }
        self.writer
            .write_all(&buf[..it.remainder().len() + 1])
            .map_err(Error::io)
    }

    fn serialize_big_integer(&mut self, v: &[u8]) -> Result<(), Error> {
        self.writer.write_u8(0x26).map_err(Error::io)?;
        self.serialize_7_bit_binary(v)
    }

    fn serialize_static_key(&mut self, v: &'static str) -> Result<(), Error> {
        KeySerializer { ser: self }.serialize_maybe_static_str(MaybeStatic::Static(v))
    }
}

impl<'a, W> serde::Serializer for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = Self;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        let b = if v { 0x23 } else { 0x22 };
        self.writer.write_u8(b).map_err(Error::io)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        let zigzag = ((v << 1) ^ (v >> 31)) as u32 as u64;

        if zigzag <= 32 {
            self.writer.write_u8(0xc0 + zigzag as u8).map_err(Error::io)
        } else {
            self.writer.write_u8(0x24).map_err(Error::io)?;
            self.serialize_vint(zigzag)
        }
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        match i32::try_from(v) {
            Ok(v) => self.serialize_i32(v),
            Err(_) => {
                self.writer.write_u8(0x25).map_err(Error::io)?;
                let zigzag = ((v << 1) ^ (v >> 63)) as u64;
                self.serialize_vint(zigzag)
            }
        }
    }

    serde_if_integer128! {
        fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
            match i64::try_from(v) {
                Ok(v) => self.serialize_i64(v),
                Err(_) => self.serialize_big_integer(&v.to_be_bytes()),
            }
        }
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        match i64::try_from(v) {
            Ok(v) => self.serialize_i64(v),
            Err(_) => self.serialize_big_integer(&v.to_be_bytes()),
        }
    }

    serde_if_integer128! {
        fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
            match i64::try_from(v) {
                Ok(v) => self.serialize_i64(v),
                Err(_) => self.serialize_big_integer(&v.to_be_bytes()),
            }
        }
    }

    // we could use serialize_7_bit_binary here, but working directly with the integer representation is a bit faster
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        let bits = v.to_bits().to_le();
        let buf = [
            0x28,
            bits as u8 & 0x7f,
            (bits >> 7) as u8 & 0x7f,
            (bits >> 14) as u8 & 0x7f,
            (bits >> 21) as u8 & 0x7f,
            (bits >> 28) as u8 & 0x7f,
        ];
        self.writer.write_all(&buf).map_err(Error::io)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        let bits = v.to_bits().to_le();
        let buf = [
            0x29,
            bits as u8 & 0x7f,
            (bits >> 7) as u8 & 0x7f,
            (bits >> 14) as u8 & 0x7f,
            (bits >> 21) as u8 & 0x7f,
            (bits >> 28) as u8 & 0x7f,
            (bits >> 35) as u8 & 0x7f,
            (bits >> 42) as u8 & 0x7f,
            (bits >> 49) as u8 & 0x7f,
            (bits >> 56) as u8 & 0x7f,
            (bits >> 63) as u8 & 0x7f,
        ];
        self.writer.write_all(&buf).map_err(Error::io)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(v.encode_utf8(&mut [0; 4]))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        if v.is_empty() {
            return self.writer.write_u8(0x20).map_err(Error::io);
        }

        if self.serialize_shared_str(v)? {
            return Ok(());
        }

        let (tiny, short, long) = if v.is_ascii() {
            (0x40, 0x60, 0xe0)
        } else {
            (0x80, 0xa0, 0xe4)
        };

        if v.len() <= 32 {
            self.writer
                .write_u8(tiny | (v.len() as u8 - 1))
                .map_err(Error::io)?;
            self.writer.write_all(v.as_bytes()).map_err(Error::io)?;
        } else if v.len() <= 64 {
            self.writer
                .write_u8(short | (v.len() as u8 - 33))
                .map_err(Error::io)?;
            self.writer.write_all(v.as_bytes()).map_err(Error::io)?;
        } else {
            self.writer.write_u8(long).map_err(Error::io)?;
            self.writer.write_all(v.as_bytes()).map_err(Error::io)?;
            self.writer.write_u8(0xfc).map_err(Error::io)?;
        }

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        if self.raw_binary {
            self.writer.write_u8(0xfd).map_err(Error::io)?;
            self.serialize_vint(v.len() as u64)?;
            self.writer.write_all(v).map_err(Error::io)
        } else {
            self.writer.write_u8(0xe8).map_err(Error::io)?;
            self.serialize_7_bit_binary(v)
        }
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_u8(0x21).map_err(Error::io)
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        let mut ser = self.serialize_map(Some(1))?;
        SerializeStruct::serialize_field(&mut ser, variant, value)?;
        SerializeStruct::end(ser)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.writer.write_u8(0xf8).map_err(Error::io)?;
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let ser = self.serialize_map(Some(1))?;
        ser.serialize_static_key(variant)?;
        ser.serialize_seq(Some(len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.writer.write_u8(0xfa).map_err(Error::io)?;
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let ser = self.serialize_map(Some(1))?;
        ser.serialize_static_key(variant)?;
        ser.serialize_map(Some(len))
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'a, W> SerializeSeq for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_u8(0xf9).map_err(Error::io)
    }
}

impl<'a, W> SerializeTuple for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl<'a, W> SerializeTupleStruct for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl<'a, W> SerializeTupleVariant for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(&[0xf9, 0xfb]).map_err(Error::io)
    }
}

impl<'a, W> SerializeMap for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        key.serialize(KeySerializer { ser: self })
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_u8(0xfb).map_err(Error::io)
    }
}

impl<'a, W> SerializeStruct for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.serialize_static_key(key)?;
        SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeMap::end(self)
    }
}

impl<'a, W> SerializeStructVariant for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(&[0xfb, 0xfb]).map_err(Error::io)
    }
}
