use crate::Error;
use memchr::memchr;
use std::io::BufRead;
use std::mem;
use std::ops::{Deref, DerefMut};

pub(crate) mod private {
    pub trait Sealed {}
}

pub trait Read<'de>: private::Sealed {
    fn next(&mut self) -> Result<Option<u8>, Error>;

    fn peek(&mut self) -> Result<Option<u8>, Error>;

    fn consume(&mut self);

    fn read<'a>(&'a mut self, n: usize) -> Result<Option<Buf<'a, 'de>>, Error>;

    fn read_mut<'a>(&'a mut self, n: usize) -> Result<Option<MutBuf<'a, 'de>>, Error>;

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
            Buf::Short(buf) => *buf,
            Buf::Long(buf) => *buf,
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
            MutBuf::Short(buf) => *buf,
            MutBuf::Long(buf) => *buf,
        }
    }
}

impl DerefMut for MutBuf<'_, '_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MutBuf::Short(buf) => *buf,
            MutBuf::Long(buf) => *buf,
        }
    }
}

pub struct SliceRead<'a> {
    slice: &'a [u8],
    index: usize,
    buf: Vec<u8>,
}

impl<'a> SliceRead<'a> {
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
        if n < s.len() {
            self.index += n;
            Ok(Some(Buf::Long(&s[..n])))
        } else {
            Ok(None)
        }
    }

    fn read_mut<'a>(&'a mut self, n: usize) -> Result<Option<MutBuf<'a, 'de>>, Error> {
        let s = &self.slice[self.index..];
        if n < s.len() {
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

pub struct MutSliceRead<'a> {
    slice: &'a mut [u8],
}

impl<'a> MutSliceRead<'a> {
    pub fn new(slice: &'a mut [u8]) -> Self {
        MutSliceRead { slice }
    }
}

impl private::Sealed for MutSliceRead<'_> {}

impl<'de> Read<'de> for MutSliceRead<'de> {
    fn next(&mut self) -> Result<Option<u8>, Error> {
        if !self.slice.is_empty() {
            let slice = mem::replace(&mut self.slice, &mut []);
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
        let slice = mem::replace(&mut self.slice, &mut []);
        self.slice = &mut slice[1..];
    }

    fn read<'a>(&'a mut self, n: usize) -> Result<Option<Buf<'a, 'de>>, Error> {
        if n < self.slice.len() {
            let (a, b) = mem::replace(&mut self.slice, &mut []).split_at_mut(n);
            self.slice = b;
            Ok(Some(Buf::Long(a)))
        } else {
            Ok(None)
        }
    }

    fn read_mut<'a>(&'a mut self, n: usize) -> Result<Option<MutBuf<'a, 'de>>, Error> {
        if n < self.slice.len() {
            let (a, b) = mem::replace(&mut self.slice, &mut []).split_at_mut(n);
            self.slice = b;
            Ok(Some(MutBuf::Long(a)))
        } else {
            Ok(None)
        }
    }

    fn read_until<'a>(&'a mut self, end: u8) -> Result<Option<Buf<'a, 'de>>, Error> {
        match memchr(end, self.slice) {
            Some(end) => {
                let (a, b) = mem::replace(&mut self.slice, &mut []).split_at_mut(end);
                self.slice = &mut b[1..];
                Ok(Some(Buf::Long(a)))
            }
            None => Ok(None),
        }
    }
}

pub struct IoRead<R> {
    reader: R,
    buf: Vec<u8>,
}

impl<R> IoRead<R>
where
    R: BufRead,
{
    pub fn new(reader: R) -> Self {
        IoRead {
            reader,
            buf: vec![],
        }
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
            self.buf.extend_from_slice(buf);
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
