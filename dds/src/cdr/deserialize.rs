pub use dust_dds_derive::CdrDeserialize;

use super::{
    deserializer::{CdrDeserializer, ClassicCdrDeserializer},
    error::CdrResult,
};

pub trait CdrDeserialize<'de>: Sized {
    fn deserialize(deserializer: &mut ClassicCdrDeserializer<'de>) -> CdrResult<Self>;
}

impl<'de> CdrDeserialize<'de> for bool {
    fn deserialize(deserializer: &mut ClassicCdrDeserializer<'de>) -> CdrResult<Self> {
        deserializer.deserialize_bool()
    }
}

impl<'de> CdrDeserialize<'de> for i8 {
    fn deserialize(deserializer: &mut ClassicCdrDeserializer<'de>) -> CdrResult<Self> {
        deserializer.deserialize_i8()
    }
}

impl<'de> CdrDeserialize<'de> for i16 {
    fn deserialize(deserializer: &mut ClassicCdrDeserializer<'de>) -> CdrResult<Self> {
        deserializer.deserialize_i16()
    }
}

impl<'de> CdrDeserialize<'de> for i32 {
    fn deserialize(deserializer: &mut ClassicCdrDeserializer<'de>) -> CdrResult<Self> {
        deserializer.deserialize_i32()
    }
}

impl<'de> CdrDeserialize<'de> for i64 {
    fn deserialize(deserializer: &mut ClassicCdrDeserializer<'de>) -> CdrResult<Self> {
        deserializer.deserialize_i64()
    }
}

impl<'de> CdrDeserialize<'de> for u8 {
    fn deserialize(deserializer: &mut ClassicCdrDeserializer<'de>) -> CdrResult<Self> {
        deserializer.deserialize_u8()
    }
}

impl<'de> CdrDeserialize<'de> for u16 {
    fn deserialize(deserializer: &mut ClassicCdrDeserializer<'de>) -> CdrResult<Self> {
        deserializer.deserialize_u16()
    }
}

impl<'de> CdrDeserialize<'de> for u32 {
    fn deserialize(deserializer: &mut ClassicCdrDeserializer<'de>) -> CdrResult<Self> {
        deserializer.deserialize_u32()
    }
}

impl<'de> CdrDeserialize<'de> for u64 {
    fn deserialize(deserializer: &mut ClassicCdrDeserializer<'de>) -> CdrResult<Self> {
        deserializer.deserialize_u64()
    }
}

impl<'de> CdrDeserialize<'de> for f32 {
    fn deserialize(deserializer: &mut ClassicCdrDeserializer<'de>) -> CdrResult<Self> {
        deserializer.deserialize_f32()
    }
}

impl<'de> CdrDeserialize<'de> for f64 {
    fn deserialize(deserializer: &mut ClassicCdrDeserializer<'de>) -> CdrResult<Self> {
        deserializer.deserialize_f64()
    }
}

impl<'de> CdrDeserialize<'de> for char {
    fn deserialize(deserializer: &mut ClassicCdrDeserializer<'de>) -> CdrResult<Self> {
        deserializer.deserialize_char()
    }
}

impl<'de> CdrDeserialize<'de> for String {
    fn deserialize(deserializer: &mut ClassicCdrDeserializer<'de>) -> CdrResult<Self> {
        deserializer.deserialize_string()
    }
}

impl<'de, const N: usize, T> CdrDeserialize<'de> for [T; N]
where
    T: CdrDeserialize<'de>,
{
    fn deserialize(deserializer: &mut ClassicCdrDeserializer<'de>) -> CdrResult<Self> {
        deserializer.deserialize_array()
    }
}

impl<'de, T> CdrDeserialize<'de> for Vec<T>
where
    T: CdrDeserialize<'de>,
{
    fn deserialize(deserializer: &mut ClassicCdrDeserializer<'de>) -> CdrResult<Self> {
        deserializer.deserialize_seq()
    }
}

impl<'de> CdrDeserialize<'de> for &'de [u8] {
    fn deserialize(deserializer: &mut ClassicCdrDeserializer<'de>) -> CdrResult<Self> {
        deserializer.deserialize_bytes()
    }
}

impl<'de, const N: usize> CdrDeserialize<'de> for &'de [u8; N] {
    fn deserialize(deserializer: &mut ClassicCdrDeserializer<'de>) -> CdrResult<Self> {
        deserializer.deserialize_byte_array()
    }
}

impl<'de> CdrDeserialize<'de> for () {
    fn deserialize(deserializer: &mut ClassicCdrDeserializer<'de>) -> CdrResult<Self> {
        deserializer.deserialize_unit()
    }
}
