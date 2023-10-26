use crate::infrastructure::error::DdsResult;

pub use dust_dds_derive::CdrSerialize;

pub trait CdrSerialize {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()>;
}

pub trait CdrSerializer {
    fn serialize_bool(&mut self, v: bool) -> DdsResult<()>;
    fn serialize_i8(&mut self, v: i8) -> DdsResult<()>;
    fn serialize_i16(&mut self, v: i16) -> DdsResult<()>;
    fn serialize_i32(&mut self, v: i32) -> DdsResult<()>;
    fn serialize_i64(&mut self, v: i64) -> DdsResult<()>;
    fn serialize_u8(&mut self, v: u8) -> DdsResult<()>;
    fn serialize_u16(&mut self, v: u16) -> DdsResult<()>;
    fn serialize_u32(&mut self, v: u32) -> DdsResult<()>;
    fn serialize_u64(&mut self, v: u64) -> DdsResult<()>;
    fn serialize_f32(&mut self, v: f32) -> DdsResult<()>;
    fn serialize_f64(&mut self, v: f64) -> DdsResult<()>;
    fn serialize_char(&mut self, v: char) -> DdsResult<()>;
    fn serialize_str(&mut self, v: &str) -> DdsResult<()>;
    fn serialize_seq(&mut self, v: &[impl CdrSerialize]) -> DdsResult<()>;
    fn serialize_array<const N: usize>(&mut self, v: &[impl CdrSerialize; N]) -> DdsResult<()>;
    fn serialize_unit(&mut self) -> DdsResult<()>;
}

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

impl CdrSerialize for bool {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        serializer.serialize_bool(*self)
    }
}

impl CdrSerialize for i8 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        serializer.serialize_i8(*self)
    }
}

impl CdrSerialize for i16 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        serializer.serialize_i16(*self)
    }
}

impl CdrSerialize for i32 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        serializer.serialize_i32(*self)
    }
}

impl CdrSerialize for i64 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        serializer.serialize_i64(*self)
    }
}

impl CdrSerialize for u8 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        serializer.serialize_u8(*self)
    }
}

impl CdrSerialize for u16 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        serializer.serialize_u16(*self)
    }
}

impl CdrSerialize for u32 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        serializer.serialize_u32(*self)
    }
}

impl CdrSerialize for u64 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        serializer.serialize_u64(*self)
    }
}

impl CdrSerialize for f32 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        serializer.serialize_f32(*self)
    }
}

impl CdrSerialize for f64 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        serializer.serialize_f64(*self)
    }
}

impl CdrSerialize for char {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        serializer.serialize_char(*self)
    }
}

impl CdrSerialize for str {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        serializer.serialize_str(self)
    }
}

impl CdrSerialize for String {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        serializer.serialize_str(self)
    }
}

impl<T> CdrSerialize for [T]
where
    T: CdrSerialize,
{
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        serializer.serialize_seq(self)
    }
}

impl<const N: usize, T> CdrSerialize for [T; N]
where
    T: CdrSerialize,
{
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        serializer.serialize_array(self)
    }
}

impl<T> CdrSerialize for Vec<T>
where
    T: CdrSerialize,
{
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        serializer.serialize_seq(self)
    }
}

impl<T> CdrSerialize for &'_ T
where
    T: CdrSerialize + ?Sized,
{
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        T::serialize(*self, serializer)
    }
}

impl<T> CdrSerialize for &'_ mut T
where
    T: CdrSerialize + ?Sized,
{
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        T::serialize(*self, serializer)
    }
}

impl<T> CdrSerialize for Box<T>
where
    T: CdrSerialize,
{
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        self.as_ref().serialize(serializer)
    }
}

impl CdrSerialize for () {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> DdsResult<()> {
        serializer.serialize_unit()
    }
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
