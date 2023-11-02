use crate::serialized_payload::{error::CdrResult, serialize::CdrSerialize};

pub trait CdrSerializer {
    fn serialize_bool(&mut self, v: bool) -> CdrResult<()>;

    fn serialize_i8(&mut self, v: i8) -> CdrResult<()>;

    fn serialize_i16(&mut self, v: i16) -> CdrResult<()>;

    fn serialize_i32(&mut self, v: i32) -> CdrResult<()>;

    fn serialize_i64(&mut self, v: i64) -> CdrResult<()>;

    fn serialize_u8(&mut self, v: u8) -> CdrResult<()>;

    fn serialize_u16(&mut self, v: u16) -> CdrResult<()>;

    fn serialize_u32(&mut self, v: u32) -> CdrResult<()>;

    fn serialize_u64(&mut self, v: u64) -> CdrResult<()>;

    fn serialize_f32(&mut self, v: f32) -> CdrResult<()>;

    fn serialize_f64(&mut self, v: f64) -> CdrResult<()>;

    fn serialize_char(&mut self, v: char) -> CdrResult<()>;

    fn serialize_str(&mut self, v: &str) -> CdrResult<()>;

    fn serialize_seq(&mut self, v: &[impl CdrSerialize]) -> CdrResult<()>;

    fn serialize_array<const N: usize>(&mut self, v: &[impl CdrSerialize; N]) -> CdrResult<()>;

    fn serialize_unit(&mut self) -> CdrResult<()>;
}
