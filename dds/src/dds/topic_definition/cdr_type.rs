use crate::infrastructure::error::DdsResult;

pub use dust_dds_derive::CdrDeserialize;

pub trait CdrDeserialize<'de>: Sized {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self>;
}

pub trait CdrDeserializer<'de> {
    fn deserialize_bool(&mut self) -> DdsResult<bool>;
    fn deserialize_i8(&mut self) -> DdsResult<i8>;
    fn deserialize_i16(&mut self) -> DdsResult<i16>;
    fn deserialize_i32(&mut self) -> DdsResult<i32>;
    fn deserialize_i64(&mut self) -> DdsResult<i64>;
    fn deserialize_u8(&mut self) -> DdsResult<u8>;
    fn deserialize_u16(&mut self) -> DdsResult<u16>;
    fn deserialize_u32(&mut self) -> DdsResult<u32>;
    fn deserialize_u64(&mut self) -> DdsResult<u64>;
    fn deserialize_f32(&mut self) -> DdsResult<f32>;
    fn deserialize_f64(&mut self) -> DdsResult<f64>;
    fn deserialize_char(&mut self) -> DdsResult<char>;
    fn deserialize_string(&mut self) -> DdsResult<String>;
    fn deserialize_seq<T>(&mut self) -> DdsResult<Vec<T>>
    where
        T: CdrDeserialize<'de>;
    fn deserialize_array<const N: usize, T>(&mut self) -> DdsResult<[T; N]>
    where
        T: CdrDeserialize<'de>;
    fn deserialize_bytes(&mut self) -> DdsResult<&'de [u8]>;
    fn deserialize_byte_array<const N: usize>(&mut self) -> DdsResult<&'de [u8; N]>;
    fn deserialize_unit(&mut self) -> DdsResult<()>;
}

/// Enumeration of the different representations defined by the RTPS standard and supported by DustDDS.
pub enum CdrRepresentationKind {
    CdrLe,
    CdrBe,
    PlCdrBe,
    PlCdrLe,
}

/// This trait defines the representation to be used by the type when serializing and deserializing.
///
/// When used in combination with [`serde::Serialize`] and [`serde::Deserialize`] a blanket implementation
/// for the [`DdsSerializeData`] and [`DdsDeserialize`] traits is provided that uses the Cdr serializer and
/// is conformant with the CDR format as specified in the RTPS standard.
pub trait CdrRepresentation {
    const REPRESENTATION: CdrRepresentationKind;
}

//// Deserialize

impl<'de> CdrDeserialize<'de> for bool {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        deserializer.deserialize_bool()
    }
}

impl<'de> CdrDeserialize<'de> for i8 {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        deserializer.deserialize_i8()
    }
}

impl<'de> CdrDeserialize<'de> for i16 {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        deserializer.deserialize_i16()
    }
}

impl<'de> CdrDeserialize<'de> for i32 {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        deserializer.deserialize_i32()
    }
}

impl<'de> CdrDeserialize<'de> for i64 {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        deserializer.deserialize_i64()
    }
}

impl<'de> CdrDeserialize<'de> for u8 {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        deserializer.deserialize_u8()
    }
}

impl<'de> CdrDeserialize<'de> for u16 {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        deserializer.deserialize_u16()
    }
}

impl<'de> CdrDeserialize<'de> for u32 {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        deserializer.deserialize_u32()
    }
}

impl<'de> CdrDeserialize<'de> for u64 {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        deserializer.deserialize_u64()
    }
}

impl<'de> CdrDeserialize<'de> for f32 {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        deserializer.deserialize_f32()
    }
}

impl<'de> CdrDeserialize<'de> for f64 {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        deserializer.deserialize_f64()
    }
}

impl<'de> CdrDeserialize<'de> for char {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        deserializer.deserialize_char()
    }
}

impl<'de> CdrDeserialize<'de> for String {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        deserializer.deserialize_string()
    }
}

impl<'de, const N: usize, T> CdrDeserialize<'de> for [T; N]
where
    T: CdrDeserialize<'de>,
{
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        deserializer.deserialize_array()
    }
}

impl<'de, T> CdrDeserialize<'de> for Vec<T>
where
    T: CdrDeserialize<'de>,
{
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        deserializer.deserialize_seq()
    }
}

impl<'de> CdrDeserialize<'de> for &'de [u8] {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        deserializer.deserialize_bytes()
    }
}

impl<'de, const N: usize> CdrDeserialize<'de> for &'de [u8; N] {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        deserializer.deserialize_byte_array()
    }
}

impl<'de> CdrDeserialize<'de> for () {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> DdsResult<Self> {
        deserializer.deserialize_unit()
    }
}
