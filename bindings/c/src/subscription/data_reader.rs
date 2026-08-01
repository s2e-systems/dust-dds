use crate::DustDdsStatusCondition;
use crate::infrastructure::condition::DustDdsStatusMask;
use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode};
use crate::infrastructure::listeners::{CDataReaderListenerWrapper, DustDdsDataReaderListener};
use crate::infrastructure::qos::DataReaderQos;
use crate::subscription::subscriber::DustDdsSubscriber;
use crate::topic_definition::topic::DustDdsTopic;
use dust_dds::xtypes::dynamic_type::DynamicData;
use std::ptr::NonNull;

/// cbindgen:opaque
pub struct DustDdsDataReader(
    pub(crate) dust_dds::subscription::data_reader::DataReader<DynamicData<'static>>,
);

impl DustDdsDataReader {
    pub fn new(
        data_reader: dust_dds::subscription::data_reader::DataReader<DynamicData<'static>>,
    ) -> Self {
        Self(data_reader)
    }

    pub fn inner(&self) -> &dust_dds::subscription::data_reader::DataReader<DynamicData<'static>> {
        &self.0
    }
}

/// Creates a new DataReader.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_subscriber_create_datareader(
    subscriber: Option<NonNull<DustDdsSubscriber>>,
    topic: Option<NonNull<DustDdsTopic>>,
    qos: *const DataReaderQos,
    listener: *const DustDdsDataReaderListener,
    mask: DustDdsStatusMask,
) -> Option<NonNull<DustDdsDataReader>> {
    let Some(subscriber) = subscriber else {
        return None;
    };
    let Some(topic) = topic else {
        return None;
    };

    let qos = if qos.is_null() {
        dust_dds::infrastructure::qos::QosKind::Default
    } else {
        dust_dds::infrastructure::qos::QosKind::Specific(unsafe { &*qos }.clone().into())
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

/// Gets the StatusCondition associated with the DataReader.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datareader_get_statuscondition(
    reader: Option<NonNull<DustDdsDataReader>>,
) -> Option<NonNull<DustDdsStatusCondition>> {
    let Some(reader) = reader else {
        return None;
    };

    let reader_ref = unsafe { reader.as_ref() };
    let condition = reader_ref.inner().get_statuscondition();
    NonNull::new(Box::into_raw(Box::new(DustDdsStatusCondition::new(
        condition,
    ))))
}
