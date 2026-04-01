use crate::{
    infrastructure::type_support::TypeSupport,
    xtypes::dynamic_type::{
        DynamicType, DynamicTypeBuilderFactory, ExtensibilityKind, TypeDescriptor, TypeKind,
    },
};
use alloc::{string::String, vec::Vec};

pub trait XTypesBinding {
    fn get_dynamic_type() -> DynamicType;
}

impl XTypesBinding for u8 {
    fn get_dynamic_type() -> DynamicType {
        DynamicType::new(
            &TypeDescriptor {
                kind: TypeKind::UINT8,
                name: "",
                base_type: None,
                discriminator_type: None,
                bound: None,
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            },
            &[],
        )
    }
}
impl XTypesBinding for i8 {
    fn get_dynamic_type() -> DynamicType {
        DynamicType::new(
            &TypeDescriptor {
                kind: TypeKind::INT8,
                name: "",
                base_type: None,
                discriminator_type: None,
                bound: None,
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            },
            &[],
        )
    }
}

impl XTypesBinding for u16 {
    fn get_dynamic_type() -> DynamicType {
        DynamicType::new(
            &TypeDescriptor {
                kind: TypeKind::UINT16,
                name: "",
                base_type: None,
                discriminator_type: None,
                bound: None,
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            },
            &[],
        )
    }
}

impl XTypesBinding for i16 {
    fn get_dynamic_type() -> DynamicType {
        DynamicType::new(
            &TypeDescriptor {
                kind: TypeKind::INT16,
                name: "",
                base_type: None,
                discriminator_type: None,
                bound: None,
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            },
            &[],
        )
    }
}

impl XTypesBinding for u32 {
    fn get_dynamic_type() -> DynamicType {
        DynamicType::new(
            &TypeDescriptor {
                kind: TypeKind::UINT32,
                name: "",
                base_type: None,
                discriminator_type: None,
                bound: None,
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            },
            &[],
        )
    }
}

impl XTypesBinding for i32 {
    fn get_dynamic_type() -> DynamicType {
        DynamicType::new(
            &TypeDescriptor {
                kind: TypeKind::INT32,
                name: "",
                base_type: None,
                discriminator_type: None,
                bound: None,
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            },
            &[],
        )
    }
}

impl XTypesBinding for u64 {
    fn get_dynamic_type() -> DynamicType {
        DynamicType::new(
            &TypeDescriptor {
                kind: TypeKind::UINT64,
                name: "",
                base_type: None,
                discriminator_type: None,
                bound: None,
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            },
            &[],
        )
    }
}

impl XTypesBinding for i64 {
    fn get_dynamic_type() -> DynamicType {
        DynamicType::new(
            &TypeDescriptor {
                kind: TypeKind::INT64,
                name: "",
                base_type: None,
                discriminator_type: None,
                bound: None,
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            },
            &[],
        )
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
        DynamicType::new(
            &TypeDescriptor {
                kind: TypeKind::BOOLEAN,
                name: "",
                base_type: None,
                discriminator_type: None,
                bound: None,
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            },
            &[],
        )
    }
}

impl XTypesBinding for f32 {
    fn get_dynamic_type() -> DynamicType {
        DynamicType::new(
            &TypeDescriptor {
                kind: TypeKind::FLOAT32,
                name: "",
                base_type: None,
                discriminator_type: None,
                bound: None,
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            },
            &[],
        )
    }
}

impl XTypesBinding for f64 {
    fn get_dynamic_type() -> DynamicType {
        DynamicType::new(
            &TypeDescriptor {
                kind: TypeKind::FLOAT64,
                name: "",
                base_type: None,
                discriminator_type: None,
                bound: None,
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            },
            &[],
        )
    }
}

impl XTypesBinding for char {
    fn get_dynamic_type() -> DynamicType {
        DynamicType::new(
            &TypeDescriptor {
                kind: TypeKind::CHAR8,
                name: "",
                base_type: None,
                discriminator_type: None,
                bound: None,
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            },
            &[],
        )
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

impl<T: XTypesBinding, const N: usize> XTypesBinding for [T; N] {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_array_type(T::get_dynamic_type(), Some(N as u32)).build()
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
