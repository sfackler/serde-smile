use crate::ser::big_integer_serializer::BigIntegerSerializer;
use crate::ser::key_serializer::KeySerializer;
use crate::{Error, Serializer};
use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use serde::Serialize;
use std::io::Write;

pub enum Mode {
    Normal,
    BigInteger,
}

pub struct Compound<'a, W> {
    pub ser: &'a mut Serializer<W>,
    pub mode: Mode,
}

impl<'a, W> SerializeSeq for Compound<'a, W>
where
    W: Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.ser.writer.write_all(&[0xf9]).map_err(Error::io)
    }
}

impl<'a, W> SerializeTuple for Compound<'a, W>
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

impl<'a, W> SerializeTupleStruct for Compound<'a, W>
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

impl<'a, W> SerializeTupleVariant for Compound<'a, W>
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
        self.ser.writer.write_all(&[0xf9, 0xfb]).map_err(Error::io)
    }
}

impl<'a, W> SerializeMap for Compound<'a, W>
where
    W: Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        key.serialize(KeySerializer {
            ser: &mut *self.ser,
        })
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.ser.writer.write_all(&[0xfb]).map_err(Error::io)
    }
}

impl<'a, W> SerializeStruct for Compound<'a, W>
where
    W: Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        match self.mode {
            Mode::BigInteger => value.serialize(BigIntegerSerializer {
                ser: &mut *self.ser,
            }),
            Mode::Normal => {
                self.ser.serialize_static_key(key)?;
                SerializeMap::serialize_value(self, value)
            }
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.mode {
            Mode::BigInteger => Ok(()),
            Mode::Normal => SerializeMap::end(self),
        }
    }
}

impl<'a, W> SerializeStructVariant for Compound<'a, W>
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
        self.ser.writer.write_all(&[0xfb, 0xfb]).map_err(Error::io)
    }
}
