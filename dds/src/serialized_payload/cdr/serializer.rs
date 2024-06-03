use super::serialize::CdrSerialize;

/// A trait representing an object with the capability of serializing a value into a CDR format.
pub trait CdrSerializer {
    /// Serialize a [`bool`] value.
    fn serialize_bool(&mut self, v: bool) -> Result<(), std::io::Error>;

    /// Serialize an [`i8`] value.
    fn serialize_i8(&mut self, v: i8) -> Result<(), std::io::Error>;

    /// Serialize an [`i16`] value.
    fn serialize_i16(&mut self, v: i16) -> Result<(), std::io::Error>;

    /// Serialize an [`i32`] value.
    fn serialize_i32(&mut self, v: i32) -> Result<(), std::io::Error>;

    /// Serialize an [`i64`] value.
    fn serialize_i64(&mut self, v: i64) -> Result<(), std::io::Error>;

    /// Serialize a [`u8`] value.
    fn serialize_u8(&mut self, v: u8) -> Result<(), std::io::Error>;

    /// Serialize a [`u16`] value.
    fn serialize_u16(&mut self, v: u16) -> Result<(), std::io::Error>;

    /// Serialize a [`u32`] value.
    fn serialize_u32(&mut self, v: u32) -> Result<(), std::io::Error>;

    /// Serialize a [`u64`] value.
    fn serialize_u64(&mut self, v: u64) -> Result<(), std::io::Error>;

    /// Serialize an [`f32`] value.
    fn serialize_f32(&mut self, v: f32) -> Result<(), std::io::Error>;

    /// Serialize an [`f64`] value.
    fn serialize_f64(&mut self, v: f64) -> Result<(), std::io::Error>;

    /// Serialize a [`char`] value.
    fn serialize_char(&mut self, v: char) -> Result<(), std::io::Error>;

    /// Serialize a [`str`] value.
    fn serialize_str(&mut self, v: &str) -> Result<(), std::io::Error>;

    /// Serialize an unknown sized sequence of values.
    fn serialize_seq(&mut self, v: &[impl CdrSerialize]) -> Result<(), std::io::Error>;

    /// Serialize an array value.
    fn serialize_array<const N: usize>(
        &mut self,
        v: &[impl CdrSerialize; N],
    ) -> Result<(), std::io::Error>;

    /// Serialize a variable sized sequence of bytes.
    fn serialize_bytes(&mut self, v: &[u8]) -> Result<(), std::io::Error>;

    /// Serialize an array of bytes.
    fn serialize_byte_array<const N: usize>(&mut self, v: &[u8; N]) -> Result<(), std::io::Error>;

    /// Serialize a unit value.
    fn serialize_unit(&mut self) -> Result<(), std::io::Error>;
}
