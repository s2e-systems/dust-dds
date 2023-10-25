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
