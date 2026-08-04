use std::ptr::NonNull;

use dust_dds::xtypes::dynamic_type::DynamicData;

use crate::{
    CDataReaderListenerWrapper, DataReaderQos, DustDdsDataReader, DustDdsDataReaderListener,
    DustDdsStatusMask, DustDdsTopic, RETCODE_BAD_PARAMETER, RETCODE_OK, RETCODE_UNSUPPORTED,
    ReturnCode, SubscriberQos, TopicQos,
};

/// cbindgen:opaque
pub struct DustDdsSubscriber(pub(crate) dust_dds::subscription::subscriber::Subscriber);

impl DustDdsSubscriber {
    pub fn new(subscriber: dust_dds::subscription::subscriber::Subscriber) -> Self {
        Self(subscriber)
    }

    pub fn inner(&self) -> &dust_dds::subscription::subscriber::Subscriber {
        &self.0
    }
}

/// Creates a new DataReader.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `subscriber` must point to a valid, initialized `DustDdsSubscriber` instance.
/// - `topic` must point to a valid, initialized `DustDdsTopic` instance.
/// - `qos` must be a valid pointer to a `DataReaderQos` instance (or null).
/// - `listener` must be a valid pointer to a `DustDdsDataReaderListener` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_subscriber_create_datareader(
    subscriber: Option<NonNull<DustDdsSubscriber>>,
    topic: Option<NonNull<DustDdsTopic>>,
    qos: *const DataReaderQos,
    listener: *const DustDdsDataReaderListener,
    mask: DustDdsStatusMask,
) -> Option<NonNull<DustDdsDataReader>> {
    let subscriber = subscriber?;
    let topic = topic?;

    let qos = if qos.is_null() {
        dust_dds::infrastructure::qos::QosKind::Default
    } else {
        dust_dds::infrastructure::qos::QosKind::Specific((*unsafe { &*qos }).into())
    };

    let status_kinds = crate::infrastructure::condition::mask_to_status_kinds(mask);

    let subscriber_ref = unsafe { subscriber.as_ref() };
    let topic_ref = unsafe { topic.as_ref() };

    if listener.is_null() {
        struct NoDataReaderListener;
        impl dust_dds::subscription::data_reader_listener::DataReaderListener<DynamicData<'static>>
            for NoDataReaderListener
        {
        }

        match subscriber_ref
            .inner()
            .create_datareader::<DynamicData<'static>>(
                topic_ref.inner(),
                qos,
                None::<NoDataReaderListener>,
                &status_kinds,
            ) {
            Ok(dr) => NonNull::new(Box::into_raw(Box::new(DustDdsDataReader::new(dr)))),
            Err(_) => None,
        }
    } else {
        let listener_wrapper = CDataReaderListenerWrapper {
            listener: unsafe { *listener },
        };

        match subscriber_ref
            .inner()
            .create_datareader::<DynamicData<'static>>(
                topic_ref.inner(),
                qos,
                Some(listener_wrapper),
                &status_kinds,
            ) {
            Ok(dr) => NonNull::new(Box::into_raw(Box::new(DustDdsDataReader::new(dr)))),
            Err(_) => None,
        }
    }
}

/// Deletes an existing DataReader.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `subscriber` must point to a valid, initialized `DustDdsSubscriber` instance.
/// - `datareader` must point to a valid, initialized `DustDdsDataReader` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_subscriber_delete_datareader(
    subscriber: Option<NonNull<DustDdsSubscriber>>,
    datareader: Option<NonNull<DustDdsDataReader>>,
) -> ReturnCode {
    let Some(datareader) = datareader else {
        return RETCODE_OK;
    };
    let Some(subscriber) = subscriber else {
        return RETCODE_BAD_PARAMETER;
    };

    let subscriber_ref = unsafe { subscriber.as_ref() };
    let datareader_ref = unsafe { datareader.as_ref() };

    match subscriber_ref
        .inner()
        .delete_datareader(datareader_ref.inner())
    {
        Ok(()) => {
            unsafe {
                drop(Box::from_raw(datareader.as_ptr()));
            }
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Deletes all the entities that were created by means of the Subscriber's create_datareader operations.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `subscriber` must point to a valid, initialized `DustDdsSubscriber` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_subscriber_delete_contained_entities(
    subscriber: Option<NonNull<DustDdsSubscriber>>,
) -> ReturnCode {
    let Some(subscriber) = subscriber else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { subscriber.as_ref() }
        .inner()
        .delete_contained_entities()
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Retrieves a previously created DataReader belonging to the Subscriber that is attached to a Topic.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `subscriber` must point to a valid, initialized `DustDdsSubscriber` instance.
/// - `topic_name` must point to a valid, null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_subscriber_lookup_datareader(
    subscriber: Option<NonNull<DustDdsSubscriber>>,
    topic_name: *const std::os::raw::c_char,
) -> Option<NonNull<DustDdsDataReader>> {
    let subscriber = subscriber?;
    if topic_name.is_null() {
        return None;
    }
    let topic_name_str = unsafe { std::ffi::CStr::from_ptr(topic_name) }
        .to_str()
        .ok()?;
    match unsafe { subscriber.as_ref() }
        .inner()
        .lookup_datareader::<DynamicData<'static>>(topic_name_str)
    {
        Ok(Some(dr)) => NonNull::new(Box::into_raw(Box::new(DustDdsDataReader::new(dr)))),
        _ => None,
    }
}

/// Invokes the on_data_available operation on the listener objects attached to contained DataReader entities.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `subscriber` must point to a valid, initialized `DustDdsSubscriber` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_subscriber_notify_datareaders(
    subscriber: Option<NonNull<DustDdsSubscriber>>,
) -> ReturnCode {
    let Some(subscriber) = subscriber else {
        return RETCODE_BAD_PARAMETER;
    };
    match unsafe { subscriber.as_ref() }.inner().notify_datareaders() {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Sets the QoS policies of the Subscriber.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `subscriber` must point to a valid, initialized `DustDdsSubscriber` instance.
/// - `qos` must be a valid pointer to a `SubscriberQos` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_subscriber_set_qos(
    subscriber: Option<NonNull<DustDdsSubscriber>>,
    qos: *const SubscriberQos,
) -> ReturnCode {
    let Some(subscriber) = subscriber else {
        return RETCODE_BAD_PARAMETER;
    };
    let qos = if qos.is_null() {
        dust_dds::infrastructure::qos::QosKind::Default
    } else {
        dust_dds::infrastructure::qos::QosKind::Specific((*unsafe { &*qos }).into())
    };
    match unsafe { subscriber.as_ref() }.inner().set_qos(qos) {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Gets the QoS policies of the Subscriber.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `subscriber` must point to a valid, initialized `DustDdsSubscriber` instance.
/// - `qos` must be a valid pointer to a `SubscriberQos` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_subscriber_get_qos(
    subscriber: Option<NonNull<DustDdsSubscriber>>,
    qos: *mut SubscriberQos,
) -> ReturnCode {
    let Some(subscriber) = subscriber else {
        return RETCODE_BAD_PARAMETER;
    };
    if qos.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { subscriber.as_ref() }.inner().get_qos() {
        Ok(q) => {
            unsafe { *qos = q.into() };
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Sets the SubscriberListener and StatusMask.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `subscriber` must point to a valid, initialized `DustDdsSubscriber` instance.
/// - `listener` must be a valid pointer to a `DustDdsSubscriberListener` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_subscriber_set_listener(
    subscriber: Option<NonNull<DustDdsSubscriber>>,
    listener: *const crate::infrastructure::listeners::DustDdsSubscriberListener,
    mask: crate::infrastructure::condition::DustDdsStatusMask,
) -> ReturnCode {
    let Some(subscriber) = subscriber else {
        return RETCODE_BAD_PARAMETER;
    };
    let status_kinds = crate::infrastructure::condition::mask_to_status_kinds(mask);
    let result = if listener.is_null() {
        unsafe { subscriber.as_ref() }.inner().set_listener(
            None::<crate::infrastructure::listeners::CSubscriberListenerWrapper>,
            &status_kinds,
        )
    } else {
        let wrapper = crate::infrastructure::listeners::CSubscriberListenerWrapper {
            listener: unsafe { *listener },
        };
        unsafe { subscriber.as_ref() }
            .inner()
            .set_listener(Some(wrapper), &status_kinds)
    };
    match result {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Begins access to the data samples.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `subscriber` must point to a valid, initialized `DustDdsSubscriber` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_subscriber_begin_access(
    subscriber: Option<NonNull<DustDdsSubscriber>>,
) -> ReturnCode {
    let Some(_) = subscriber else {
        return RETCODE_BAD_PARAMETER;
    };
    RETCODE_UNSUPPORTED
}

/// Ends access to the data samples.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `subscriber` must point to a valid, initialized `DustDdsSubscriber` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_subscriber_end_access(
    subscriber: Option<NonNull<DustDdsSubscriber>>,
) -> ReturnCode {
    let Some(_) = subscriber else {
        return RETCODE_BAD_PARAMETER;
    };
    RETCODE_UNSUPPORTED
}

/// Returns the DomainParticipant to which the Subscriber belongs.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `subscriber` must point to a valid, initialized `DustDdsSubscriber` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_subscriber_get_participant(
    subscriber: Option<NonNull<DustDdsSubscriber>>,
) -> Option<NonNull<crate::DustDdsDomainParticipant>> {
    let subscriber = subscriber?;
    let participant = unsafe { subscriber.as_ref() }.inner().get_participant();
    NonNull::new(Box::into_raw(Box::new(
        crate::DustDdsDomainParticipant::new(participant),
    )))
}

/// Sets the default DataReaderQos.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `subscriber` must point to a valid, initialized `DustDdsSubscriber` instance.
/// - `qos` must be a valid pointer to a `DataReaderQos` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_subscriber_set_default_datareader_qos(
    subscriber: Option<NonNull<DustDdsSubscriber>>,
    qos: *const DataReaderQos,
) -> ReturnCode {
    let Some(subscriber) = subscriber else {
        return RETCODE_BAD_PARAMETER;
    };
    let qos = if qos.is_null() {
        dust_dds::infrastructure::qos::QosKind::Default
    } else {
        dust_dds::infrastructure::qos::QosKind::Specific((*unsafe { &*qos }).into())
    };
    match unsafe { subscriber.as_ref() }
        .inner()
        .set_default_datareader_qos(qos)
    {
        Ok(()) => RETCODE_OK,
        Err(e) => e.into(),
    }
}

/// Gets the default DataReaderQos.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `subscriber` must point to a valid, initialized `DustDdsSubscriber` instance.
/// - `qos` must be a valid pointer to a `DataReaderQos` instance for writing (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_subscriber_get_default_datareader_qos(
    subscriber: Option<NonNull<DustDdsSubscriber>>,
    qos: *mut DataReaderQos,
) -> ReturnCode {
    let Some(subscriber) = subscriber else {
        return RETCODE_BAD_PARAMETER;
    };
    if qos.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    match unsafe { subscriber.as_ref() }
        .inner()
        .get_default_datareader_qos()
    {
        Ok(q) => {
            unsafe { *qos = q.into() };
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Copies the policies in the TopicQos to the corresponding policies in the DataReaderQos.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `subscriber` must point to a valid, initialized `DustDdsSubscriber` instance.
/// - `a_datareader_qos` must be a valid pointer to a `DataReaderQos` instance for writing (or null).
/// - `a_topic_qos` must be a valid pointer to a `TopicQos` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_subscriber_copy_from_topic_qos(
    subscriber: Option<NonNull<DustDdsSubscriber>>,
    a_datareader_qos: *mut DataReaderQos,
    a_topic_qos: *const TopicQos,
) -> ReturnCode {
    let Some(_subscriber) = subscriber else {
        return RETCODE_BAD_PARAMETER;
    };
    if a_datareader_qos.is_null() || a_topic_qos.is_null() {
        return RETCODE_BAD_PARAMETER;
    }
    let mut rust_datareader_qos: dust_dds::infrastructure::qos::DataReaderQos =
        unsafe { *a_datareader_qos }.into();
    let rust_topic_qos: dust_dds::infrastructure::qos::TopicQos = unsafe { *a_topic_qos }.into();
    match dust_dds::subscription::subscriber::Subscriber::copy_from_topic_qos(
        &mut rust_datareader_qos,
        &rust_topic_qos,
    ) {
        Ok(()) => {
            unsafe { *a_datareader_qos = rust_datareader_qos.into() };
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Retrieves the DataReader entities.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `subscriber` must point to a valid, initialized `DustDdsSubscriber` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DDS_subscriber_get_datareaders(
    subscriber: Option<NonNull<DustDdsSubscriber>>,
) -> ReturnCode {
    let Some(_) = subscriber else {
        return RETCODE_BAD_PARAMETER;
    };
    RETCODE_UNSUPPORTED
}
