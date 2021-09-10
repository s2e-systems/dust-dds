//! A serialization/deserialization implementation for Common Data Representation.
//!
//! # Examples
//!
//! ```rust
//! use cdr::{CdrBe, Infinite};
//! use serde_derive::{Deserialize, Serialize};
//!
//! #[derive(Deserialize, Serialize, PartialEq)]
//! struct Point {
//!     x: f64,
//!     y: f64,
//! }
//!
//! #[derive(Deserialize, Serialize, PartialEq)]
//! struct Polygon(Vec<Point>);
//!
//!     let triangle = Polygon(vec![Point { x: -1.0, y: -1.0 },
//!                                 Point { x: 1.0, y: -1.0 },
//!                                 Point { x: 0.0, y: 0.73 }]);
//!
//!     let encoded = cdr::serialize::<_, _, CdrBe>(&triangle, Infinite).unwrap();
//!     let decoded = cdr::deserialize::<Polygon>(&encoded[..]).unwrap();
//!
//!     assert!(triangle == decoded);

//! ```

pub use byteorder::{BigEndian, LittleEndian};

pub mod de;
#[doc(inline)]
pub use crate::de::Deserializer;

mod encapsulation;
pub use crate::encapsulation::{CdrBe, CdrLe, Encapsulation, PlCdrBe, PlCdrLe};

mod error;
pub use crate::error::{Error, Result};

pub mod ser;
#[doc(inline)]
pub use crate::ser::Serializer;

pub mod size;
#[doc(inline)]
pub use crate::size::{Bounded, Infinite, SizeLimit};

use std::io::{Read, Write};

/// Returns the size that an object would be if serialized with a encapsulation.
pub fn calc_serialized_size<T: ?Sized>(value: &T) -> u64
where
    T: serde::Serialize,
{
    size::calc_serialized_data_size(value) + encapsulation::ENCAPSULATION_HEADER_SIZE
}

/// Given a maximum size limit, check how large an object would be if it were
/// to be serialized with a encapsulation.
pub fn calc_serialized_size_bounded<T: ?Sized>(value: &T, max: u64) -> Result<u64>
where
    T: serde::Serialize,
{
    use crate::encapsulation::ENCAPSULATION_HEADER_SIZE;

    if max < ENCAPSULATION_HEADER_SIZE {
        Err(Error::SizeLimit)
    } else {
        size::calc_serialized_data_size_bounded(value, max)
            .map(|size| size + ENCAPSULATION_HEADER_SIZE)
    }
}

/// Serializes a serializable object into a `Vec` of bytes with the encapsulation.
pub fn serialize<T: ?Sized, S, C>(value: &T, size_limit: S) -> Result<Vec<u8>>
where
    T: serde::Serialize,
    S: SizeLimit,
    C: Encapsulation,
{
    let mut writer = match size_limit.limit() {
        Some(limit) => {
            let actual_size = calc_serialized_size_bounded(value, limit)?;
            Vec::with_capacity(actual_size as usize)
        }
        None => {
            let size = calc_serialized_size(value) as usize;
            Vec::with_capacity(size)
        }
    };

    serialize_into::<_, _, _, C>(&mut writer, value, Infinite)?;
    Ok(writer)
}

/// Serializes an object directly into a `Write` with the encapsulation.
pub fn serialize_into<W, T: ?Sized, S, C>(writer: W, value: &T, size_limit: S) -> Result<()>
where
    W: Write,
    T: serde::ser::Serialize,
    S: SizeLimit,
    C: Encapsulation,
{
    if let Some(limit) = size_limit.limit() {
        calc_serialized_size_bounded(value, limit)?;
    }

    let mut serializer = Serializer::<_, C::E>::new(writer);

    serde::Serialize::serialize(&C::id(), &mut serializer)?;
    serde::Serialize::serialize(&C::option(), &mut serializer)?;
    serializer.reset_pos();
    serde::Serialize::serialize(value, &mut serializer)
}

/// Deserializes a slice of bytes into an object.
pub fn deserialize<'de, T>(bytes: &[u8]) -> Result<T>
where
    T: serde::Deserialize<'de>,
{
    deserialize_from::<_, _, _>(bytes, Infinite)
}

/// Deserializes an object directly from a `Read`.
pub fn deserialize_from<'de, R, T, S>(reader: R, size_limit: S) -> Result<T>
where
    R: Read,
    T: serde::Deserialize<'de>,
    S: SizeLimit,
{
    use crate::encapsulation::ENCAPSULATION_HEADER_SIZE;

    let mut deserializer = Deserializer::<_, S, BigEndian>::new(reader, size_limit);

    let v: [u8; ENCAPSULATION_HEADER_SIZE as usize] =
        serde::Deserialize::deserialize(&mut deserializer)?;
    deserializer.reset_pos();
    match v[1] {
        0 | 2 => serde::Deserialize::deserialize(&mut deserializer),
        1 | 3 => serde::Deserialize::deserialize(
            &mut Into::<Deserializer<_, _, LittleEndian>>::into(deserializer),
        ),
        _ => Err(Error::InvalidEncapsulation),
    }
}
