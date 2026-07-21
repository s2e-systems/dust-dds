use std::ptr::NonNull;
use std::sync::{Mutex, OnceLock};
use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_ERROR, RETCODE_OK, ReturnCode};
use dust_dds::xtypes::dynamic_type::{
    DynamicType, DynamicTypeBuilder, DynamicTypeBuilderFactory, ExtensibilityKind,
    MemberDescriptor, TryConstructKind, TypeDescriptor, TypeKind,
};

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

#[derive(Copy, Clone, PartialEq, Eq)]
struct SafeNonNull(NonNull<DustDdsDynamicType>);
unsafe impl Send for SafeNonNull {}
unsafe impl Sync for SafeNonNull {}

static PRIMITIVES: OnceLock<Mutex<[Option<SafeNonNull>; 256]>> = OnceLock::new();

/// Returns a DynamicType representing the specified primitive type kind.
/// Returns a raw pointer to DustDdsDynamicType on success, or NULL on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_type_get_primitive_type(
    kind: u8,
) -> Option<NonNull<DustDdsDynamicType>> {
    let type_kind = match kind {
        0x01 => TypeKind::BOOLEAN,
        0x02 => TypeKind::BYTE,
        0x03 => TypeKind::INT16,
        0x04 => TypeKind::INT32,
        0x05 => TypeKind::INT64,
        0x06 => TypeKind::UINT16,
        0x07 => TypeKind::UINT32,
        0x08 => TypeKind::UINT64,
        0x09 => TypeKind::FLOAT32,
        0x0A => TypeKind::FLOAT64,
        0x0B => TypeKind::FLOAT128,
        0x0C => TypeKind::INT8,
        0x0D => TypeKind::UINT8,
        0x10 => TypeKind::CHAR8,
        0x11 => TypeKind::CHAR16,
        0x20 => TypeKind::STRING8,
        0x21 => TypeKind::STRING16,
        0x30 => TypeKind::ALIAS,
        0x40 => TypeKind::ENUM,
        0x41 => TypeKind::BITMASK,
        0x50 => TypeKind::ANNOTATION,
        0x51 => TypeKind::STRUCTURE,
        0x52 => TypeKind::UNION,
        0x53 => TypeKind::BITSET,
        0x60 => TypeKind::SEQUENCE,
        0x61 => TypeKind::ARRAY,
        0x62 => TypeKind::MAP,
        _ => return None,
    };

    let mutex = PRIMITIVES.get_or_init(|| Mutex::new([None; 256]));
    let mut guard = mutex.lock().unwrap();
    let idx = kind as usize;
    if guard[idx].is_none() {
        let dynamic_type = DynamicTypeBuilderFactory::get_primitive_type(type_kind);
        let ptr = NonNull::new(Box::into_raw(Box::new(DustDdsDynamicType::new(dynamic_type)))).unwrap();
        guard[idx] = Some(SafeNonNull(ptr));
    }
    guard[idx].map(|s| s.0)
}

/// Creates a DynamicType for a string with the specified bound.
/// Returns a raw pointer to DustDdsDynamicType on success, or NULL on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_type_create_string_type(
    bound: u32,
) -> Option<NonNull<DustDdsDynamicType>> {
    let builder = DynamicTypeBuilderFactory::create_string_type(bound);
    let dynamic_type = builder.build();
    NonNull::new(Box::into_raw(Box::new(DustDdsDynamicType::new(dynamic_type))))
}

/// Frees a DynamicType object.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_type_free(
    dynamic_type: Option<NonNull<DustDdsDynamicType>>,
) {
    if let Some(dt) = dynamic_type {
        let is_primitive = if let Some(mutex) = PRIMITIVES.get() {
            let guard = mutex.lock().unwrap();
            guard.iter().any(|&p| p == Some(SafeNonNull(dt)))
        } else {
            false
        };

        if !is_primitive {
            unsafe {
                drop(Box::from_raw(dt.as_ptr()));
            }
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
    NonNull::new(Box::into_raw(Box::new(DustDdsDynamicTypeBuilder::new(builder))))
}

/// Adds a member to a structure being built.
/// Returns RETCODE_OK on success, or standard DDS return code on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_type_builder_add_member(
    builder: Option<NonNull<DustDdsDynamicTypeBuilder>>,
    name: *const std::os::raw::c_char,
    id: u32,
    r#type: Option<NonNull<DustDdsDynamicType>>,
) -> ReturnCode {
    let Some(mut builder) = builder else { return RETCODE_BAD_PARAMETER; };
    let Some(r#type) = r#type else { return RETCODE_BAD_PARAMETER; };
    if name.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let name_str = match unsafe { std::ffi::CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(_) => return RETCODE_BAD_PARAMETER,
    };

    let type_ref = unsafe { r#type.as_ref() };
    let builder_ref = unsafe { builder.as_mut() };

    let member_descriptor = MemberDescriptor {
        name: name_str.to_string().leak(),
        id,
        r#type: type_ref.0,
        default_value: None,
        index: id,
        label: &[],
        try_construct_kind: TryConstructKind::UseDefault,
        is_key: false,
        is_optional: false,
        is_must_understand: true,
        is_shared: false,
        is_default_label: false,
        is_external: false,
    };

    match builder_ref.0.add_member(member_descriptor) {
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
    let Some(builder) = builder else { return None; };
    let builder_val = unsafe { *Box::from_raw(builder.as_ptr()) };
    let dynamic_type = builder_val.0.build();
    NonNull::new(Box::into_raw(Box::new(DustDdsDynamicType::new(dynamic_type))))
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
