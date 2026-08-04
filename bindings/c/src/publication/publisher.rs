use std::ptr::NonNull;

use dust_dds::xtypes::dynamic_type::DynamicData;

use crate::{
    CDataWriterListenerWrapper, DataWriter, DataWriterListener, StatusMask, Topic,
    infrastructure::{
        error::{RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode},
        qos::{DataWriterQos, PublisherQos, TopicQos},
    },
};

/// cbindgen:opaque
pub struct Publisher(pub(crate) dust_dds::publication::publisher::Publisher);

impl Publisher {
    pub fn new(publisher: dust_dds::publication::publisher::Publisher) -> Self {
        Self(publisher)
    }

    pub fn inner(&self) -> &dust_dds::publication::publisher::Publisher {
        &self.0
    }
}

/// Creates a new DataWriter.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `publisher` must point to a valid, initialized `Publisher` instance.
/// - `topic` must point to a valid, initialized `Topic` instance.
/// - `qos` must be a valid pointer to a `DataWriterQos` instance (or null).
/// - `listener` must be a valid pointer to a `DataWriterListener` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Publisher_create_datawriter(
    publisher: Option<NonNull<Publisher>>,
    topic: Option<NonNull<Topic>>,
    qos: *const DataWriterQos,
    listener: *const DataWriterListener,
    mask: StatusMask,
) -> Option<NonNull<DataWriter>> {
    let publisher = publisher?;
    let topic = topic?;

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
        Ok(dw) => NonNull::new(Box::into_raw(Box::new(DataWriter::new(dw)))),
        Err(_) => None,
    }
}

/// Deletes an existing DataWriter.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `publisher` must point to a valid, initialized `Publisher` instance.
/// - `datawriter` must point to a valid, initialized `DataWriter` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Publisher_delete_datawriter(
    publisher: Option<NonNull<Publisher>>,
    datawriter: Option<NonNull<DataWriter>>,
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

/// Deletes all the entities that were created by means of the Publisher's create_datawriter operations.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `publisher` must point to a valid, initialized `Publisher` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Publisher_delete_contained_entities(
    publisher: Option<NonNull<Publisher>>,
) -> ReturnCode {
    let Some(publisher) = publisher else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { publisher.as_ref() }
        .inner()
        .delete_contained_entities()
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Sets the QoS policies of the Publisher.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `publisher` must point to a valid, initialized `Publisher` instance.
/// - `qos` must be a valid pointer to a `PublisherQos` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Publisher_set_qos(
    publisher: Option<NonNull<Publisher>>,
    qos: *const PublisherQos,
) -> ReturnCode {
    let Some(publisher) = publisher else {
        return RETCODE_BAD_PARAMETER;
    };
    let qos = if qos.is_null() {
        dust_dds::infrastructure::qos::QosKind::Default
    } else {
        dust_dds::infrastructure::qos::QosKind::Specific((*unsafe { &*qos }).into())
    };
    match unsafe { publisher.as_ref() }.inner().set_qos(qos) {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Gets the QoS policies of the Publisher.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `publisher` must point to a valid, initialized `Publisher` instance.
/// - `qos` must be a valid pointer to a `PublisherQos` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Publisher_get_qos(
    publisher: Option<NonNull<Publisher>>,
    qos: *mut PublisherQos,
) -> ReturnCode {
    let Some(publisher) = publisher else {
        return RETCODE_BAD_PARAMETER;
    };
    if qos.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { publisher.as_ref() }.inner().get_qos() {
        Ok(q) => {
            unsafe { *qos = q.into() };
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Sets the PublisherListener and StatusMask.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `publisher` must point to a valid, initialized `Publisher` instance.
/// - `listener` must be a valid pointer to a `PublisherListener` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Publisher_set_listener(
    publisher: Option<NonNull<Publisher>>,
    listener: *const crate::infrastructure::listeners::PublisherListener,
    mask: crate::infrastructure::condition::StatusMask,
) -> ReturnCode {
    let Some(publisher) = publisher else {
        return RETCODE_BAD_PARAMETER;
    };
    let status_kinds = crate::infrastructure::condition::mask_to_status_kinds(mask);
    let result = if listener.is_null() {
        unsafe { publisher.as_ref() }.inner().set_listener(
            None::<crate::infrastructure::listeners::CPublisherListenerWrapper>,
            &status_kinds,
        )
    } else {
        let wrapper = crate::infrastructure::listeners::CPublisherListenerWrapper {
            listener: unsafe { *listener },
        };
        unsafe { publisher.as_ref() }
            .inner()
            .set_listener(Some(wrapper), &status_kinds)
    };
    match result {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Suspends publications.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `publisher` must point to a valid, initialized `Publisher` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Publisher_suspend_publications(
    publisher: Option<NonNull<Publisher>>,
) -> ReturnCode {
    let Some(publisher) = publisher else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { publisher.as_ref() }.inner().suspend_publications() {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Resumes publications.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `publisher` must point to a valid, initialized `Publisher` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Publisher_resume_publications(
    publisher: Option<NonNull<Publisher>>,
) -> ReturnCode {
    let Some(publisher) = publisher else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { publisher.as_ref() }.inner().resume_publications() {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Begins a coherent set of changes.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `publisher` must point to a valid, initialized `Publisher` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Publisher_begin_coherent_changes(
    publisher: Option<NonNull<Publisher>>,
) -> ReturnCode {
    let Some(publisher) = publisher else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { publisher.as_ref() }
        .inner()
        .begin_coherent_changes()
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Ends a coherent set of changes.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `publisher` must point to a valid, initialized `Publisher` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Publisher_end_coherent_changes(
    publisher: Option<NonNull<Publisher>>,
) -> ReturnCode {
    let Some(publisher) = publisher else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { publisher.as_ref() }.inner().end_coherent_changes() {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Blocks the calling thread until all data written by the reliable DataWriter entities is acknowledged.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `publisher` must point to a valid, initialized `Publisher` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Publisher_wait_for_acknowledgments(
    publisher: Option<NonNull<Publisher>>,
    max_wait: crate::infrastructure::wait_set::Duration,
) -> ReturnCode {
    let Some(publisher) = publisher else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { publisher.as_ref() }
        .inner()
        .wait_for_acknowledgments(max_wait.into())
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Returns the DomainParticipant to which the Publisher belongs.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `publisher` must point to a valid, initialized `Publisher` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Publisher_get_participant(
    publisher: Option<NonNull<Publisher>>,
) -> Option<NonNull<crate::DomainParticipant>> {
    let publisher = publisher?;
    let participant = unsafe { publisher.as_ref() }.inner().get_participant();
    NonNull::new(Box::into_raw(Box::new(crate::DomainParticipant::new(
        participant,
    ))))
}

/// Sets the default DataWriterQos.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `publisher` must point to a valid, initialized `Publisher` instance.
/// - `qos` must be a valid pointer to a `DataWriterQos` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Publisher_set_default_datawriter_qos(
    publisher: Option<NonNull<Publisher>>,
    qos: *const DataWriterQos,
) -> ReturnCode {
    let Some(publisher) = publisher else {
        return RETCODE_BAD_PARAMETER;
    };
    let qos = if qos.is_null() {
        dust_dds::infrastructure::qos::QosKind::Default
    } else {
        dust_dds::infrastructure::qos::QosKind::Specific((*unsafe { &*qos }).into())
    };
    match unsafe { publisher.as_ref() }
        .inner()
        .set_default_datawriter_qos(qos)
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Gets the default DataWriterQos.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `publisher` must point to a valid, initialized `Publisher` instance.
/// - `qos` must be a valid pointer to a `DataWriterQos` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Publisher_get_default_datawriter_qos(
    publisher: Option<NonNull<Publisher>>,
    qos: *mut DataWriterQos,
) -> ReturnCode {
    let Some(publisher) = publisher else {
        return RETCODE_BAD_PARAMETER;
    };
    if qos.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { publisher.as_ref() }
        .inner()
        .get_default_datawriter_qos()
    {
        Ok(q) => {
            unsafe { *qos = q.into() };
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Copies the policies in the TopicQos to the corresponding policies in the DataWriterQos.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `publisher` must point to a valid, initialized `Publisher` instance.
/// - `a_datawriter_qos` must be a valid pointer to a `DataWriterQos` instance for writing (or null).
/// - `a_topic_qos` must be a valid pointer to a `TopicQos` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_Publisher_copy_from_topic_qos(
    publisher: Option<NonNull<Publisher>>,
    a_datawriter_qos: *mut DataWriterQos,
    a_topic_qos: *const TopicQos,
) -> ReturnCode {
    let Some(publisher) = publisher else {
        return RETCODE_BAD_PARAMETER;
    };
    if a_datawriter_qos.is_null() || a_topic_qos.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let publisher_ref = unsafe { publisher.as_ref() };
    let mut rust_datawriter_qos: dust_dds::infrastructure::qos::DataWriterQos =
        unsafe { *a_datawriter_qos }.into();
    let rust_topic_qos: dust_dds::infrastructure::qos::TopicQos = unsafe { *a_topic_qos }.into();
    match publisher_ref
        .inner()
        .copy_from_topic_qos(&mut rust_datawriter_qos, &rust_topic_qos)
    {
        Ok(()) => {
            unsafe { *a_datawriter_qos = rust_datawriter_qos.into() };
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}
