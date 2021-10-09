use crate::ser::Serializer;
use crate::Error;
use byteorder::WriteBytesExt;
use serde::ser::Impossible;
use serde::{serde_if_integer128, Serialize, Serializer as _};
use std::borrow::Cow;
use std::io::Write;
use std::ops::Deref;

pub(crate) struct KeySerializer<'a, W> {
    pub(crate) ser: &'a mut Serializer<W>,
}

impl<'a, W> KeySerializer<'a, W>
where
    W: Write,
{
    fn serialize_int<I>(self, v: I) -> Result<(), Error>
    where
        I: itoa::Integer,
    {
        let mut buffer = itoa::Buffer::new();
        let v = buffer.format(v);
        self.serialize_str(v)
    }

    fn serialize_shared_property(&mut self, v: MaybeStatic<'_, str>) -> Result<bool, Error> {
        let shared_properties = match &mut self.ser.shared_properties {
            Some(shared_properties) => shared_properties,
            None => return Ok(false),
        };

        if v.len() > 64 {
            return Ok(false);
        }

        match shared_properties.get(&v) {
            Some(backref) => {
                if backref <= 63 {
                    self.ser
                        .writer
                        .write_u8(0x40 + backref as u8)
                        .map_err(Error::io)?;
                } else {
                    let buf = [0x30 | (backref >> 8) as u8, backref as u8];
                    self.ser.writer.write_all(&buf).map_err(Error::io)?;
                }
                Ok(true)
            }
            None => {
                let cow = match v {
                    MaybeStatic::Static(v) => Cow::Borrowed(v),
                    MaybeStatic::Nonstatic(v) => Cow::Owned(v.to_string()),
                };
                shared_properties.intern(cow);
                Ok(false)
            }
        }
    }

    pub(crate) fn serialize_maybe_static_str(
        &mut self,
        v: MaybeStatic<'_, str>,
    ) -> Result<(), Error> {
        if v.is_empty() {
            return self.ser.writer.write_u8(0x20).map_err(Error::io);
        }

        if self.serialize_shared_property(v)? {
            return Ok(());
        }

        if v.len() <= 64 && v.is_ascii() {
            self.ser
                .writer
                .write_u8(0x80 + v.len() as u8 - 1)
                .map_err(Error::io)?;
            self.ser.writer.write_all(v.as_bytes()).map_err(Error::io)?;
        } else if v.len() < 57 {
            self.ser
                .writer
                .write_u8(0xc0 + v.len() as u8 - 2)
                .map_err(Error::io)?;
            self.ser.writer.write_all(v.as_bytes()).map_err(Error::io)?;
        } else {
            self.ser.writer.write_u8(0x34).map_err(Error::io)?;
            self.ser.writer.write_all(v.as_bytes()).map_err(Error::io)?;
            self.ser.writer.write_u8(0xfc).map_err(Error::io)?;
        }

        Ok(())
    }
}

impl<'a, W> serde::Serializer for KeySerializer<'a, W>
where
    W: Write,
{
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;

    type SerializeTuple = Impossible<(), Error>;

    type SerializeTupleStruct = Impossible<(), Error>;

    type SerializeTupleVariant = Impossible<(), Error>;

    type SerializeMap = Impossible<(), Error>;

    type SerializeStruct = Impossible<(), Error>;

    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(Error::key_must_be_a_string())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_int(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_int(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_int(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.serialize_int(v)
    }

    serde_if_integer128! {
        fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
            self.serialize_int(v)
        }
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_int(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_int(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_int(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_int(v)
    }

    serde_if_integer128! {
        fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
            self.serialize_int(v)
        }
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(Error::key_must_be_a_string())
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(Error::key_must_be_a_string())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(v.encode_utf8(&mut [0; 4]))
    }

    fn serialize_str(mut self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.serialize_maybe_static_str(MaybeStatic::Nonstatic(v))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::key_must_be_a_string())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(Error::key_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::key_must_be_a_string())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::key_must_be_a_string())
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(Error::key_must_be_a_string())
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(Error::key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::key_must_be_a_string())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::key_must_be_a_string())
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

pub(crate) enum MaybeStatic<'a, T>
where
    T: ?Sized + 'static,
{
    Static(&'static T),
    Nonstatic(&'a T),
}

impl<T> Copy for MaybeStatic<'_, T> where T: ?Sized {}

impl<T> Clone for MaybeStatic<'_, T>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        match self {
            MaybeStatic::Static(v) => MaybeStatic::Static(*v),
            MaybeStatic::Nonstatic(v) => MaybeStatic::Nonstatic(*v),
        }
    }
}

impl<'a, T> Deref for MaybeStatic<'a, T>
where
    T: ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            MaybeStatic::Static(v) => *v,
            MaybeStatic::Nonstatic(v) => *v,
        }
    }
}
