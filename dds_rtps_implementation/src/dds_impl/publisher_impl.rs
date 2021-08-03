use rust_dds_api::{
    dcps_psm::{InstanceHandle, StatusMask},
    domain::domain_participant::DomainParticipant,
    infrastructure::{
        entity::StatusCondition,
        qos::{DataWriterQos, PublisherQos, TopicQos},
        qos_policy::ReliabilityQosPolicyKind,
    },
    publication::{
        data_writer::DataWriter, data_writer_listener::DataWriterListener,
        publisher::DataWriterFactory, publisher_listener::PublisherListener,
    },
    return_type::{DDSError, DDSResult},
};
use rust_rtps_pim::{
    behavior::writer::stateless_writer::RtpsStatelessWriterOperations,
    structure::{
        types::{EntityId, EntityKind, Guid, ReliabilityKind, TopicKind},
        RtpsEntity,
    },
};

use crate::{
    dds_type::DDSType,
    rtps_impl::{rtps_group_impl::RtpsGroupImpl, rtps_writer_impl::RtpsWriterImpl},
    utils::shared_object::{RtpsShared, RtpsWeak},
};

use super::{
    data_writer_impl::{DataWriterImpl, DataWriterStorage},
    topic_impl::TopicImpl,
};

pub struct PublisherStorage {
    qos: PublisherQos,
    rtps_group: RtpsGroupImpl,
    data_writer_storage_list: Vec<RtpsShared<DataWriterStorage>>,
    user_defined_data_writer_counter: u8,
    default_datawriter_qos: DataWriterQos,
}

impl PublisherStorage {
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroupImpl,
        data_writer_storage_list: Vec<RtpsShared<DataWriterStorage>>,
    ) -> Self {
        Self {
            qos,
            rtps_group,
            data_writer_storage_list,
            user_defined_data_writer_counter: 0,
            default_datawriter_qos: DataWriterQos::default(),
        }
    }

    /// Get a reference to the publisher storage's data writer storage list.
    pub fn data_writer_storage_list(&self) -> &[RtpsShared<DataWriterStorage>] {
        self.data_writer_storage_list.as_slice()
    }
}

pub struct PublisherImpl<'p> {
    participant: &'p dyn DomainParticipant,
    publisher_storage: RtpsWeak<PublisherStorage>,
}

impl<'p> PublisherImpl<'p> {
    pub fn new(
        participant: &'p dyn DomainParticipant,
        publisher_storage: RtpsWeak<PublisherStorage>,
    ) -> Self {
        Self {
            participant,
            publisher_storage,
        }
    }

    /// Get a reference to the publisher impl's publisher storage.
    pub(crate) fn publisher_storage(&self) -> &RtpsWeak<PublisherStorage> {
        &self.publisher_storage
    }
}

impl<'dw, 'p: 'dw, 't: 'dw, T: DDSType + 'static> DataWriterFactory<'dw, 't, T>
    for PublisherImpl<'p>
{
    type TopicType = TopicImpl<'t, T>;
    type DataWriterType = DataWriterImpl<'dw, T>;

    fn create_datawriter(
        &'dw self,
        a_topic: &'dw Self::TopicType,
        qos: Option<DataWriterQos>,
        _a_listener: Option<&'static dyn DataWriterListener<DataPIM = T>>,
        _mask: StatusMask,
    ) -> Option<Self::DataWriterType> {
        let publisher_storage = self.publisher_storage.upgrade().ok()?;
        let mut publisher_storage_lock = publisher_storage.lock();
        let qos = qos.unwrap_or(publisher_storage_lock.default_datawriter_qos.clone());
        publisher_storage_lock.user_defined_data_writer_counter += 1;
        let (entity_kind, topic_kind) = match T::has_key() {
            true => (EntityKind::UserDefinedWriterWithKey, TopicKind::WithKey),
            false => (EntityKind::UserDefinedWriterNoKey, TopicKind::NoKey),
        };
        let entity_id = EntityId::new(
            [
                publisher_storage_lock
                    .rtps_group
                    .guid()
                    .entity_id()
                    .entity_key()[0],
                publisher_storage_lock.user_defined_data_writer_counter,
                0,
            ],
            entity_kind,
        );
        let guid = Guid::new(
            *publisher_storage_lock.rtps_group.guid().prefix(),
            entity_id,
        );
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };
        let unicast_locator_list = &[];
        let multicast_locator_list = &[];
        let push_mode = true;
        let heartbeat_period = rust_rtps_pim::behavior::types::Duration::new(0, 200_000_000);
        let nack_response_delay = rust_rtps_pim::behavior::types::DURATION_ZERO;
        let nack_suppression_duration = rust_rtps_pim::behavior::types::DURATION_ZERO;
        let data_max_size_serialized = None;
        let rtps_writer = RtpsWriterImpl::new(
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        );
        let data_writer_storage = DataWriterStorage::new(qos, rtps_writer);
        let data_writer_storage_shared = RtpsShared::new(data_writer_storage);
        let datawriter = DataWriterImpl::new(self, a_topic, data_writer_storage_shared.downgrade());
        publisher_storage_lock
            .data_writer_storage_list
            .push(data_writer_storage_shared);
        Some(datawriter)
    }

    fn delete_datawriter(&self, a_datawriter: &Self::DataWriterType) -> DDSResult<()> {
        if std::ptr::eq(a_datawriter.get_publisher(), self) {
            todo!()
            // self.rtps_writer_group_impl
            // .upgrade()?
            // .delete_datawriter(a_datawriter.get_instance_handle()?)
        } else {
            Err(DDSError::PreconditionNotMet(
                "Data writer can only be deleted from its parent publisher",
            ))
        }
    }

    fn lookup_datawriter(&'dw self, _topic: &'dw Self::TopicType) -> Option<Self::DataWriterType> {
        todo!()
    }
}

impl<'p> rust_dds_api::publication::publisher::Publisher for PublisherImpl<'p> {
    fn suspend_publications(&self) -> DDSResult<()> {
        // self.rtps_writer_group_impl
        //     .upgrade()?
        //     .suspend_publications()
        todo!()
    }

    fn resume_publications(&self) -> DDSResult<()> {
        // self.rtps_writer_group_impl.upgrade()?.resume_publications()
        todo!()
    }

    fn begin_coherent_changes(&self) -> DDSResult<()> {
        todo!()
    }

    fn end_coherent_changes(&self) -> DDSResult<()> {
        todo!()
    }

    fn wait_for_acknowledgments(
        &self,
        _max_wait: rust_dds_api::dcps_psm::Duration,
    ) -> DDSResult<()> {
        todo!()
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_datawriter_qos(&self, _qos: Option<DataWriterQos>) -> DDSResult<()> {
        // self.rtps_writer_group_impl
        //     .upgrade()?
        //     .set_default_datawriter_qos(qos)
        todo!()
    }

    fn get_default_datawriter_qos(&self) -> DataWriterQos {
        // self.default_datawriter_qos.lock().unwrap().clone()
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_participant(&self) -> &dyn DomainParticipant {
        self.participant
    }
}

impl<'p> rust_dds_api::infrastructure::entity::Entity for PublisherImpl<'p> {
    type Qos = PublisherQos;
    type Listener = &'static dyn PublisherListener;

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        todo!()
        // Ok(self
        //     .impl_ref
        //     .upgrade()
        //     .ok_or(DDSError::AlreadyDeleted)?
        //     .lock()
        //     .unwrap()
        //     .set_qos(qos))
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        todo!()
        // Ok(self
        //     .impl_ref
        //     .upgrade()
        //     .ok_or(DDSError::AlreadyDeleted)?
        //     .lock()
        //     .unwrap()
        //     .get_qos()
        //     .clone())
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
        // Ok(crate::utils::instance_handle_from_guid(
        //     &self.rtps_writer_group_impl.upgrade()?.lock().guid(),
        // ))
    }
}

#[cfg(test)]
mod tests {
    use rust_dds_api::{
        domain::domain_participant_listener::DomainParticipantListener,
        infrastructure::{entity::Entity, qos::DomainParticipantQos},
    };
    use rust_rtps_pim::structure::types::GUID_UNKNOWN;

    use crate::dds_impl::topic_impl::TopicStorage;

    use super::*;

    #[derive(serde::Serialize)]
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

        fn ignore_publication(&self, handle: InstanceHandle) -> DDSResult<()> {
            todo!()
        }

        fn ignore_subscription(&self, handle: InstanceHandle) -> DDSResult<()> {
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

        fn set_default_publisher_qos(&self, qos: Option<PublisherQos>) -> DDSResult<()> {
            todo!()
        }

        fn get_default_publisher_qos(&self) -> PublisherQos {
            todo!()
        }

        fn set_default_subscriber_qos(
            &self,
            qos: Option<rust_dds_api::infrastructure::qos::SubscriberQos>,
        ) -> DDSResult<()> {
            todo!()
        }

        fn get_default_subscriber_qos(&self) -> rust_dds_api::infrastructure::qos::SubscriberQos {
            todo!()
        }

        fn set_default_topic_qos(&self, qos: Option<TopicQos>) -> DDSResult<()> {
            todo!()
        }

        fn get_default_topic_qos(&self) -> TopicQos {
            todo!()
        }

        fn get_discovered_participants(
            &self,
            participant_handles: &mut [InstanceHandle],
        ) -> DDSResult<()> {
            todo!()
        }

        fn get_discovered_participant_data(
            &self,
            participant_data: rust_dds_api::builtin_topics::ParticipantBuiltinTopicData,
            participant_handle: InstanceHandle,
        ) -> DDSResult<()> {
            todo!()
        }

        fn get_discovered_topics(&self, topic_handles: &mut [InstanceHandle]) -> DDSResult<()> {
            todo!()
        }

        fn get_discovered_topic_data(
            &self,
            topic_data: rust_dds_api::builtin_topics::TopicBuiltinTopicData,
            topic_handle: InstanceHandle,
        ) -> DDSResult<()> {
            todo!()
        }

        fn contains_entity(&self, a_handle: InstanceHandle) -> bool {
            todo!()
        }

        fn get_current_time(&self) -> DDSResult<rust_dds_api::dcps_psm::Time> {
            todo!()
        }
    }

    impl Entity for MockDomainParticipant {
        type Qos = DomainParticipantQos;
        type Listener = &'static dyn DomainParticipantListener;

        fn set_qos(&self, qos: Option<Self::Qos>) -> DDSResult<()> {
            todo!()
        }

        fn get_qos(&self) -> DDSResult<Self::Qos> {
            todo!()
        }

        fn set_listener(
            &self,
            a_listener: Option<Self::Listener>,
            mask: StatusMask,
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
    fn create_datawriter() {
        let participant = MockDomainParticipant;
        let rtps_group = RtpsGroupImpl::new(GUID_UNKNOWN);
        let data_writer_storage_list = vec![];
        let publisher_storage = PublisherStorage::new(
            PublisherQos::default(),
            rtps_group,
            data_writer_storage_list,
        );
        let publisher_storage_shared = RtpsShared::new(publisher_storage);
        let publisher = PublisherImpl::new(&participant, publisher_storage_shared.downgrade());
        let topic_storage = TopicStorage::new(TopicQos::default());
        let topic_storage_shared = RtpsShared::new(topic_storage);
        let topic = TopicImpl::<MockKeyedType>::new(&participant, topic_storage_shared.downgrade());

        let datawriter = publisher.create_datawriter(&topic, None, None, 0);

        assert!(datawriter.is_some());
    }
}
