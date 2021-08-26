use rust_dds_api::{
    dcps_psm::{
        InstanceHandle, InstanceStateKind, SampleLostStatus, SampleStateKind, StatusMask,
        ViewStateKind,
    },
    domain::domain_participant::DomainParticipant,
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataReaderQos, SubscriberQos, TopicQos},
        qos_policy::ReliabilityQosPolicyKind,
    },
    return_type::DDSResult,
    subscription::{
        data_reader::AnyDataReader, data_reader_listener::DataReaderListener,
        subscriber_listener::SubscriberListener,
    },
};
use rust_rtps_pim::{
    behavior::reader::stateful_reader::RtpsStatefulReaderOperations,
    structure::{
        types::{EntityId, EntityKind, Guid, ReliabilityKind, TopicKind},
        RtpsEntity,
    },
};

use crate::{
    dds_type::DDSType,
    rtps_impl::{rtps_group_impl::RtpsGroupImpl, rtps_reader_impl::RtpsReaderImpl},
    utils::shared_object::{RtpsShared, RtpsWeak},
};

use super::{data_reader_impl::DataReaderImpl, data_reader_proxy::DataReaderProxy, topic_proxy::TopicProxy};

pub struct SubscriberImpl {
    qos: SubscriberQos,
    rtps_group: RtpsGroupImpl,
    data_reader_storage_list: Vec<RtpsShared<DataReaderImpl>>,
    user_defined_data_reader_counter: u8,
    default_data_reader_qos: DataReaderQos,
}

impl SubscriberImpl {
    pub fn new(
        qos: SubscriberQos,
        rtps_group: RtpsGroupImpl,
        data_reader_storage_list: Vec<RtpsShared<DataReaderImpl>>,
    ) -> Self {
        Self {
            qos,
            rtps_group,
            data_reader_storage_list,
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
        }
    }

    /// Get a reference to the subscriber storage's readers.
    pub fn readers(&self) -> &[RtpsShared<DataReaderImpl>] {
        self.data_reader_storage_list.as_slice()
    }
}

pub struct SubscriberProxy<'s> {
    participant: &'s dyn DomainParticipant,
    subscriber_storage: RtpsWeak<SubscriberImpl>,
}

impl<'s> SubscriberProxy<'s> {
    pub fn new(
        participant: &'s dyn DomainParticipant,
        subscriber_storage: RtpsWeak<SubscriberImpl>,
    ) -> Self {
        Self {
            participant,
            subscriber_storage,
        }
    }

    /// Get a reference to the subscriber impl's subscriber storage.
    pub(crate) fn subscriber_storage(&self) -> &RtpsWeak<SubscriberImpl> {
        &self.subscriber_storage
    }
}

impl<'dr, 's: 'dr, 't: 'dr, T: DDSType + 'static>
    rust_dds_api::subscription::subscriber::DataReaderFactory<'dr, 't, T> for SubscriberProxy<'s>
where
    T: for<'de> serde::Deserialize<'de>,
{
    type TopicType = TopicProxy<'t, T>;
    type DataReaderType = DataReaderProxy<'dr, T>;

    fn create_datareader(
        &'dr self,
        a_topic: &'dr Self::TopicType,
        qos: Option<DataReaderQos>,
        _a_listener: Option<&'static dyn DataReaderListener<DataPIM = T>>,
        _mask: StatusMask,
    ) -> Option<Self::DataReaderType> {
        let subscriber_storage = self.subscriber_storage.upgrade().ok()?;
        let subscriber_storage_lock = subscriber_storage.lock();
        let qos = qos.unwrap_or(subscriber_storage_lock.default_data_reader_qos.clone());
        qos.is_consistent().ok()?;

        let (entity_kind, topic_kind) = match T::has_key() {
            true => (EntityKind::UserDefinedWriterWithKey, TopicKind::WithKey),
            false => (EntityKind::UserDefinedWriterNoKey, TopicKind::NoKey),
        };
        let entity_id = EntityId::new(
            [
                subscriber_storage_lock
                    .rtps_group
                    .guid()
                    .entity_id()
                    .entity_key()[0],
                subscriber_storage_lock.user_defined_data_reader_counter,
                0,
            ],
            entity_kind,
        );
        let guid = Guid::new(
            *subscriber_storage_lock.rtps_group.guid().prefix(),
            entity_id,
        );
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };

        let unicast_locator_list = &[];
        let multicast_locator_list = &[];
        let heartbeat_response_delay = rust_rtps_pim::behavior::types::DURATION_ZERO;
        let heartbeat_supression_duration = rust_rtps_pim::behavior::types::DURATION_ZERO;
        let expects_inline_qos = false;
        let rtps_reader = RtpsReaderImpl::new(
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_response_delay,
            heartbeat_supression_duration,
            expects_inline_qos,
        );
        let reader_storage = DataReaderImpl::new(rtps_reader, qos);
        let reader_storage_shared = RtpsShared::new(reader_storage);
        let data_reader = DataReaderProxy::new(self, a_topic, reader_storage_shared.downgrade());
        Some(data_reader)
    }

    fn delete_datareader(&self, _a_datareader: &Self::DataReaderType) -> DDSResult<()> {
        todo!()
    }

    fn lookup_datareader<'a>(
        &'a self,
        _topic: &'a Self::TopicType,
    ) -> Option<Self::DataReaderType> {
        todo!()
    }
}

impl<'s> rust_dds_api::subscription::subscriber::Subscriber for SubscriberProxy<'s> {
    fn begin_access(&self) -> DDSResult<()> {
        todo!()
    }

    fn end_access(&self) -> DDSResult<()> {
        todo!()
    }

    fn notify_datareaders(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_sample_lost_status(&self, _status: &mut SampleLostStatus) -> DDSResult<()> {
        todo!()
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_datareader_qos(&self, _qos: Option<DataReaderQos>) -> DDSResult<()> {
        todo!()
    }

    fn get_default_datareader_qos(&self) -> DDSResult<DataReaderQos> {
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_datareaders(
        &self,
        _readers: &mut [&mut dyn AnyDataReader],
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }

    /// This operation returns the DomainParticipant to which the Subscriber belongs.
    fn get_participant(&self) -> &dyn DomainParticipant {
        self.participant
    }
}

impl<'s> Entity for SubscriberProxy<'s> {
    type Qos = SubscriberQos;
    type Listener = &'static dyn SubscriberListener;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        let subscriber_storage = self.subscriber_storage.upgrade()?;
        let mut subscriber_storage_lock = subscriber_storage.lock();
        subscriber_storage_lock.qos = qos;
        Ok(())
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        Ok(self.subscriber_storage.upgrade()?.lock().qos.clone())
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(&self) -> StatusCondition {
        todo!()
    }

    fn get_status_changes(&self) -> StatusMask {
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use rust_dds_api::{
        domain::domain_participant_listener::DomainParticipantListener,
        infrastructure::qos::{DomainParticipantQos, PublisherQos},
        subscription::subscriber::Subscriber,
    };
    use rust_rtps_pim::structure::types::GUID_UNKNOWN;

    use crate::{dds_impl::topic_proxy::TopicImpl, dds_type::DDSType};

    use super::*;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct MockKeyedType;

    impl DDSType for MockKeyedType {
        fn type_name() -> &'static str {
            todo!()
        }

        fn has_key() -> bool {
            true
        }
    }

    struct MockDomainParticipant;

    impl DomainParticipant for MockDomainParticipant {
        fn lookup_topicdescription<'t, T>(
            &'t self,
            _name: &'t str,
        ) -> Option<&'t dyn rust_dds_api::topic::topic_description::TopicDescription<T>>
        where
            Self: Sized,
        {
            todo!()
        }

        fn ignore_participant(&self, _handle: InstanceHandle) -> DDSResult<()> {
            todo!()
        }

        fn ignore_topic(&self, _handle: InstanceHandle) -> DDSResult<()> {
            todo!()
        }

        fn ignore_publication(&self, _handle: InstanceHandle) -> DDSResult<()> {
            todo!()
        }

        fn ignore_subscription(&self, _handle: InstanceHandle) -> DDSResult<()> {
            todo!()
        }

        fn get_domain_id(&self) -> rust_dds_api::dcps_psm::DomainId {
            todo!()
        }

        fn delete_contained_entities(&self) -> DDSResult<()> {
            todo!()
        }

        fn assert_liveliness(&self) -> DDSResult<()> {
            todo!()
        }

        fn set_default_publisher_qos(&self, _qos: Option<PublisherQos>) -> DDSResult<()> {
            todo!()
        }

        fn get_default_publisher_qos(&self) -> PublisherQos {
            todo!()
        }

        fn set_default_subscriber_qos(
            &self,
            _qos: Option<rust_dds_api::infrastructure::qos::SubscriberQos>,
        ) -> DDSResult<()> {
            todo!()
        }

        fn get_default_subscriber_qos(&self) -> rust_dds_api::infrastructure::qos::SubscriberQos {
            todo!()
        }

        fn set_default_topic_qos(&self, _qos: Option<TopicQos>) -> DDSResult<()> {
            todo!()
        }

        fn get_default_topic_qos(&self) -> TopicQos {
            todo!()
        }

        fn get_discovered_participants(
            &self,
            _participant_handles: &mut [InstanceHandle],
        ) -> DDSResult<()> {
            todo!()
        }

        fn get_discovered_participant_data(
            &self,
            _participant_data: rust_dds_api::builtin_topics::ParticipantBuiltinTopicData,
            _participant_handle: InstanceHandle,
        ) -> DDSResult<()> {
            todo!()
        }

        fn get_discovered_topics(&self, _topic_handles: &mut [InstanceHandle]) -> DDSResult<()> {
            todo!()
        }

        fn get_discovered_topic_data(
            &self,
            _topic_data: rust_dds_api::builtin_topics::TopicBuiltinTopicData,
            _topic_handle: InstanceHandle,
        ) -> DDSResult<()> {
            todo!()
        }

        fn contains_entity(&self, _a_handle: InstanceHandle) -> bool {
            todo!()
        }

        fn get_current_time(&self) -> DDSResult<rust_dds_api::dcps_psm::Time> {
            todo!()
        }
    }

    impl Entity for MockDomainParticipant {
        type Qos = DomainParticipantQos;
        type Listener = &'static dyn DomainParticipantListener;

        fn set_qos(&self, _qos: Option<Self::Qos>) -> DDSResult<()> {
            todo!()
        }

        fn get_qos(&self) -> DDSResult<Self::Qos> {
            todo!()
        }

        fn set_listener(
            &self,
            _a_listener: Option<Self::Listener>,
            _mask: StatusMask,
        ) -> DDSResult<()> {
            todo!()
        }

        fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
            todo!()
        }

        fn get_statuscondition(&self) -> StatusCondition {
            todo!()
        }

        fn get_status_changes(&self) -> StatusMask {
            todo!()
        }

        fn enable(&self) -> DDSResult<()> {
            todo!()
        }

        fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
            todo!()
        }
    }

    #[test]
    fn create_datareader() {
        let participant = MockDomainParticipant;
        let rtps_group = RtpsGroupImpl::new(GUID_UNKNOWN);
        let data_reader_storage_list = vec![];
        let subscriber_storage = SubscriberImpl::new(
            SubscriberQos::default(),
            rtps_group,
            data_reader_storage_list,
        );
        let subscriber_storage_shared = RtpsShared::new(subscriber_storage);
        let subscriber = SubscriberProxy::new(&participant, subscriber_storage_shared.downgrade());
        let topic_storage = TopicImpl::new(TopicQos::default());
        let topic_storage_shared = RtpsShared::new(topic_storage);
        let topic = TopicProxy::<MockKeyedType>::new(&participant, topic_storage_shared.downgrade());

        let datareader = subscriber.create_datareader(&topic, None, None, 0);

        assert!(datareader.is_some());
    }
}
