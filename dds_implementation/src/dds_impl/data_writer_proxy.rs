use std::{marker::PhantomData, ops::DerefMut};

use crate::{
    dds_type::{DdsSerialize, LittleEndian},
    utils::{
        rtps_structure::RtpsStructure,
        shared_object::{DdsRwLock, DdsShared, DdsWeak},
    },
};
use dds_api::{
    builtin_topics::SubscriptionBuiltinTopicData,
    dcps_psm::{
        Duration, InstanceHandle, LivelinessLostStatus, OfferedDeadlineMissedStatus,
        OfferedIncompatibleQosStatus, PublicationMatchedStatus, StatusMask, Time,
    },
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::DataWriterQos,
    },
    publication::{data_writer::DataWriter, data_writer_listener::DataWriterListener},
    return_type::{DDSError, DDSResult},
};
use rtps_pim::{
    behavior::writer::writer::{RtpsWriterAttributes, RtpsWriterOperations},
    structure::{history_cache::RtpsHistoryCacheOperations, types::ChangeKind},
};

use super::{
    publisher_proxy::{PublisherAttributes, PublisherProxy},
    topic_proxy::{TopicAttributes, TopicProxy},
};

pub enum RtpsWriter<Rtps>
where
    Rtps: RtpsStructure,
{
    Stateless(Rtps::StatelessWriter),
    Stateful(Rtps::StatefulWriter),
}

impl<Rtps> RtpsWriter<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn try_as_stateless_writer(&mut self) -> DDSResult<&mut Rtps::StatelessWriter> {
        match self {
            RtpsWriter::Stateless(x) => Ok(x),
            RtpsWriter::Stateful(_) => Err(DDSError::PreconditionNotMet(
                "Not a stateless writer".to_string(),
            )),
        }
    }
    pub fn try_as_stateful_writer(&mut self) -> DDSResult<&mut Rtps::StatefulWriter> {
        match self {
            RtpsWriter::Stateless(_) => Err(DDSError::PreconditionNotMet(
                "Not a stateful writer".to_string(),
            )),
            RtpsWriter::Stateful(x) => Ok(x),
        }
    }
}

pub struct DataWriterAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub _qos: DataWriterQos,
    pub rtps_writer: DdsRwLock<RtpsWriter<Rtps>>,
    pub listener: DdsRwLock<Option<Box<dyn DataWriterListener + Send + Sync>>>,
    pub topic: DdsShared<TopicAttributes<Rtps>>,
    pub publisher: DdsWeak<PublisherAttributes<Rtps>>,
    pub status: DdsRwLock<PublicationMatchedStatus>,
}

impl<Rtps> DataWriterAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn new(
        qos: DataWriterQos,
        rtps_writer: RtpsWriter<Rtps>,
        listener: Option<Box<dyn DataWriterListener + Send + Sync>>,
        topic: DdsShared<TopicAttributes<Rtps>>,
        publisher: DdsWeak<PublisherAttributes<Rtps>>,
    ) -> Self {
        Self {
            _qos: qos,
            rtps_writer: DdsRwLock::new(rtps_writer),
            listener: DdsRwLock::new(listener),
            topic,
            publisher,
            status: DdsRwLock::new(PublicationMatchedStatus {
                total_count: 0,
                total_count_change: 0,
                last_subscription_handle: 0,
                current_count: 0,
                current_count_change: 0,
            }),
        }
    }
}

pub struct DataWriterProxy<Foo, Rtps>
where
    Rtps: RtpsStructure,
{
    data_writer_impl: DdsWeak<DataWriterAttributes<Rtps>>,
    phantom: PhantomData<Foo>,
}

// Not automatically derived because in that case it is only available if Foo: Clone
impl<Foo, Rtps> Clone for DataWriterProxy<Foo, Rtps>
where
    Rtps: RtpsStructure,
{
    fn clone(&self) -> Self {
        Self {
            data_writer_impl: self.data_writer_impl.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<Foo, Rtps> DataWriterProxy<Foo, Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn new(data_writer_impl: DdsWeak<DataWriterAttributes<Rtps>>) -> Self {
        Self {
            data_writer_impl,
            phantom: PhantomData,
        }
    }
}

impl<Foo, Rtps> AsRef<DdsWeak<DataWriterAttributes<Rtps>>> for DataWriterProxy<Foo, Rtps>
where
    Rtps: RtpsStructure,
{
    fn as_ref(&self) -> &DdsWeak<DataWriterAttributes<Rtps>> {
        &self.data_writer_impl
    }
}

impl<Foo, Rtps> DataWriter<Foo> for DataWriterProxy<Foo, Rtps>
where
    Foo: DdsSerialize,
    Rtps: RtpsStructure,
    Rtps::StatelessWriter: RtpsWriterOperations<DataType = Vec<u8>, ParameterListType = Vec<u8>>
        + RtpsWriterAttributes,
    Rtps::StatefulWriter: RtpsWriterOperations<DataType = Vec<u8>, ParameterListType = Vec<u8>>
        + RtpsWriterAttributes,
    <Rtps::StatelessWriter as RtpsWriterAttributes>::HistoryCacheType: RtpsHistoryCacheOperations<
        CacheChangeType = <Rtps::StatelessWriter as RtpsWriterOperations>::CacheChangeType,
    >,
    <Rtps::StatefulWriter as RtpsWriterAttributes>::HistoryCacheType: RtpsHistoryCacheOperations<
        CacheChangeType = <Rtps::StatefulWriter as RtpsWriterOperations>::CacheChangeType,
    >,
{
    type Publisher = PublisherProxy<Rtps>;
    type Topic = TopicProxy<Foo, Rtps>;

    fn register_instance(&self, _instance: Foo) -> DDSResult<Option<InstanceHandle>> {
        // let timestamp = self.publisher.get_participant()?.get_current_time()?;
        // self.register_instance_w_timestamp(instance, timestamp)
        todo!()
    }

    fn register_instance_w_timestamp(
        &self,
        _instance: Foo,
        _timestamp: Time,
    ) -> DDSResult<Option<InstanceHandle>> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.data_writer_impl)?)
        // .register_instance_w_timestamp(instance, timestamp)
        todo!()
    }

    fn unregister_instance(
        &self,
        _instance: Foo,
        _handle: Option<InstanceHandle>,
    ) -> DDSResult<()> {
        todo!()
        // let timestamp = self.publisher.get_participant()?.get_current_time()?;
        // self.unregister_instance_w_timestamp(instance, handle, timestamp)
    }

    fn unregister_instance_w_timestamp(
        &self,
        _instance: Foo,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DDSResult<()> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.data_writer_impl)?)
        // .unregister_instance_w_timestamp(instance, handle, timestamp)
        todo!()
    }

    fn get_key_value(&self, _key_holder: &mut Foo, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn lookup_instance(&self, _instance: &Foo) -> DDSResult<Option<InstanceHandle>> {
        todo!()
    }

    fn write(&self, _data: &Foo, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        // let timestamp = self.publisher.get_participant()?.get_current_time()?;
        // self.write_w_timestamp(data, handle, timestamp)
        todo!()
    }

    fn write_w_timestamp(
        &self,
        data: &Foo,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DDSResult<()> {
        let mut serialized_data = Vec::new();
        data.serialize::<_, LittleEndian>(&mut serialized_data)?;

        let data_writer_shared = self.data_writer_impl.upgrade()?;

        match data_writer_shared.rtps_writer.write_lock().deref_mut() {
            RtpsWriter::Stateless(rtps_writer) => {
                let change = rtps_writer.new_change(ChangeKind::Alive, serialized_data, vec![], 0);
                rtps_writer.writer_cache().add_change(change);
            }
            RtpsWriter::Stateful(rtps_writer) => {
                let change = rtps_writer.new_change(ChangeKind::Alive, serialized_data, vec![], 0);
                rtps_writer.writer_cache().add_change(change);
            }
        }

        Ok(())
    }

    fn dispose(&self, _data: Foo, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        todo!()
    }

    fn dispose_w_timestamp(
        &self,
        _data: Foo,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DDSResult<()> {
        todo!()
    }

    fn wait_for_acknowledgments(&self, _max_wait: Duration) -> DDSResult<()> {
        todo!()
    }

    fn get_liveliness_lost_status(&self, _status: &mut LivelinessLostStatus) -> DDSResult<()> {
        todo!()
    }

    fn get_offered_deadline_missed_status(
        &self,
        _status: &mut OfferedDeadlineMissedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_offered_incompatible_qos_status(
        &self,
        _status: &mut OfferedIncompatibleQosStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_publication_matched_status(
        &self,
        _status: &mut PublicationMatchedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_topic(&self) -> DDSResult<Self::Topic> {
        // Ok(self.topic.clone())
        todo!()
    }

    fn get_publisher(&self) -> DDSResult<Self::Publisher> {
        // Ok(self.publisher.clone())
        todo!()
    }

    fn assert_liveliness(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_matched_subscription_data(
        &self,
        _subscription_data: SubscriptionBuiltinTopicData,
        _subscription_handle: InstanceHandle,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_matched_subscriptions(
        &self,
        _subscription_handles: &mut [InstanceHandle],
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<Foo, Rtps> Entity for DataWriterProxy<Foo, Rtps>
where
    Rtps: RtpsStructure,
{
    type Qos = DataWriterQos;
    type Listener = Box<dyn DataWriterListener + Send + Sync>;

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).set_qos(qos)
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_qos()
        todo!()
    }

    fn set_listener(&self, listener: Option<Self::Listener>, _mask: StatusMask) -> DDSResult<()> {
        *self.as_ref().upgrade()?.listener.write_lock() = listener;
        Ok(())
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_listener()
        todo!()
    }

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_statuscondition()
        todo!()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_status_changes()
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).enable()
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_instance_handle()
        todo!()
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use dds_api::dcps_psm::{InstanceHandle, Time};
    use dds_api::infrastructure::qos::{DataWriterQos, TopicQos};
    use dds_api::publication::data_writer::DataWriter;
    use dds_api::return_type::DDSResult;
    use mockall::mock;
    use rtps_pim::behavior::types::Duration;
    use rtps_pim::behavior::writer::writer::{RtpsWriterAttributes, RtpsWriterOperations};
    use rtps_pim::structure::history_cache::RtpsHistoryCacheOperations;
    use rtps_pim::structure::types::{ChangeKind, SequenceNumber};

    use crate::dds_impl::topic_proxy::TopicAttributes;
    use crate::dds_type::{DdsSerialize, Endianness};
    use crate::utils::rtps_structure::RtpsStructure;
    use crate::utils::shared_object::{DdsShared, DdsWeak};

    use super::{DataWriterAttributes, DataWriterProxy, RtpsWriter};

    mock! {
        WriterHistoryCacheType {
            fn add_change_(&mut self, change: ());
            fn get_seq_num_max_(&self) -> Option<SequenceNumber>;
            fn get_seq_num_min_(&self) -> Option<SequenceNumber>;
        }
    }

    impl RtpsHistoryCacheOperations for MockWriterHistoryCacheType {
        type CacheChangeType = ();
        fn add_change(&mut self, change: ()) {
            self.add_change_(change)
        }

        fn remove_change<F>(&mut self, _f: F)
        where
            F: FnMut(&Self::CacheChangeType) -> bool,
        {
            todo!()
        }

        fn get_seq_num_min(&self) -> Option<SequenceNumber> {
            self.get_seq_num_min_()
        }

        fn get_seq_num_max(&self) -> Option<SequenceNumber> {
            self.get_seq_num_max_()
        }
    }

    mock! {
        Writer {}

        impl RtpsWriterOperations for Writer {
            type DataType = Vec<u8>;
            type ParameterListType = Vec<u8>;
            type CacheChangeType = ();

            fn new_change(
                &mut self,
                kind: ChangeKind,
                data: Vec<u8>,
                inline_qos: Vec<u8>,
                handle: InstanceHandle,
            );
        }

        impl RtpsWriterAttributes for Writer {
            type HistoryCacheType = MockWriterHistoryCacheType;

            fn push_mode(&self) -> bool;
            fn heartbeat_period(&self) -> Duration;
            fn nack_response_delay(&self) -> Duration;
            fn nack_suppression_duration(&self) -> Duration;
            fn last_change_sequence_number(&self) -> SequenceNumber;
            fn data_max_size_serialized(&self) -> Option<i32>;
            fn writer_cache(&mut self) -> &mut MockWriterHistoryCacheType;
        }
    }

    mock! {
        Rtps {}

        impl RtpsStructure for Rtps {
            type Group           = ();
            type Participant     = ();
            type StatelessWriter = MockWriter;
            type StatefulWriter  = MockWriter;
            type StatelessReader = ();
            type StatefulReader  = ();
        }
    }

    struct MockFoo {}

    impl DdsSerialize for MockFoo {
        fn serialize<W: Write, E: Endianness>(&self, _writer: W) -> DDSResult<()> {
            Ok(())
        }
    }

    #[test]
    fn try_as_stateful_writer_on_stateful_is_ok() {
        assert!(RtpsWriter::<MockRtps>::Stateful(MockWriter::new())
            .try_as_stateful_writer()
            .is_ok());
    }

    #[test]
    fn try_as_stateful_writer_on_stateless_is_err() {
        assert!(RtpsWriter::<MockRtps>::Stateless(MockWriter::new())
            .try_as_stateful_writer()
            .is_err());
    }

    #[test]
    fn try_as_stateless_writer_on_stateless_is_ok() {
        assert!(RtpsWriter::<MockRtps>::Stateless(MockWriter::new())
            .try_as_stateless_writer()
            .is_ok());
    }

    #[test]
    fn try_as_stateless_writer_on_stateful_is_err() {
        assert!(RtpsWriter::<MockRtps>::Stateful(MockWriter::new())
            .try_as_stateless_writer()
            .is_err());
    }

    #[test]
    fn write_w_timestamp_stateless() {
        let mut mock_writer_history_cache = MockWriterHistoryCacheType::new();
        mock_writer_history_cache
            .expect_add_change_()
            .once()
            .return_const(());

        let mut mock_writer = MockWriter::new();
        mock_writer.expect_new_change().once().return_const(());
        mock_writer
            .expect_writer_cache()
            .once()
            .return_var(mock_writer_history_cache);

        let dummy_topic = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            "",
            "",
            DdsWeak::new(),
        ));

        let data_writer: DataWriterAttributes<MockRtps> = DataWriterAttributes::new(
            DataWriterQos::default(),
            RtpsWriter::Stateless(mock_writer),
            None,
            dummy_topic,
            DdsWeak::new(),
        );

        let shared_data_writer = DdsShared::new(data_writer);
        let weak_data_writer = shared_data_writer.downgrade();

        let data_writer_proxy = DataWriterProxy::<MockFoo, MockRtps>::new(weak_data_writer);
        data_writer_proxy
            .write_w_timestamp(&MockFoo {}, None, Time { sec: 0, nanosec: 0 })
            .unwrap();
    }

    #[test]
    fn write_w_timestamp_stateful() {
        let mut mock_writer_history_cache = MockWriterHistoryCacheType::new();
        mock_writer_history_cache
            .expect_add_change_()
            .once()
            .return_const(());

        let mut mock_writer = MockWriter::new();
        mock_writer.expect_new_change().once().return_const(());
        mock_writer
            .expect_writer_cache()
            .once()
            .return_var(mock_writer_history_cache);

        let dummy_topic = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            "",
            "",
            DdsWeak::new(),
        ));

        let data_writer: DataWriterAttributes<MockRtps> = DataWriterAttributes::new(
            DataWriterQos::default(),
            RtpsWriter::Stateful(mock_writer),
            None,
            dummy_topic,
            DdsWeak::new(),
        );

        let shared_data_writer = DdsShared::new(data_writer);
        let weak_data_writer = shared_data_writer.downgrade();

        let data_writer_proxy = DataWriterProxy::<MockFoo, MockRtps>::new(weak_data_writer);
        data_writer_proxy
            .write_w_timestamp(&MockFoo {}, None, Time { sec: 0, nanosec: 0 })
            .unwrap();
    }
}
