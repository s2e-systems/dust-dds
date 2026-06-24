use crate::xtypes::dynamic_type::{
    DynamicData, DynamicType, ExtensibilityKind, TypeDescriptor, TypeKind,
};
use alloc::{boxed::Box, string::String, vec::Vec};
pub use dust_dds_derive::TypeSupport;

/// The Type trait represents static type information of Rust types
pub trait Type {
    /// This constant represent the ['DynamicType'] object corresponding to the TypeSupport’s data type
    const TYPE: DynamicType<'static>;
}

/// The TypeSupport trait represents a type that can be transmitted by DDS.
pub trait TypeSupport: Type {
    /// This constant represent the ['DynamicType'] object corresponding to the TypeSupport’s data type
    fn get_type() -> DynamicType<'static> {
        Self::TYPE
    }

    /// Create a sample of the TypeSupport’s data type with the contents of an input DynamicData object.
    fn create_sample(src: &mut DynamicData<'static>) -> Self;

    /// Create a 'DynamicData' object with the contents of an input sample of the TypeSupport’s data type.
    fn create_dynamic_sample(self, data: &mut DynamicData<'static>);
}

/// Preregistered String type as per Annex E: Built-in Types
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
pub struct _String {
    /// value
    pub value: String,
}
impl From<String> for _String {
    fn from(value: String) -> Self {
        Self { value }
    }
}
impl From<_String> for String {
    fn from(value: _String) -> Self {
        value.value
    }
}

impl Type for i8 {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::INT8,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: true,
        },
        member_list: &[],
    };
}

impl Type for u8 {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::UINT8,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: true,
        },
        member_list: &[],
    };
}

impl Type for i16 {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::INT16,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: true,
        },
        member_list: &[],
    };
}

impl Type for u16 {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::UINT16,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: true,
        },
        member_list: &[],
    };
}

impl Type for i32 {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::INT32,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: true,
        },
        member_list: &[],
    };
}

impl Type for u32 {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::UINT32,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: true,
        },
        member_list: &[],
    };
}

impl Type for i64 {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::INT64,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: true,
        },
        member_list: &[],
    };
}

impl Type for u64 {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::UINT64,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: true,
        },
        member_list: &[],
    };
}

impl Type for f32 {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::FLOAT32,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: true,
        },
        member_list: &[],
    };
}

impl Type for f64 {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::FLOAT64,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: true,
        },
        member_list: &[],
    };
}

impl Type for bool {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::BOOLEAN,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: true,
        },
        member_list: &[],
    };
}

impl Type for char {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::CHAR8,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: true,
        },
        member_list: &[],
    };
}

impl<T> Type for Box<T> {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::ALIAS,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl<T> Type for Option<T> {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::ALIAS,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl Type for String {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::STRING8,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl<T: Type, const N: usize> Type for [T; N] {
    const TYPE: DynamicType<'static> = crate::xtypes::dynamic_type::DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::ARRAY,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(N as u32),
            element_type: Some(T::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl Type for &[u8] {
    const TYPE: DynamicType<'static> = crate::xtypes::dynamic_type::DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::ARRAY,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(u8::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl Type for &str {
    const TYPE: DynamicType<'static> = crate::xtypes::dynamic_type::DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::STRING8,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl<T: TypeSupport> Type for Vec<T> {
    const TYPE: DynamicType<'static> = crate::xtypes::dynamic_type::DynamicType {
        descriptor: &crate::xtypes::dynamic_type::TypeDescriptor {
            kind: crate::xtypes::dynamic_type::TypeKind::SEQUENCE,
            name: "SequenceComplexValue",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(T::TYPE),
            key_element_type: None,
            extensibility_kind: crate::xtypes::dynamic_type::ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl Type for Vec<i8> {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(u8::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl Type for Vec<u8> {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(u8::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl Type for Vec<i16> {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(i16::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl Type for Vec<u16> {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(u16::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl Type for Vec<i32> {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(i32::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl Type for Vec<u32> {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(u32::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl Type for Vec<i64> {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(i64::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl Type for Vec<u64> {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(u64::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl Type for Vec<f32> {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(f32::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl Type for Vec<f64> {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(f64::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl Type for Vec<String> {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(String::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xtypes::dynamic_type::DynamicDataFactory;

    fn create_dynamic_sample<T: TypeSupport>(v: T) -> DynamicData<'static> {
        let mut data = crate::xtypes::dynamic_type::DynamicDataFactory::create_data(T::TYPE);
        v.create_dynamic_sample(&mut data);
        data
    }

    #[test]
    fn basic_type_should_create_sample() {
        #[derive(TypeSupport)]
        pub struct Inner {
            x: u16,
        }

        let mut inner = DynamicDataFactory::create_data(Inner::TYPE);
        inner.set_uint16_value(0, 2).unwrap();

        let v = Inner { x: 2 };
        assert_eq!(create_dynamic_sample(v), inner);
    }

    #[test]
    fn complex_type_should_create_sample() {
        #[derive(TypeSupport)]
        pub struct Inner {
            x: u16,
        }
        #[derive(TypeSupport)]
        pub struct Outer {
            y: Inner,
        }

        let v = Outer { y: Inner { x: 2 } };
        let mut inner = DynamicDataFactory::create_data(Inner::TYPE);
        inner.set_uint16_value(0, 2).unwrap();
        let mut outer = DynamicDataFactory::create_data(Outer::TYPE);
        outer.set_complex_value(0, inner).unwrap();

        assert_eq!(create_dynamic_sample(v), outer);
    }

    #[test]
    fn vector_of_basic_type_should_create_sample() {
        #[derive(TypeSupport)]
        pub struct Inner {
            x: Vec<u16>,
        }
        let mut inner = DynamicDataFactory::create_data(Inner::TYPE);
        inner.set_uint16_values(0, vec![2]).unwrap();

        let v = Inner { x: vec![2] };
        assert_eq!(create_dynamic_sample(v), inner);
    }

    #[test]
    fn vector_of_complex_type_should_create_sample() {
        #[derive(TypeSupport)]
        pub struct Inner {
            x: u16,
        }
        #[derive(TypeSupport)]
        pub struct Outer {
            list: Vec<Inner>,
        }

        let v = Outer {
            list: vec![Inner { x: 2 }],
        };
        let mut inner = DynamicDataFactory::create_data(Inner::TYPE);
        inner.set_uint16_value(0, 2).unwrap();
        let mut outer = DynamicDataFactory::create_data(Outer::TYPE);
        outer.set_complex_values(0, vec![inner]).unwrap();

        assert_eq!(create_dynamic_sample(v), outer);
    }
}
