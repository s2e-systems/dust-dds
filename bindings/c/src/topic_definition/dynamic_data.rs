use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_ERROR, RETCODE_OK, ReturnCode};
use crate::topic_definition::dynamic_type::DustDdsDynamicType;
use dust_dds::xtypes::dynamic_type::{DynamicData, DynamicDataFactory};
use std::ptr::NonNull;

/// cbindgen:opaque
pub struct DustDdsDynamicData(pub(crate) DynamicData<'static>);

impl DustDdsDynamicData {
    pub fn new(d: DynamicData<'static>) -> Self {
        Self(d)
    }

    pub fn inner(&self) -> &DynamicData<'static> {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut DynamicData<'static> {
        &mut self.0
    }
}

/// Creates a new DynamicData instance for a given DynamicType.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `r#type` must be a valid pointer to a `DustDdsDynamicType` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_create(
    r#type: *const DustDdsDynamicType,
) -> Option<NonNull<DustDdsDynamicData>> {
    if r#type.is_null() {
        return None;
    }
    let dynamic_data = DynamicDataFactory::create_data(*unsafe { &*r#type }.inner());
    NonNull::new(Box::into_raw(Box::new(DustDdsDynamicData::new(
        dynamic_data,
    ))))
}

/// Frees a DynamicData instance.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_free(data: Option<NonNull<DustDdsDynamicData>>) {
    if let Some(d) = data {
        unsafe {
            drop(Box::from_raw(d.as_ptr()));
        }
    }
}

/// Frees a string allocated by the Rust bindings.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `ptr` must be a valid pointer to a `c_char` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_string_free(ptr: *mut std::os::raw::c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(std::ffi::CString::from_raw(ptr));
        }
    }
}

// Explicit primitive getters
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `value` must be a valid pointer to a `bool` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_get_boolean_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: *mut bool,
) -> ReturnCode {
    let Some(data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    if value.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { data.as_ref() }.inner().get_boolean_value(id) {
        Ok(val) => {
            unsafe { *value = *val };
            RETCODE_OK
        }
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `value` must be a valid pointer to a `i8` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_get_int8_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: *mut i8,
) -> ReturnCode {
    let Some(data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    if value.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { data.as_ref() }.inner().get_int8_value(id) {
        Ok(val) => {
            unsafe { *value = *val };
            RETCODE_OK
        }
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `value` must be a valid pointer to a `u8` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_get_uint8_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: *mut u8,
) -> ReturnCode {
    let Some(data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    if value.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { data.as_ref() }.inner().get_uint8_value(id) {
        Ok(val) => {
            unsafe { *value = *val };
            RETCODE_OK
        }
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `value` must be a valid pointer to a `i16` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_get_int16_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: *mut i16,
) -> ReturnCode {
    let Some(data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    if value.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { data.as_ref() }.inner().get_int16_value(id) {
        Ok(val) => {
            unsafe { *value = *val };
            RETCODE_OK
        }
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `value` must be a valid pointer to a `u16` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_get_uint16_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: *mut u16,
) -> ReturnCode {
    let Some(data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    if value.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { data.as_ref() }.inner().get_uint16_value(id) {
        Ok(val) => {
            unsafe { *value = *val };
            RETCODE_OK
        }
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `value` must be a valid pointer to a `i32` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_get_int32_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: *mut i32,
) -> ReturnCode {
    let Some(data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    if value.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { data.as_ref() }.inner().get_int32_value(id) {
        Ok(val) => {
            unsafe { *value = *val };
            RETCODE_OK
        }
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `value` must be a valid pointer to a `u32` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_get_uint32_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: *mut u32,
) -> ReturnCode {
    let Some(data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    if value.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { data.as_ref() }.inner().get_uint32_value(id) {
        Ok(val) => {
            unsafe { *value = *val };
            RETCODE_OK
        }
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `value` must be a valid pointer to a `i64` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_get_int64_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: *mut i64,
) -> ReturnCode {
    let Some(data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    if value.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { data.as_ref() }.inner().get_int64_value(id) {
        Ok(val) => {
            unsafe { *value = *val };
            RETCODE_OK
        }
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `value` must be a valid pointer to a `u64` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_get_uint64_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: *mut u64,
) -> ReturnCode {
    let Some(data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    if value.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { data.as_ref() }.inner().get_uint64_value(id) {
        Ok(val) => {
            unsafe { *value = *val };
            RETCODE_OK
        }
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `value` must be a valid pointer to a `f32` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_get_float32_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: *mut f32,
) -> ReturnCode {
    let Some(data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    if value.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { data.as_ref() }.inner().get_float32_value(id) {
        Ok(val) => {
            unsafe { *value = *val };
            RETCODE_OK
        }
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `value` must be a valid pointer to a `f64` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_get_float64_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: *mut f64,
) -> ReturnCode {
    let Some(data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    if value.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { data.as_ref() }.inner().get_float64_value(id) {
        Ok(val) => {
            unsafe { *value = *val };
            RETCODE_OK
        }
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `value` must be a valid pointer to a `c_char` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_get_char8_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: *mut std::os::raw::c_char,
) -> ReturnCode {
    let Some(data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    if value.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { data.as_ref() }.inner().get_char8_value(id) {
        Ok(val) => {
            unsafe { *value = *val as u8 as std::os::raw::c_char };
            RETCODE_OK
        }
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `value` must be a valid pointer to a `c_char` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_get_string_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: *mut *mut std::os::raw::c_char,
) -> ReturnCode {
    let Some(data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    if value.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { data.as_ref() }.inner().get_string_value(id) {
        Ok(s) => {
            let c_string = match std::ffi::CString::new(s.as_str()) {
                Ok(cs) => cs,
                Err(_) => return RETCODE_ERROR,
            };
            unsafe { *value = c_string.into_raw() };
            RETCODE_OK
        }
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `value` must be a valid pointer to a `DustDdsDynamicData` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_get_complex_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: *mut *mut DustDdsDynamicData,
) -> ReturnCode {
    let Some(data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    if value.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { data.as_ref() }.inner().get_complex_value(id) {
        Ok(complex_data) => {
            let owned_complex = DustDdsDynamicData::new(complex_data.clone());
            unsafe { *value = Box::into_raw(Box::new(owned_complex)) };
            RETCODE_OK
        }
        Err(_) => RETCODE_ERROR,
    }
}

// Explicit primitive setters
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_set_boolean_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: bool,
) -> ReturnCode {
    let Some(mut data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { data.as_mut() }
        .inner_mut()
        .set_boolean_value(id, value)
    {
        Ok(()) => RETCODE_OK,
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_set_int8_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: i8,
) -> ReturnCode {
    let Some(mut data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { data.as_mut() }
        .inner_mut()
        .set_int8_value(id, value)
    {
        Ok(()) => RETCODE_OK,
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_set_uint8_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: u8,
) -> ReturnCode {
    let Some(mut data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { data.as_mut() }
        .inner_mut()
        .set_uint8_value(id, value)
    {
        Ok(()) => RETCODE_OK,
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_set_int16_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: i16,
) -> ReturnCode {
    let Some(mut data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { data.as_mut() }
        .inner_mut()
        .set_int16_value(id, value)
    {
        Ok(()) => RETCODE_OK,
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_set_uint16_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: u16,
) -> ReturnCode {
    let Some(mut data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { data.as_mut() }
        .inner_mut()
        .set_uint16_value(id, value)
    {
        Ok(()) => RETCODE_OK,
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_set_int32_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: i32,
) -> ReturnCode {
    let Some(mut data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { data.as_mut() }
        .inner_mut()
        .set_int32_value(id, value)
    {
        Ok(()) => RETCODE_OK,
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_set_uint32_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: u32,
) -> ReturnCode {
    let Some(mut data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { data.as_mut() }
        .inner_mut()
        .set_uint32_value(id, value)
    {
        Ok(()) => RETCODE_OK,
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_set_int64_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: i64,
) -> ReturnCode {
    let Some(mut data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { data.as_mut() }
        .inner_mut()
        .set_int64_value(id, value)
    {
        Ok(()) => RETCODE_OK,
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_set_uint64_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: u64,
) -> ReturnCode {
    let Some(mut data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { data.as_mut() }
        .inner_mut()
        .set_uint64_value(id, value)
    {
        Ok(()) => RETCODE_OK,
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_set_float32_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: f32,
) -> ReturnCode {
    let Some(mut data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { data.as_mut() }
        .inner_mut()
        .set_float32_value(id, value)
    {
        Ok(()) => RETCODE_OK,
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_set_float64_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: f64,
) -> ReturnCode {
    let Some(mut data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { data.as_mut() }
        .inner_mut()
        .set_float64_value(id, value)
    {
        Ok(()) => RETCODE_OK,
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_set_char8_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: std::os::raw::c_char,
) -> ReturnCode {
    let Some(mut data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    let r_char = value as u8 as char;
    match unsafe { data.as_mut() }
        .inner_mut()
        .set_char8_value(id, r_char)
    {
        Ok(()) => RETCODE_OK,
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `value` must be a valid pointer to a `c_char` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_set_string_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: *const std::os::raw::c_char,
) -> ReturnCode {
    let Some(mut data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    if value.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let c_str = unsafe { std::ffi::CStr::from_ptr(value) };
    let string_val = match c_str.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return RETCODE_BAD_PARAMETER,
    };
    match unsafe { data.as_mut() }
        .inner_mut()
        .set_string_value(id, string_val)
    {
        Ok(()) => RETCODE_OK,
        Err(_) => RETCODE_ERROR,
    }
}
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `value` must point to a valid, initialized `DustDdsDynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_set_complex_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: Option<NonNull<DustDdsDynamicData>>,
) -> ReturnCode {
    let Some(mut data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    let Some(value) = value else {
        return RETCODE_BAD_PARAMETER;
    };
    let complex_val = unsafe { value.as_ref() }.inner().clone();
    match unsafe { data.as_mut() }
        .inner_mut()
        .set_complex_value(id, complex_val)
    {
        Ok(()) => RETCODE_OK,
        Err(_) => RETCODE_ERROR,
    }
}

/// Writes data using the generic DustDdsDataWriter.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `writer` must point to a valid, initialized `DustDdsDataWriter` instance.
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datawriter_write(
    writer: Option<NonNull<crate::publication::data_writer::DustDdsDataWriter>>,
    data: Option<NonNull<DustDdsDynamicData>>,
    handle: *const crate::infrastructure::status::InstanceHandle_t,
) -> ReturnCode {
    let Some(writer) = writer else {
        return RETCODE_BAD_PARAMETER;
    };
    let Some(data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    let rust_handle = if handle.is_null() {
        None
    } else {
        Some(dust_dds::infrastructure::instance::InstanceHandle::new(
            unsafe { *handle },
        ))
    };
    let data_val = unsafe { data.as_ref() }.inner().clone();
    match unsafe { writer.as_ref() }
        .inner()
        .write(data_val, rust_handle)
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Registers an instance.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `writer` must point to a valid, initialized `DustDdsDataWriter` instance.
/// - `instance_data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datawriter_register_instance(
    writer: Option<NonNull<crate::publication::data_writer::DustDdsDataWriter>>,
    instance_data: Option<NonNull<DustDdsDynamicData>>,
    handle: *mut crate::infrastructure::status::InstanceHandle_t,
) -> ReturnCode {
    let Some(writer) = writer else {
        return RETCODE_BAD_PARAMETER;
    };
    let Some(instance_data) = instance_data else {
        return RETCODE_BAD_PARAMETER;
    };
    if handle.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let data_val = unsafe { instance_data.as_ref() }.inner().clone();
    match unsafe { writer.as_ref() }
        .inner()
        .register_instance(data_val)
    {
        Ok(h) => {
            unsafe {
                *handle = match h {
                    Some(handle_val) => <[u8; 16]>::from(handle_val),
                    None => [0; 16],
                };
            }
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Registers an instance with timestamp.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `writer` must point to a valid, initialized `DustDdsDataWriter` instance.
/// - `instance_data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datawriter_register_instance_w_timestamp(
    writer: Option<NonNull<crate::publication::data_writer::DustDdsDataWriter>>,
    instance_data: Option<NonNull<DustDdsDynamicData>>,
    source_timestamp: crate::infrastructure::qos_policy::Time_t,
    handle: *mut crate::infrastructure::status::InstanceHandle_t,
) -> ReturnCode {
    let Some(writer) = writer else {
        return RETCODE_BAD_PARAMETER;
    };
    let Some(instance_data) = instance_data else {
        return RETCODE_BAD_PARAMETER;
    };
    if handle.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let data_val = unsafe { instance_data.as_ref() }.inner().clone();
    match unsafe { writer.as_ref() }
        .inner()
        .register_instance_w_timestamp(data_val, source_timestamp.into())
    {
        Ok(h) => {
            unsafe {
                *handle = match h {
                    Some(handle_val) => <[u8; 16]>::from(handle_val),
                    None => [0; 16],
                };
            }
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Unregisters an instance.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `writer` must point to a valid, initialized `DustDdsDataWriter` instance.
/// - `instance_data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datawriter_unregister_instance(
    writer: Option<NonNull<crate::publication::data_writer::DustDdsDataWriter>>,
    instance_data: Option<NonNull<DustDdsDynamicData>>,
    handle: *const crate::infrastructure::status::InstanceHandle_t,
) -> ReturnCode {
    let Some(writer) = writer else {
        return RETCODE_BAD_PARAMETER;
    };
    let Some(instance_data) = instance_data else {
        return RETCODE_BAD_PARAMETER;
    };
    let rust_handle = if handle.is_null() {
        None
    } else {
        Some(dust_dds::infrastructure::instance::InstanceHandle::new(
            unsafe { *handle },
        ))
    };
    let data_val = unsafe { instance_data.as_ref() }.inner().clone();
    match unsafe { writer.as_ref() }
        .inner()
        .unregister_instance(data_val, rust_handle)
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Unregisters an instance with timestamp.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `writer` must point to a valid, initialized `DustDdsDataWriter` instance.
/// - `instance_data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datawriter_unregister_instance_w_timestamp(
    writer: Option<NonNull<crate::publication::data_writer::DustDdsDataWriter>>,
    instance_data: Option<NonNull<DustDdsDynamicData>>,
    handle: *const crate::infrastructure::status::InstanceHandle_t,
    source_timestamp: crate::infrastructure::qos_policy::Time_t,
) -> ReturnCode {
    let Some(writer) = writer else {
        return RETCODE_BAD_PARAMETER;
    };
    let Some(instance_data) = instance_data else {
        return RETCODE_BAD_PARAMETER;
    };
    let rust_handle = if handle.is_null() {
        None
    } else {
        Some(dust_dds::infrastructure::instance::InstanceHandle::new(
            unsafe { *handle },
        ))
    };
    let data_val = unsafe { instance_data.as_ref() }.inner().clone();
    match unsafe { writer.as_ref() }
        .inner()
        .unregister_instance_w_timestamp(data_val, rust_handle, source_timestamp.into())
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Writes data with timestamp using the generic DustDdsDataWriter.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `writer` must point to a valid, initialized `DustDdsDataWriter` instance.
/// - `data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datawriter_write_w_timestamp(
    writer: Option<NonNull<crate::publication::data_writer::DustDdsDataWriter>>,
    data: Option<NonNull<DustDdsDynamicData>>,
    handle: *const crate::infrastructure::status::InstanceHandle_t,
    source_timestamp: crate::infrastructure::qos_policy::Time_t,
) -> ReturnCode {
    let Some(writer) = writer else {
        return RETCODE_BAD_PARAMETER;
    };
    let Some(data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    let rust_handle = if handle.is_null() {
        None
    } else {
        Some(dust_dds::infrastructure::instance::InstanceHandle::new(
            unsafe { *handle },
        ))
    };
    let data_val = unsafe { data.as_ref() }.inner().clone();
    match unsafe { writer.as_ref() }.inner().write_w_timestamp(
        data_val,
        rust_handle,
        source_timestamp.into(),
    ) {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Disposes an instance.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `writer` must point to a valid, initialized `DustDdsDataWriter` instance.
/// - `instance_data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datawriter_dispose(
    writer: Option<NonNull<crate::publication::data_writer::DustDdsDataWriter>>,
    instance_data: Option<NonNull<DustDdsDynamicData>>,
    handle: *const crate::infrastructure::status::InstanceHandle_t,
) -> ReturnCode {
    let Some(writer) = writer else {
        return RETCODE_BAD_PARAMETER;
    };
    let Some(instance_data) = instance_data else {
        return RETCODE_BAD_PARAMETER;
    };
    let rust_handle = if handle.is_null() {
        None
    } else {
        Some(dust_dds::infrastructure::instance::InstanceHandle::new(
            unsafe { *handle },
        ))
    };
    let data_val = unsafe { instance_data.as_ref() }.inner().clone();
    match unsafe { writer.as_ref() }
        .inner()
        .dispose(data_val, rust_handle)
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Disposes an instance with timestamp.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `writer` must point to a valid, initialized `DustDdsDataWriter` instance.
/// - `instance_data` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datawriter_dispose_w_timestamp(
    writer: Option<NonNull<crate::publication::data_writer::DustDdsDataWriter>>,
    instance_data: Option<NonNull<DustDdsDynamicData>>,
    handle: *const crate::infrastructure::status::InstanceHandle_t,
    source_timestamp: crate::infrastructure::qos_policy::Time_t,
) -> ReturnCode {
    let Some(writer) = writer else {
        return RETCODE_BAD_PARAMETER;
    };
    let Some(instance_data) = instance_data else {
        return RETCODE_BAD_PARAMETER;
    };
    let rust_handle = if handle.is_null() {
        None
    } else {
        Some(dust_dds::infrastructure::instance::InstanceHandle::new(
            unsafe { *handle },
        ))
    };
    let data_val = unsafe { instance_data.as_ref() }.inner().clone();
    match unsafe { writer.as_ref() }.inner().dispose_w_timestamp(
        data_val,
        rust_handle,
        source_timestamp.into(),
    ) {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Retrieves the instance key value.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `writer` must point to a valid, initialized `DustDdsDataWriter` instance.
/// - `key_holder` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datawriter_get_key_value(
    writer: Option<NonNull<crate::publication::data_writer::DustDdsDataWriter>>,
    key_holder: Option<NonNull<DustDdsDynamicData>>,
    handle: *const crate::infrastructure::status::InstanceHandle_t,
) -> ReturnCode {
    let Some(writer) = writer else {
        return RETCODE_BAD_PARAMETER;
    };
    let Some(mut key_holder) = key_holder else {
        return RETCODE_BAD_PARAMETER;
    };
    if handle.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let rust_handle = dust_dds::infrastructure::instance::InstanceHandle::new(unsafe { *handle });
    match unsafe { writer.as_ref() }
        .inner()
        .get_key_value(unsafe { key_holder.as_mut() }.inner_mut(), rust_handle)
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Looks up the handle of an instance.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `writer` must point to a valid, initialized `DustDdsDataWriter` instance.
/// - `key_holder` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datawriter_lookup_instance(
    writer: Option<NonNull<crate::publication::data_writer::DustDdsDataWriter>>,
    key_holder: Option<NonNull<DustDdsDynamicData>>,
    handle: *mut crate::infrastructure::status::InstanceHandle_t,
) -> ReturnCode {
    let Some(writer) = writer else {
        return RETCODE_BAD_PARAMETER;
    };
    let Some(key_holder) = key_holder else {
        return RETCODE_BAD_PARAMETER;
    };
    if handle.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let data_val = unsafe { key_holder.as_ref() }.inner().clone();
    match unsafe { writer.as_ref() }.inner().lookup_instance(data_val) {
        Ok(h) => {
            unsafe {
                *handle = match h {
                    Some(handle_val) => <[u8; 16]>::from(handle_val),
                    None => [0; 16],
                };
            }
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

use crate::subscription::data_reader::{
    InstanceStateMask, SampleInfo, SampleStateMask, ViewStateMask, instance_states_from_mask,
    sample_states_from_mask, view_states_from_mask,
};

/// Reads data using the generic DustDdsDataReader.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `reader` must point to a valid, initialized `DustDdsDataReader` instance.
/// - `data_values` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `sample_infos` must be a valid pointer to a `SampleInfo` instance for writing (or null).
/// - `received_samples` must be a valid pointer to a `i32` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datareader_read(
    reader: Option<NonNull<crate::subscription::data_reader::DustDdsDataReader>>,
    data_values: *mut Option<NonNull<DustDdsDynamicData>>,
    sample_infos: *mut SampleInfo,
    max_samples: i32,
    sample_states: SampleStateMask,
    view_states: ViewStateMask,
    instance_states: InstanceStateMask,
    received_samples: *mut i32,
) -> ReturnCode {
    let Some(reader) = reader else {
        return RETCODE_BAD_PARAMETER;
    };
    if data_values.is_null() || received_samples.is_null() {
        return RETCODE_BAD_PARAMETER;
    }

    let reader_ref = unsafe { reader.as_ref() };
    let sample_states_vec = sample_states_from_mask(sample_states);
    let view_states_vec = view_states_from_mask(view_states);
    let instance_states_vec = instance_states_from_mask(instance_states);

    match reader_ref.inner().read(
        max_samples,
        &sample_states_vec,
        &view_states_vec,
        &instance_states_vec,
    ) {
        Ok(samples) => {
            let count = samples.len() as i32;
            unsafe {
                *received_samples = count;
                for (i, sample) in samples.into_iter().enumerate() {
                    if let Some(dynamic_data) = sample.data {
                        let wrapper = DustDdsDynamicData::new(dynamic_data);
                        let ptr = NonNull::new(Box::into_raw(Box::new(wrapper)));
                        *data_values.add(i) = ptr;
                    } else {
                        *data_values.add(i) = None;
                    }
                    if !sample_infos.is_null() {
                        *sample_infos.add(i) = sample.sample_info.into();
                    }
                }
            }
            RETCODE_OK
        }
        Err(e) => {
            if let dust_dds::infrastructure::error::DdsError::NoData = e {
                unsafe { *received_samples = 0 };
            }
            e.into()
        }
    }
}

/// Takes data using the generic DustDdsDataReader.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `reader` must point to a valid, initialized `DustDdsDataReader` instance.
/// - `data_values` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `sample_infos` must be a valid pointer to a `SampleInfo` instance for writing (or null).
/// - `received_samples` must be a valid pointer to a `i32` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datareader_take(
    reader: Option<NonNull<crate::subscription::data_reader::DustDdsDataReader>>,
    data_values: *mut Option<NonNull<DustDdsDynamicData>>,
    sample_infos: *mut SampleInfo,
    max_samples: i32,
    sample_states: SampleStateMask,
    view_states: ViewStateMask,
    instance_states: InstanceStateMask,
    received_samples: *mut i32,
) -> ReturnCode {
    let Some(reader) = reader else {
        return RETCODE_BAD_PARAMETER;
    };
    if data_values.is_null() || received_samples.is_null() {
        return RETCODE_BAD_PARAMETER;
    }

    let reader_ref = unsafe { reader.as_ref() };
    let sample_states_vec = sample_states_from_mask(sample_states);
    let view_states_vec = view_states_from_mask(view_states);
    let instance_states_vec = instance_states_from_mask(instance_states);

    match reader_ref.inner().take(
        max_samples,
        &sample_states_vec,
        &view_states_vec,
        &instance_states_vec,
    ) {
        Ok(samples) => {
            let count = samples.len() as i32;
            unsafe {
                *received_samples = count;
                for (i, sample) in samples.into_iter().enumerate() {
                    if let Some(dynamic_data) = sample.data {
                        let wrapper = DustDdsDynamicData::new(dynamic_data);
                        let ptr = NonNull::new(Box::into_raw(Box::new(wrapper)));
                        *data_values.add(i) = ptr;
                    } else {
                        *data_values.add(i) = None;
                    }
                    if !sample_infos.is_null() {
                        *sample_infos.add(i) = sample.sample_info.into();
                    }
                }
            }
            RETCODE_OK
        }
        Err(e) => {
            if let dust_dds::infrastructure::error::DdsError::NoData = e {
                unsafe { *received_samples = 0 };
            }
            e.into()
        }
    }
}

/// Reads the next sample.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `reader` must point to a valid, initialized `DustDdsDataReader` instance.
/// - `data_value` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `sample_info` must be a valid pointer to a `SampleInfo` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datareader_read_next_sample(
    reader: Option<NonNull<crate::subscription::data_reader::DustDdsDataReader>>,
    data_value: *mut Option<NonNull<DustDdsDynamicData>>,
    sample_info: *mut SampleInfo,
) -> ReturnCode {
    let Some(reader) = reader else {
        return RETCODE_BAD_PARAMETER;
    };
    if data_value.is_null() || sample_info.is_null() {
        return RETCODE_BAD_PARAMETER;
    }

    let reader_ref = unsafe { reader.as_ref() };
    match reader_ref.inner().read_next_sample() {
        Ok(sample) => {
            unsafe {
                if let Some(dynamic_data) = sample.data {
                    let wrapper = DustDdsDynamicData::new(dynamic_data);
                    let ptr = NonNull::new(Box::into_raw(Box::new(wrapper)));
                    *data_value = ptr;
                } else {
                    *data_value = None;
                }
                *sample_info = sample.sample_info.into();
            }
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Takes the next sample.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `reader` must point to a valid, initialized `DustDdsDataReader` instance.
/// - `data_value` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `sample_info` must be a valid pointer to a `SampleInfo` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datareader_take_next_sample(
    reader: Option<NonNull<crate::subscription::data_reader::DustDdsDataReader>>,
    data_value: *mut Option<NonNull<DustDdsDynamicData>>,
    sample_info: *mut SampleInfo,
) -> ReturnCode {
    let Some(reader) = reader else {
        return RETCODE_BAD_PARAMETER;
    };
    if data_value.is_null() || sample_info.is_null() {
        return RETCODE_BAD_PARAMETER;
    }

    let reader_ref = unsafe { reader.as_ref() };
    match reader_ref.inner().take_next_sample() {
        Ok(sample) => {
            unsafe {
                if let Some(dynamic_data) = sample.data {
                    let wrapper = DustDdsDynamicData::new(dynamic_data);
                    let ptr = NonNull::new(Box::into_raw(Box::new(wrapper)));
                    *data_value = ptr;
                } else {
                    *data_value = None;
                }
                *sample_info = sample.sample_info.into();
            }
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Reads a specific instance.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `reader` must point to a valid, initialized `DustDdsDataReader` instance.
/// - `data_values` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `sample_infos` must be a valid pointer to a `SampleInfo` instance for writing (or null).
/// - `a_handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
/// - `received_samples` must be a valid pointer to a `i32` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datareader_read_instance(
    reader: Option<NonNull<crate::subscription::data_reader::DustDdsDataReader>>,
    data_values: *mut Option<NonNull<DustDdsDynamicData>>,
    sample_infos: *mut SampleInfo,
    max_samples: i32,
    a_handle: *const crate::infrastructure::status::InstanceHandle_t,
    sample_states: SampleStateMask,
    view_states: ViewStateMask,
    instance_states: InstanceStateMask,
    received_samples: *mut i32,
) -> ReturnCode {
    let Some(reader) = reader else {
        return RETCODE_BAD_PARAMETER;
    };
    if data_values.is_null() || received_samples.is_null() || a_handle.is_null() {
        return RETCODE_BAD_PARAMETER;
    }

    let reader_ref = unsafe { reader.as_ref() };
    let rust_handle = dust_dds::infrastructure::instance::InstanceHandle::new(unsafe { *a_handle });
    let sample_states_vec = sample_states_from_mask(sample_states);
    let view_states_vec = view_states_from_mask(view_states);
    let instance_states_vec = instance_states_from_mask(instance_states);

    match reader_ref.inner().read_instance(
        max_samples,
        rust_handle,
        &sample_states_vec,
        &view_states_vec,
        &instance_states_vec,
    ) {
        Ok(samples) => {
            let count = samples.len() as i32;
            unsafe {
                *received_samples = count;
                for (i, sample) in samples.into_iter().enumerate() {
                    if let Some(dynamic_data) = sample.data {
                        let wrapper = DustDdsDynamicData::new(dynamic_data);
                        let ptr = NonNull::new(Box::into_raw(Box::new(wrapper)));
                        *data_values.add(i) = ptr;
                    } else {
                        *data_values.add(i) = None;
                    }
                    if !sample_infos.is_null() {
                        *sample_infos.add(i) = sample.sample_info.into();
                    }
                }
            }
            RETCODE_OK
        }
        Err(e) => {
            if let dust_dds::infrastructure::error::DdsError::NoData = e {
                unsafe { *received_samples = 0 };
            }
            e.into()
        }
    }
}

/// Takes a specific instance.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `reader` must point to a valid, initialized `DustDdsDataReader` instance.
/// - `data_values` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `sample_infos` must be a valid pointer to a `SampleInfo` instance for writing (or null).
/// - `a_handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
/// - `received_samples` must be a valid pointer to a `i32` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datareader_take_instance(
    reader: Option<NonNull<crate::subscription::data_reader::DustDdsDataReader>>,
    data_values: *mut Option<NonNull<DustDdsDynamicData>>,
    sample_infos: *mut SampleInfo,
    max_samples: i32,
    a_handle: *const crate::infrastructure::status::InstanceHandle_t,
    sample_states: SampleStateMask,
    view_states: ViewStateMask,
    instance_states: InstanceStateMask,
    received_samples: *mut i32,
) -> ReturnCode {
    let Some(reader) = reader else {
        return RETCODE_BAD_PARAMETER;
    };
    if data_values.is_null() || received_samples.is_null() || a_handle.is_null() {
        return RETCODE_BAD_PARAMETER;
    }

    let reader_ref = unsafe { reader.as_ref() };
    let rust_handle = dust_dds::infrastructure::instance::InstanceHandle::new(unsafe { *a_handle });
    let sample_states_vec = sample_states_from_mask(sample_states);
    let view_states_vec = view_states_from_mask(view_states);
    let instance_states_vec = instance_states_from_mask(instance_states);

    match reader_ref.inner().take_instance(
        max_samples,
        rust_handle,
        &sample_states_vec,
        &view_states_vec,
        &instance_states_vec,
    ) {
        Ok(samples) => {
            let count = samples.len() as i32;
            unsafe {
                *received_samples = count;
                for (i, sample) in samples.into_iter().enumerate() {
                    if let Some(dynamic_data) = sample.data {
                        let wrapper = DustDdsDynamicData::new(dynamic_data);
                        let ptr = NonNull::new(Box::into_raw(Box::new(wrapper)));
                        *data_values.add(i) = ptr;
                    } else {
                        *data_values.add(i) = None;
                    }
                    if !sample_infos.is_null() {
                        *sample_infos.add(i) = sample.sample_info.into();
                    }
                }
            }
            RETCODE_OK
        }
        Err(e) => {
            if let dust_dds::infrastructure::error::DdsError::NoData = e {
                unsafe { *received_samples = 0 };
            }
            e.into()
        }
    }
}

/// Reads the next instance.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `reader` must point to a valid, initialized `DustDdsDataReader` instance.
/// - `data_values` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `sample_infos` must be a valid pointer to a `SampleInfo` instance for writing (or null).
/// - `previous_handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
/// - `received_samples` must be a valid pointer to a `i32` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datareader_read_next_instance(
    reader: Option<NonNull<crate::subscription::data_reader::DustDdsDataReader>>,
    data_values: *mut Option<NonNull<DustDdsDynamicData>>,
    sample_infos: *mut SampleInfo,
    max_samples: i32,
    previous_handle: *const crate::infrastructure::status::InstanceHandle_t,
    sample_states: SampleStateMask,
    view_states: ViewStateMask,
    instance_states: InstanceStateMask,
    received_samples: *mut i32,
) -> ReturnCode {
    let Some(reader) = reader else {
        return RETCODE_BAD_PARAMETER;
    };
    if data_values.is_null() || received_samples.is_null() {
        return RETCODE_BAD_PARAMETER;
    }

    let reader_ref = unsafe { reader.as_ref() };
    let rust_handle = if previous_handle.is_null() {
        None
    } else {
        let handle_val = unsafe { *previous_handle };
        if handle_val == [0; 16] {
            None
        } else {
            Some(dust_dds::infrastructure::instance::InstanceHandle::new(
                handle_val,
            ))
        }
    };
    let sample_states_vec = sample_states_from_mask(sample_states);
    let view_states_vec = view_states_from_mask(view_states);
    let instance_states_vec = instance_states_from_mask(instance_states);

    match reader_ref.inner().read_next_instance(
        max_samples,
        rust_handle,
        &sample_states_vec,
        &view_states_vec,
        &instance_states_vec,
    ) {
        Ok(samples) => {
            let count = samples.len() as i32;
            unsafe {
                *received_samples = count;
                for (i, sample) in samples.into_iter().enumerate() {
                    if let Some(dynamic_data) = sample.data {
                        let wrapper = DustDdsDynamicData::new(dynamic_data);
                        let ptr = NonNull::new(Box::into_raw(Box::new(wrapper)));
                        *data_values.add(i) = ptr;
                    } else {
                        *data_values.add(i) = None;
                    }
                    if !sample_infos.is_null() {
                        *sample_infos.add(i) = sample.sample_info.into();
                    }
                }
            }
            RETCODE_OK
        }
        Err(e) => {
            if let dust_dds::infrastructure::error::DdsError::NoData = e {
                unsafe { *received_samples = 0 };
            }
            e.into()
        }
    }
}

/// Takes the next instance.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `reader` must point to a valid, initialized `DustDdsDataReader` instance.
/// - `data_values` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `sample_infos` must be a valid pointer to a `SampleInfo` instance for writing (or null).
/// - `previous_handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
/// - `received_samples` must be a valid pointer to a `i32` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datareader_take_next_instance(
    reader: Option<NonNull<crate::subscription::data_reader::DustDdsDataReader>>,
    data_values: *mut Option<NonNull<DustDdsDynamicData>>,
    sample_infos: *mut SampleInfo,
    max_samples: i32,
    previous_handle: *const crate::infrastructure::status::InstanceHandle_t,
    sample_states: SampleStateMask,
    view_states: ViewStateMask,
    instance_states: InstanceStateMask,
    received_samples: *mut i32,
) -> ReturnCode {
    let Some(reader) = reader else {
        return RETCODE_BAD_PARAMETER;
    };
    if data_values.is_null() || received_samples.is_null() {
        return RETCODE_BAD_PARAMETER;
    }

    let reader_ref = unsafe { reader.as_ref() };
    let rust_handle = if previous_handle.is_null() {
        None
    } else {
        let handle_val = unsafe { *previous_handle };
        if handle_val == [0; 16] {
            None
        } else {
            Some(dust_dds::infrastructure::instance::InstanceHandle::new(
                handle_val,
            ))
        }
    };
    let sample_states_vec = sample_states_from_mask(sample_states);
    let view_states_vec = view_states_from_mask(view_states);
    let instance_states_vec = instance_states_from_mask(instance_states);

    match reader_ref.inner().take_next_instance(
        max_samples,
        rust_handle,
        &sample_states_vec,
        &view_states_vec,
        &instance_states_vec,
    ) {
        Ok(samples) => {
            let count = samples.len() as i32;
            unsafe {
                *received_samples = count;
                for (i, sample) in samples.into_iter().enumerate() {
                    if let Some(dynamic_data) = sample.data {
                        let wrapper = DustDdsDynamicData::new(dynamic_data);
                        let ptr = NonNull::new(Box::into_raw(Box::new(wrapper)));
                        *data_values.add(i) = ptr;
                    } else {
                        *data_values.add(i) = None;
                    }
                    if !sample_infos.is_null() {
                        *sample_infos.add(i) = sample.sample_info.into();
                    }
                }
            }
            RETCODE_OK
        }
        Err(e) => {
            if let dust_dds::infrastructure::error::DdsError::NoData = e {
                unsafe { *received_samples = 0 };
            }
            e.into()
        }
    }
}

/// Returns the loan of the sample and info collections.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `_reader` must point to a valid, initialized `DustDdsDataReader` instance.
/// - `_data_values` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `_sample_infos` must be a valid pointer to a `SampleInfo` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datareader_return_loan(
    _reader: Option<NonNull<crate::subscription::data_reader::DustDdsDataReader>>,
    _data_values: *mut Option<NonNull<DustDdsDynamicData>>,
    _sample_infos: *mut SampleInfo,
) -> ReturnCode {
    RETCODE_OK
}

/// Retrieves the key value of an instance.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `reader` must point to a valid, initialized `DustDdsDataReader` instance.
/// - `key_holder` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datareader_get_key_value(
    reader: Option<NonNull<crate::subscription::data_reader::DustDdsDataReader>>,
    key_holder: Option<NonNull<DustDdsDynamicData>>,
    handle: *const crate::infrastructure::status::InstanceHandle_t,
) -> ReturnCode {
    let Some(reader) = reader else {
        return RETCODE_BAD_PARAMETER;
    };
    let Some(mut key_holder) = key_holder else {
        return RETCODE_BAD_PARAMETER;
    };
    if handle.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let rust_handle = dust_dds::infrastructure::instance::InstanceHandle::new(unsafe { *handle });
    match unsafe { reader.as_ref() }
        .inner()
        .get_key_value(unsafe { key_holder.as_mut() }.inner_mut(), rust_handle)
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Looks up the handle of an instance.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `reader` must point to a valid, initialized `DustDdsDataReader` instance.
/// - `key_holder` must point to a valid, initialized `DustDdsDynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datareader_lookup_instance(
    reader: Option<NonNull<crate::subscription::data_reader::DustDdsDataReader>>,
    key_holder: Option<NonNull<DustDdsDynamicData>>,
    handle: *mut crate::infrastructure::status::InstanceHandle_t,
) -> ReturnCode {
    let Some(reader) = reader else {
        return RETCODE_BAD_PARAMETER;
    };
    let Some(key_holder) = key_holder else {
        return RETCODE_BAD_PARAMETER;
    };
    if handle.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let data_val = unsafe { key_holder.as_ref() }.inner().clone();
    match unsafe { reader.as_ref() }
        .inner()
        .lookup_instance(&data_val)
    {
        Ok(h) => {
            unsafe {
                *handle = match h {
                    Some(handle_val) => <[u8; 16]>::from(handle_val),
                    None => [0; 16],
                };
            }
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}
