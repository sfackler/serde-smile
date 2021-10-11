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
