use dust_dds::xtypes::dynamic_type::DynamicData;
use std::ptr::NonNull;

use crate::{
    infrastructure::status::{
        InconsistentTopicStatus, LivelinessChangedStatus, LivelinessLostStatus,
        OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    publication::data_writer::DataWriter,
    subscription::{data_reader::DataReader, subscriber::Subscriber},
    topic_definition::topic::Topic,
};

// =========================================================================
// DataReaderListener
// =========================================================================

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DataReaderListener {
    pub listener_data: *mut std::ffi::c_void,
    pub on_data_available: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_sample_rejected: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            status: SampleRejectedStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_liveliness_changed: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            status: LivelinessChangedStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_requested_deadline_missed: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            status: RequestedDeadlineMissedStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_requested_incompatible_qos: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            status: RequestedIncompatibleQosStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_subscription_matched: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            status: SubscriptionMatchedStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_sample_lost: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            status: SampleLostStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
}

unsafe impl Send for DataReaderListener {}
unsafe impl Sync for DataReaderListener {}

pub(crate) struct CDataReaderListenerWrapper {
    pub listener: DataReaderListener,
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
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(the_reader),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
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
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(the_reader),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
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
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(the_reader),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
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
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(the_reader),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
            unsafe {
                on_requested_deadline_missed(
                    reader_ptr,
                    status.into(),
                    self.listener.listener_data,
                );
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
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(the_reader),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
            unsafe {
                on_requested_incompatible_qos(
                    reader_ptr,
                    status.into(),
                    self.listener.listener_data,
                );
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
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(the_reader),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
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
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(the_reader),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
            unsafe {
                on_sample_lost(reader_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }
}

// =========================================================================
// DataWriterListener
// =========================================================================

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DataWriterListener {
    pub listener_data: *mut std::ffi::c_void,
    pub on_liveliness_lost: Option<
        unsafe extern "C" fn(
            writer: Option<NonNull<DataWriter>>,
            status: LivelinessLostStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_offered_deadline_missed: Option<
        unsafe extern "C" fn(
            writer: Option<NonNull<DataWriter>>,
            status: OfferedDeadlineMissedStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_offered_incompatible_qos: Option<
        unsafe extern "C" fn(
            writer: Option<NonNull<DataWriter>>,
            status: OfferedIncompatibleQosStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_publication_matched: Option<
        unsafe extern "C" fn(
            writer: Option<NonNull<DataWriter>>,
            status: PublicationMatchedStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
}

unsafe impl Send for DataWriterListener {}
unsafe impl Sync for DataWriterListener {}

pub(crate) struct CDataWriterListenerWrapper {
    pub listener: DataWriterListener,
}

unsafe impl Send for CDataWriterListenerWrapper {}
unsafe impl Sync for CDataWriterListenerWrapper {}

impl dust_dds::publication::data_writer_listener::DataWriterListener<DynamicData<'static>>
    for CDataWriterListenerWrapper
{
    fn on_liveliness_lost(
        &mut self,
        the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<DynamicData<'static>>,
        status: dust_dds::infrastructure::status::LivelinessLostStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_liveliness_lost) = self.listener.on_liveliness_lost {
            let mut writer_wrapper = DataWriter::new(
                dust_dds::publication::data_writer::DataWriter::from(the_writer),
            );
            let writer_ptr = NonNull::new(&mut writer_wrapper as *mut DataWriter);
            unsafe {
                on_liveliness_lost(writer_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_offered_deadline_missed(
        &mut self,
        the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<DynamicData<'static>>,
        status: dust_dds::infrastructure::status::OfferedDeadlineMissedStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_offered_deadline_missed) = self.listener.on_offered_deadline_missed {
            let mut writer_wrapper = DataWriter::new(
                dust_dds::publication::data_writer::DataWriter::from(the_writer),
            );
            let writer_ptr = NonNull::new(&mut writer_wrapper as *mut DataWriter);
            unsafe {
                on_offered_deadline_missed(writer_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_offered_incompatible_qos(
        &mut self,
        the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<DynamicData<'static>>,
        status: dust_dds::infrastructure::status::OfferedIncompatibleQosStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_offered_incompatible_qos) = self.listener.on_offered_incompatible_qos {
            let mut writer_wrapper = DataWriter::new(
                dust_dds::publication::data_writer::DataWriter::from(the_writer),
            );
            let writer_ptr = NonNull::new(&mut writer_wrapper as *mut DataWriter);
            unsafe {
                on_offered_incompatible_qos(writer_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_publication_matched(
        &mut self,
        the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<DynamicData<'static>>,
        status: dust_dds::infrastructure::status::PublicationMatchedStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_publication_matched) = self.listener.on_publication_matched {
            let mut writer_wrapper = DataWriter::new(
                dust_dds::publication::data_writer::DataWriter::from(the_writer),
            );
            let writer_ptr = NonNull::new(&mut writer_wrapper as *mut DataWriter);
            unsafe {
                on_publication_matched(writer_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }
}

// =========================================================================
// TopicListener
// =========================================================================

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TopicListener {
    pub listener_data: *mut std::ffi::c_void,
    pub on_inconsistent_topic: Option<
        unsafe extern "C" fn(
            topic: Option<NonNull<Topic>>,
            status: InconsistentTopicStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
}

unsafe impl Send for TopicListener {}
unsafe impl Sync for TopicListener {}

pub(crate) struct CTopicListenerWrapper {
    pub listener: TopicListener,
}

unsafe impl Send for CTopicListenerWrapper {}
unsafe impl Sync for CTopicListenerWrapper {}

impl dust_dds::topic_definition::topic_listener::TopicListener for CTopicListenerWrapper {
    fn on_inconsistent_topic(
        &mut self,
        the_topic: dust_dds::dds_async::topic::TopicAsync,
        status: dust_dds::infrastructure::status::InconsistentTopicStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_inconsistent_topic) = self.listener.on_inconsistent_topic {
            let mut topic_wrapper =
                Topic::new(dust_dds::topic_definition::topic::Topic::from(the_topic));
            let topic_ptr = NonNull::new(&mut topic_wrapper as *mut Topic);
            unsafe {
                on_inconsistent_topic(topic_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }
}

// =========================================================================
// PublisherListener
// =========================================================================

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PublisherListener {
    pub listener_data: *mut std::ffi::c_void,
    pub on_liveliness_lost: Option<
        unsafe extern "C" fn(
            writer: Option<NonNull<DataWriter>>,
            status: LivelinessLostStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_offered_deadline_missed: Option<
        unsafe extern "C" fn(
            writer: Option<NonNull<DataWriter>>,
            status: OfferedDeadlineMissedStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_offered_incompatible_qos: Option<
        unsafe extern "C" fn(
            writer: Option<NonNull<DataWriter>>,
            status: OfferedIncompatibleQosStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_publication_matched: Option<
        unsafe extern "C" fn(
            writer: Option<NonNull<DataWriter>>,
            status: PublicationMatchedStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
}

unsafe impl Send for PublisherListener {}
unsafe impl Sync for PublisherListener {}

pub(crate) struct CPublisherListenerWrapper {
    pub listener: PublisherListener,
}

unsafe impl Send for CPublisherListenerWrapper {}
unsafe impl Sync for CPublisherListenerWrapper {}

impl dust_dds::publication::publisher_listener::PublisherListener for CPublisherListenerWrapper {
    fn on_liveliness_lost(
        &mut self,
        the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<()>,
        status: dust_dds::infrastructure::status::LivelinessLostStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_liveliness_lost) = self.listener.on_liveliness_lost {
            let writer_async_dynamic: dust_dds::dds_async::data_writer::DataWriterAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_writer) };
            let mut writer_wrapper = DataWriter::new(
                dust_dds::publication::data_writer::DataWriter::from(writer_async_dynamic),
            );
            let writer_ptr = NonNull::new(&mut writer_wrapper as *mut DataWriter);
            unsafe {
                on_liveliness_lost(writer_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_offered_deadline_missed(
        &mut self,
        the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<()>,
        status: dust_dds::infrastructure::status::OfferedDeadlineMissedStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_offered_deadline_missed) = self.listener.on_offered_deadline_missed {
            let writer_async_dynamic: dust_dds::dds_async::data_writer::DataWriterAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_writer) };
            let mut writer_wrapper = DataWriter::new(
                dust_dds::publication::data_writer::DataWriter::from(writer_async_dynamic),
            );
            let writer_ptr = NonNull::new(&mut writer_wrapper as *mut DataWriter);
            unsafe {
                on_offered_deadline_missed(writer_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_offered_incompatible_qos(
        &mut self,
        the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<()>,
        status: dust_dds::infrastructure::status::OfferedIncompatibleQosStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_offered_incompatible_qos) = self.listener.on_offered_incompatible_qos {
            let writer_async_dynamic: dust_dds::dds_async::data_writer::DataWriterAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_writer) };
            let mut writer_wrapper = DataWriter::new(
                dust_dds::publication::data_writer::DataWriter::from(writer_async_dynamic),
            );
            let writer_ptr = NonNull::new(&mut writer_wrapper as *mut DataWriter);
            unsafe {
                on_offered_incompatible_qos(writer_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_publication_matched(
        &mut self,
        the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<()>,
        status: dust_dds::infrastructure::status::PublicationMatchedStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_publication_matched) = self.listener.on_publication_matched {
            let writer_async_dynamic: dust_dds::dds_async::data_writer::DataWriterAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_writer) };
            let mut writer_wrapper = DataWriter::new(
                dust_dds::publication::data_writer::DataWriter::from(writer_async_dynamic),
            );
            let writer_ptr = NonNull::new(&mut writer_wrapper as *mut DataWriter);
            unsafe {
                on_publication_matched(writer_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }
}

// =========================================================================
// SubscriberListener
// =========================================================================

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SubscriberListener {
    pub listener_data: *mut std::ffi::c_void,
    pub on_data_on_readers: Option<
        unsafe extern "C" fn(
            subscriber: Option<NonNull<Subscriber>>,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_data_available: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_sample_rejected: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            status: SampleRejectedStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_liveliness_changed: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            status: LivelinessChangedStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_requested_deadline_missed: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            status: RequestedDeadlineMissedStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_requested_incompatible_qos: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            status: RequestedIncompatibleQosStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_subscription_matched: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            status: SubscriptionMatchedStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_sample_lost: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            status: SampleLostStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
}

unsafe impl Send for SubscriberListener {}
unsafe impl Sync for SubscriberListener {}

pub(crate) struct CSubscriberListenerWrapper {
    pub listener: SubscriberListener,
}

unsafe impl Send for CSubscriberListenerWrapper {}
unsafe impl Sync for CSubscriberListenerWrapper {}

impl dust_dds::subscription::subscriber_listener::SubscriberListener
    for CSubscriberListenerWrapper
{
    fn on_data_on_readers(
        &mut self,
        the_subscriber: dust_dds::dds_async::subscriber::SubscriberAsync,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_data_on_readers) = self.listener.on_data_on_readers {
            let mut subscriber_wrapper = Subscriber::new(
                dust_dds::subscription::subscriber::Subscriber::from(the_subscriber),
            );
            let subscriber_ptr = NonNull::new(&mut subscriber_wrapper as *mut Subscriber);
            unsafe {
                on_data_on_readers(subscriber_ptr, self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_data_available(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_data_available) = self.listener.on_data_available {
            let reader_async_dynamic: dust_dds::dds_async::data_reader::DataReaderAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_reader) };
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(reader_async_dynamic),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
            unsafe {
                on_data_available(reader_ptr, self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_sample_rejected(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::SampleRejectedStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_sample_rejected) = self.listener.on_sample_rejected {
            let reader_async_dynamic: dust_dds::dds_async::data_reader::DataReaderAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_reader) };
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(reader_async_dynamic),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
            unsafe {
                on_sample_rejected(reader_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_liveliness_changed(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::LivelinessChangedStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_liveliness_changed) = self.listener.on_liveliness_changed {
            let reader_async_dynamic: dust_dds::dds_async::data_reader::DataReaderAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_reader) };
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(reader_async_dynamic),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
            unsafe {
                on_liveliness_changed(reader_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_requested_deadline_missed(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::RequestedDeadlineMissedStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_requested_deadline_missed) = self.listener.on_requested_deadline_missed {
            let reader_async_dynamic: dust_dds::dds_async::data_reader::DataReaderAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_reader) };
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(reader_async_dynamic),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
            unsafe {
                on_requested_deadline_missed(
                    reader_ptr,
                    status.into(),
                    self.listener.listener_data,
                );
            }
        }
        std::future::ready(())
    }

    fn on_requested_incompatible_qos(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::RequestedIncompatibleQosStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_requested_incompatible_qos) = self.listener.on_requested_incompatible_qos {
            let reader_async_dynamic: dust_dds::dds_async::data_reader::DataReaderAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_reader) };
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(reader_async_dynamic),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
            unsafe {
                on_requested_incompatible_qos(
                    reader_ptr,
                    status.into(),
                    self.listener.listener_data,
                );
            }
        }
        std::future::ready(())
    }

    fn on_subscription_matched(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::SubscriptionMatchedStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_subscription_matched) = self.listener.on_subscription_matched {
            let reader_async_dynamic: dust_dds::dds_async::data_reader::DataReaderAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_reader) };
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(reader_async_dynamic),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
            unsafe {
                on_subscription_matched(reader_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_sample_lost(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::SampleLostStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_sample_lost) = self.listener.on_sample_lost {
            let reader_async_dynamic: dust_dds::dds_async::data_reader::DataReaderAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_reader) };
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(reader_async_dynamic),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
            unsafe {
                on_sample_lost(reader_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }
}

// =========================================================================
// DomainParticipantListener
// =========================================================================

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DomainParticipantListener {
    pub listener_data: *mut std::ffi::c_void,
    pub on_inconsistent_topic: Option<
        unsafe extern "C" fn(
            topic: Option<NonNull<Topic>>,
            status: InconsistentTopicStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_liveliness_lost: Option<
        unsafe extern "C" fn(
            writer: Option<NonNull<DataWriter>>,
            status: LivelinessLostStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_offered_deadline_missed: Option<
        unsafe extern "C" fn(
            writer: Option<NonNull<DataWriter>>,
            status: OfferedDeadlineMissedStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_offered_incompatible_qos: Option<
        unsafe extern "C" fn(
            writer: Option<NonNull<DataWriter>>,
            status: OfferedIncompatibleQosStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_sample_lost: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            status: SampleLostStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_data_available: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_sample_rejected: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            status: SampleRejectedStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_liveliness_changed: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            status: LivelinessChangedStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_requested_deadline_missed: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            status: RequestedDeadlineMissedStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_requested_incompatible_qos: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            status: RequestedIncompatibleQosStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_publication_matched: Option<
        unsafe extern "C" fn(
            writer: Option<NonNull<DataWriter>>,
            status: PublicationMatchedStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
    pub on_subscription_matched: Option<
        unsafe extern "C" fn(
            reader: Option<NonNull<DataReader>>,
            status: SubscriptionMatchedStatus,
            listener_data: *mut std::ffi::c_void,
        ),
    >,
}

unsafe impl Send for DomainParticipantListener {}
unsafe impl Sync for DomainParticipantListener {}

pub(crate) struct CDomainParticipantListenerWrapper {
    pub listener: DomainParticipantListener,
}

unsafe impl Send for CDomainParticipantListenerWrapper {}
unsafe impl Sync for CDomainParticipantListenerWrapper {}

impl dust_dds::domain::domain_participant_listener::DomainParticipantListener
    for CDomainParticipantListenerWrapper
{
    fn on_inconsistent_topic(
        &mut self,
        the_topic: dust_dds::dds_async::topic::TopicAsync,
        status: dust_dds::infrastructure::status::InconsistentTopicStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_inconsistent_topic) = self.listener.on_inconsistent_topic {
            let mut topic_wrapper =
                Topic::new(dust_dds::topic_definition::topic::Topic::from(the_topic));
            let topic_ptr = NonNull::new(&mut topic_wrapper as *mut Topic);
            unsafe {
                on_inconsistent_topic(topic_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_liveliness_lost(
        &mut self,
        the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<()>,
        status: dust_dds::infrastructure::status::LivelinessLostStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_liveliness_lost) = self.listener.on_liveliness_lost {
            let writer_async_dynamic: dust_dds::dds_async::data_writer::DataWriterAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_writer) };
            let mut writer_wrapper = DataWriter::new(
                dust_dds::publication::data_writer::DataWriter::from(writer_async_dynamic),
            );
            let writer_ptr = NonNull::new(&mut writer_wrapper as *mut DataWriter);
            unsafe {
                on_liveliness_lost(writer_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_offered_deadline_missed(
        &mut self,
        the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<()>,
        status: dust_dds::infrastructure::status::OfferedDeadlineMissedStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_offered_deadline_missed) = self.listener.on_offered_deadline_missed {
            let writer_async_dynamic: dust_dds::dds_async::data_writer::DataWriterAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_writer) };
            let mut writer_wrapper = DataWriter::new(
                dust_dds::publication::data_writer::DataWriter::from(writer_async_dynamic),
            );
            let writer_ptr = NonNull::new(&mut writer_wrapper as *mut DataWriter);
            unsafe {
                on_offered_deadline_missed(writer_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_offered_incompatible_qos(
        &mut self,
        the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<()>,
        status: dust_dds::infrastructure::status::OfferedIncompatibleQosStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_offered_incompatible_qos) = self.listener.on_offered_incompatible_qos {
            let writer_async_dynamic: dust_dds::dds_async::data_writer::DataWriterAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_writer) };
            let mut writer_wrapper = DataWriter::new(
                dust_dds::publication::data_writer::DataWriter::from(writer_async_dynamic),
            );
            let writer_ptr = NonNull::new(&mut writer_wrapper as *mut DataWriter);
            unsafe {
                on_offered_incompatible_qos(writer_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_sample_lost(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::SampleLostStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_sample_lost) = self.listener.on_sample_lost {
            let reader_async_dynamic: dust_dds::dds_async::data_reader::DataReaderAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_reader) };
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(reader_async_dynamic),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
            unsafe {
                on_sample_lost(reader_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_data_available(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_data_available) = self.listener.on_data_available {
            let reader_async_dynamic: dust_dds::dds_async::data_reader::DataReaderAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_reader) };
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(reader_async_dynamic),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
            unsafe {
                on_data_available(reader_ptr, self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_sample_rejected(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::SampleRejectedStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_sample_rejected) = self.listener.on_sample_rejected {
            let reader_async_dynamic: dust_dds::dds_async::data_reader::DataReaderAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_reader) };
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(reader_async_dynamic),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
            unsafe {
                on_sample_rejected(reader_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_liveliness_changed(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::LivelinessChangedStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_liveliness_changed) = self.listener.on_liveliness_changed {
            let reader_async_dynamic: dust_dds::dds_async::data_reader::DataReaderAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_reader) };
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(reader_async_dynamic),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
            unsafe {
                on_liveliness_changed(reader_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_requested_deadline_missed(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::RequestedDeadlineMissedStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_requested_deadline_missed) = self.listener.on_requested_deadline_missed {
            let reader_async_dynamic: dust_dds::dds_async::data_reader::DataReaderAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_reader) };
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(reader_async_dynamic),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
            unsafe {
                on_requested_deadline_missed(
                    reader_ptr,
                    status.into(),
                    self.listener.listener_data,
                );
            }
        }
        std::future::ready(())
    }

    fn on_requested_incompatible_qos(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::RequestedIncompatibleQosStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_requested_incompatible_qos) = self.listener.on_requested_incompatible_qos {
            let reader_async_dynamic: dust_dds::dds_async::data_reader::DataReaderAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_reader) };
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(reader_async_dynamic),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
            unsafe {
                on_requested_incompatible_qos(
                    reader_ptr,
                    status.into(),
                    self.listener.listener_data,
                );
            }
        }
        std::future::ready(())
    }

    fn on_publication_matched(
        &mut self,
        the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<()>,
        status: dust_dds::infrastructure::status::PublicationMatchedStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_publication_matched) = self.listener.on_publication_matched {
            let writer_async_dynamic: dust_dds::dds_async::data_writer::DataWriterAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_writer) };
            let mut writer_wrapper = DataWriter::new(
                dust_dds::publication::data_writer::DataWriter::from(writer_async_dynamic),
            );
            let writer_ptr = NonNull::new(&mut writer_wrapper as *mut DataWriter);
            unsafe {
                on_publication_matched(writer_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }

    fn on_subscription_matched(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::SubscriptionMatchedStatus,
    ) -> impl std::future::Future<Output = ()> + Send {
        if let Some(on_subscription_matched) = self.listener.on_subscription_matched {
            let reader_async_dynamic: dust_dds::dds_async::data_reader::DataReaderAsync<
                DynamicData<'static>,
            > = unsafe { std::mem::transmute(the_reader) };
            let mut reader_wrapper = DataReader::new(
                dust_dds::subscription::data_reader::DataReader::from(reader_async_dynamic),
            );
            let reader_ptr = NonNull::new(&mut reader_wrapper as *mut DataReader);
            unsafe {
                on_subscription_matched(reader_ptr, status.into(), self.listener.listener_data);
            }
        }
        std::future::ready(())
    }
}
