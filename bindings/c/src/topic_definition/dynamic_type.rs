use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_ERROR, RETCODE_OK, ReturnCode};
use dust_dds::xtypes::dynamic_type::{
    DynamicType, DynamicTypeBuilder, DynamicTypeBuilderFactory, ExtensibilityKind,
    MemberDescriptor, TryConstructKind, TypeDescriptor, TypeKind,
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
pub struct DustDdsDynamicType(pub(crate) DynamicType<'static>);

impl DustDdsDynamicType {
    pub fn new(t: DynamicType<'static>) -> Self {
        Self(t)
    }

    pub fn inner(&self) -> &DynamicType<'static> {
        &self.0
    }
}

/// cbindgen:opaque
pub struct DustDdsDynamicTypeBuilder(pub(crate) DynamicTypeBuilder);

impl DustDdsDynamicTypeBuilder {
    pub fn new(b: DynamicTypeBuilder) -> Self {
        Self(b)
    }

    pub fn inner(&self) -> &DynamicTypeBuilder {
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
///
/// cbindgen:opaque
pub struct DustDdsMemberDescriptor {
    pub(crate) name: &'static str,
    pub(crate) id: u32,
    pub(crate) r#type: DynamicType<'static>,
    pub(crate) is_key: bool,
    pub(crate) is_optional: bool,
    pub(crate) is_must_understand: bool,
}

// Compile-time static instances of DustDdsDynamicType for standard primitive types
use dust_dds::xtypes::type_support::Type;
static BOOLEAN_TYPE: DustDdsDynamicType = DustDdsDynamicType(bool::TYPE);
static INT8_TYPE: DustDdsDynamicType = DustDdsDynamicType(i8::TYPE);
static UINT8_TYPE: DustDdsDynamicType = DustDdsDynamicType(u8::TYPE);
static INT16_TYPE: DustDdsDynamicType = DustDdsDynamicType(i16::TYPE);
static UINT16_TYPE: DustDdsDynamicType = DustDdsDynamicType(u16::TYPE);
static INT32_TYPE: DustDdsDynamicType = DustDdsDynamicType(i32::TYPE);
static UINT32_TYPE: DustDdsDynamicType = DustDdsDynamicType(u32::TYPE);
static INT64_TYPE: DustDdsDynamicType = DustDdsDynamicType(i64::TYPE);
static UINT64_TYPE: DustDdsDynamicType = DustDdsDynamicType(u64::TYPE);
static FLOAT32_TYPE: DustDdsDynamicType = DustDdsDynamicType(f32::TYPE);
static FLOAT64_TYPE: DustDdsDynamicType = DustDdsDynamicType(f64::TYPE);
static CHAR8_TYPE: DustDdsDynamicType = DustDdsDynamicType(char::TYPE);

/// Returns a DynamicType representing the specified primitive type kind.
/// Returns a raw pointer to DustDdsDynamicType on success, or NULL on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_type_get_primitive_type(
    kind: u8,
) -> Option<NonNull<DustDdsDynamicType>> {
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
/// Returns a raw pointer to DustDdsDynamicType on success, or NULL on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_type_create_string_type(
    bound: u32,
) -> Option<NonNull<DustDdsDynamicType>> {
    let builder = DynamicTypeBuilderFactory::create_string_type(bound);
    let dynamic_type = builder.build();
    NonNull::new(Box::into_raw(Box::new(DustDdsDynamicType::new(
        dynamic_type,
    ))))
}

/// Frees a DynamicType object.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_type_free(
    dynamic_type: Option<NonNull<DustDdsDynamicType>>,
) {
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
/// Returns a raw pointer to DustDdsDynamicTypeBuilder on success, or NULL on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_type_builder_create_struct(
    name: *const std::os::raw::c_char,
) -> Option<NonNull<DustDdsDynamicTypeBuilder>> {
    if name.is_null() {
        return None;
    }
    let name_str = unsafe { std::ffi::CStr::from_ptr(name) }.to_str().ok()?;
    let descriptor = TypeDescriptor {
        kind: TypeKind::STRUCTURE,
        name: name_str.to_string().leak(),
        base_type: None,
        discriminator_type: None,
        bound: None,
        element_type: None,
        key_element_type: None,
        extensibility_kind: ExtensibilityKind::Final,
        is_nested: false,
    };
    let builder = DynamicTypeBuilderFactory::create_type(descriptor);
    NonNull::new(Box::into_raw(Box::new(DustDdsDynamicTypeBuilder::new(
        builder,
    ))))
}

/// Sets the extensibility kind on a DynamicTypeBuilder.
///
/// `kind`: one of `EXTENSIBILITY_KIND_FINAL` (0), `EXTENSIBILITY_KIND_APPENDABLE` (1),
/// or `EXTENSIBILITY_KIND_MUTABLE` (2).
///
/// Returns RETCODE_OK on success, RETCODE_BAD_PARAMETER on invalid arguments.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_type_builder_set_extensibility(
    builder: Option<NonNull<DustDdsDynamicTypeBuilder>>,
    kind: u8,
) -> ReturnCode {
    let Some(mut builder) = builder else {
        return RETCODE_BAD_PARAMETER;
    };
    let extensibility = match kind {
        EXTENSIBILITY_KIND_FINAL => ExtensibilityKind::Final,
        EXTENSIBILITY_KIND_APPENDABLE => ExtensibilityKind::Appendable,
        EXTENSIBILITY_KIND_MUTABLE => ExtensibilityKind::Mutable,
        _ => return RETCODE_BAD_PARAMETER,
    };
    unsafe { builder.as_mut() }.0.set_extensibility(extensibility);
    RETCODE_OK
}

/// Adds a member described by `descriptor` to a structure being built.
///
/// Mirrors the DDS spec `add_member(in MemberDescriptor descriptor)` interface.
/// The caller retains ownership of `descriptor` and must call
/// `dust_dds_member_descriptor_free` when done.
///
/// Returns RETCODE_OK on success, or a standard DDS return code on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_type_builder_add_member(
    builder: Option<NonNull<DustDdsDynamicTypeBuilder>>,
    descriptor: Option<NonNull<DustDdsMemberDescriptor>>,
) -> ReturnCode {
    let Some(mut builder) = builder else {
        return RETCODE_BAD_PARAMETER;
    };
    let Some(descriptor) = descriptor else {
        return RETCODE_BAD_PARAMETER;
    };
    let desc_ref = unsafe { descriptor.as_ref() };
    let member_descriptor = MemberDescriptor {
        name: desc_ref.name,
        id: desc_ref.id,
        r#type: desc_ref.r#type,
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
/// Returns a raw pointer to DustDdsDynamicType on success, or NULL on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_type_builder_build(
    builder: Option<NonNull<DustDdsDynamicTypeBuilder>>,
) -> Option<NonNull<DustDdsDynamicType>> {
    let Some(builder) = builder else {
        return None;
    };
    let builder_val = unsafe { *Box::from_raw(builder.as_ptr()) };
    let dynamic_type = builder_val.0.build();
    NonNull::new(Box::into_raw(Box::new(DustDdsDynamicType::new(
        dynamic_type,
    ))))
}

/// Frees a DynamicTypeBuilder object.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_type_builder_free(
    builder: Option<NonNull<DustDdsDynamicTypeBuilder>>,
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

/// Creates a new `DustDdsMemberDescriptor` with the given name, member ID, and type.
///
/// Defaults: `is_key = false`, `is_optional = false`, `is_must_understand = true`.
/// The `type` pointer must remain valid for the lifetime of the descriptor.
///
/// Returns NULL on failure (NULL arguments or invalid UTF-8 name).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_member_descriptor_new(
    name: *const std::os::raw::c_char,
    id: u32,
    r#type: Option<NonNull<DustDdsDynamicType>>,
) -> Option<NonNull<DustDdsMemberDescriptor>> {
    if name.is_null() {
        return None;
    }
    let Some(r#type) = r#type else {
        return None;
    };
    let name_str = match unsafe { std::ffi::CStr::from_ptr(name) }.to_str() {
        Ok(s) => s.to_string().leak() as &'static str,
        Err(_) => return None,
    };
    let desc = DustDdsMemberDescriptor {
        name: name_str,
        id,
        r#type: unsafe { r#type.as_ref() }.0,
        is_key: false,
        is_optional: false,
        is_must_understand: true,
    };
    NonNull::new(Box::into_raw(Box::new(desc)))
}

/// Sets the `is_key` flag on a `DustDdsMemberDescriptor`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_member_descriptor_set_is_key(
    descriptor: Option<NonNull<DustDdsMemberDescriptor>>,
    is_key: bool,
) {
    if let Some(mut d) = descriptor {
        unsafe { d.as_mut() }.is_key = is_key;
    }
}

/// Sets the `is_optional` flag on a `DustDdsMemberDescriptor`.
/// Setting `is_optional = true` also clears `is_must_understand`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_member_descriptor_set_is_optional(
    descriptor: Option<NonNull<DustDdsMemberDescriptor>>,
    is_optional: bool,
) {
    if let Some(mut d) = descriptor {
        let desc = unsafe { d.as_mut() };
        desc.is_optional = is_optional;
        if is_optional {
            desc.is_must_understand = false;
        }
    }
}

/// Frees a `DustDdsMemberDescriptor` created by `dust_dds_member_descriptor_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_member_descriptor_free(
    descriptor: Option<NonNull<DustDdsMemberDescriptor>>,
) {
    if let Some(d) = descriptor {
        unsafe {
            drop(Box::from_raw(d.as_ptr()));
        }
    }
}

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
