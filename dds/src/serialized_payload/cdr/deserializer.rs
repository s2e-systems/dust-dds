use super::deserialize::CdrDeserialize;

pub trait CdrDeserializer<'de> {
    fn deserialize_bool(&mut self) -> Result<bool, std::io::Error>;

    fn deserialize_i8(&mut self) -> Result<i8, std::io::Error>;

    fn deserialize_i16(&mut self) -> Result<i16, std::io::Error>;

    fn deserialize_i32(&mut self) -> Result<i32, std::io::Error>;

    fn deserialize_i64(&mut self) -> Result<i64, std::io::Error>;

    fn deserialize_u8(&mut self) -> Result<u8, std::io::Error>;

    fn deserialize_u16(&mut self) -> Result<u16, std::io::Error>;

    fn deserialize_u32(&mut self) -> Result<u32, std::io::Error>;

    fn deserialize_u64(&mut self) -> Result<u64, std::io::Error>;

    fn deserialize_f32(&mut self) -> Result<f32, std::io::Error>;

    fn deserialize_f64(&mut self) -> Result<f64, std::io::Error>;

    fn deserialize_char(&mut self) -> Result<char, std::io::Error>;

    fn deserialize_string(&mut self) -> Result<String, std::io::Error>;

    fn deserialize_seq<T>(&mut self) -> Result<Vec<T>, std::io::Error>
    where
        T: CdrDeserialize<'de>;

    fn deserialize_array<const N: usize, T>(&mut self) -> Result<[T; N], std::io::Error>
    where
        T: CdrDeserialize<'de>;

    fn deserialize_bytes(&mut self) -> Result<&'de [u8], std::io::Error>;

    fn deserialize_byte_array<const N: usize>(&mut self) -> Result<&'de [u8; N], std::io::Error>;

    fn deserialize_unit(&mut self) -> Result<(), std::io::Error>;
}
