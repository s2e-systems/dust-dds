use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_ERROR, RETCODE_OK, ReturnCode};
use dust_dds::xtypes::dynamic_type::{
    DynamicType as RustDynamicType, DynamicTypeBuilder as RustDynamicTypeBuilder, DynamicTypeBuilderFactory, ExtensibilityKind,
    MemberDescriptor as RustMemberDescriptor, TryConstructKind, TypeDescriptor as RustTypeDescriptor, TypeKind,
};
use std::ptr::NonNull;

// Re-export type kind constants derived from Rust TypeKind (as literals for cbindgen)
pub const TYPE_KIND_NONE: u8 = 0x00;
pub const TYPE_KIND_BOOLEAN: u8 = 0x01;
pub const TYPE_KIND_BYTE: u8 = 0x02;
pub const TYPE_KIND_INT16: u8 = 0x03;
pub const TYPE_KIND_INT32: u8 = 0x04;
pub const TYPE_KIND_INT64: u8 = 0x05;
pub const TYPE_KIND_UINT16: u8 = 0x06;
pub const TYPE_KIND_UINT32: u8 = 0x07;
pub const TYPE_KIND_UINT64: u8 = 0x08;
pub const TYPE_KIND_FLOAT32: u8 = 0x09;
pub const TYPE_KIND_FLOAT64: u8 = 0x0A;
pub const TYPE_KIND_FLOAT128: u8 = 0x0B;
pub const TYPE_KIND_INT8: u8 = 0x0C;
pub const TYPE_KIND_UINT8: u8 = 0x0D;
pub const TYPE_KIND_CHAR8: u8 = 0x10;
pub const TYPE_KIND_CHAR16: u8 = 0x11;
pub const TYPE_KIND_STRING8: u8 = 0x20;
pub const TYPE_KIND_STRING16: u8 = 0x21;
pub const TYPE_KIND_ALIAS: u8 = 0x30;
pub const TYPE_KIND_ENUM: u8 = 0x40;
pub const TYPE_KIND_BITMASK: u8 = 0x41;
pub const TYPE_KIND_ANNOTATION: u8 = 0x50;
pub const TYPE_KIND_STRUCTURE: u8 = 0x51;
pub const TYPE_KIND_UNION: u8 = 0x52;
pub const TYPE_KIND_BITSET: u8 = 0x53;
pub const TYPE_KIND_SEQUENCE: u8 = 0x60;
pub const TYPE_KIND_ARRAY: u8 = 0x61;
pub const TYPE_KIND_MAP: u8 = 0x62;

// Extensibility kind constants (matching DDS spec ExtensibilityKind)
pub const EXTENSIBILITY_KIND_FINAL: u8 = 0;
pub const EXTENSIBILITY_KIND_APPENDABLE: u8 = 1;
pub const EXTENSIBILITY_KIND_MUTABLE: u8 = 2;

/// cbindgen:opaque
pub struct DynamicType(pub(crate) RustDynamicType<'static>);

impl DynamicType {
    pub fn new(t: RustDynamicType<'static>) -> Self {
        Self(t)
    }

    pub fn inner(&self) -> &RustDynamicType<'static> {
        &self.0
    }
}

/// cbindgen:opaque
pub struct DynamicTypeBuilder(pub(crate) RustDynamicTypeBuilder);

impl DynamicTypeBuilder {
    pub fn new(b: RustDynamicTypeBuilder) -> Self {
        Self(b)
    }

    pub fn inner(&self) -> &RustDynamicTypeBuilder {
        &self.0
    }
}

/// Opaque C wrapper around the DDS MemberDescriptor valuetype.
///
/// Mirrors the DDS spec:
/// ```text
/// valuetype MemberDescriptor {
///   public ObjectName name;
///   public MemberId   id;
///   public DynamicType type;
///   public boolean    is_key;
///   public boolean    is_optional;
///   public boolean    is_must_understand;
///   ...
/// };
/// ```
/// C representation of the DDS MemberDescriptor valuetype.
#[repr(C)]
pub struct MemberDescriptor {
    pub name: *const std::os::raw::c_char,
    pub id: u32,
    pub r#type: *const DynamicType,
    pub is_key: bool,
    pub is_optional: bool,
    pub is_must_understand: bool,
}

/// C representation of the DDS TypeDescriptor valuetype.
#[repr(C)]
pub struct TypeDescriptor {
    pub kind: u8,
    pub name: *const std::os::raw::c_char,
    pub base_type: *const DynamicType,
    pub discriminator_type: *const DynamicType,
    pub bound: *const u32,
    pub element_type: *const DynamicType,
    pub key_element_type: *const DynamicType,
    pub extensibility_kind: u8,
    pub is_nested: bool,
}

fn type_kind_from_u8(value: u8) -> Option<TypeKind> {
    match value {
        TYPE_KIND_NONE => Some(TypeKind::NONE),
        TYPE_KIND_BOOLEAN => Some(TypeKind::BOOLEAN),
        TYPE_KIND_BYTE => Some(TypeKind::BYTE),
        TYPE_KIND_INT16 => Some(TypeKind::INT16),
        TYPE_KIND_INT32 => Some(TypeKind::INT32),
        TYPE_KIND_INT64 => Some(TypeKind::INT64),
        TYPE_KIND_UINT16 => Some(TypeKind::UINT16),
        TYPE_KIND_UINT32 => Some(TypeKind::UINT32),
        TYPE_KIND_UINT64 => Some(TypeKind::UINT64),
        TYPE_KIND_FLOAT32 => Some(TypeKind::FLOAT32),
        TYPE_KIND_FLOAT64 => Some(TypeKind::FLOAT64),
        TYPE_KIND_FLOAT128 => Some(TypeKind::FLOAT128),
        TYPE_KIND_INT8 => Some(TypeKind::INT8),
        TYPE_KIND_UINT8 => Some(TypeKind::UINT8),
        TYPE_KIND_CHAR8 => Some(TypeKind::CHAR8),
        TYPE_KIND_CHAR16 => Some(TypeKind::CHAR16),
        TYPE_KIND_STRING8 => Some(TypeKind::STRING8),
        TYPE_KIND_STRING16 => Some(TypeKind::STRING16),
        TYPE_KIND_ALIAS => Some(TypeKind::ALIAS),
        TYPE_KIND_ENUM => Some(TypeKind::ENUM),
        TYPE_KIND_BITMASK => Some(TypeKind::BITMASK),
        TYPE_KIND_ANNOTATION => Some(TypeKind::ANNOTATION),
        TYPE_KIND_STRUCTURE => Some(TypeKind::STRUCTURE),
        TYPE_KIND_UNION => Some(TypeKind::UNION),
        TYPE_KIND_BITSET => Some(TypeKind::BITSET),
        TYPE_KIND_SEQUENCE => Some(TypeKind::SEQUENCE),
        TYPE_KIND_ARRAY => Some(TypeKind::ARRAY),
        TYPE_KIND_MAP => Some(TypeKind::MAP),
        _ => None,
    }
}

fn extensibility_kind_from_u8(value: u8) -> Option<ExtensibilityKind> {
    match value {
        EXTENSIBILITY_KIND_FINAL => Some(ExtensibilityKind::Final),
        EXTENSIBILITY_KIND_APPENDABLE => Some(ExtensibilityKind::Appendable),
        EXTENSIBILITY_KIND_MUTABLE => Some(ExtensibilityKind::Mutable),
        _ => None,
    }
}

/// Creates a new DynamicTypeBuilder using the provided TypeDescriptor.
/// Returns a raw pointer to DynamicTypeBuilder on success, or NULL on failure.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `descriptor` must be a valid pointer to a `TypeDescriptor` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_type_builder_factory_create_type(
    descriptor: *const TypeDescriptor,
) -> Option<NonNull<DynamicTypeBuilder>> {
    if descriptor.is_null() {
        return None;
    }
    let descriptor = unsafe { &*descriptor };
    let name_str = if descriptor.name.is_null() {
        ""
    } else {
        unsafe { std::ffi::CStr::from_ptr(descriptor.name) }
            .to_str()
            .ok()?
            .to_string()
            .leak()
    };

    let base_type = if descriptor.base_type.is_null() {
        None
    } else {
        Some(unsafe { &*descriptor.base_type }.0)
    };

    let discriminator_type = if descriptor.discriminator_type.is_null() {
        None
    } else {
        Some(unsafe { &*descriptor.discriminator_type }.0)
    };

    let bound: &'static [u32] = if descriptor.bound.is_null() {
        &[]
    } else {
        Box::leak(vec![unsafe { *descriptor.bound }].into_boxed_slice())
    };

    let element_type = if descriptor.element_type.is_null() {
        None
    } else {
        Some(unsafe { &*descriptor.element_type }.0)
    };

    let key_element_type = if descriptor.key_element_type.is_null() {
        None
    } else {
        Some(unsafe { &*descriptor.key_element_type }.0)
    };

    let kind = type_kind_from_u8(descriptor.kind)?;
    let extensibility_kind = extensibility_kind_from_u8(descriptor.extensibility_kind)?;

    let type_desc = RustTypeDescriptor {
        kind,
        name: name_str,
        base_type,
        discriminator_type,
        bound,
        element_type,
        key_element_type,
        extensibility_kind,
        is_nested: descriptor.is_nested,
    };

    let builder = DynamicTypeBuilderFactory::create_type(type_desc);
    NonNull::new(Box::into_raw(Box::new(DynamicTypeBuilder::new(
        builder,
    ))))
}

// Compile-time static instances of DynamicType for standard primitive types
use dust_dds::xtypes::type_support::Type;
static BOOLEAN_TYPE: DynamicType = DynamicType(bool::TYPE);
static INT8_TYPE: DynamicType = DynamicType(i8::TYPE);
static UINT8_TYPE: DynamicType = DynamicType(u8::TYPE);
static INT16_TYPE: DynamicType = DynamicType(i16::TYPE);
static UINT16_TYPE: DynamicType = DynamicType(u16::TYPE);
static INT32_TYPE: DynamicType = DynamicType(i32::TYPE);
static UINT32_TYPE: DynamicType = DynamicType(u32::TYPE);
static INT64_TYPE: DynamicType = DynamicType(i64::TYPE);
static UINT64_TYPE: DynamicType = DynamicType(u64::TYPE);
static FLOAT32_TYPE: DynamicType = DynamicType(f32::TYPE);
static FLOAT64_TYPE: DynamicType = DynamicType(f64::TYPE);
static CHAR8_TYPE: DynamicType = DynamicType(char::TYPE);

/// Returns a DynamicType representing the specified primitive type kind.
/// Returns a raw pointer to DynamicType on success, or NULL on failure.
///
/// # Safety
///
/// There are no special safety invariants to be observed when calling this function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_type_get_primitive_type(
    kind: u8,
) -> Option<NonNull<DynamicType>> {
    let ptr = match kind {
        TYPE_KIND_BOOLEAN => &BOOLEAN_TYPE,
        TYPE_KIND_INT8 => &INT8_TYPE,
        TYPE_KIND_UINT8 => &UINT8_TYPE,
        TYPE_KIND_INT16 => &INT16_TYPE,
        TYPE_KIND_UINT16 => &UINT16_TYPE,
        TYPE_KIND_INT32 => &INT32_TYPE,
        TYPE_KIND_UINT32 => &UINT32_TYPE,
        TYPE_KIND_INT64 => &INT64_TYPE,
        TYPE_KIND_UINT64 => &UINT64_TYPE,
        TYPE_KIND_FLOAT32 => &FLOAT32_TYPE,
        TYPE_KIND_FLOAT64 => &FLOAT64_TYPE,
        TYPE_KIND_CHAR8 => &CHAR8_TYPE,
        _ => return None,
    };

    Some(NonNull::from(ptr))
}

/// Creates a DynamicType for a string with the specified bound.
/// Returns a raw pointer to DynamicType on success, or NULL on failure.
///
/// # Safety
///
/// There are no special safety invariants to be observed when calling this function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_type_create_string_type(
    bound: u32,
) -> Option<NonNull<DynamicType>> {
    let builder = DynamicTypeBuilderFactory::create_string_type(bound);
    let dynamic_type = builder.build();
    NonNull::new(Box::into_raw(Box::new(DynamicType::new(
        dynamic_type,
    ))))
}

/// Frees a DynamicType object.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `dynamic_type` must point to a valid, initialized `DynamicType` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_type_free(dynamic_type: Option<NonNull<DynamicType>>) {
    if let Some(dt) = dynamic_type {
        let ptr_val = dt.as_ptr() as usize;
        let is_static_primitive = ptr_val == &BOOLEAN_TYPE as *const _ as usize
            || ptr_val == &INT8_TYPE as *const _ as usize
            || ptr_val == &UINT8_TYPE as *const _ as usize
            || ptr_val == &INT16_TYPE as *const _ as usize
            || ptr_val == &UINT16_TYPE as *const _ as usize
            || ptr_val == &INT32_TYPE as *const _ as usize
            || ptr_val == &UINT32_TYPE as *const _ as usize
            || ptr_val == &INT64_TYPE as *const _ as usize
            || ptr_val == &UINT64_TYPE as *const _ as usize
            || ptr_val == &FLOAT32_TYPE as *const _ as usize
            || ptr_val == &FLOAT64_TYPE as *const _ as usize
            || ptr_val == &CHAR8_TYPE as *const _ as usize;

        if is_static_primitive {
            return;
        }

        unsafe {
            drop(Box::from_raw(dt.as_ptr()));
        }
    }
}

/// Creates a new DynamicTypeBuilder for a structure type.
/// Returns a raw pointer to DynamicTypeBuilder on success, or NULL on failure.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `name` must be a valid pointer to a `c_char` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_type_builder_create_struct(
    name: *const std::os::raw::c_char,
) -> Option<NonNull<DynamicTypeBuilder>> {
    if name.is_null() {
        return None;
    }
    let name_str = unsafe { std::ffi::CStr::from_ptr(name) }.to_str().ok()?;
    let descriptor = RustTypeDescriptor {
        kind: TypeKind::STRUCTURE,
        name: name_str.to_string().leak(),
        base_type: None,
        discriminator_type: None,
        bound: &[],
        element_type: None,
        key_element_type: None,
        extensibility_kind: ExtensibilityKind::Final,
        is_nested: false,
    };
    let builder = DynamicTypeBuilderFactory::create_type(descriptor);
    NonNull::new(Box::into_raw(Box::new(DynamicTypeBuilder::new(
        builder,
    ))))
}

/// Adds a member described by `descriptor` to a structure being built.
///
/// Mirrors the DDS spec `add_member(in MemberDescriptor descriptor)` interface.
/// The caller retains ownership of `descriptor` and must call
/// `DDS_member_descriptor_free` when done.
///
/// Returns RETCODE_OK on success, or a standard DDS return code on failure.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `builder` must point to a valid, initialized `DynamicTypeBuilder` instance.
/// - `descriptor` must be a valid pointer to a `MemberDescriptor` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_type_builder_add_member(
    builder: Option<NonNull<DynamicTypeBuilder>>,
    descriptor: *const MemberDescriptor,
) -> ReturnCode {
    let Some(mut builder) = builder else {
        return RETCODE_BAD_PARAMETER;
    };
    if descriptor.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let desc_ref = unsafe { &*descriptor };
    if desc_ref.name.is_null() || desc_ref.r#type.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let name_str = match unsafe { std::ffi::CStr::from_ptr(desc_ref.name) }.to_str() {
        Ok(s) => s.to_string().leak() as &'static str,
        Err(_) => return RETCODE_BAD_PARAMETER,
    };
    let member_descriptor = RustMemberDescriptor {
        name: name_str,
        id: desc_ref.id,
        r#type: unsafe { &*desc_ref.r#type }.0,
        default_value: None,
        index: desc_ref.id,
        label: &[],
        try_construct_kind: TryConstructKind::UseDefault,
        is_key: desc_ref.is_key,
        is_optional: desc_ref.is_optional,
        is_must_understand: desc_ref.is_must_understand,
        is_shared: false,
        is_default_label: false,
        is_external: false,
    };
    match unsafe { builder.as_mut() }.0.add_member(member_descriptor) {
        Ok(()) => RETCODE_OK,
        Err(_) => RETCODE_ERROR,
    }
}

/// Builds the DynamicType from the builder and consumes the builder.
/// Returns a raw pointer to DynamicType on success, or NULL on failure.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `builder` must point to a valid, initialized `DynamicTypeBuilder` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_type_builder_build(
    builder: Option<NonNull<DynamicTypeBuilder>>,
) -> Option<NonNull<DynamicType>> {
    let builder = builder?;
    let builder_val = unsafe { *Box::from_raw(builder.as_ptr()) };
    let dynamic_type = builder_val.0.build();
    NonNull::new(Box::into_raw(Box::new(DynamicType::new(
        dynamic_type,
    ))))
}

/// Frees a DynamicTypeBuilder object.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `builder` must point to a valid, initialized `DynamicTypeBuilder` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_type_builder_free(
    builder: Option<NonNull<DynamicTypeBuilder>>,
) {
    if let Some(b) = builder {
        unsafe {
            drop(Box::from_raw(b.as_ptr()));
        }
    }
}

// ---------------------------------------------------------------------------
// MemberDescriptor lifecycle
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_kind_constants() {
        assert_eq!(TYPE_KIND_NONE, TypeKind::NONE as u8);
        assert_eq!(TYPE_KIND_BOOLEAN, TypeKind::BOOLEAN as u8);
        assert_eq!(TYPE_KIND_BYTE, TypeKind::BYTE as u8);
        assert_eq!(TYPE_KIND_INT16, TypeKind::INT16 as u8);
        assert_eq!(TYPE_KIND_INT32, TypeKind::INT32 as u8);
        assert_eq!(TYPE_KIND_INT64, TypeKind::INT64 as u8);
        assert_eq!(TYPE_KIND_UINT16, TypeKind::UINT16 as u8);
        assert_eq!(TYPE_KIND_UINT32, TypeKind::UINT32 as u8);
        assert_eq!(TYPE_KIND_UINT64, TypeKind::UINT64 as u8);
        assert_eq!(TYPE_KIND_FLOAT32, TypeKind::FLOAT32 as u8);
        assert_eq!(TYPE_KIND_FLOAT64, TypeKind::FLOAT64 as u8);
        assert_eq!(TYPE_KIND_FLOAT128, TypeKind::FLOAT128 as u8);
        assert_eq!(TYPE_KIND_INT8, TypeKind::INT8 as u8);
        assert_eq!(TYPE_KIND_UINT8, TypeKind::UINT8 as u8);
        assert_eq!(TYPE_KIND_CHAR8, TypeKind::CHAR8 as u8);
        assert_eq!(TYPE_KIND_CHAR16, TypeKind::CHAR16 as u8);
        assert_eq!(TYPE_KIND_STRING8, TypeKind::STRING8 as u8);
        assert_eq!(TYPE_KIND_STRING16, TypeKind::STRING16 as u8);
        assert_eq!(TYPE_KIND_ALIAS, TypeKind::ALIAS as u8);
        assert_eq!(TYPE_KIND_ENUM, TypeKind::ENUM as u8);
        assert_eq!(TYPE_KIND_BITMASK, TypeKind::BITMASK as u8);
        assert_eq!(TYPE_KIND_ANNOTATION, TypeKind::ANNOTATION as u8);
        assert_eq!(TYPE_KIND_STRUCTURE, TypeKind::STRUCTURE as u8);
        assert_eq!(TYPE_KIND_UNION, TypeKind::UNION as u8);
        assert_eq!(TYPE_KIND_BITSET, TypeKind::BITSET as u8);
        assert_eq!(TYPE_KIND_SEQUENCE, TypeKind::SEQUENCE as u8);
        assert_eq!(TYPE_KIND_ARRAY, TypeKind::ARRAY as u8);
        assert_eq!(TYPE_KIND_MAP, TypeKind::MAP as u8);
    }
}
