use std::{collections::HashMap, marker::PhantomData, ops::DerefMut};

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
    domain::domain_participant::DomainParticipant,
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::DataWriterQos,
    },
    publication::{
        data_writer::DataWriter, data_writer_listener::DataWriterListener, publisher::Publisher,
    },
    return_type::{DdsError, DdsResult},
};
use rtps_pim::{
    behavior::writer::writer::{RtpsWriterAttributes, RtpsWriterOperations},
    structure::{
        cache_change::RtpsCacheChangeAttributes,
        history_cache::RtpsHistoryCacheOperations,
        types::{ChangeKind, SequenceNumber},
    },
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
    pub fn try_as_stateless_writer(&mut self) -> DdsResult<&mut Rtps::StatelessWriter> {
        match self {
            RtpsWriter::Stateless(x) => Ok(x),
            RtpsWriter::Stateful(_) => Err(DdsError::PreconditionNotMet(
                "Not a stateless writer".to_string(),
            )),
        }
    }
    pub fn try_as_stateful_writer(&mut self) -> DdsResult<&mut Rtps::StatefulWriter> {
        match self {
            RtpsWriter::Stateless(_) => Err(DdsError::PreconditionNotMet(
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
    pub sample_info: DdsRwLock<HashMap<SequenceNumber, Time>>,
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
            sample_info: DdsRwLock::new(HashMap::new()),
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
{
    type Publisher = PublisherProxy<Rtps>;
    type Topic = TopicProxy<Foo, Rtps>;

    fn register_instance(&self, _instance: Foo) -> DdsResult<Option<InstanceHandle>> {
        // let timestamp = self.publisher.get_participant()?.get_current_time()?;
        // self.register_instance_w_timestamp(instance, timestamp)
        todo!()
    }

    fn register_instance_w_timestamp(
        &self,
        _instance: Foo,
        _timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.data_writer_impl)?)
        // .register_instance_w_timestamp(instance, timestamp)
        todo!()
    }

    fn unregister_instance(
        &self,
        _instance: Foo,
        _handle: Option<InstanceHandle>,
    ) -> DdsResult<()> {
        todo!()
        // let timestamp = self.publisher.get_participant()?.get_current_time()?;
        // self.unregister_instance_w_timestamp(instance, handle, timestamp)
    }

    fn unregister_instance_w_timestamp(
        &self,
        _instance: Foo,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DdsResult<()> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.data_writer_impl)?)
        // .unregister_instance_w_timestamp(instance, handle, timestamp)
        todo!()
    }

    fn get_key_value(&self, _key_holder: &mut Foo, _handle: InstanceHandle) -> DdsResult<()> {
        todo!()
    }

    fn lookup_instance(&self, _instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        todo!()
    }

    fn write(&self, data: &Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        let timestamp = self
            .get_publisher()?
            .get_participant()?
            .get_current_time()?;
        self.write_w_timestamp(data, handle, timestamp)
    }

    fn write_w_timestamp(
        &self,
        data: &Foo,
        _handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let mut serialized_data = Vec::new();
        data.serialize::<_, LittleEndian>(&mut serialized_data)?;

        let data_writer_shared = self.data_writer_impl.upgrade()?;

        match data_writer_shared.rtps_writer.write_lock().deref_mut() {
            RtpsWriter::Stateless(rtps_writer) => {
                let change = rtps_writer.new_change(ChangeKind::Alive, serialized_data, vec![], 0);
                data_writer_shared
                    .sample_info
                    .write_lock()
                    .insert(change.sequence_number(), timestamp);
                rtps_writer.writer_cache().add_change(change);
            }
            RtpsWriter::Stateful(rtps_writer) => {
                let change = rtps_writer.new_change(ChangeKind::Alive, serialized_data, vec![], 0);
                data_writer_shared
                    .sample_info
                    .write_lock()
                    .insert(change.sequence_number(), timestamp);
                rtps_writer.writer_cache().add_change(change);
            }
        }

        Ok(())
    }

    fn dispose(&self, _data: Foo, _handle: Option<InstanceHandle>) -> DdsResult<()> {
        todo!()
    }

    fn dispose_w_timestamp(
        &self,
        _data: Foo,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DdsResult<()> {
        todo!()
    }

    fn wait_for_acknowledgments(&self, _max_wait: Duration) -> DdsResult<()> {
        todo!()
    }

    fn get_liveliness_lost_status(&self, _status: &mut LivelinessLostStatus) -> DdsResult<()> {
        todo!()
    }

    fn get_offered_deadline_missed_status(
        &self,
        _status: &mut OfferedDeadlineMissedStatus,
    ) -> DdsResult<()> {
        todo!()
    }

    fn get_offered_incompatible_qos_status(
        &self,
        _status: &mut OfferedIncompatibleQosStatus,
    ) -> DdsResult<()> {
        todo!()
    }

    fn get_publication_matched_status(
        &self,
        _status: &mut PublicationMatchedStatus,
    ) -> DdsResult<()> {
        todo!()
    }

    fn get_topic(&self) -> DdsResult<Self::Topic> {
        // Ok(self.topic.clone())
        todo!()
    }

    fn get_publisher(&self) -> DdsResult<Self::Publisher> {
        // Ok(self.publisher.clone())
        todo!()
    }

    fn assert_liveliness(&self) -> DdsResult<()> {
        todo!()
    }

    fn get_matched_subscription_data(
        &self,
        _subscription_data: SubscriptionBuiltinTopicData,
        _subscription_handle: InstanceHandle,
    ) -> DdsResult<()> {
        todo!()
    }

    fn get_matched_subscriptions(
        &self,
        _subscription_handles: &mut [InstanceHandle],
    ) -> DdsResult<()> {
        todo!()
    }
}

impl<Foo, Rtps> Entity for DataWriterProxy<Foo, Rtps>
where
    Rtps: RtpsStructure,
{
    type Qos = DataWriterQos;
    type Listener = Box<dyn DataWriterListener + Send + Sync>;

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DdsResult<()> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).set_qos(qos)
        todo!()
    }

    fn get_qos(&self) -> DdsResult<Self::Qos> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_qos()
        todo!()
    }

    fn set_listener(&self, listener: Option<Self::Listener>, _mask: StatusMask) -> DdsResult<()> {
        *self.as_ref().upgrade()?.listener.write_lock() = listener;
        Ok(())
    }

    fn get_listener(&self) -> DdsResult<Option<Self::Listener>> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_listener()
        todo!()
    }

    fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_statuscondition()
        todo!()
    }

    fn get_status_changes(&self) -> DdsResult<StatusMask> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_status_changes()
        todo!()
    }

    fn enable(&self) -> DdsResult<()> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).enable()
        todo!()
    }

    fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_instance_handle()
        todo!()
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use dds_api::infrastructure::qos::TopicQos;

    use crate::{
        dds_type::Endianness,
        test_utils::{
            mock_rtps::MockRtps, mock_rtps_cache_change::MockRtpsCacheChange,
            mock_rtps_history_cache::MockRtpsHistoryCache,
            mock_rtps_stateful_writer::MockRtpsStatefulWriter,
            mock_rtps_stateless_writer::MockRtpsStatelessWriter,
        },
    };

    use super::*;

    struct MockFoo {}

    impl DdsSerialize for MockFoo {
        fn serialize<W: Write, E: Endianness>(&self, _writer: W) -> DdsResult<()> {
            Ok(())
        }
    }

    #[test]
    fn try_as_stateful_writer_on_stateful_is_ok() {
        assert!(
            RtpsWriter::<MockRtps>::Stateful(MockRtpsStatefulWriter::new())
                .try_as_stateful_writer()
                .is_ok()
        );
    }

    #[test]
    fn try_as_stateful_writer_on_stateless_is_err() {
        assert!(
            RtpsWriter::<MockRtps>::Stateless(MockRtpsStatelessWriter::new())
                .try_as_stateful_writer()
                .is_err()
        );
    }

    #[test]
    fn try_as_stateless_writer_on_stateless_is_ok() {
        assert!(
            RtpsWriter::<MockRtps>::Stateless(MockRtpsStatelessWriter::new())
                .try_as_stateless_writer()
                .is_ok()
        );
    }

    #[test]
    fn try_as_stateless_writer_on_stateful_is_err() {
        assert!(
            RtpsWriter::<MockRtps>::Stateful(MockRtpsStatefulWriter::new())
                .try_as_stateless_writer()
                .is_err()
        );
    }

    #[test]
    fn write_w_timestamp_stateless() {
        let mut mock_writer_history_cache = MockRtpsHistoryCache::new();
        mock_writer_history_cache
            .expect_add_change_()
            .once()
            .return_const(());

        let mut mock_writer = MockRtpsStatelessWriter::new();
        mock_writer
            .expect_new_change()
            .once()
            .return_once(|_, _, _, _| {
                let mut mock_cache_change = MockRtpsCacheChange::new();
                mock_cache_change.expect_sequence_number().return_const(1);
                mock_cache_change
            });
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
        let mut mock_writer_history_cache = MockRtpsHistoryCache::new();
        mock_writer_history_cache
            .expect_add_change_()
            .once()
            .return_const(());

        let mut mock_writer = MockRtpsStatefulWriter::new();
        mock_writer
            .expect_new_change()
            .once()
            .return_once(|_, _, _, _| {
                let mut mock_cache_change = MockRtpsCacheChange::new();
                mock_cache_change.expect_sequence_number().return_const(1);
                mock_cache_change
            });
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
