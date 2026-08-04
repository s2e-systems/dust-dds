use crate::{
    infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_ERROR, RETCODE_OK, ReturnCode},
    topic_definition::dynamic_type::DynamicType,
};
use dust_dds::xtypes::dynamic_type::{DynamicData as RustDynamicData, DynamicDataFactory};
use std::ptr::NonNull;

/// cbindgen:opaque
pub struct DynamicData(pub(crate) RustDynamicData<'static>);

impl DynamicData {
    pub fn new(d: RustDynamicData<'static>) -> Self {
        Self(d)
    }

    pub fn inner(&self) -> &RustDynamicData<'static> {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut RustDynamicData<'static> {
        &mut self.0
    }
}

/// Creates a new DynamicData instance for a given DynamicType.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `r#type` must be a valid pointer to a `DynamicType` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_create(
    r#type: *const DynamicType,
) -> Option<NonNull<DynamicData>> {
    if r#type.is_null() {
        return None;
    }
    let dynamic_data = DynamicDataFactory::create_data(*unsafe { &*r#type }.inner());
    NonNull::new(Box::into_raw(Box::new(DynamicData::new(
        dynamic_data,
    ))))
}

/// Frees a DynamicData instance.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `data` must point to a valid, initialized `DynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_free(data: Option<NonNull<DynamicData>>) {
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
pub unsafe extern "C" fn DDS_string_free(ptr: *mut std::os::raw::c_char) {
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
/// - `value` must be a valid pointer to a `bool` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_get_boolean_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
/// - `value` must be a valid pointer to a `i8` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_get_int8_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
/// - `value` must be a valid pointer to a `u8` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_get_uint8_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
/// - `value` must be a valid pointer to a `i16` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_get_int16_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
/// - `value` must be a valid pointer to a `u16` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_get_uint16_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
/// - `value` must be a valid pointer to a `i32` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_get_int32_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
/// - `value` must be a valid pointer to a `u32` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_get_uint32_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
/// - `value` must be a valid pointer to a `i64` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_get_int64_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
/// - `value` must be a valid pointer to a `u64` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_get_uint64_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
/// - `value` must be a valid pointer to a `f32` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_get_float32_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
/// - `value` must be a valid pointer to a `f64` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_get_float64_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
/// - `value` must be a valid pointer to a `c_char` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_get_char8_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
/// - `value` must be a valid pointer to a `c_char` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_get_string_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
/// - `value` must be a valid pointer to a `DynamicData` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_get_complex_value(
    data: Option<NonNull<DynamicData>>,
    id: u32,
    value: *mut *mut DynamicData,
) -> ReturnCode {
    let Some(data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    if value.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { data.as_ref() }.inner().get_complex_value(id) {
        Ok(complex_data) => {
            let owned_complex = DynamicData::new(complex_data.clone());
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_set_boolean_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_set_int8_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_set_uint8_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_set_int16_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_set_uint16_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_set_int32_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_set_uint32_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_set_int64_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_set_uint64_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_set_float32_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_set_float64_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_set_char8_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
/// - `value` must be a valid pointer to a `c_char` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_set_string_value(
    data: Option<NonNull<DynamicData>>,
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
/// - `data` must point to a valid, initialized `DynamicData` instance.
/// - `value` must point to a valid, initialized `DynamicData` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_dynamic_data_set_complex_value(
    data: Option<NonNull<DynamicData>>,
    id: u32,
    value: Option<NonNull<DynamicData>>,
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
