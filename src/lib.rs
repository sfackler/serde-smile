//! A Smile implementation for Serde.
//!
//! [Smile] is a binary data format created by the developers of the Jackson serialization library for Java. It is
//! designed to be a binary equivalent of JSON.
//!
//! [Smile]: https://github.com/FasterXML/smile-format-specification
#![warn(missing_docs)]

#[doc(inline)]
pub use de::{from_mut_slice, from_reader, from_slice, Deserializer};
#[doc(inline)]
pub use error::Error;
#[doc(inline)]
pub use ser::{to_vec, to_writer, Serializer};

pub mod de;
mod error;
pub mod ser;
#[cfg(test)]
mod test;
