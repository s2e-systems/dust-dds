use super::deserialize::CdrDeserialize;

/// A trait representing an object with the capability of deserializing a value from a CDR format.
pub trait CdrDeserializer<'de> {
    /// Deserialize a [`bool`] value.
    fn deserialize_bool(&mut self) -> Result<bool, std::io::Error>;

    /// Deserialize an [`i8`] value.
    fn deserialize_i8(&mut self) -> Result<i8, std::io::Error>;

    /// Deserialize an [`i16`] value.
    fn deserialize_i16(&mut self) -> Result<i16, std::io::Error>;

    /// Deserialize an [`i32`] value.
    fn deserialize_i32(&mut self) -> Result<i32, std::io::Error>;

    /// Deserialize an [`i64`] value.
    fn deserialize_i64(&mut self) -> Result<i64, std::io::Error>;

    /// Deserialize a [`u8`] value.
    fn deserialize_u8(&mut self) -> Result<u8, std::io::Error>;

    /// Deserialize a [`u16`] value.
    fn deserialize_u16(&mut self) -> Result<u16, std::io::Error>;

    /// Deserialize a [`u32`] value.
    fn deserialize_u32(&mut self) -> Result<u32, std::io::Error>;

    /// Deserialize a [`u64`] value.
    fn deserialize_u64(&mut self) -> Result<u64, std::io::Error>;

    /// Deserialize an [`f32`] value.
    fn deserialize_f32(&mut self) -> Result<f32, std::io::Error>;

    /// Deserialize an [`f64`] value.
    fn deserialize_f64(&mut self) -> Result<f64, std::io::Error>;

    /// Deserialize a [`char`] value.
    fn deserialize_char(&mut self) -> Result<char, std::io::Error>;

    /// Deserialize a [`String`] value.
    fn deserialize_string(&mut self) -> Result<String, std::io::Error>;

    /// Deserialize a variable sized sequence of values.
    fn deserialize_seq<T>(&mut self) -> Result<Vec<T>, std::io::Error>
    where
        T: CdrDeserialize<'de>;

    /// Deserialize an array of values.
    fn deserialize_array<const N: usize, T>(&mut self) -> Result<[T; N], std::io::Error>
    where
        T: CdrDeserialize<'de>;

    /// Deserialize a variable sized sequence of bytes by borrowing from the original byte array.
    fn deserialize_bytes(&mut self) -> Result<&'de [u8], std::io::Error>;

    /// Deserialize an array of bytes by borrowing from the original byte array.
    fn deserialize_byte_array<const N: usize>(&mut self) -> Result<&'de [u8; N], std::io::Error>;

    /// Deserialize a unit value.
    fn deserialize_unit(&mut self) -> Result<(), std::io::Error>;
}
