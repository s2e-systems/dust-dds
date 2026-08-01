use std::ptr::NonNull;
use crate::DustDdsStatusCondition;
use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode};
use crate::infrastructure::qos::DataReaderQos;
use crate::subscription::subscriber::DustDdsSubscriber;
use crate::topic_definition::topic::DustDdsTopic;
use dust_dds::xtypes::dynamic_type::DynamicData;
use crate::infrastructure::condition::DustDdsStatusMask;

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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DustDdsSampleLostStatus {
    pub total_count: i32,
    pub total_count_change: i32,
}

impl From<dust_dds::infrastructure::status::SampleLostStatus> for DustDdsSampleLostStatus {
    fn from(status: dust_dds::infrastructure::status::SampleLostStatus) -> Self {
        Self {
            total_count: status.total_count,
            total_count_change: status.total_count_change,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DustDdsSampleRejectedStatus {
    pub total_count: i32,
    pub total_count_change: i32,
    pub last_reason: i32,
    pub last_instance_handle: [u8; 16],
}

impl From<dust_dds::infrastructure::status::SampleRejectedStatus> for DustDdsSampleRejectedStatus {
    fn from(status: dust_dds::infrastructure::status::SampleRejectedStatus) -> Self {
        let last_reason = match status.last_reason {
            dust_dds::infrastructure::status::SampleRejectedStatusKind::NotRejected => 0,
            dust_dds::infrastructure::status::SampleRejectedStatusKind::RejectedByInstancesLimit => 1,
            dust_dds::infrastructure::status::SampleRejectedStatusKind::RejectedBySamplesLimit => 2,
            dust_dds::infrastructure::status::SampleRejectedStatusKind::RejectedBySamplesPerInstanceLimit => 3,
        };
        Self {
            total_count: status.total_count,
            total_count_change: status.total_count_change,
            last_reason,
            last_instance_handle: status.last_instance_handle.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DustDdsLivelinessChangedStatus {
    pub alive_count: i32,
    pub not_alive_count: i32,
    pub alive_count_change: i32,
    pub not_alive_count_change: i32,
    pub last_publication_handle: [u8; 16],
}

impl From<dust_dds::infrastructure::status::LivelinessChangedStatus> for DustDdsLivelinessChangedStatus {
    fn from(status: dust_dds::infrastructure::status::LivelinessChangedStatus) -> Self {
        Self {
            alive_count: status.alive_count,
            not_alive_count: status.not_alive_count,
            alive_count_change: status.alive_count_change,
            not_alive_count_change: status.not_alive_count_change,
            last_publication_handle: status.last_publication_handle.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DustDdsRequestedDeadlineMissedStatus {
    pub total_count: i32,
    pub total_count_change: i32,
    pub last_instance_handle: [u8; 16],
}

impl From<dust_dds::infrastructure::status::RequestedDeadlineMissedStatus> for DustDdsRequestedDeadlineMissedStatus {
    fn from(status: dust_dds::infrastructure::status::RequestedDeadlineMissedStatus) -> Self {
        Self {
            total_count: status.total_count,
            total_count_change: status.total_count_change,
            last_instance_handle: status.last_instance_handle.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DustDdsRequestedIncompatibleQosStatus {
    pub total_count: i32,
    pub total_count_change: i32,
    pub last_policy_id: i32,
}

impl From<dust_dds::infrastructure::status::RequestedIncompatibleQosStatus> for DustDdsRequestedIncompatibleQosStatus {
    fn from(status: dust_dds::infrastructure::status::RequestedIncompatibleQosStatus) -> Self {
        Self {
            total_count: status.total_count,
            total_count_change: status.total_count_change,
            last_policy_id: status.last_policy_id,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DustDdsSubscriptionMatchedStatus {
    pub total_count: i32,
    pub total_count_change: i32,
    pub last_publication_handle: [u8; 16],
    pub current_count: i32,
    pub current_count_change: i32,
}

impl From<dust_dds::infrastructure::status::SubscriptionMatchedStatus> for DustDdsSubscriptionMatchedStatus {
    fn from(status: dust_dds::infrastructure::status::SubscriptionMatchedStatus) -> Self {
        Self {
            total_count: status.total_count,
            total_count_change: status.total_count_change,
            last_publication_handle: status.last_publication_handle.into(),
            current_count: status.current_count,
            current_count_change: status.current_count_change,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DustDdsDataReaderListener {
    pub listener_data: *mut std::ffi::c_void,
    pub on_data_available: Option<unsafe extern "C" fn(
        reader: Option<NonNull<DustDdsDataReader>>,
        listener_data: *mut std::ffi::c_void,
    )>,
    pub on_sample_rejected: Option<unsafe extern "C" fn(
        reader: Option<NonNull<DustDdsDataReader>>,
        status: DustDdsSampleRejectedStatus,
        listener_data: *mut std::ffi::c_void,
    )>,
    pub on_liveliness_changed: Option<unsafe extern "C" fn(
        reader: Option<NonNull<DustDdsDataReader>>,
        status: DustDdsLivelinessChangedStatus,
        listener_data: *mut std::ffi::c_void,
    )>,
    pub on_requested_deadline_missed: Option<unsafe extern "C" fn(
        reader: Option<NonNull<DustDdsDataReader>>,
        status: DustDdsRequestedDeadlineMissedStatus,
        listener_data: *mut std::ffi::c_void,
    )>,
    pub on_requested_incompatible_qos: Option<unsafe extern "C" fn(
        reader: Option<NonNull<DustDdsDataReader>>,
        status: DustDdsRequestedIncompatibleQosStatus,
        listener_data: *mut std::ffi::c_void,
    )>,
    pub on_subscription_matched: Option<unsafe extern "C" fn(
        reader: Option<NonNull<DustDdsDataReader>>,
        status: DustDdsSubscriptionMatchedStatus,
        listener_data: *mut std::ffi::c_void,
    )>,
    pub on_sample_lost: Option<unsafe extern "C" fn(
        reader: Option<NonNull<DustDdsDataReader>>,
        status: DustDdsSampleLostStatus,
        listener_data: *mut std::ffi::c_void,
    )>,
}

unsafe impl Send for DustDdsDataReaderListener {}
unsafe impl Sync for DustDdsDataReaderListener {}

struct CDataReaderListenerWrapper {
    listener: DustDdsDataReaderListener,
}

unsafe impl Send for CDataReaderListenerWrapper {}
unsafe impl Sync for CDataReaderListenerWrapper {}

impl dust_dds::subscription::data_reader_listener::DataReaderListener<DynamicData<'static>>
    for CDataReaderListenerWrapper
{
    fn on_data_available(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<DynamicData<'static>>,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_data_available) = self.listener.on_data_available {
            let mut reader_wrapper = DustDdsDataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(the_reader)
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DustDdsDataReader);
            unsafe {
                on_data_available(reader_ptr, self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_sample_rejected(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<DynamicData<'static>>,
        status: dust_dds::infrastructure::status::SampleRejectedStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_sample_rejected) = self.listener.on_sample_rejected {
            let mut reader_wrapper = DustDdsDataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(the_reader)
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DustDdsDataReader);
            unsafe {
                on_sample_rejected(reader_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_liveliness_changed(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<DynamicData<'static>>,
        status: dust_dds::infrastructure::status::LivelinessChangedStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_liveliness_changed) = self.listener.on_liveliness_changed {
            let mut reader_wrapper = DustDdsDataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(the_reader)
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DustDdsDataReader);
            unsafe {
                on_liveliness_changed(reader_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_requested_deadline_missed(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<DynamicData<'static>>,
        status: dust_dds::infrastructure::status::RequestedDeadlineMissedStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_requested_deadline_missed) = self.listener.on_requested_deadline_missed {
            let mut reader_wrapper = DustDdsDataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(the_reader)
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DustDdsDataReader);
            unsafe {
                on_requested_deadline_missed(reader_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_requested_incompatible_qos(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<DynamicData<'static>>,
        status: dust_dds::infrastructure::status::RequestedIncompatibleQosStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_requested_incompatible_qos) = self.listener.on_requested_incompatible_qos {
            let mut reader_wrapper = DustDdsDataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(the_reader)
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DustDdsDataReader);
            unsafe {
                on_requested_incompatible_qos(reader_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_subscription_matched(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<DynamicData<'static>>,
        status: dust_dds::infrastructure::status::SubscriptionMatchedStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_subscription_matched) = self.listener.on_subscription_matched {
            let mut reader_wrapper = DustDdsDataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(the_reader)
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DustDdsDataReader);
            unsafe {
                on_subscription_matched(reader_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_sample_lost(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<DynamicData<'static>>,
        status: dust_dds::infrastructure::status::SampleLostStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_sample_lost) = self.listener.on_sample_lost {
            let mut reader_wrapper = DustDdsDataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(the_reader)
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DustDdsDataReader);
            unsafe {
                on_sample_lost(reader_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
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
            for NoDataReaderListener {}

        match subscriber_ref.inner().create_datareader::<DynamicData<'static>>(
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

        match subscriber_ref.inner().create_datareader::<DynamicData<'static>>(
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
    NonNull::new(Box::into_raw(Box::new(DustDdsStatusCondition::new(condition))))
}
