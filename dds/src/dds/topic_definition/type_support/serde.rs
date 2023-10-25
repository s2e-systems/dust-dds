use crate::infrastructure::error::DdsResult;

pub use dust_dds_derive::DdsSerialize;

pub trait DdsSerialize {
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()>;
}

pub trait DdsSerializer {
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
    fn serialize_seq(&mut self, v: &[impl DdsSerialize]) -> DdsResult<()>;
    fn serialize_array<const N: usize>(&mut self, v: &[impl DdsSerialize; N]) -> DdsResult<()>;
}

impl DdsSerialize for bool {
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        serializer.serialize_bool(*self)
    }
}

impl DdsSerialize for i8 {
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        serializer.serialize_i8(*self)
    }
}

impl DdsSerialize for i16 {
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        serializer.serialize_i16(*self)
    }
}

impl DdsSerialize for i32 {
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        serializer.serialize_i32(*self)
    }
}

impl DdsSerialize for i64 {
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        serializer.serialize_i64(*self)
    }
}

impl DdsSerialize for u8 {
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        serializer.serialize_u8(*self)
    }
}

impl DdsSerialize for u16 {
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        serializer.serialize_u16(*self)
    }
}

impl DdsSerialize for u32 {
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        serializer.serialize_u32(*self)
    }
}

impl DdsSerialize for u64 {
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        serializer.serialize_u64(*self)
    }
}

impl DdsSerialize for f32 {
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        serializer.serialize_f32(*self)
    }
}

impl DdsSerialize for f64 {
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        serializer.serialize_f64(*self)
    }
}

impl DdsSerialize for char {
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        serializer.serialize_char(*self)
    }
}

impl DdsSerialize for str {
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        serializer.serialize_str(self)
    }
}

impl DdsSerialize for String {
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        serializer.serialize_str(self)
    }
}

impl<T> DdsSerialize for [T]
where
    T: DdsSerialize,
{
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        serializer.serialize_seq(self)
    }
}

impl<const N: usize, T> DdsSerialize for [T; N]
where
    T: DdsSerialize,
{
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        serializer.serialize_array(self)
    }
}

impl<T> DdsSerialize for Vec<T>
where
    T: DdsSerialize,
{
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        serializer.serialize_seq(self)
    }
}

impl<T> DdsSerialize for &'_ T
where
    T: DdsSerialize + ?Sized,
{
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        T::serialize(*self, serializer)
    }
}

impl<T> DdsSerialize for &'_ mut T
where
    T: DdsSerialize + ?Sized,
{
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        T::serialize(*self, serializer)
    }
}

impl<T> DdsSerialize for Box<T>
where
    T: DdsSerialize,
{
    fn serialize(&self, serializer: &mut impl DdsSerializer) -> DdsResult<()> {
        self.as_ref().serialize(serializer)
    }
}
