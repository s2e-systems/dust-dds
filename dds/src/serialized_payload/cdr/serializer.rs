use super::serialize::CdrSerialize;

pub trait CdrSerializer {
    fn serialize_bool(&mut self, v: bool) -> Result<(), std::io::Error>;

    fn serialize_i8(&mut self, v: i8) -> Result<(), std::io::Error>;

    fn serialize_i16(&mut self, v: i16) -> Result<(), std::io::Error>;

    fn serialize_i32(&mut self, v: i32) -> Result<(), std::io::Error>;

    fn serialize_i64(&mut self, v: i64) -> Result<(), std::io::Error>;

    fn serialize_u8(&mut self, v: u8) -> Result<(), std::io::Error>;

    fn serialize_u16(&mut self, v: u16) -> Result<(), std::io::Error>;

    fn serialize_u32(&mut self, v: u32) -> Result<(), std::io::Error>;

    fn serialize_u64(&mut self, v: u64) -> Result<(), std::io::Error>;

    fn serialize_f32(&mut self, v: f32) -> Result<(), std::io::Error>;

    fn serialize_f64(&mut self, v: f64) -> Result<(), std::io::Error>;

    fn serialize_char(&mut self, v: char) -> Result<(), std::io::Error>;

    fn serialize_str(&mut self, v: &str) -> Result<(), std::io::Error>;

    fn serialize_seq(&mut self, v: &[impl CdrSerialize]) -> Result<(), std::io::Error>;

    fn serialize_array<const N: usize>(&mut self, v: &[impl CdrSerialize; N]) -> Result<(), std::io::Error>;

    fn serialize_unit(&mut self) -> Result<(), std::io::Error>;
}
