use crate::de::Read;
use crate::{Deserializer, Error};
use serde::Deserialize;
use std::marker::PhantomData;

/// An iterator that deserializes a stream into multiple Smile values.
///
/// A stream deserializer can be created from any Smile deserializer using the [`Deserializer::into_iter`] method.
///
/// The iterator will stop at either the Smile end-of-stream marker or the end of the underlying reader's stream.
pub struct StreamDeserializer<'de, R, T> {
    pub(crate) de: Deserializer<'de, R>,
    pub(crate) done: bool,
    pub(crate) _p: PhantomData<T>,
}

impl<'de, R, T> Iterator for StreamDeserializer<'de, R, T>
where
    R: Read<'de>,
    T: Deserialize<'de>,
{
    type Item = Result<T, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        if let Err(e) = self.de.read_header() {
            self.done = true;
            return Some(Err(e));
        }

        match self.de.reader.peek() {
            Ok(Some(0xff)) => {
                self.de.reader.consume();
                self.done = true;
                return None;
            }
            Ok(Some(_)) => {}
            Ok(None) => {
                self.done = true;
                return None;
            }
            Err(e) => {
                self.done = true;
                return Some(Err(e));
            }
        }

        match T::deserialize(&mut self.de) {
            Ok(value) => Some(Ok(value)),
            Err(e) => {
                self.done = true;
                Some(Err(e))
            }
        }
    }
}
