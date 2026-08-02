use std::ptr::NonNull;

use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode};
use crate::infrastructure::qos::{PublisherQos, DataWriterQos, TopicQos};

/// cbindgen:opaque
pub struct DustDdsPublisher(pub(crate) dust_dds::publication::publisher::Publisher);

pub type Publisher = DustDdsPublisher;

impl DustDdsPublisher {
    pub fn new(publisher: dust_dds::publication::publisher::Publisher) -> Self {
        Self(publisher)
    }

    pub fn inner(&self) -> &dust_dds::publication::publisher::Publisher {
        &self.0
    }
}

/// Deletes all the entities that were created by means of the Publisher's create_datawriter operations.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_publisher_delete_contained_entities(
    publisher: Option<NonNull<DustDdsPublisher>>,
) -> ReturnCode {
    let Some(publisher) = publisher else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { publisher.as_ref() }.inner().delete_contained_entities() {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Sets the QoS policies of the Publisher.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_publisher_set_qos(
    publisher: Option<NonNull<DustDdsPublisher>>,
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_publisher_get_qos(
    publisher: Option<NonNull<DustDdsPublisher>>,
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_publisher_set_listener(
    publisher: Option<NonNull<DustDdsPublisher>>,
    listener: *const crate::infrastructure::listeners::DustDdsPublisherListener,
    mask: crate::infrastructure::condition::DustDdsStatusMask,
) -> ReturnCode {
    let Some(publisher) = publisher else {
        return RETCODE_BAD_PARAMETER;
    };
    let status_kinds = crate::infrastructure::condition::mask_to_status_kinds(mask);
    let result = if listener.is_null() {
        unsafe { publisher.as_ref() }
            .inner()
            .set_listener(None::<crate::infrastructure::listeners::CPublisherListenerWrapper>, &status_kinds)
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_publisher_suspend_publications(
    publisher: Option<NonNull<DustDdsPublisher>>,
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_publisher_resume_publications(
    publisher: Option<NonNull<DustDdsPublisher>>,
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_publisher_begin_coherent_changes(
    publisher: Option<NonNull<DustDdsPublisher>>,
) -> ReturnCode {
    let Some(publisher) = publisher else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { publisher.as_ref() }.inner().begin_coherent_changes() {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Ends a coherent set of changes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_publisher_end_coherent_changes(
    publisher: Option<NonNull<DustDdsPublisher>>,
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_publisher_wait_for_acknowledgments(
    publisher: Option<NonNull<DustDdsPublisher>>,
    max_wait: crate::infrastructure::wait_set::DustDdsDuration,
) -> ReturnCode {
    let Some(publisher) = publisher else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { publisher.as_ref() }.inner().wait_for_acknowledgments(max_wait.into()) {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Returns the DomainParticipant to which the Publisher belongs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_publisher_get_participant(
    publisher: Option<NonNull<DustDdsPublisher>>,
) -> Option<NonNull<crate::DustDdsDomainParticipant>> {
    let Some(publisher) = publisher else {
        return None;
    };
    let participant = unsafe { publisher.as_ref() }.inner().get_participant();
    NonNull::new(Box::into_raw(Box::new(crate::DustDdsDomainParticipant::new(
        participant,
    ))))
}

/// Sets the default DataWriterQos.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_publisher_set_default_datawriter_qos(
    publisher: Option<NonNull<DustDdsPublisher>>,
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
    match unsafe { publisher.as_ref() }.inner().set_default_datawriter_qos(qos) {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Gets the default DataWriterQos.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_publisher_get_default_datawriter_qos(
    publisher: Option<NonNull<DustDdsPublisher>>,
    qos: *mut DataWriterQos,
) -> ReturnCode {
    let Some(publisher) = publisher else {
        return RETCODE_BAD_PARAMETER;
    };
    if qos.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { publisher.as_ref() }.inner().get_default_datawriter_qos() {
        Ok(q) => {
            unsafe { *qos = q.into() };
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Copies the policies in the TopicQos to the corresponding policies in the DataWriterQos.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_publisher_copy_from_topic_qos(
    publisher: Option<NonNull<DustDdsPublisher>>,
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
    let mut rust_datawriter_qos: dust_dds::infrastructure::qos::DataWriterQos = unsafe { *a_datawriter_qos }.into();
    let rust_topic_qos: dust_dds::infrastructure::qos::TopicQos = unsafe { *a_topic_qos }.into();
    match publisher_ref.inner().copy_from_topic_qos(&mut rust_datawriter_qos, &rust_topic_qos) {
        Ok(()) => {
            unsafe { *a_datawriter_qos = rust_datawriter_qos.into() };
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}
