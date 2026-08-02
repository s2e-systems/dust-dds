use crate::DustDdsStatusCondition;
use crate::infrastructure::condition::DustDdsStatusMask;
use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode};
use crate::infrastructure::listeners::{CDataWriterListenerWrapper, DustDdsDataWriterListener};
use crate::infrastructure::qos::DataWriterQos;
use crate::publication::publisher::DustDdsPublisher;
use crate::topic_definition::topic::DustDdsTopic;
use dust_dds::xtypes::dynamic_type::DynamicData;
use std::ptr::NonNull;

/// cbindgen:opaque
pub struct DustDdsDataWriter(
    pub(crate) dust_dds::publication::data_writer::DataWriter<DynamicData<'static>>,
);

impl DustDdsDataWriter {
    pub fn new(
        data_writer: dust_dds::publication::data_writer::DataWriter<DynamicData<'static>>,
    ) -> Self {
        Self(data_writer)
    }

    pub fn inner(&self) -> &dust_dds::publication::data_writer::DataWriter<DynamicData<'static>> {
        &self.0
    }
}

/// Creates a new DataWriter.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_publisher_create_datawriter(
    publisher: Option<NonNull<DustDdsPublisher>>,
    topic: Option<NonNull<DustDdsTopic>>,
    qos: *const DataWriterQos,
    listener: *const DustDdsDataWriterListener,
    mask: DustDdsStatusMask,
) -> Option<NonNull<DustDdsDataWriter>> {
    let Some(publisher) = publisher else {
        return None;
    };
    let Some(topic) = topic else {
        return None;
    };

    let qos = if qos.is_null() {
        dust_dds::infrastructure::qos::QosKind::Default
    } else {
        dust_dds::infrastructure::qos::QosKind::Specific((*unsafe { &*qos }).into())
    };

    let status_kinds = crate::infrastructure::condition::mask_to_status_kinds(mask);

    let publisher_ref = unsafe { publisher.as_ref() };
    let topic_ref = unsafe { topic.as_ref() };

    let result = if listener.is_null() {
        struct NoDataWriterListener;
        impl dust_dds::publication::data_writer_listener::DataWriterListener<DynamicData<'static>>
            for NoDataWriterListener
        {
        }

        publisher_ref
            .inner()
            .create_datawriter::<DynamicData<'static>>(
                topic_ref.inner(),
                qos,
                None::<NoDataWriterListener>,
                &status_kinds,
            )
    } else {
        let listener_wrapper = CDataWriterListenerWrapper {
            listener: unsafe { *listener },
        };

        publisher_ref
            .inner()
            .create_datawriter::<DynamicData<'static>>(
                topic_ref.inner(),
                qos,
                Some(listener_wrapper),
                &status_kinds,
            )
    };

    match result {
        Ok(dw) => NonNull::new(Box::into_raw(Box::new(DustDdsDataWriter::new(dw)))),
        Err(_) => None,
    }
}

/// Deletes an existing DataWriter.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_publisher_delete_datawriter(
    publisher: Option<NonNull<DustDdsPublisher>>,
    datawriter: Option<NonNull<DustDdsDataWriter>>,
) -> ReturnCode {
    let Some(datawriter) = datawriter else {
        return RETCODE_OK;
    };
    let Some(publisher) = publisher else {
        return RETCODE_BAD_PARAMETER;
    };

    let publisher_ref = unsafe { publisher.as_ref() };
    let datawriter_ref = unsafe { datawriter.as_ref() };

    match publisher_ref
        .inner()
        .delete_datawriter(datawriter_ref.inner())
    {
        Ok(()) => {
            unsafe {
                drop(Box::from_raw(datawriter.as_ptr()));
            }
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Gets the StatusCondition associated with the DataWriter.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datawriter_get_statuscondition(
    writer: Option<NonNull<DustDdsDataWriter>>,
) -> Option<NonNull<DustDdsStatusCondition>> {
    let Some(writer) = writer else {
        return None;
    };

    let writer_ref = unsafe { writer.as_ref() };
    let condition = writer_ref.inner().get_statuscondition();
    NonNull::new(Box::into_raw(Box::new(DustDdsStatusCondition::new(
        condition,
    ))))
}

/// Waits for all acknowledged samples.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datawriter_wait_for_acknowledgments(
    writer: Option<NonNull<DustDdsDataWriter>>,
    max_wait: crate::infrastructure::wait_set::DustDdsDuration,
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_publisher_lookup_datawriter(
    publisher: Option<NonNull<DustDdsPublisher>>,
    topic_name: *const std::os::raw::c_char,
) -> Option<NonNull<DustDdsDataWriter>> {
    let Some(publisher) = publisher else {
        return None;
    };
    if topic_name.is_null() {
        return None;
    }
    let topic_name_str = unsafe { std::ffi::CStr::from_ptr(topic_name) }.to_str().ok()?;

    match unsafe { publisher.as_ref() }
        .inner()
        .lookup_datawriter::<DynamicData<'static>>(topic_name_str)
    {
        Ok(Some(dw)) => NonNull::new(Box::into_raw(Box::new(DustDdsDataWriter::new(dw)))),
        _ => None,
    }
}
