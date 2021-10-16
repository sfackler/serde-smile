//! A Smile implementation for Serde.
//!
//! [Smile] is a binary data format created by the developers of the Jackson serialization library for Java. It is
//! designed to be a binary equivalent of JSON.
//!
//! # Serialization Options
//!
//! Smile defines several optional features that can be enabled or disabled during serialization:
//!
//! * [`Builder::raw_binary`]: If enabled, binary data will be encoded directly as "raw" bytes, rather than using
//!     Smile's 7-bit "safe" encoding. The raw format is 14% smaller and faster to serialize and deserialize, but usage
//!     means that encoded values may contain Smile control characters such as the end-of-stream token `0xff`. Disabled
//!     by default.
//! * [`Builder::shared_strings`]: If enabled, string values 64 bytes and smaller will be deduplicated in the encoded
//!     format. This increases the memory overhead of serialization and deserialization, but can significantly shrink
//!     the size of the encoded value when strings are repeated. Disabled by default.
//! * [`Builder::shared_properties`]: If enabled, map keys 64 bytes and smaller will be deduplicated in the encoded
//!     format. This increases the memory overhead of serialization and deserialization, but can significantly shrink
//!     the size of the encoded value when keys are repeated (particularly struct field names). Enabled by default.
//! * [`Serializer::end`]: A sequence of Smile values can optionally be terminated by the end-of-stream token `0xff`.
//!     Calling this method will write the token into the output stream.
//!
//! # Special Types
//!
//! Smile supports two kinds of values that Serde does not natively handle: arbitrary precision integer and decimals.
//! This crate defines special types [`BigInteger`] and [`BigDecimal`] which will serialize to and deserialize from
//! their respective Smile types. However, they should only be used with the serializers and deserializers defined
//! within this crate as they will produce nonsensical values when used with other Serde libraries.
//!
//! # Encoding Notes
//!
//! Rust integer values that cannot be stored in an `i64` will be serialized as Smile `BigInteger` values. In the other
//! direction, `BigInteger` values will be deserialized to Rust integer types if the value is small enough.
//!
//! # Examples
//!
//! Serialize a Rust object into a Smile value:
//! ```rust
//! use serde::Serialize;
//! use serde_smile::Error;
//!
//! #[derive(Serialize)]
//! struct Address {
//!     number: u32,
//!     street: String,
//! }
//!
//! fn main() -> Result<(), Error> {
//!     let address = Address {
//!         number: 1600,
//!         street: "Pennsylvania Avenue".to_string(),
//!     };
//!
//!     let value = serde_smile::to_vec(&address)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! Deserialize a Smile value into a Rust object:
//! ```rust
//! use serde::Deserialize;
//! use serde_smile::Error;
//!
//! #[derive(Deserialize)]
//! struct Address {
//!     number: u32,
//!     street: String,
//! }
//!
//! fn main() -> Result<(), Error> {
//!     let smile = b":)\n\x01\xfa\x85number\x24\x32\x80\x85street\x52Pennsylvania Avenue\xfb";
//!
//!     let address: Address = serde_smile::from_slice(smile)?;
//!
//!     println!("{} {}", address.number, address.street);
//!
//!     Ok(())
//! }
//! ```
//!
//! [Smile]: https://github.com/FasterXML/smile-format-specification
//! [`Builder::raw_binary`]: ser::Builder::raw_binary
//! [`Builder::shared_strings`]: ser::Builder::shared_strings
//! [`Builder::shared_properties`]: ser::Builder::shared_properties
//! [`BigInteger`]: value::BigInteger
//! [`BigDecimal`]: value::BigDecimal
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
pub mod value;
