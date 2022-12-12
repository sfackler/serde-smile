use crate::Error;
use memchr::memchr;
use std::io::BufRead;
use std::mem;
use std::ops::{Deref, DerefMut};

pub(crate) mod private {
    pub trait Sealed {}
}

/// A trait used by [`Deserializer`](crate::Deserializer) to abstract over input types.
///
/// This trait is sealed and cannot be implemented outside of `serde_smile`. The contents of the trait are not
/// considered part of the crate's public API and are subject to change at any time.
pub trait Read<'de>: private::Sealed {
    #[doc(hidden)]
    fn next(&mut self) -> Result<Option<u8>, Error>;

    #[doc(hidden)]
    fn peek(&mut self) -> Result<Option<u8>, Error>;

    #[doc(hidden)]
    fn consume(&mut self);

    #[doc(hidden)]
    fn read<'a>(&'a mut self, n: usize) -> Result<Option<Buf<'a, 'de>>, Error>;

    #[doc(hidden)]
    fn read_mut<'a>(&'a mut self, n: usize) -> Result<Option<MutBuf<'a, 'de>>, Error>;

    #[doc(hidden)]
    fn read_until<'a>(&'a mut self, end: u8) -> Result<Option<Buf<'a, 'de>>, Error>;
}

pub enum Buf<'a, 'de> {
    Short(&'a [u8]),
    Long(&'de [u8]),
}

impl Deref for Buf<'_, '_> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Buf::Short(buf) => buf,
            Buf::Long(buf) => buf,
        }
    }
}

pub enum MutBuf<'a, 'de> {
    Short(&'a mut [u8]),
    Long(&'de mut [u8]),
}

impl Deref for MutBuf<'_, '_> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            MutBuf::Short(buf) => buf,
            MutBuf::Long(buf) => buf,
        }
    }
}

impl DerefMut for MutBuf<'_, '_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MutBuf::Short(buf) => buf,
            MutBuf::Long(buf) => buf,
        }
    }
}

/// A [`Read`] implementation for shared slices.
pub struct SliceRead<'a> {
    slice: &'a [u8],
    index: usize,
    buf: Vec<u8>,
}

impl<'a> SliceRead<'a> {
    /// Creates a new `SliceRead`.
    pub fn new(slice: &'a [u8]) -> Self {
        SliceRead {
            slice,
            index: 0,
            buf: vec![],
        }
    }
}

impl private::Sealed for SliceRead<'_> {}

impl<'de> Read<'de> for SliceRead<'de> {
    #[inline]
    fn next(&mut self) -> Result<Option<u8>, Error> {
        if self.index < self.slice.len() {
            let ch = self.slice[self.index];
            self.index += 1;
            Ok(Some(ch))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn peek(&mut self) -> Result<Option<u8>, Error> {
        if self.index < self.slice.len() {
            Ok(Some(self.slice[self.index]))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn consume(&mut self) {
        self.index += 1;
    }

    #[inline]
    fn read<'a>(&'a mut self, n: usize) -> Result<Option<Buf<'a, 'de>>, Error> {
        let s = &self.slice[self.index..];
        if n <= s.len() {
            self.index += n;
            Ok(Some(Buf::Long(&s[..n])))
        } else {
            Ok(None)
        }
    }

    fn read_mut<'a>(&'a mut self, n: usize) -> Result<Option<MutBuf<'a, 'de>>, Error> {
        let s = &self.slice[self.index..];
        if n <= s.len() {
            self.index += n;
            self.buf.clear();
            self.buf.extend_from_slice(&s[..n]);
            Ok(Some(MutBuf::Short(&mut self.buf)))
        } else {
            Ok(None)
        }
    }

    fn read_until<'a>(&'a mut self, end: u8) -> Result<Option<Buf<'a, 'de>>, Error> {
        let s = &self.slice[self.index..];
        match memchr(end, s) {
            Some(end) => {
                self.index += end + 1;
                Ok(Some(Buf::Long(&s[..end])))
            }
            None => Ok(None),
        }
    }
}

/// A [`Read`] implementation for mutable slices.
pub struct MutSliceRead<'a> {
    slice: &'a mut [u8],
}

impl<'a> MutSliceRead<'a> {
    /// Creates a new `MutSliceRead`.
    pub fn new(slice: &'a mut [u8]) -> Self {
        MutSliceRead { slice }
    }
}

impl private::Sealed for MutSliceRead<'_> {}

impl<'de> Read<'de> for MutSliceRead<'de> {
    fn next(&mut self) -> Result<Option<u8>, Error> {
        if !self.slice.is_empty() {
            let slice = mem::take(&mut self.slice);
            let b = slice[0];

            self.slice = &mut slice[1..];
            Ok(Some(b))
        } else {
            Ok(None)
        }
    }

    fn peek(&mut self) -> Result<Option<u8>, Error> {
        if !self.slice.is_empty() {
            Ok(Some(self.slice[0]))
        } else {
            Ok(None)
        }
    }

    fn consume(&mut self) {
        let slice = mem::take(&mut self.slice);
        self.slice = &mut slice[1..];
    }

    fn read<'a>(&'a mut self, n: usize) -> Result<Option<Buf<'a, 'de>>, Error> {
        if n <= self.slice.len() {
            let (a, b) = mem::take(&mut self.slice).split_at_mut(n);
            self.slice = b;
            Ok(Some(Buf::Long(a)))
        } else {
            Ok(None)
        }
    }

    fn read_mut<'a>(&'a mut self, n: usize) -> Result<Option<MutBuf<'a, 'de>>, Error> {
        if n <= self.slice.len() {
            let (a, b) = mem::take(&mut self.slice).split_at_mut(n);
            self.slice = b;
            Ok(Some(MutBuf::Long(a)))
        } else {
            Ok(None)
        }
    }

    fn read_until<'a>(&'a mut self, end: u8) -> Result<Option<Buf<'a, 'de>>, Error> {
        match memchr(end, self.slice) {
            Some(end) => {
                let (a, b) = mem::take(&mut self.slice).split_at_mut(end);
                self.slice = &mut b[1..];
                Ok(Some(Buf::Long(a)))
            }
            None => Ok(None),
        }
    }
}

/// A [`Read`] implementation for buffered IO streams.
pub struct IoRead<R> {
    reader: R,
    buf: Vec<u8>,
}

impl<R> IoRead<R>
where
    R: BufRead,
{
    /// Creates a new `IoRead`.
    pub fn new(reader: R) -> Self {
        IoRead {
            reader,
            buf: vec![],
        }
    }

    /// Returns a shared reference to the inner reader.
    pub fn get_ref(&self) -> &R {
        &self.reader
    }

    /// Returns a mutable reference to the inner reader.
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Consumes the `IoRead`, returning the inner reader.
    pub fn into_inner(self) -> R {
        self.reader
    }

    fn fill_buf(&mut self, n: usize) -> Result<bool, Error> {
        self.buf.clear();
        // defend against malicious input pretending to be huge by limiting growth
        self.buf.reserve(usize::min(n, 16 * 1024));

        let mut remaining = n;
        while remaining > 0 {
            let buf = self.reader.fill_buf().map_err(Error::io)?;
            if buf.is_empty() {
                return Ok(false);
            }

            let len = usize::min(remaining, buf.len());
            self.buf.extend_from_slice(&buf[..len]);
            self.reader.consume(len);
            remaining -= len;
        }

        Ok(true)
    }
}

impl<R> private::Sealed for IoRead<R> {}

impl<'de, R> Read<'de> for IoRead<R>
where
    R: BufRead,
{
    fn next(&mut self) -> Result<Option<u8>, Error> {
        let r = self.peek();
        if let Ok(Some(_)) = r {
            self.consume();
        }
        r
    }

    fn peek(&mut self) -> Result<Option<u8>, Error> {
        let buf = self.reader.fill_buf().map_err(Error::io)?;
        if buf.is_empty() {
            Ok(None)
        } else {
            Ok(Some(buf[0]))
        }
    }

    fn consume(&mut self) {
        self.reader.consume(1);
    }

    // FIXME ideally we'd be able to avoid a copy by directly referencing the reader's buffer when it has enough data
    // but that would require some kind of deferred consume handling.
    fn read<'a>(&'a mut self, n: usize) -> Result<Option<Buf<'a, 'de>>, Error> {
        if self.fill_buf(n)? {
            Ok(Some(Buf::Short(&self.buf)))
        } else {
            Ok(None)
        }
    }

    fn read_mut<'a>(&'a mut self, n: usize) -> Result<Option<MutBuf<'a, 'de>>, Error> {
        if self.fill_buf(n)? {
            Ok(Some(MutBuf::Short(&mut self.buf)))
        } else {
            Ok(None)
        }
    }

    fn read_until<'a>(&'a mut self, end: u8) -> Result<Option<Buf<'a, 'de>>, Error> {
        self.buf.clear();

        loop {
            let buf = self.reader.fill_buf().map_err(Error::io)?;
            if buf.is_empty() {
                return Ok(None);
            }

            match memchr(end, buf) {
                Some(end) => {
                    self.buf.extend_from_slice(&buf[..end]);
                    self.reader.consume(end + 1);
                    return Ok(Some(Buf::Short(&self.buf)));
                }
                None => {
                    self.buf.extend(buf);
                    let len = buf.len();
                    self.reader.consume(len);
                }
            }
        }
    }
}
