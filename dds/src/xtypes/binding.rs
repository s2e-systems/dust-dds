use crate::{
    infrastructure::type_support::TypeSupport,
    xtypes::dynamic_type::{
        DynamicType, ExtensibilityKind, StaticTypeInformation, TypeDescriptor, TypeKind,
    },
};
use alloc::{string::String, vec::Vec};

pub trait XTypesBinding {
    const TYPE_INFORMATION: &'static dyn DynamicType;
}

impl XTypesBinding for u8 {
    const TYPE_INFORMATION: &'static dyn DynamicType = &StaticTypeInformation {
        descriptor: &TypeDescriptor {
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
        member_list: &[],
    };
}
impl XTypesBinding for i8 {
    const TYPE_INFORMATION: &'static dyn DynamicType = &StaticTypeInformation {
        descriptor: &TypeDescriptor {
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
        member_list: &[],
    };
}

impl XTypesBinding for u16 {
    const TYPE_INFORMATION: &'static dyn DynamicType = &StaticTypeInformation {
        descriptor: &TypeDescriptor {
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
        member_list: &[],
    };
}

impl XTypesBinding for i16 {
    const TYPE_INFORMATION: &'static dyn DynamicType = &StaticTypeInformation {
        descriptor: &TypeDescriptor {
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
        member_list: &[],
    };
}

impl XTypesBinding for u32 {
    const TYPE_INFORMATION: &'static dyn DynamicType = &StaticTypeInformation {
        descriptor: &TypeDescriptor {
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
        member_list: &[],
    };
}

impl XTypesBinding for i32 {
    const TYPE_INFORMATION: &'static dyn DynamicType = &StaticTypeInformation {
        descriptor: &TypeDescriptor {
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
        member_list: &[],
    };
}

impl XTypesBinding for u64 {
    const TYPE_INFORMATION: &'static dyn DynamicType = &StaticTypeInformation {
        descriptor: &TypeDescriptor {
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
        member_list: &[],
    };
}

impl XTypesBinding for i64 {
    const TYPE_INFORMATION: &'static dyn DynamicType = &StaticTypeInformation {
        descriptor: &TypeDescriptor {
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
        member_list: &[],
    };
}

impl XTypesBinding for String {
    const TYPE_INFORMATION: &'static dyn DynamicType = &StaticTypeInformation {
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

impl XTypesBinding for &'_ str {
    const TYPE_INFORMATION: &'static dyn DynamicType = &StaticTypeInformation {
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

impl XTypesBinding for bool {
    const TYPE_INFORMATION: &'static dyn DynamicType = &StaticTypeInformation {
        descriptor: &TypeDescriptor {
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
        member_list: &[],
    };
}

impl XTypesBinding for f32 {
    const TYPE_INFORMATION: &'static dyn DynamicType = &StaticTypeInformation {
        descriptor: &TypeDescriptor {
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
        member_list: &[],
    };
}

impl XTypesBinding for f64 {
    const TYPE_INFORMATION: &'static dyn DynamicType = &StaticTypeInformation {
        descriptor: &TypeDescriptor {
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
        member_list: &[],
    };
}

impl XTypesBinding for char {
    const TYPE_INFORMATION: &'static dyn DynamicType = &StaticTypeInformation {
        descriptor: &TypeDescriptor {
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
        member_list: &[],
    };
}

impl XTypesBinding for &'_ [u8] {
    const TYPE_INFORMATION: &'static dyn DynamicType = &StaticTypeInformation {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(u8::TYPE_INFORMATION),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl<T: XTypesBinding> XTypesBinding for Vec<T> {
    const TYPE_INFORMATION: &'static dyn DynamicType = &StaticTypeInformation {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(T::TYPE_INFORMATION),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl<T: TypeSupport> XTypesBinding for T {
    const TYPE_INFORMATION: &'static dyn DynamicType = T::TYPE;
}

impl<T: XTypesBinding, const N: usize> XTypesBinding for [T; N] {
    const TYPE_INFORMATION: &'static dyn DynamicType = &StaticTypeInformation {
        descriptor: &TypeDescriptor {
            kind: TypeKind::ARRAY,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(N as u32),
            element_type: Some(T::TYPE_INFORMATION),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}

impl<T: XTypesBinding> XTypesBinding for Option<T> {
    const TYPE_INFORMATION: &'static dyn DynamicType = T::TYPE_INFORMATION;
}
