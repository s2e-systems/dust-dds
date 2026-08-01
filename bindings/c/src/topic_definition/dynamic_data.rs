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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_dynamic_data_free(data: Option<NonNull<DustDdsDynamicData>>) {
    if let Some(d) = data {
        unsafe {
            drop(Box::from_raw(d.as_ptr()));
        }
    }
}

/// Frees a string allocated by the Rust bindings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_string_free(ptr: *mut std::os::raw::c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(std::ffi::CString::from_raw(ptr));
        }
    }
}

// Explicit primitive getters
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datawriter_write(
    writer: Option<NonNull<crate::publication::data_writer::DustDdsDataWriter>>,
    data: Option<NonNull<DustDdsDynamicData>>,
) -> ReturnCode {
    let Some(writer) = writer else {
        return RETCODE_BAD_PARAMETER;
    };
    let Some(data) = data else {
        return RETCODE_BAD_PARAMETER;
    };
    let data_val = unsafe { data.as_ref() }.inner().clone();
    match unsafe { writer.as_ref() }.inner().write(data_val, None) {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Reads data using the generic DustDdsDataReader.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datareader_read(
    reader: Option<NonNull<crate::subscription::data_reader::DustDdsDataReader>>,
    data_values: *mut Option<NonNull<DustDdsDynamicData>>,
    max_samples: i32,
    received_samples: *mut i32,
) -> ReturnCode {
    let Some(reader) = reader else {
        return RETCODE_BAD_PARAMETER;
    };
    if data_values.is_null() || received_samples.is_null() {
        return RETCODE_BAD_PARAMETER;
    }

    let reader_ref = unsafe { reader.as_ref() };
    match reader_ref.inner().read(
        max_samples,
        dust_dds::infrastructure::sample_info::ANY_SAMPLE_STATE,
        dust_dds::infrastructure::sample_info::ANY_VIEW_STATE,
        dust_dds::infrastructure::sample_info::ANY_INSTANCE_STATE,
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
