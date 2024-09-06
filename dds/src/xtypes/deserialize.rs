use super::{
    bytes::Bytes,
    deserializer::{DeserializeArray, XTypesDeserializer},
    error::XTypesError,
};

pub use dust_dds_derive::XTypesDeserialize;

/// A trait representing a structure that can be deserialized from a CDR format.
pub trait XTypesDeserialize<'de>: Sized {
    /// Method to deserialize this value using the given deserializer.
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError>;
}

impl<'de> XTypesDeserialize<'de> for bool {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        deserializer.deserialize_boolean()
    }
}

impl<'de> XTypesDeserialize<'de> for i8 {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        deserializer.deserialize_int8()
    }
}

impl<'de> XTypesDeserialize<'de> for i16 {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        deserializer.deserialize_int16()
    }
}

impl<'de> XTypesDeserialize<'de> for i32 {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        deserializer.deserialize_int32()
    }
}

impl<'de> XTypesDeserialize<'de> for i64 {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        deserializer.deserialize_int64()
    }
}

impl<'de> XTypesDeserialize<'de> for u8 {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        deserializer.deserialize_uint8()
    }
}

impl<'de> XTypesDeserialize<'de> for u16 {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        deserializer.deserialize_uint16()
    }
}

impl<'de> XTypesDeserialize<'de> for u32 {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        deserializer.deserialize_uint32()
    }
}

impl<'de> XTypesDeserialize<'de> for u64 {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        deserializer.deserialize_uint64()
    }
}

impl<'de> XTypesDeserialize<'de> for f32 {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        deserializer.deserialize_float32()
    }
}

impl<'de> XTypesDeserialize<'de> for f64 {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        deserializer.deserialize_float64()
    }
}

impl<'de> XTypesDeserialize<'de> for char {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        deserializer.deserialize_char8()
    }
}
impl<'de> XTypesDeserialize<'de> for &'de str {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        deserializer.deserialize_string()
    }
}

impl<'de> XTypesDeserialize<'de> for &'de [u8] {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        deserializer.deserialize_byte_sequence()
    }
}
impl<'de, const N: usize> XTypesDeserialize<'de> for &'de [u8; N] {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        deserializer.deserialize_byte_array()
    }
}
impl<'de, T: XTypesDeserialize<'de>, const N: usize> XTypesDeserialize<'de> for [T; N] {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        let mut seq = deserializer.deserialize_array()?;
        let v: [Result<T, _>; N] = core::array::from_fn(|_| seq.deserialize_element());
        if let Some(_e) = v.iter().find(|f| f.is_err()) {
            return Err(XTypesError::InvalidData);
        }
        let mut iter = v.into_iter();
        Ok(core::array::from_fn(|_| {
            iter.next()
                .expect("same amount of elements guaranteed")
                .expect("error handled above")
        }))
    }
}

impl<'de: 'a, 'a> XTypesDeserialize<'de> for Bytes<'a> {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        Ok(Self(deserializer.deserialize_byte_sequence()?))
    }
}

#[cfg(feature = "std")]
impl<'de> XTypesDeserialize<'de> for super::bytes::ByteBuf {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        Ok(Self(deserializer.deserialize_byte_sequence()?.to_owned()))
    }
}

#[cfg(feature = "std")]
impl<'de, T> XTypesDeserialize<'de> for Vec<T>
where
    T: XTypesDeserialize<'de>,
{
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        let mut vec = Vec::new();
        let mut seq = deserializer.deserialize_sequence()?;
        for _ in 0..super::deserializer::DeserializeSequence::len(&seq) {
            vec.push(super::deserializer::DeserializeSequence::deserialize_element(&mut seq)?);
        }
        Ok(vec)
    }
}

#[cfg(feature = "std")]
impl<'de> XTypesDeserialize<'de> for String {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XTypesError> {
        Ok(deserializer.deserialize_string()?.to_string())
    }
}
