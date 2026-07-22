use std::ptr::NonNull;
use crate::DustDdsStatusCondition;
use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode};
use crate::infrastructure::qos::DustDdsDataReaderQos;
use crate::subscription::subscriber::DustDdsSubscriber;
use crate::topic_definition::topic::DustDdsTopic;
use dust_dds::xtypes::dynamic_type::DynamicData;

/// cbindgen:opaque
pub struct DustDdsDataReader(pub(crate) dust_dds::subscription::data_reader::DataReader<DynamicData<'static>>);

impl DustDdsDataReader {
    pub fn new(data_reader: dust_dds::subscription::data_reader::DataReader<DynamicData<'static>>) -> Self {
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
    qos: Option<NonNull<DustDdsDataReaderQos>>,
) -> Option<NonNull<DustDdsDataReader>> {
    let Some(subscriber) = subscriber else {
        return None;
    };
    let Some(topic) = topic else {
        return None;
    };

    let qos = match qos {
        Some(q) => dust_dds::infrastructure::qos::QosKind::Specific(unsafe { q.as_ref() }.inner().clone()),
        None => dust_dds::infrastructure::qos::QosKind::Default,
    };

    struct NoDataReaderListener;
    impl dust_dds::subscription::data_reader_listener::DataReaderListener<DynamicData<'static>>
        for NoDataReaderListener {}

    let subscriber_ref = unsafe { subscriber.as_ref() };
    let topic_ref = unsafe { topic.as_ref() };

    match subscriber_ref.inner().create_datareader::<DynamicData<'static>>(
        topic_ref.inner(),
        qos,
        None::<NoDataReaderListener>,
        &[],
    ) {
        Ok(dr) => NonNull::new(Box::into_raw(Box::new(DustDdsDataReader::new(dr)))),
        Err(_) => None,
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
    NonNull::new(Box::into_raw(Box::new(DustDdsStatusCondition::new(condition))))
}
