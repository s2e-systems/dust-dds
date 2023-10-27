use super::{deserialize::CdrDeserialize, error::CdrResult};

pub trait CdrDeserializer<'de> {
    fn deserialize_bool(&mut self) -> CdrResult<bool>;
    fn deserialize_i8(&mut self) -> CdrResult<i8>;
    fn deserialize_i16(&mut self) -> CdrResult<i16>;
    fn deserialize_i32(&mut self) -> CdrResult<i32>;
    fn deserialize_i64(&mut self) -> CdrResult<i64>;
    fn deserialize_u8(&mut self) -> CdrResult<u8>;
    fn deserialize_u16(&mut self) -> CdrResult<u16>;
    fn deserialize_u32(&mut self) -> CdrResult<u32>;
    fn deserialize_u64(&mut self) -> CdrResult<u64>;
    fn deserialize_f32(&mut self) -> CdrResult<f32>;
    fn deserialize_f64(&mut self) -> CdrResult<f64>;
    fn deserialize_char(&mut self) -> CdrResult<char>;
    fn deserialize_string(&mut self) -> CdrResult<String>;
    fn deserialize_seq<T>(&mut self) -> CdrResult<Vec<T>>
    where
        T: CdrDeserialize<'de>;
    fn deserialize_array<const N: usize, T>(&mut self) -> CdrResult<[T; N]>
    where
        T: CdrDeserialize<'de>;
    fn deserialize_bytes(&mut self) -> CdrResult<&'de [u8]>;
    fn deserialize_byte_array<const N: usize>(&mut self) -> CdrResult<&'de [u8; N]>;
    fn deserialize_unit(&mut self) -> CdrResult<()>;
}
