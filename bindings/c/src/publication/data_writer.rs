use crate::{
    DynamicData, StatusCondition,
    infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode},
    publication::publisher::Publisher,
};
use dust_dds::xtypes::dynamic_type::DynamicData as RustDynamicData;
use std::ptr::NonNull;

/// cbindgen:opaque
pub struct DataWriter(
    pub(crate) dust_dds::publication::data_writer::DataWriter<RustDynamicData<'static>>,
);

impl DataWriter {
    pub fn new(
        data_writer: dust_dds::publication::data_writer::DataWriter<RustDynamicData<'static>>,
    ) -> Self {
        Self(data_writer)
    }

    pub fn inner(&self) -> &dust_dds::publication::data_writer::DataWriter<RustDynamicData<'static>> {
        &self.0
    }
}

/// Writes data using the generic DataWriter.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `writer` must point to a valid, initialized `DataWriter` instance.
/// - `data` must point to a valid, initialized `DynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_datawriter_write(
    writer: Option<NonNull<crate::publication::data_writer::DataWriter>>,
    data: Option<NonNull<DynamicData>>,
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
/// - `writer` must point to a valid, initialized `DataWriter` instance.
/// - `instance_data` must point to a valid, initialized `DynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_datawriter_register_instance(
    writer: Option<NonNull<crate::publication::data_writer::DataWriter>>,
    instance_data: Option<NonNull<DynamicData>>,
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
/// - `writer` must point to a valid, initialized `DataWriter` instance.
/// - `instance_data` must point to a valid, initialized `DynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_datawriter_register_instance_w_timestamp(
    writer: Option<NonNull<crate::publication::data_writer::DataWriter>>,
    instance_data: Option<NonNull<DynamicData>>,
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
/// - `writer` must point to a valid, initialized `DataWriter` instance.
/// - `instance_data` must point to a valid, initialized `DynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_datawriter_unregister_instance(
    writer: Option<NonNull<crate::publication::data_writer::DataWriter>>,
    instance_data: Option<NonNull<DynamicData>>,
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
/// - `writer` must point to a valid, initialized `DataWriter` instance.
/// - `instance_data` must point to a valid, initialized `DynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_datawriter_unregister_instance_w_timestamp(
    writer: Option<NonNull<crate::publication::data_writer::DataWriter>>,
    instance_data: Option<NonNull<DynamicData>>,
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

/// Writes data with timestamp using the generic DataWriter.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `writer` must point to a valid, initialized `DataWriter` instance.
/// - `data` must point to a valid, initialized `DynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_datawriter_write_w_timestamp(
    writer: Option<NonNull<crate::publication::data_writer::DataWriter>>,
    data: Option<NonNull<DynamicData>>,
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
/// - `writer` must point to a valid, initialized `DataWriter` instance.
/// - `instance_data` must point to a valid, initialized `DynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_datawriter_dispose(
    writer: Option<NonNull<crate::publication::data_writer::DataWriter>>,
    instance_data: Option<NonNull<DynamicData>>,
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
/// - `writer` must point to a valid, initialized `DataWriter` instance.
/// - `instance_data` must point to a valid, initialized `DynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_datawriter_dispose_w_timestamp(
    writer: Option<NonNull<crate::publication::data_writer::DataWriter>>,
    instance_data: Option<NonNull<DynamicData>>,
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
/// - `writer` must point to a valid, initialized `DataWriter` instance.
/// - `key_holder` must point to a valid, initialized `DynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_datawriter_get_key_value(
    writer: Option<NonNull<crate::publication::data_writer::DataWriter>>,
    key_holder: Option<NonNull<DynamicData>>,
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
/// - `writer` must point to a valid, initialized `DataWriter` instance.
/// - `key_holder` must point to a valid, initialized `DynamicData` instance.
/// - `handle` must be a valid pointer to a `InstanceHandle_t` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_datawriter_lookup_instance(
    writer: Option<NonNull<crate::publication::data_writer::DataWriter>>,
    key_holder: Option<NonNull<DynamicData>>,
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

/// Gets the StatusCondition associated with the DataWriter.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `writer` must point to a valid, initialized `DataWriter` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_datawriter_get_statuscondition(
    writer: Option<NonNull<DataWriter>>,
) -> Option<NonNull<StatusCondition>> {
    let writer = writer?;

    let writer_ref = unsafe { writer.as_ref() };
    let condition = writer_ref.inner().get_statuscondition();
    NonNull::new(Box::into_raw(Box::new(StatusCondition::new(
        condition,
    ))))
}

/// Waits for all acknowledged samples.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `writer` must point to a valid, initialized `DataWriter` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_datawriter_wait_for_acknowledgments(
    writer: Option<NonNull<DataWriter>>,
    max_wait: crate::infrastructure::wait_set::Duration,
) -> ReturnCode {
    let Some(writer) = writer else {
        return RETCODE_BAD_PARAMETER;
    };

    let writer_ref = unsafe { writer.as_ref() };
    match writer_ref.inner().wait_for_acknowledgments(max_wait.into()) {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Looks up an existing DataWriter.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `publisher` must point to a valid, initialized `Publisher` instance.
/// - `topic_name` must be a valid pointer to a `c_char` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_publisher_lookup_datawriter(
    publisher: Option<NonNull<Publisher>>,
    topic_name: *const std::os::raw::c_char,
) -> Option<NonNull<DataWriter>> {
    let publisher = publisher?;
    if topic_name.is_null() {
        return None;
    }
    let topic_name_str = unsafe { std::ffi::CStr::from_ptr(topic_name) }
        .to_str()
        .ok()?;

    match unsafe { publisher.as_ref() }
        .inner()
        .lookup_datawriter::<RustDynamicData<'static>>(topic_name_str)
    {
        Ok(Some(dw)) => NonNull::new(Box::into_raw(Box::new(DataWriter::new(dw)))),
        _ => None,
    }
}
