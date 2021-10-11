use serde::{de, ser};
use std::{error, fmt, io};

#[derive(Debug)]
enum ErrorKind {
    Io(io::Error),
    Custom(String),
    KeyMustBeAString,
    EofWhileParsingValue,
    ReservedToken,
    InvalidStringReference,
    UnterminatedVint,
    BufferLengthOverflow,
    UnsupportedBigInteger,
    UnsupportedBigDecimal,
    InvalidUtf8,
    RecursionLimitExceeded,
    TrailingData,
    EofWhileParsingArray,
    UnexpectedToken,
    EofWhileParsingMap,
    InvalidHeader,
    UnsupportedVersion,
    EofWhileParsingHeader,
}

/// An error encountered when serializing or deserializing to or from Smile.
#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self.0 {
            ErrorKind::Io(_) => f.write_str("IO error"),
            ErrorKind::Custom(e) => f.write_str(e),
            ErrorKind::KeyMustBeAString => f.write_str("key must be a string"),
            ErrorKind::EofWhileParsingValue => f.write_str("EOF while parsing a value"),
            ErrorKind::ReservedToken => f.write_str("reserved token"),
            ErrorKind::InvalidStringReference => f.write_str("invalid string reference"),
            ErrorKind::UnterminatedVint => f.write_str("unterminated vint"),
            ErrorKind::BufferLengthOverflow => f.write_str("buffer length overflow"),
            ErrorKind::UnsupportedBigInteger => f.write_str("unsupported BigInteger"),
            ErrorKind::UnsupportedBigDecimal => f.write_str("unsupported BigDecimal"),
            ErrorKind::InvalidUtf8 => f.write_str("invalid UTF-8"),
            ErrorKind::RecursionLimitExceeded => f.write_str("recursion limit exceeded"),
            ErrorKind::TrailingData => f.write_str("trailing data"),
            ErrorKind::EofWhileParsingArray => f.write_str("EOF while parsing array"),
            ErrorKind::UnexpectedToken => f.write_str("unexpected token"),
            ErrorKind::EofWhileParsingMap => f.write_str("EOF while parsing map"),
            ErrorKind::InvalidHeader => f.write_str("invalid header"),
            ErrorKind::UnsupportedVersion => f.write_str("unsupported version"),
            ErrorKind::EofWhileParsingHeader => f.write_str("EOF while parsing header"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &*self.0 {
            ErrorKind::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error(Box::new(ErrorKind::Custom(msg.to_string())))
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error(Box::new(ErrorKind::Custom(msg.to_string())))
    }
}

impl Error {
    pub(crate) fn io(e: io::Error) -> Self {
        Error(Box::new(ErrorKind::Io(e)))
    }

    pub(crate) fn key_must_be_a_string() -> Self {
        Error(Box::new(ErrorKind::KeyMustBeAString))
    }

    pub(crate) fn eof_while_parsing_value() -> Self {
        Error(Box::new(ErrorKind::EofWhileParsingValue))
    }

    pub(crate) fn reserved_token() -> Self {
        Error(Box::new(ErrorKind::ReservedToken))
    }

    pub(crate) fn invalid_string_reference() -> Self {
        Error(Box::new(ErrorKind::InvalidStringReference))
    }

    pub(crate) fn unterminated_vint() -> Self {
        Error(Box::new(ErrorKind::UnterminatedVint))
    }

    pub(crate) fn buffer_length_overflow() -> Self {
        Error(Box::new(ErrorKind::BufferLengthOverflow))
    }

    pub(crate) fn unsupported_big_integer() -> Self {
        Error(Box::new(ErrorKind::UnsupportedBigInteger))
    }

    pub(crate) fn unsupported_big_decimal() -> Self {
        Error(Box::new(ErrorKind::UnsupportedBigDecimal))
    }

    pub(crate) fn invalid_utf8() -> Self {
        Error(Box::new(ErrorKind::InvalidUtf8))
    }

    pub(crate) fn recursion_limit_exceeded() -> Self {
        Error(Box::new(ErrorKind::RecursionLimitExceeded))
    }

    pub(crate) fn trailing_data() -> Self {
        Error(Box::new(ErrorKind::TrailingData))
    }

    pub(crate) fn eof_while_parsing_array() -> Self {
        Error(Box::new(ErrorKind::EofWhileParsingArray))
    }

    pub(crate) fn unexpected_token() -> Self {
        Error(Box::new(ErrorKind::UnexpectedToken))
    }

    pub(crate) fn eof_while_parsing_map() -> Self {
        Error(Box::new(ErrorKind::EofWhileParsingMap))
    }

    pub(crate) fn invalid_header() -> Self {
        Error(Box::new(ErrorKind::InvalidHeader))
    }

    pub(crate) fn unsupported_version() -> Self {
        Error(Box::new(ErrorKind::UnsupportedVersion))
    }

    pub(crate) fn eof_while_parsing_header() -> Self {
        Error(Box::new(ErrorKind::EofWhileParsingHeader))
    }
}
