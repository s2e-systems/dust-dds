pub use super::{error::XcdrError, serializer::SerializeCollection, serializer::XTypesSerializer};
pub use dust_dds_derive::XTypesSerialize;

/// A trait representing a structure that can be serialized into a CDR format.
pub trait XTypesSerialize {
    /// Method to serialize this value using the given serializer.
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError>;
}

impl XTypesSerialize for bool {
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        serializer.serialize_boolean(*self)
    }
}

impl XTypesSerialize for i8 {
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        serializer.serialize_int8(*self)
    }
}

impl XTypesSerialize for i16 {
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        serializer.serialize_int16(*self)
    }
}

impl XTypesSerialize for i32 {
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        serializer.serialize_int32(*self)
    }
}

impl XTypesSerialize for i64 {
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        serializer.serialize_int64(*self)
    }
}

impl XTypesSerialize for u8 {
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        serializer.serialize_uint8(*self)
    }
}

impl XTypesSerialize for u16 {
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        serializer.serialize_uint16(*self)
    }
}

impl XTypesSerialize for u32 {
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        serializer.serialize_uint32(*self)
    }
}

impl XTypesSerialize for u64 {
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        serializer.serialize_uint64(*self)
    }
}

impl XTypesSerialize for f32 {
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        serializer.serialize_float32(*self)
    }
}

impl XTypesSerialize for f64 {
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        serializer.serialize_float64(*self)
    }
}

impl XTypesSerialize for char {
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        serializer.serialize_char8(*self)
    }
}

impl XTypesSerialize for str {
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        serializer.serialize_string(self)
    }
}

impl<T> XTypesSerialize for &'_ T
where
    T: XTypesSerialize + ?Sized,
{
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        T::serialize(*self, serializer)
    }
}

impl<T> XTypesSerialize for &'_ mut T
where
    T: XTypesSerialize + ?Sized,
{
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        T::serialize(*self, serializer)
    }
}

impl XTypesSerialize for () {
    fn serialize(&self, _serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        Ok(())
    }
}

impl<T: XTypesSerialize, const N: usize> XTypesSerialize for [T; N] {
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        let mut s = serializer.serialize_array()?;
        for e in self {
            s.serialize_element(e)?;
        }
        Ok(())
    }
}

// impl XTypesSerialize for &[u8] {
//     fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
//         serializer.serialize_byte_sequence(self)
//     }
// }

impl<T: XTypesSerialize> XTypesSerialize for &[T] {
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        let mut s = serializer.serialize_sequence(self.len())?;
        for e in self.iter() {
            s.serialize_element(e)?;
        }
        Ok(())
    }
}

impl<T> XTypesSerialize for Vec<T>
where
    T: XTypesSerialize,
{
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        let mut s = serializer.serialize_sequence(self.len())?;
        for e in self.iter() {
            s.serialize_element(e)?;
        }
        Ok(())
        // serializer.serialize_byte_sequence(self)
    }
}

impl XTypesSerialize for String {
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        serializer.serialize_string(self.as_str())
    }
}