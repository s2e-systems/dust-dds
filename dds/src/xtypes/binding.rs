use crate::{
    infrastructure::type_support::TypeSupport,
    xtypes::dynamic_type::{DynamicType, DynamicTypeBuilderFactory, TypeKind},
};
use alloc::{string::String, vec, vec::Vec};

pub trait XTypesBinding {
    fn get_dynamic_type() -> DynamicType;
}

impl XTypesBinding for u8 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::UINT8)
    }
}
impl XTypesBinding for i8 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::INT8)
    }
}

impl XTypesBinding for u16 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::UINT16)
    }
}

impl XTypesBinding for i16 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::INT16)
    }
}

impl XTypesBinding for u32 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::UINT32)
    }
}

impl XTypesBinding for i32 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::INT32)
    }
}

impl XTypesBinding for u64 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::UINT64)
    }
}

impl XTypesBinding for i64 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::INT64)
    }
}

impl XTypesBinding for String {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_string_type(u32::MAX).build()
    }
}

impl XTypesBinding for &'_ str {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_string_type(u32::MAX).build()
    }
}

impl XTypesBinding for bool {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::BOOLEAN)
    }
}

impl XTypesBinding for f32 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::FLOAT32)
    }
}

impl XTypesBinding for f64 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::FLOAT64)
    }
}

impl XTypesBinding for char {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::CHAR8)
    }
}

impl<const N: usize> XTypesBinding for [u8; N] {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_array_type(u8::get_dynamic_type(), vec![N as u32]).build()
    }
}

impl<const N: usize> XTypesBinding for [i16; N] {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_array_type(i16::get_dynamic_type(), vec![N as u32])
            .build()
    }
}

impl XTypesBinding for &'_ [u8] {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u8::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<u8> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u8::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<u16> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u16::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<u32> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u32::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<u64> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u64::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<i8> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(i8::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<i16> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(i16::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<i32> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(i32::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<i64> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(i64::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<f32> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(f32::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<f64> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(f64::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<String> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(String::get_dynamic_type(), u32::MAX)
            .build()
    }
}

impl<T: TypeSupport> XTypesBinding for T {
    fn get_dynamic_type() -> DynamicType {
        T::get_type()
    }
}

impl<T: TypeSupport> XTypesBinding for Vec<T> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(T::get_type(), u32::MAX).build()
    }
}

impl<T: XTypesBinding> XTypesBinding for Option<T> {
    fn get_dynamic_type() -> DynamicType {
        T::get_dynamic_type()
    }
}
