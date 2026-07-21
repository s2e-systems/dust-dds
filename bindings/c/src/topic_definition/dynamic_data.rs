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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_data_create(
    r#type: *const DustDdsDynamicType,
) -> Option<NonNull<DustDdsDynamicData>> {
    if r#type.is_null() {
        return None;
    }
    let dynamic_data = DynamicDataFactory::create_data(unsafe { &*r#type }.inner().clone());
    NonNull::new(Box::into_raw(Box::new(DustDdsDynamicData::new(dynamic_data))))
}

/// Frees a DynamicData instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_data_free(
    data: Option<NonNull<DustDdsDynamicData>>,
) {
    if let Some(d) = data {
        unsafe {
            drop(Box::from_raw(d.as_ptr()));
        }
    }
}

/// Frees a string allocated by the Rust bindings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_string_free(
    ptr: *mut std::os::raw::c_char,
) {
    if !ptr.is_null() {
        unsafe {
            drop(std::ffi::CString::from_raw(ptr));
        }
    }
}

// Macro helper to implement primitive getters
macro_rules! impl_get_value {
    ($fn_name:ident, $t:ty, $rust_fn:ident) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $fn_name(
            data: Option<NonNull<DustDdsDynamicData>>,
            id: u32,
            value: *mut $t,
        ) -> ReturnCode {
            let Some(data) = data else {
                return RETCODE_BAD_PARAMETER;
            };
            if value.is_null() {
                return RETCODE_BAD_PARAMETER;
            }
            match unsafe { data.as_ref() }.inner().$rust_fn(id) {
                Ok(val) => {
                    unsafe { *value = *val };
                    RETCODE_OK
                }
                Err(_) => RETCODE_ERROR,
            }
        }
    };
}

impl_get_value!(dust_dds_dynamic_data_get_boolean_value, bool, get_boolean_value);
impl_get_value!(dust_dds_dynamic_data_get_int8_value, i8, get_int8_value);
impl_get_value!(dust_dds_dynamic_data_get_uint8_value, u8, get_uint8_value);
impl_get_value!(dust_dds_dynamic_data_get_int16_value, i16, get_int16_value);
impl_get_value!(dust_dds_dynamic_data_get_uint16_value, u16, get_uint16_value);
impl_get_value!(dust_dds_dynamic_data_get_int32_value, i32, get_int32_value);
impl_get_value!(dust_dds_dynamic_data_get_uint32_value, u32, get_uint32_value);
impl_get_value!(dust_dds_dynamic_data_get_int64_value, i64, get_int64_value);
impl_get_value!(dust_dds_dynamic_data_get_uint64_value, u64, get_uint64_value);
impl_get_value!(dust_dds_dynamic_data_get_float32_value, f32, get_float32_value);
impl_get_value!(dust_dds_dynamic_data_get_float64_value, f64, get_float64_value);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_data_get_char8_value(
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_data_get_string_value(
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_data_get_complex_value(
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

// Macro helper to implement primitive setters
macro_rules! impl_set_value {
    ($fn_name:ident, $t:ty, $rust_fn:ident) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $fn_name(
            data: Option<NonNull<DustDdsDynamicData>>,
            id: u32,
            value: $t,
        ) -> ReturnCode {
            let Some(mut data) = data else {
                return RETCODE_BAD_PARAMETER;
            };
            match unsafe { data.as_mut() }.inner_mut().$rust_fn(id, value) {
                Ok(()) => RETCODE_OK,
                Err(_) => RETCODE_ERROR,
            }
        }
    };
}

impl_set_value!(dust_dds_dynamic_data_set_boolean_value, bool, set_boolean_value);
impl_set_value!(dust_dds_dynamic_data_set_int8_value, i8, set_int8_value);
impl_set_value!(dust_dds_dynamic_data_set_uint8_value, u8, set_uint8_value);
impl_set_value!(dust_dds_dynamic_data_set_int16_value, i16, set_int16_value);
impl_set_value!(dust_dds_dynamic_data_set_uint16_value, u16, set_uint16_value);
impl_set_value!(dust_dds_dynamic_data_set_int32_value, i32, set_int32_value);
impl_set_value!(dust_dds_dynamic_data_set_uint32_value, u32, set_uint32_value);
impl_set_value!(dust_dds_dynamic_data_set_int64_value, i64, set_int64_value);
impl_set_value!(dust_dds_dynamic_data_set_uint64_value, u64, set_uint64_value);
impl_set_value!(dust_dds_dynamic_data_set_float32_value, f32, set_float32_value);
impl_set_value!(dust_dds_dynamic_data_set_float64_value, f64, set_float64_value);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_data_set_char8_value(
    data: Option<NonNull<DustDdsDynamicData>>,
    id: u32,
    value: std::os::raw::c_char,
) -> ReturnCode {
    let Some(mut data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    let r_char = value as u8 as char;
    match unsafe { data.as_mut() }.inner_mut().set_char8_value(id, r_char) {
        Ok(()) => RETCODE_OK,
        Err(_) => RETCODE_ERROR,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_data_set_string_value(
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
    match unsafe { data.as_mut() }.inner_mut().set_string_value(id, string_val) {
        Ok(()) => RETCODE_OK,
        Err(_) => RETCODE_ERROR,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_dynamic_data_set_complex_value(
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
    match unsafe { data.as_mut() }.inner_mut().set_complex_value(id, complex_val) {
        Ok(()) => RETCODE_OK,
        Err(_) => RETCODE_ERROR,
    }
}
