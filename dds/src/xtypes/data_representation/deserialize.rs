use crate::xtypes::error::XTypesResult;

pub trait PrimitiveTypeDeserializer {
    fn deserialize_u8(&mut self) -> XTypesResult<u8>;
    fn deserialize_u16(&mut self) -> XTypesResult<u16>;
    fn deserialize_u32(&mut self) -> XTypesResult<u32>;
    fn deserialize_u64(&mut self) -> XTypesResult<u64>;
    fn deserialize_i8(&mut self) -> XTypesResult<i8>;
    fn deserialize_i16(&mut self) -> XTypesResult<i16>;
    fn deserialize_i32(&mut self) -> XTypesResult<i32>;
    fn deserialize_i64(&mut self) -> XTypesResult<i64>;
    fn deserialize_f32(&mut self) -> XTypesResult<f32>;
    fn deserialize_f64(&mut self) -> XTypesResult<f64>;
    fn deserialize_boolean(&mut self) -> XTypesResult<bool>;
    fn deserialize_char(&mut self) -> XTypesResult<char>;

    fn deserialize_primitive_type_sequence<T: PrimitiveTypeDeserialize>(
        &mut self,
    ) -> XTypesResult<Vec<T>>;
}

// This trait is only meant to be implemented on Rust basic types
// to allow for generalizing the process of deserializing those types
pub trait PrimitiveTypeDeserialize: Sized {
    fn deserialize(d: &mut impl PrimitiveTypeDeserializer) -> XTypesResult<Self>;
}

impl PrimitiveTypeDeserialize for u8 {
    fn deserialize(d: &mut impl PrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_u8()
    }
}
impl PrimitiveTypeDeserialize for u16 {
    fn deserialize(d: &mut impl PrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_u16()
    }
}
impl PrimitiveTypeDeserialize for u32 {
    fn deserialize(d: &mut impl PrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_u32()
    }
}
impl PrimitiveTypeDeserialize for u64 {
    fn deserialize(d: &mut impl PrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_u64()
    }
}
impl PrimitiveTypeDeserialize for i8 {
    fn deserialize(d: &mut impl PrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_i8()
    }
}
impl PrimitiveTypeDeserialize for i16 {
    fn deserialize(d: &mut impl PrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_i16()
    }
}
impl PrimitiveTypeDeserialize for i32 {
    fn deserialize(d: &mut impl PrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_i32()
    }
}
impl PrimitiveTypeDeserialize for i64 {
    fn deserialize(d: &mut impl PrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_i64()
    }
}
impl PrimitiveTypeDeserialize for f32 {
    fn deserialize(d: &mut impl PrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_f32()
    }
}
impl PrimitiveTypeDeserialize for f64 {
    fn deserialize(d: &mut impl PrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_f64()
    }
}
impl PrimitiveTypeDeserialize for bool {
    fn deserialize(d: &mut impl PrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_boolean()
    }
}
impl PrimitiveTypeDeserialize for char {
    fn deserialize(d: &mut impl PrimitiveTypeDeserializer) -> XTypesResult<Self> {
        d.deserialize_char()
    }
}
