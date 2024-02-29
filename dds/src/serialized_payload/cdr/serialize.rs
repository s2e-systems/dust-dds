pub use dust_dds_derive::CdrSerialize;

use super::serializer::CdrSerializer;

/// A trait representing a structure that can be serialized into a CDR format.
pub trait CdrSerialize {
    /// Method to serialize this value using the given serializer.
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error>;
}

impl CdrSerialize for bool {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        serializer.serialize_bool(*self)
    }
}

impl CdrSerialize for i8 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        serializer.serialize_i8(*self)
    }
}

impl CdrSerialize for i16 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        serializer.serialize_i16(*self)
    }
}

impl CdrSerialize for i32 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        serializer.serialize_i32(*self)
    }
}

impl CdrSerialize for i64 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        serializer.serialize_i64(*self)
    }
}

impl CdrSerialize for u8 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        serializer.serialize_u8(*self)
    }
}

impl CdrSerialize for u16 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        serializer.serialize_u16(*self)
    }
}

impl CdrSerialize for u32 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        serializer.serialize_u32(*self)
    }
}

impl CdrSerialize for u64 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        serializer.serialize_u64(*self)
    }
}

impl CdrSerialize for f32 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        serializer.serialize_f32(*self)
    }
}

impl CdrSerialize for f64 {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        serializer.serialize_f64(*self)
    }
}

impl CdrSerialize for char {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        serializer.serialize_char(*self)
    }
}

impl CdrSerialize for str {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        serializer.serialize_str(self)
    }
}

impl CdrSerialize for String {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        serializer.serialize_str(self)
    }
}

impl<T> CdrSerialize for [T]
where
    T: CdrSerialize,
{
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        serializer.serialize_seq(self)
    }
}

impl<const N: usize, T> CdrSerialize for [T; N]
where
    T: CdrSerialize,
{
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        serializer.serialize_array(self)
    }
}

impl<T> CdrSerialize for Vec<T>
where
    T: CdrSerialize,
{
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        serializer.serialize_seq(self)
    }
}

impl<T> CdrSerialize for &'_ T
where
    T: CdrSerialize + ?Sized,
{
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        T::serialize(*self, serializer)
    }
}

impl<T> CdrSerialize for &'_ mut T
where
    T: CdrSerialize + ?Sized,
{
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        T::serialize(*self, serializer)
    }
}

impl<T> CdrSerialize for Box<T>
where
    T: CdrSerialize,
{
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        self.as_ref().serialize(serializer)
    }
}

impl CdrSerialize for () {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        serializer.serialize_unit()
    }
}
