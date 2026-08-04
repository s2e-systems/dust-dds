use crate::{DynamicData, RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode, StatusCondition};
use dust_dds::xtypes::dynamic_type::DynamicData as RustDynamicData;
use std::ptr::NonNull;

/// cbindgen:opaque
pub struct DataReader(
    pub(crate) dust_dds::subscription::data_reader::DataReader<RustDynamicData<'static>>,
);

impl DataReader {
    pub fn new(
        data_reader: dust_dds::subscription::data_reader::DataReader<RustDynamicData<'static>>,
    ) -> Self {
        Self(data_reader)
    }

    pub fn inner(
        &self,
    ) -> &dust_dds::subscription::data_reader::DataReader<RustDynamicData<'static>> {
        &self.0
    }
}

/// Reads data using the generic DataReader.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `reader` must point to a valid, initialized `DataReader` instance.
/// - `data_values` must point to a valid, initialized `DynamicData` instance.
/// - `sample_infos` must be a valid pointer to a `SampleInfo` instance for writing (or null).
/// - `received_samples` must be a valid pointer to a `i32` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DataReader_read(
    reader: Option<NonNull<crate::subscription::data_reader::DataReader>>,
    data_values: *mut Option<NonNull<DynamicData>>,
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
                        let wrapper = DynamicData::new(dynamic_data);
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

/// Takes data using the generic DataReader.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `reader` must point to a valid, initialized `DataReader` instance.
/// - `data_values` must point to a valid, initialized `DynamicData` instance.
/// - `sample_infos` must be a valid pointer to a `SampleInfo` instance for writing (or null).
/// - `received_samples` must be a valid pointer to a `i32` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DataReader_take(
    reader: Option<NonNull<crate::subscription::data_reader::DataReader>>,
    data_values: *mut Option<NonNull<DynamicData>>,
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
                        let wrapper = DynamicData::new(dynamic_data);
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
/// - `reader` must point to a valid, initialized `DataReader` instance.
/// - `data_value` must point to a valid, initialized `DynamicData` instance.
/// - `sample_info` must be a valid pointer to a `SampleInfo` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DataReader_read_next_sample(
    reader: Option<NonNull<crate::subscription::data_reader::DataReader>>,
    data_value: *mut Option<NonNull<DynamicData>>,
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
                    let wrapper = DynamicData::new(dynamic_data);
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
/// - `reader` must point to a valid, initialized `DataReader` instance.
/// - `data_value` must point to a valid, initialized `DynamicData` instance.
/// - `sample_info` must be a valid pointer to a `SampleInfo` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DataReader_take_next_sample(
    reader: Option<NonNull<crate::subscription::data_reader::DataReader>>,
    data_value: *mut Option<NonNull<DynamicData>>,
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
                    let wrapper = DynamicData::new(dynamic_data);
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
/// - `reader` must point to a valid, initialized `DataReader` instance.
/// - `data_values` must point to a valid, initialized `DynamicData` instance.
/// - `sample_infos` must be a valid pointer to a `SampleInfo` instance for writing (or null).
/// - `a_handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
/// - `received_samples` must be a valid pointer to a `i32` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DataReader_read_instance(
    reader: Option<NonNull<crate::subscription::data_reader::DataReader>>,
    data_values: *mut Option<NonNull<DynamicData>>,
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
                        let wrapper = DynamicData::new(dynamic_data);
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
/// - `reader` must point to a valid, initialized `DataReader` instance.
/// - `data_values` must point to a valid, initialized `DynamicData` instance.
/// - `sample_infos` must be a valid pointer to a `SampleInfo` instance for writing (or null).
/// - `a_handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
/// - `received_samples` must be a valid pointer to a `i32` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DataReader_take_instance(
    reader: Option<NonNull<crate::subscription::data_reader::DataReader>>,
    data_values: *mut Option<NonNull<DynamicData>>,
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
                        let wrapper = DynamicData::new(dynamic_data);
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
/// - `reader` must point to a valid, initialized `DataReader` instance.
/// - `data_values` must point to a valid, initialized `DynamicData` instance.
/// - `sample_infos` must be a valid pointer to a `SampleInfo` instance for writing (or null).
/// - `previous_handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
/// - `received_samples` must be a valid pointer to a `i32` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DataReader_read_next_instance(
    reader: Option<NonNull<crate::subscription::data_reader::DataReader>>,
    data_values: *mut Option<NonNull<DynamicData>>,
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
                        let wrapper = DynamicData::new(dynamic_data);
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
/// - `reader` must point to a valid, initialized `DataReader` instance.
/// - `data_values` must point to a valid, initialized `DynamicData` instance.
/// - `sample_infos` must be a valid pointer to a `SampleInfo` instance for writing (or null).
/// - `previous_handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
/// - `received_samples` must be a valid pointer to a `i32` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DataReader_take_next_instance(
    reader: Option<NonNull<crate::subscription::data_reader::DataReader>>,
    data_values: *mut Option<NonNull<DynamicData>>,
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
                        let wrapper = DynamicData::new(dynamic_data);
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
/// - `_reader` must point to a valid, initialized `DataReader` instance.
/// - `_data_values` must point to a valid, initialized `DynamicData` instance.
/// - `_sample_infos` must be a valid pointer to a `SampleInfo` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DataReader_return_loan(
    _reader: Option<NonNull<crate::subscription::data_reader::DataReader>>,
    _data_values: *mut Option<NonNull<DynamicData>>,
    _sample_infos: *mut SampleInfo,
) -> ReturnCode {
    RETCODE_OK
}

/// Retrieves the key value of an instance.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `reader` must point to a valid, initialized `DataReader` instance.
/// - `key_holder` must point to a valid, initialized `DynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DataReader_get_key_value(
    reader: Option<NonNull<crate::subscription::data_reader::DataReader>>,
    key_holder: Option<NonNull<DynamicData>>,
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
/// - `reader` must point to a valid, initialized `DataReader` instance.
/// - `key_holder` must point to a valid, initialized `DynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DataReader_lookup_instance(
    reader: Option<NonNull<crate::subscription::data_reader::DataReader>>,
    key_holder: Option<NonNull<DynamicData>>,
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

/// Gets the StatusCondition associated with the DataReader.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `reader` must point to a valid, initialized `DataReader` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_DataReader_get_statuscondition(
    reader: Option<NonNull<DataReader>>,
) -> Option<NonNull<StatusCondition>> {
    let reader = reader?;

    let reader_ref = unsafe { reader.as_ref() };
    let condition = reader_ref.inner().get_statuscondition();
    NonNull::new(Box::into_raw(Box::new(StatusCondition::new(condition))))
}

pub type SampleStateMask = u32;
pub type ViewStateMask = u32;
pub type InstanceStateMask = u32;

pub const READ_SAMPLE_STATE: SampleStateMask = 0x0001;
pub const NOT_READ_SAMPLE_STATE: SampleStateMask = 0x0002;
pub const ANY_SAMPLE_STATE: SampleStateMask = 0xffff;

pub const NEW_VIEW_STATE: ViewStateMask = 0x0001;
pub const NOT_NEW_VIEW_STATE: ViewStateMask = 0x0002;
pub const ANY_VIEW_STATE: ViewStateMask = 0xffff;

pub const ALIVE_INSTANCE_STATE: InstanceStateMask = 0x0001;
pub const NOT_ALIVE_DISPOSED_INSTANCE_STATE: InstanceStateMask = 0x0002;
pub const NOT_ALIVE_NO_WRITERS_INSTANCE_STATE: InstanceStateMask = 0x0004;
pub const ANY_INSTANCE_STATE: InstanceStateMask = 0xffff;
pub const NOT_ALIVE_INSTANCE_STATE: InstanceStateMask = 0x0006;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SampleInfo {
    pub sample_state: SampleStateMask,
    pub view_state: ViewStateMask,
    pub instance_state: InstanceStateMask,
    pub disposed_generation_count: i32,
    pub no_writers_generation_count: i32,
    pub sample_rank: i32,
    pub generation_rank: i32,
    pub absolute_generation_rank: i32,
    pub source_timestamp: crate::infrastructure::qos_policy::Time_t,
    pub instance_handle: crate::infrastructure::status::InstanceHandle_t,
    pub publication_handle: crate::infrastructure::status::InstanceHandle_t,
    pub valid_data: bool,
}

pub(crate) fn sample_state_kind_to_mask(
    value: dust_dds::infrastructure::sample_info::SampleStateKind,
) -> SampleStateMask {
    match value {
        dust_dds::infrastructure::sample_info::SampleStateKind::Read => READ_SAMPLE_STATE,
        dust_dds::infrastructure::sample_info::SampleStateKind::NotRead => NOT_READ_SAMPLE_STATE,
    }
}

pub(crate) fn view_state_kind_to_mask(
    value: dust_dds::infrastructure::sample_info::ViewStateKind,
) -> ViewStateMask {
    match value {
        dust_dds::infrastructure::sample_info::ViewStateKind::New => NEW_VIEW_STATE,
        dust_dds::infrastructure::sample_info::ViewStateKind::NotNew => NOT_NEW_VIEW_STATE,
    }
}

pub(crate) fn instance_state_kind_to_mask(
    value: dust_dds::infrastructure::sample_info::InstanceStateKind,
) -> InstanceStateMask {
    match value {
        dust_dds::infrastructure::sample_info::InstanceStateKind::Alive => ALIVE_INSTANCE_STATE,
        dust_dds::infrastructure::sample_info::InstanceStateKind::NotAliveDisposed => {
            NOT_ALIVE_DISPOSED_INSTANCE_STATE
        }
        dust_dds::infrastructure::sample_info::InstanceStateKind::NotAliveNoWriters => {
            NOT_ALIVE_NO_WRITERS_INSTANCE_STATE
        }
    }
}

impl From<dust_dds::infrastructure::sample_info::SampleInfo> for SampleInfo {
    fn from(value: dust_dds::infrastructure::sample_info::SampleInfo) -> Self {
        Self {
            sample_state: sample_state_kind_to_mask(value.sample_state),
            view_state: view_state_kind_to_mask(value.view_state),
            instance_state: instance_state_kind_to_mask(value.instance_state),
            disposed_generation_count: value.disposed_generation_count,
            no_writers_generation_count: value.no_writers_generation_count,
            sample_rank: value.sample_rank,
            generation_rank: value.generation_rank,
            absolute_generation_rank: value.absolute_generation_rank,
            source_timestamp: value.source_timestamp.into(),
            instance_handle: value.instance_handle.into(),
            publication_handle: value.publication_handle.into(),
            valid_data: value.valid_data,
        }
    }
}

pub(crate) fn sample_states_from_mask(
    mask: SampleStateMask,
) -> Vec<dust_dds::infrastructure::sample_info::SampleStateKind> {
    let mut states = Vec::new();
    if (mask & READ_SAMPLE_STATE) != 0 {
        states.push(dust_dds::infrastructure::sample_info::SampleStateKind::Read);
    }
    if (mask & NOT_READ_SAMPLE_STATE) != 0 {
        states.push(dust_dds::infrastructure::sample_info::SampleStateKind::NotRead);
    }
    states
}

pub(crate) fn view_states_from_mask(
    mask: ViewStateMask,
) -> Vec<dust_dds::infrastructure::sample_info::ViewStateKind> {
    let mut states = Vec::new();
    if (mask & NEW_VIEW_STATE) != 0 {
        states.push(dust_dds::infrastructure::sample_info::ViewStateKind::New);
    }
    if (mask & NOT_NEW_VIEW_STATE) != 0 {
        states.push(dust_dds::infrastructure::sample_info::ViewStateKind::NotNew);
    }
    states
}

pub(crate) fn instance_states_from_mask(
    mask: InstanceStateMask,
) -> Vec<dust_dds::infrastructure::sample_info::InstanceStateKind> {
    let mut states = Vec::new();
    if (mask & ALIVE_INSTANCE_STATE) != 0 {
        states.push(dust_dds::infrastructure::sample_info::InstanceStateKind::Alive);
    }
    if (mask & NOT_ALIVE_DISPOSED_INSTANCE_STATE) != 0 {
        states.push(dust_dds::infrastructure::sample_info::InstanceStateKind::NotAliveDisposed);
    }
    if (mask & NOT_ALIVE_NO_WRITERS_INSTANCE_STATE) != 0 {
        states.push(dust_dds::infrastructure::sample_info::InstanceStateKind::NotAliveNoWriters);
    }
    states
}
