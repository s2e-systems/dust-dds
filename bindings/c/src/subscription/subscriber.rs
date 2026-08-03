use std::ptr::NonNull;

use dust_dds::xtypes::dynamic_type::DynamicData;

use crate::{
    CDataReaderListenerWrapper, DataReaderQos, DustDdsDataReader, DustDdsDataReaderListener,
    DustDdsStatusMask, DustDdsTopic, RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode,
};

/// cbindgen:opaque
pub struct DustDdsSubscriber(pub(crate) dust_dds::subscription::subscriber::Subscriber);

pub type Subscriber = DustDdsSubscriber;

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
pub unsafe extern "C" fn dds_subscriber_create_datareader(
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
pub unsafe extern "C" fn dds_subscriber_delete_datareader(
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
