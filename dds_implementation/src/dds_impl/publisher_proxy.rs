use std::sync::atomic::{self, AtomicU8};

use dds_api::{
    builtin_topics::PublicationBuiltinTopicData,
    dcps_psm::{BuiltInTopicKey, Duration, InstanceHandle, StatusMask},
    domain::domain_participant::DomainParticipantTopicFactory,
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataWriterQos, PublisherQos, TopicQos},
        qos_policy::{
            DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
            DurabilityServiceQosPolicy, GroupDataQosPolicy, LatencyBudgetQosPolicy,
            LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, OwnershipStrengthQosPolicy,
            PartitionQosPolicy, PresentationQosPolicy, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind, TopicDataQosPolicy, UserDataQosPolicy,
        },
    },
    publication::{
        data_writer::DataWriter,
        publisher::{Publisher, PublisherDataWriterFactory},
        publisher_listener::PublisherListener,
    },
    return_type::{DdsError, DDSResult},
};

use rtps_pim::{
    behavior::writer::stateful_writer::RtpsStatefulWriterConstructor,
    structure::{
        entity::RtpsEntityAttributes,
        participant::RtpsParticipantAttributes,
        types::{
            EntityId, Guid, ReliabilityKind, TopicKind, USER_DEFINED_WRITER_NO_KEY,
            USER_DEFINED_WRITER_WITH_KEY,
        },
    },
};

use crate::{
    data_representation_builtin_endpoints::sedp_discovered_writer_data::{
        RtpsWriterProxy, SedpDiscoveredWriterData, DCPS_PUBLICATION,
    },
    dds_type::{DdsSerialize, DdsType},
    utils::{
        rtps_structure::RtpsStructure,
        shared_object::{DdsRwLock, DdsShared, DdsWeak},
    },
};

use super::{
    data_writer_proxy::DataWriterAttributes,
    data_writer_proxy::{DataWriterProxy, RtpsWriter},
    domain_participant_proxy::{DomainParticipantAttributes, DomainParticipantProxy},
    topic_proxy::TopicProxy,
};

pub struct PublisherAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub _qos: PublisherQos,
    pub rtps_group: Rtps::Group,
    pub data_writer_list: DdsRwLock<Vec<DdsShared<DataWriterAttributes<Rtps>>>>,
    pub user_defined_data_writer_counter: AtomicU8,
    pub default_datawriter_qos: DataWriterQos,
    pub parent_participant: DdsWeak<DomainParticipantAttributes<Rtps>>,
}

impl<Rtps> PublisherAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn new(
        qos: PublisherQos,
        rtps_group: Rtps::Group,
        parent_participant: DdsWeak<DomainParticipantAttributes<Rtps>>,
    ) -> Self {
        Self {
            _qos: qos,
            rtps_group,
            data_writer_list: DdsRwLock::new(Vec::new()),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
            parent_participant,
        }
    }
}

#[derive(Clone)]
pub struct PublisherProxy<Rtps>(pub(crate) DdsWeak<PublisherAttributes<Rtps>>)
where
    Rtps: RtpsStructure;

impl<Rtps> PublisherProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn new(publisher_impl: DdsWeak<PublisherAttributes<Rtps>>) -> Self {
        Self(publisher_impl)
    }
}

impl<Rtps> AsRef<DdsWeak<PublisherAttributes<Rtps>>> for PublisherProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    fn as_ref(&self) -> &DdsWeak<PublisherAttributes<Rtps>> {
        &self.0
    }
}

impl<Foo, Rtps> PublisherDataWriterFactory<Foo> for PublisherProxy<Rtps>
where
    Foo: DdsType + DdsSerialize + Send + Sync + 'static,
    Rtps: RtpsStructure,
{
    type TopicType = TopicProxy<Foo, Rtps>;
    type DataWriterType = DataWriterProxy<Foo, Rtps>;

    fn datawriter_factory_create_datawriter(
        &self,
        topic: &Self::TopicType,
        qos: Option<DataWriterQos>,
        listener: Option<<Self::DataWriterType as Entity>::Listener>,
        _mask: StatusMask,
    ) -> DDSResult<Self::DataWriterType> {
        let publisher_shared = self.0.upgrade()?;

        let topic_shared = topic.as_ref().upgrade()?;

        // /////// Build the GUID
        let guid = {
            let user_defined_data_writer_counter = publisher_shared
                .user_defined_data_writer_counter
                .fetch_add(1, atomic::Ordering::SeqCst);

            let entity_kind = match Foo::has_key() {
                true => USER_DEFINED_WRITER_WITH_KEY,
                false => USER_DEFINED_WRITER_NO_KEY,
            };

            Guid::new(
                publisher_shared.rtps_group.guid().prefix(),
                EntityId::new(
                    [
                        publisher_shared.rtps_group.guid().entity_id().entity_key()[0],
                        user_defined_data_writer_counter,
                        0,
                    ],
                    entity_kind,
                ),
            )
        };

        // /////// Create data writer
        let data_writer_shared = {
            let qos = qos.unwrap_or(publisher_shared.default_datawriter_qos.clone());
            qos.is_consistent()?;

            let topic_kind = match Foo::has_key() {
                true => TopicKind::WithKey,
                false => TopicKind::NoKey,
            };

            let reliability_level = match qos.reliability.kind {
                ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
                ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
            };

            let domain_participant = publisher_shared.parent_participant.upgrade()?;
            let rtps_writer_impl = RtpsWriter::Stateful(Rtps::StatefulWriter::new(
                guid,
                topic_kind,
                reliability_level,
                &domain_participant
                    .rtps_participant
                    .default_unicast_locator_list(),
                &domain_participant
                    .rtps_participant
                    .default_multicast_locator_list(),
                true,
                rtps_pim::behavior::types::Duration::new(0, 200_000_000),
                rtps_pim::behavior::types::DURATION_ZERO,
                rtps_pim::behavior::types::DURATION_ZERO,
                None,
            ));

            let data_writer_shared = DdsShared::new(DataWriterAttributes::new(
                qos,
                rtps_writer_impl,
                listener,
                topic_shared.clone(),
                publisher_shared.downgrade(),
            ));

            publisher_shared
                .data_writer_list
                .write_lock()
                .push(data_writer_shared.clone());

            data_writer_shared
        };

        // /////// Announce the data writer creation
        {
            let domain_participant = publisher_shared.parent_participant.upgrade()?;
            let domain_participant_proxy =
                DomainParticipantProxy::new(domain_participant.downgrade());
            let builtin_publisher_option = domain_participant.builtin_publisher.read_lock().clone();
            if let Some(builtin_publisher) = builtin_publisher_option {
                let builtin_publisher_proxy = PublisherProxy::new(builtin_publisher.downgrade());

                if let Ok(publication_topic) =
                    domain_participant_proxy.topic_factory_lookup_topicdescription(DCPS_PUBLICATION)
                {
                    if let Ok(sedp_builtin_publications_announcer) = builtin_publisher_proxy
                        .datawriter_factory_lookup_datawriter(&publication_topic)
                    {
                        let sedp_discovered_writer_data = SedpDiscoveredWriterData {
                            writer_proxy: RtpsWriterProxy {
                                remote_writer_guid: guid,
                                unicast_locator_list: domain_participant
                                    .rtps_participant
                                    .default_unicast_locator_list()
                                    .to_vec(),
                                multicast_locator_list: domain_participant
                                    .rtps_participant
                                    .default_multicast_locator_list()
                                    .to_vec(),
                                data_max_size_serialized: None,
                                remote_group_entity_id: EntityId::new([0; 3], 0),
                            },

                            publication_builtin_topic_data: PublicationBuiltinTopicData {
                                key: BuiltInTopicKey { value: guid.into() },
                                participant_key: BuiltInTopicKey { value: [1; 16] },
                                topic_name: topic_shared.topic_name.clone(),
                                type_name: Foo::type_name().to_string(),
                                durability: DurabilityQosPolicy::default(),
                                durability_service: DurabilityServiceQosPolicy::default(),
                                deadline: DeadlineQosPolicy::default(),
                                latency_budget: LatencyBudgetQosPolicy::default(),
                                liveliness: LivelinessQosPolicy::default(),
                                reliability: ReliabilityQosPolicy {
                                    kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos,
                                    max_blocking_time: Duration::new(3, 0),
                                },
                                lifespan: LifespanQosPolicy::default(),
                                user_data: UserDataQosPolicy::default(),
                                ownership: OwnershipQosPolicy::default(),
                                ownership_strength: OwnershipStrengthQosPolicy::default(),
                                destination_order: DestinationOrderQosPolicy::default(),
                                presentation: PresentationQosPolicy::default(),
                                partition: PartitionQosPolicy::default(),
                                topic_data: TopicDataQosPolicy::default(),
                                group_data: GroupDataQosPolicy::default(),
                            },
                        };

                        sedp_builtin_publications_announcer
                            .write_w_timestamp(
                                &sedp_discovered_writer_data,
                                None,
                                dds_api::dcps_psm::Time { sec: 0, nanosec: 0 },
                            )
                            .unwrap();
                    }
                }
            }
        }

        Ok(DataWriterProxy::new(data_writer_shared.downgrade()))
    }

    fn datawriter_factory_delete_datawriter(
        &self,
        datawriter: &Self::DataWriterType,
    ) -> DDSResult<()> {
        let publisher_shared = self.0.upgrade()?;
        let datawriter_shared = datawriter.as_ref().upgrade()?;

        let data_writer_list = &mut publisher_shared.data_writer_list.write_lock();
        let data_writer_list_position = data_writer_list
            .iter()
            .position(|x| x == &datawriter_shared)
            .ok_or(DdsError::PreconditionNotMet(
                "Data writer can only be deleted from its parent publisher".to_string(),
            ))?;
        data_writer_list.remove(data_writer_list_position);

        Ok(())
    }

    fn datawriter_factory_lookup_datawriter(
        &self,
        topic: &Self::TopicType,
    ) -> DDSResult<Self::DataWriterType> {
        let publisher_shared = self.0.upgrade()?;
        let data_writer_list = &publisher_shared.data_writer_list.write_lock();

        let topic_shared = topic.as_ref().upgrade()?;

        data_writer_list
            .iter()
            .find_map(|data_writer_shared| {
                let data_writer_topic = &data_writer_shared.topic;

                if data_writer_topic.topic_name == topic_shared.topic_name
                    && data_writer_topic.type_name == Foo::type_name()
                {
                    Some(DataWriterProxy::new(data_writer_shared.downgrade()))
                } else {
                    None
                }
            })
            .ok_or(DdsError::PreconditionNotMet("Not found".to_string()))
    }
}

impl<Rtps> Publisher for PublisherProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    type DomainParticipant = DomainParticipantProxy<Rtps>;

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

    fn wait_for_acknowledgments(&self, _max_wait: dds_api::dcps_psm::Duration) -> DDSResult<()> {
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

    fn get_default_datawriter_qos(&self) -> DDSResult<DataWriterQos> {
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

    fn get_participant(&self) -> DDSResult<Self::DomainParticipant> {
        let publisher_attributes = self.0.upgrade()?;
        Ok(DomainParticipantProxy::new(
            publisher_attributes.parent_participant.clone(),
        ))
    }
}

impl<Rtps> Entity for PublisherProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    type Qos = PublisherQos;
    type Listener = &'static dyn PublisherListener;

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.publisher_impl)?).set_qos(qos)
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?).get_qos()
        todo!()
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DDSResult<()> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?)
        //     .set_listener(a_listener, mask)
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?).get_listener()
        todo!()
    }

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?).get_statuscondition()
        todo!()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?).get_status_changes()
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?).enable()
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.publisher_impl)?).get_instance_handle()
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{dds_type::Endianness, test_utils::mock_rtps_group::MockRtpsGroup};
    use dds_api::{dcps_psm::DomainId, infrastructure::qos::DomainParticipantQos};
    use rtps_pim::structure::types::GuidPrefix;
    use std::io::Write;

    use crate::{dds_impl::topic_proxy::TopicAttributes, test_utils::mock_rtps::MockRtps};

    use super::*;

    macro_rules! make_empty_dds_type {
        ($type_name:ident) => {
            struct $type_name {}

            impl DdsSerialize for $type_name {
                fn serialize<W: Write, E: Endianness>(&self, _writer: W) -> DDSResult<()> {
                    Ok(())
                }
            }

            impl DdsType for $type_name {
                fn type_name() -> &'static str {
                    stringify!($type_name)
                }

                fn has_key() -> bool {
                    false
                }
            }
        };
    }

    make_empty_dds_type!(Foo);

    #[test]
    fn datawriter_factory_create_datawriter() {
        let mut domain_participant_attributes: DomainParticipantAttributes<MockRtps> =
            DomainParticipantAttributes::new(
                GuidPrefix([1; 12]),
                DomainId::default(),
                "".to_string(),
                DomainParticipantQos::default(),
                vec![],
                vec![],
                vec![],
                vec![],
            );
        domain_participant_attributes
            .rtps_participant
            .expect_default_unicast_locator_list()
            .return_const(vec![]);
        domain_participant_attributes
            .rtps_participant
            .expect_default_multicast_locator_list()
            .return_const(vec![]);

        let domain_participant = DdsShared::new(domain_participant_attributes);

        *domain_participant.builtin_publisher.write_lock() =
            Some(DdsShared::new(PublisherAttributes::new(
                PublisherQos::default(),
                MockRtpsGroup::new(),
                domain_participant.downgrade(),
            )));

        let mut publisher_attributes: PublisherAttributes<MockRtps> = PublisherAttributes {
            _qos: PublisherQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_writer_list: DdsRwLock::new(Vec::new()),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
            parent_participant: domain_participant.downgrade(),
        };
        publisher_attributes
            .rtps_group
            .expect_guid()
            .return_const(Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)));
        let publisher = DdsShared::new(publisher_attributes);
        let publisher_proxy = PublisherProxy::new(publisher.downgrade());

        let topic = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));
        let topic_proxy = TopicProxy::<Foo, MockRtps>::new(topic.downgrade());

        let data_writer =
            publisher_proxy.datawriter_factory_create_datawriter(&topic_proxy, None, None, 0);

        assert!(data_writer.is_ok());
        assert_eq!(1, publisher.data_writer_list.read_lock().len());
    }

    #[test]
    fn datawriter_factory_delete_datawriter() {
        let mut domain_participant_attributes: DomainParticipantAttributes<MockRtps> =
            DomainParticipantAttributes::new(
                GuidPrefix([1; 12]),
                DomainId::default(),
                "".to_string(),
                DomainParticipantQos::default(),
                vec![],
                vec![],
                vec![],
                vec![],
            );
        domain_participant_attributes
            .rtps_participant
            .expect_default_unicast_locator_list()
            .return_const(vec![]);
        domain_participant_attributes
            .rtps_participant
            .expect_default_multicast_locator_list()
            .return_const(vec![]);

        let domain_participant = DdsShared::new(domain_participant_attributes);

        *domain_participant.builtin_publisher.write_lock() =
            Some(DdsShared::new(PublisherAttributes::new(
                PublisherQos::default(),
                MockRtpsGroup::new(),
                domain_participant.downgrade(),
            )));

        let mut publisher_attributes: PublisherAttributes<MockRtps> = PublisherAttributes {
            _qos: PublisherQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_writer_list: DdsRwLock::new(Vec::new()),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
            parent_participant: domain_participant.downgrade(),
        };
        publisher_attributes
            .rtps_group
            .expect_guid()
            .return_const(Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)));
        let publisher = DdsShared::new(publisher_attributes);
        let publisher_proxy = PublisherProxy::new(publisher.downgrade());

        let topic = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));
        let topic_proxy = TopicProxy::<Foo, _>::new(topic.downgrade());

        let data_writer = publisher_proxy
            .datawriter_factory_create_datawriter(&topic_proxy, None, None, 0)
            .unwrap();

        assert_eq!(1, publisher.data_writer_list.read_lock().len());

        publisher_proxy
            .datawriter_factory_delete_datawriter(&data_writer)
            .unwrap();

        assert_eq!(0, publisher.data_writer_list.read_lock().len());
        assert!(data_writer.as_ref().upgrade().is_err())
    }

    #[test]
    fn datawriter_factory_delete_datawriter_from_other_publisher() {
        let mut domain_participant_attributes: DomainParticipantAttributes<MockRtps> =
            DomainParticipantAttributes::new(
                GuidPrefix([1; 12]),
                DomainId::default(),
                "".to_string(),
                DomainParticipantQos::default(),
                vec![],
                vec![],
                vec![],
                vec![],
            );
        domain_participant_attributes
            .rtps_participant
            .expect_default_unicast_locator_list()
            .return_const(vec![]);
        domain_participant_attributes
            .rtps_participant
            .expect_default_multicast_locator_list()
            .return_const(vec![]);

        let domain_participant = DdsShared::new(domain_participant_attributes);

        *domain_participant.builtin_publisher.write_lock() =
            Some(DdsShared::new(PublisherAttributes::new(
                PublisherQos::default(),
                MockRtpsGroup::new(),
                domain_participant.downgrade(),
            )));

        let mut publisher_attributes: PublisherAttributes<MockRtps> = PublisherAttributes {
            _qos: PublisherQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_writer_list: DdsRwLock::new(Vec::new()),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
            parent_participant: domain_participant.downgrade(),
        };
        publisher_attributes
            .rtps_group
            .expect_guid()
            .return_const(Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)));
        let publisher = DdsShared::new(publisher_attributes);
        let publisher_proxy = PublisherProxy::new(publisher.downgrade());

        let publisher2_attributes: PublisherAttributes<MockRtps> = PublisherAttributes {
            _qos: PublisherQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_writer_list: DdsRwLock::new(Vec::new()),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
            parent_participant: domain_participant.downgrade(),
        };
        let publisher2 = DdsShared::new(publisher2_attributes);
        let publisher2_proxy = PublisherProxy::new(publisher2.downgrade());

        let topic = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));
        let topic_proxy = TopicProxy::<Foo, _>::new(topic.downgrade());

        let data_writer = publisher_proxy
            .datawriter_factory_create_datawriter(&topic_proxy, None, None, 0)
            .unwrap();

        assert_eq!(1, publisher.data_writer_list.read_lock().len());
        assert_eq!(0, publisher2.data_writer_list.read_lock().len());

        assert!(matches!(
            publisher2_proxy.datawriter_factory_delete_datawriter(&data_writer),
            Err(DdsError::PreconditionNotMet(_))
        ));
        assert!(data_writer.as_ref().upgrade().is_ok())
    }

    #[test]
    fn datawriter_factory_lookup_datawriter_with_no_datawriter() {
        let domain_participant_attributes: DomainParticipantAttributes<MockRtps> =
            DomainParticipantAttributes::new(
                GuidPrefix([1; 12]),
                DomainId::default(),
                "".to_string(),
                DomainParticipantQos::default(),
                vec![],
                vec![],
                vec![],
                vec![],
            );

        let domain_participant = DdsShared::new(domain_participant_attributes);

        *domain_participant.builtin_publisher.write_lock() =
            Some(DdsShared::new(PublisherAttributes::new(
                PublisherQos::default(),
                MockRtpsGroup::new(),
                domain_participant.downgrade(),
            )));

        let publisher_attributes: PublisherAttributes<MockRtps> = PublisherAttributes {
            _qos: PublisherQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_writer_list: DdsRwLock::new(Vec::new()),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
            parent_participant: domain_participant.downgrade(),
        };
        let publisher = DdsShared::new(publisher_attributes);
        let publisher_proxy = PublisherProxy::new(publisher.downgrade());

        let topic = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));
        let topic_proxy = TopicProxy::<Foo, _>::new(topic.downgrade());

        assert!(publisher_proxy
            .datawriter_factory_lookup_datawriter(&topic_proxy)
            .is_err());
    }

    #[test]
    fn datawriter_factory_lookup_datawriter_with_one_datawriter() {
        let mut domain_participant_attributes: DomainParticipantAttributes<MockRtps> =
            DomainParticipantAttributes::new(
                GuidPrefix([1; 12]),
                DomainId::default(),
                "".to_string(),
                DomainParticipantQos::default(),
                vec![],
                vec![],
                vec![],
                vec![],
            );
        domain_participant_attributes
            .rtps_participant
            .expect_default_unicast_locator_list()
            .return_const(vec![]);
        domain_participant_attributes
            .rtps_participant
            .expect_default_multicast_locator_list()
            .return_const(vec![]);

        let domain_participant = DdsShared::new(domain_participant_attributes);

        *domain_participant.builtin_publisher.write_lock() =
            Some(DdsShared::new(PublisherAttributes::new(
                PublisherQos::default(),
                MockRtpsGroup::new(),
                domain_participant.downgrade(),
            )));

        let mut publisher_attributes: PublisherAttributes<MockRtps> = PublisherAttributes {
            _qos: PublisherQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_writer_list: DdsRwLock::new(Vec::new()),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
            parent_participant: domain_participant.downgrade(),
        };
        publisher_attributes
            .rtps_group
            .expect_guid()
            .return_const(Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)));
        let publisher = DdsShared::new(publisher_attributes);
        let publisher_proxy = PublisherProxy::new(publisher.downgrade());

        let topic = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));
        let topic_proxy = TopicProxy::<Foo, _>::new(topic.downgrade());

        let data_writer = publisher_proxy
            .datawriter_factory_create_datawriter(&topic_proxy, None, None, 0)
            .unwrap();

        assert!(
            publisher_proxy
                .datawriter_factory_lookup_datawriter(&topic_proxy)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == data_writer.as_ref().upgrade().unwrap()
        );
    }

    make_empty_dds_type!(Bar);

    #[test]
    fn datawriter_factory_lookup_datawriter_with_one_datawriter_with_wrong_type() {
        let mut domain_participant_attributes: DomainParticipantAttributes<MockRtps> =
            DomainParticipantAttributes::new(
                GuidPrefix([1; 12]),
                DomainId::default(),
                "".to_string(),
                DomainParticipantQos::default(),
                vec![],
                vec![],
                vec![],
                vec![],
            );
        domain_participant_attributes
            .rtps_participant
            .expect_default_unicast_locator_list()
            .return_const(vec![]);
        domain_participant_attributes
            .rtps_participant
            .expect_default_multicast_locator_list()
            .return_const(vec![]);

        let domain_participant = DdsShared::new(domain_participant_attributes);

        *domain_participant.builtin_publisher.write_lock() =
            Some(DdsShared::new(PublisherAttributes::new(
                PublisherQos::default(),
                MockRtpsGroup::new(),
                domain_participant.downgrade(),
            )));

        let mut publisher_attributes: PublisherAttributes<MockRtps> = PublisherAttributes {
            _qos: PublisherQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_writer_list: DdsRwLock::new(Vec::new()),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
            parent_participant: domain_participant.downgrade(),
        };
        publisher_attributes
            .rtps_group
            .expect_guid()
            .return_const(Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)));
        let publisher = DdsShared::new(publisher_attributes);
        let publisher_proxy = PublisherProxy::new(publisher.downgrade());

        let topic_foo = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));
        let topic_foo_proxy = TopicProxy::<Foo, _>::new(topic_foo.downgrade());

        let topic_bar = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Bar::type_name(),
            "topic",
            DdsWeak::new(),
        ));
        let topic_bar_proxy = TopicProxy::<Bar, _>::new(topic_bar.downgrade());

        publisher_proxy
            .datawriter_factory_create_datawriter(&topic_bar_proxy, None, None, 0)
            .unwrap();

        assert!(publisher_proxy
            .datawriter_factory_lookup_datawriter(&topic_foo_proxy)
            .is_err());
    }

    #[test]
    fn datawriter_factory_lookup_datawriter_with_one_datawriter_with_wrong_topic() {
        let mut domain_participant_attributes: DomainParticipantAttributes<MockRtps> =
            DomainParticipantAttributes::new(
                GuidPrefix([1; 12]),
                DomainId::default(),
                "".to_string(),
                DomainParticipantQos::default(),
                vec![],
                vec![],
                vec![],
                vec![],
            );
        domain_participant_attributes
            .rtps_participant
            .expect_default_unicast_locator_list()
            .return_const(vec![]);
        domain_participant_attributes
            .rtps_participant
            .expect_default_multicast_locator_list()
            .return_const(vec![]);

        let domain_participant = DdsShared::new(domain_participant_attributes);

        *domain_participant.builtin_publisher.write_lock() =
            Some(DdsShared::new(PublisherAttributes::new(
                PublisherQos::default(),
                MockRtpsGroup::new(),
                domain_participant.downgrade(),
            )));

        let mut publisher_attributes: PublisherAttributes<MockRtps> = PublisherAttributes {
            _qos: PublisherQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_writer_list: DdsRwLock::new(Vec::new()),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
            parent_participant: domain_participant.downgrade(),
        };
        publisher_attributes
            .rtps_group
            .expect_guid()
            .return_const(Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)));
        let publisher = DdsShared::new(publisher_attributes);
        let publisher_proxy = PublisherProxy::new(publisher.downgrade());

        let topic1 = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic1",
            DdsWeak::new(),
        ));
        let topic1_proxy = TopicProxy::<Foo, _>::new(topic1.downgrade());

        let topic2 = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic2",
            DdsWeak::new(),
        ));
        let topic2_proxy = TopicProxy::<Foo, _>::new(topic2.downgrade());

        publisher_proxy
            .datawriter_factory_create_datawriter(&topic2_proxy, None, None, 0)
            .unwrap();

        assert!(publisher_proxy
            .datawriter_factory_lookup_datawriter(&topic1_proxy)
            .is_err());
    }

    #[test]
    fn datawriter_factory_lookup_datawriter_with_two_dawriters_with_different_types() {
        let mut domain_participant_attributes: DomainParticipantAttributes<MockRtps> =
            DomainParticipantAttributes::new(
                GuidPrefix([1; 12]),
                DomainId::default(),
                "".to_string(),
                DomainParticipantQos::default(),
                vec![],
                vec![],
                vec![],
                vec![],
            );
        domain_participant_attributes
            .rtps_participant
            .expect_default_unicast_locator_list()
            .return_const(vec![]);
        domain_participant_attributes
            .rtps_participant
            .expect_default_multicast_locator_list()
            .return_const(vec![]);

        let domain_participant = DdsShared::new(domain_participant_attributes);

        *domain_participant.builtin_publisher.write_lock() =
            Some(DdsShared::new(PublisherAttributes::new(
                PublisherQos::default(),
                MockRtpsGroup::new(),
                domain_participant.downgrade(),
            )));

        let mut publisher_attributes: PublisherAttributes<MockRtps> = PublisherAttributes {
            _qos: PublisherQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_writer_list: DdsRwLock::new(Vec::new()),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
            parent_participant: domain_participant.downgrade(),
        };
        publisher_attributes
            .rtps_group
            .expect_guid()
            .return_const(Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)));
        let publisher = DdsShared::new(publisher_attributes);
        let publisher_proxy = PublisherProxy::new(publisher.downgrade());

        let topic_foo = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));
        let topic_foo_proxy = TopicProxy::<Foo, _>::new(topic_foo.downgrade());

        let topic_bar = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Bar::type_name(),
            "topic",
            DdsWeak::new(),
        ));
        let topic_bar_proxy = TopicProxy::<Bar, _>::new(topic_bar.downgrade());

        let data_writer_foo = publisher_proxy
            .datawriter_factory_create_datawriter(&topic_foo_proxy, None, None, 0)
            .unwrap();
        let data_writer_bar = publisher_proxy
            .datawriter_factory_create_datawriter(&topic_bar_proxy, None, None, 0)
            .unwrap();

        assert!(
            publisher_proxy
                .datawriter_factory_lookup_datawriter(&topic_foo_proxy)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == data_writer_foo.as_ref().upgrade().unwrap()
        );

        assert!(
            publisher_proxy
                .datawriter_factory_lookup_datawriter(&topic_bar_proxy)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == data_writer_bar.as_ref().upgrade().unwrap()
        );
    }

    #[test]
    fn datawriter_factory_lookup_datawriter_with_two_datawriters_with_different_topics() {
        let mut domain_participant_attributes: DomainParticipantAttributes<MockRtps> =
            DomainParticipantAttributes::new(
                GuidPrefix([1; 12]),
                DomainId::default(),
                "".to_string(),
                DomainParticipantQos::default(),
                vec![],
                vec![],
                vec![],
                vec![],
            );
        domain_participant_attributes
            .rtps_participant
            .expect_default_unicast_locator_list()
            .return_const(vec![]);
        domain_participant_attributes
            .rtps_participant
            .expect_default_multicast_locator_list()
            .return_const(vec![]);

        let domain_participant = DdsShared::new(domain_participant_attributes);

        *domain_participant.builtin_publisher.write_lock() =
            Some(DdsShared::new(PublisherAttributes::new(
                PublisherQos::default(),
                MockRtpsGroup::new(),
                domain_participant.downgrade(),
            )));

        let mut publisher_attributes: PublisherAttributes<MockRtps> = PublisherAttributes {
            _qos: PublisherQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_writer_list: DdsRwLock::new(Vec::new()),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
            parent_participant: domain_participant.downgrade(),
        };
        publisher_attributes
            .rtps_group
            .expect_guid()
            .return_const(Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)));
        let publisher = DdsShared::new(publisher_attributes);
        let publisher_proxy = PublisherProxy::new(publisher.downgrade());

        let topic1 = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic1",
            DdsWeak::new(),
        ));
        let topic1_proxy = TopicProxy::<Foo, _>::new(topic1.downgrade());

        let topic2 = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic2",
            DdsWeak::new(),
        ));
        let topic2_proxy = TopicProxy::<Foo, _>::new(topic2.downgrade());

        let data_writer1 = publisher_proxy
            .datawriter_factory_create_datawriter(&topic1_proxy, None, None, 0)
            .unwrap();
        let data_writer2 = publisher_proxy
            .datawriter_factory_create_datawriter(&topic2_proxy, None, None, 0)
            .unwrap();

        assert!(
            publisher_proxy
                .datawriter_factory_lookup_datawriter(&topic1_proxy)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == data_writer1.as_ref().upgrade().unwrap()
        );

        assert!(
            publisher_proxy
                .datawriter_factory_lookup_datawriter(&topic2_proxy)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == data_writer2.as_ref().upgrade().unwrap()
        );
    }
}
