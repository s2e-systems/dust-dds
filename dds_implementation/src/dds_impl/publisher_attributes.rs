use std::sync::atomic::{self, AtomicU8};

use dds_api::{
    builtin_topics::PublicationBuiltinTopicData,
    dcps_psm::{BuiltInTopicKey, Duration, StatusMask},
    domain::domain_participant::DomainParticipantTopicFactory,
    infrastructure::{
        entity::Entity,
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
    return_type::{DdsError, DdsResult},
    topic::topic_description::TopicDescription,
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
    data_representation_builtin_endpoints::discovered_writer_data::{
        DiscoveredWriterData, RtpsWriterProxy, DCPS_PUBLICATION,
    },
    dds_type::DdsType,
    utils::{
        rtps_structure::RtpsStructure,
        shared_object::{DdsRwLock, DdsShared, DdsWeak},
    },
};

use super::{
    data_writer_attributes::{DataWriterAttributes, RtpsWriter},
    domain_participant_attributes::DomainParticipantAttributes,
    topic_attributes::TopicAttributes,
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

impl<Rtps, Foo> PublisherDataWriterFactory<Foo> for DdsShared<PublisherAttributes<Rtps>>
where
    Rtps: RtpsStructure,
    Foo: DdsType,
{
    type TopicType = DdsShared<TopicAttributes<Rtps>>;
    type DataWriterType = DdsShared<DataWriterAttributes<Rtps>>;

    fn datawriter_factory_create_datawriter(
        &self,
        a_topic: &Self::TopicType,
        qos: Option<DataWriterQos>,
        a_listener: Option<<Self::DataWriterType as Entity>::Listener>,
        _mask: StatusMask,
    ) -> DdsResult<Self::DataWriterType>
    where
        Self::DataWriterType: Entity,
    {
        let topic_shared = a_topic;

        // /////// Build the GUID
        let guid = {
            let user_defined_data_writer_counter = self
                .user_defined_data_writer_counter
                .fetch_add(1, atomic::Ordering::SeqCst);

            let entity_kind = match Foo::has_key() {
                true => USER_DEFINED_WRITER_WITH_KEY,
                false => USER_DEFINED_WRITER_NO_KEY,
            };

            Guid::new(
                self.rtps_group.guid().prefix(),
                EntityId::new(
                    [
                        self.rtps_group.guid().entity_id().entity_key()[0],
                        user_defined_data_writer_counter,
                        0,
                    ],
                    entity_kind,
                ),
            )
        };

        // /////// Create data writer
        let data_writer_shared = {
            let qos = qos.unwrap_or(self.default_datawriter_qos.clone());
            qos.is_consistent()?;

            let topic_kind = match Foo::has_key() {
                true => TopicKind::WithKey,
                false => TopicKind::NoKey,
            };

            let reliability_level = match qos.reliability.kind {
                ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
                ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
            };

            let domain_participant = self.parent_participant.upgrade()?;
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
                a_listener,
                topic_shared.clone(),
                self.downgrade(),
            ));

            self.data_writer_list
                .write_lock()
                .push(data_writer_shared.clone());

            data_writer_shared
        };

        // /////// Announce the data writer creation
        {
            let domain_participant = self.parent_participant.upgrade()?;

            let builtin_publisher_option = domain_participant.builtin_publisher.read_lock().clone();
            if let Some(builtin_publisher) = builtin_publisher_option {
                if let Ok(publication_topic) =
                    DomainParticipantTopicFactory::<DiscoveredWriterData>::topic_factory_lookup_topicdescription(&domain_participant, DCPS_PUBLICATION)
                {
                    if let Ok(sedp_builtin_publications_announcer) =
                        PublisherDataWriterFactory::<DiscoveredWriterData>::datawriter_factory_lookup_datawriter(&builtin_publisher, &publication_topic)
                    {
                        let sedp_discovered_writer_data = DiscoveredWriterData {
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
                                topic_name: topic_shared.get_name().unwrap(),
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
                            .write(&sedp_discovered_writer_data, None)
                            .unwrap();
                    }
                }
            }
        }

        Ok(data_writer_shared)
    }

    fn datawriter_factory_delete_datawriter(
        &self,
        a_datawriter: &Self::DataWriterType,
    ) -> DdsResult<()> {
        let data_writer_list = &mut self.data_writer_list.write_lock();
        let data_writer_list_position = data_writer_list
            .iter()
            .position(|x| x == a_datawriter)
            .ok_or(DdsError::PreconditionNotMet(
                "Data writer can only be deleted from its parent publisher".to_string(),
            ))?;
        data_writer_list.remove(data_writer_list_position);

        Ok(())
    }

    fn datawriter_factory_lookup_datawriter(
        &self,
        topic: &Self::TopicType,
    ) -> DdsResult<Self::DataWriterType> {
        let data_writer_list = &self.data_writer_list.write_lock();

        data_writer_list
            .iter()
            .find_map(|data_writer_shared| {
                let data_writer_topic = &data_writer_shared.topic;

                if data_writer_topic.get_name().ok()? == topic.get_name().ok()?
                    && data_writer_topic.get_type_name().ok()? == Foo::type_name()
                {
                    Some(data_writer_shared.clone())
                } else {
                    None
                }
            })
            .ok_or(DdsError::PreconditionNotMet("Not found".to_string()))
    }
}

impl<Rtps> Publisher for DdsShared<PublisherAttributes<Rtps>>
where
    Rtps: RtpsStructure,
{
    type DomainParticipantType = DdsShared<DomainParticipantAttributes<Rtps>>;

    fn suspend_publications(&self) -> DdsResult<()> {
        todo!()
    }

    fn resume_publications(&self) -> DdsResult<()> {
        todo!()
    }

    fn begin_coherent_changes(&self) -> DdsResult<()> {
        todo!()
    }

    fn end_coherent_changes(&self) -> DdsResult<()> {
        todo!()
    }

    fn wait_for_acknowledgments(&self, _max_wait: Duration) -> DdsResult<()> {
        todo!()
    }

    fn get_participant(&self) -> DdsResult<Self::DomainParticipantType> {
        Ok(self.parent_participant.upgrade()?.clone())
    }

    fn delete_contained_entities(&self) -> DdsResult<()> {
        todo!()
    }

    fn set_default_datawriter_qos(&self, _qos: Option<DataWriterQos>) -> DdsResult<()> {
        todo!()
    }

    fn get_default_datawriter_qos(&self) -> DdsResult<DataWriterQos> {
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        todo!()
    }
}

impl<Rtps> Entity for DdsShared<PublisherAttributes<Rtps>>
where
    Rtps: RtpsStructure,
{
    type Qos = PublisherQos;
    type Listener = Box<dyn PublisherListener>;

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DdsResult<()> {
        todo!()
    }

    fn get_qos(&self) -> DdsResult<Self::Qos> {
        todo!()
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DdsResult<()> {
        todo!()
    }

    fn get_listener(&self) -> DdsResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(&self) -> DdsResult<dds_api::infrastructure::entity::StatusCondition> {
        todo!()
    }

    fn get_status_changes(&self) -> DdsResult<StatusMask> {
        todo!()
    }

    fn enable(&self) -> DdsResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DdsResult<dds_api::dcps_psm::InstanceHandle> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        dds_type::{DdsSerialize, DdsType, Endianness},
        test_utils::mock_rtps_group::MockRtpsGroup,
    };
    use dds_api::{
        dcps_psm::DomainId,
        infrastructure::qos::{DomainParticipantQos, TopicQos},
    };
    use rtps_pim::structure::types::GuidPrefix;
    use std::io::Write;

    use crate::{dds_impl::topic_attributes::TopicAttributes, test_utils::mock_rtps::MockRtps};

    use super::*;

    macro_rules! make_empty_dds_type {
        ($type_name:ident) => {
            struct $type_name {}

            impl DdsSerialize for $type_name {
                fn serialize<W: Write, E: Endianness>(&self, _writer: W) -> DdsResult<()> {
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

        let topic = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));

        let data_writer = publisher.create_datawriter::<Foo>(&topic, None, None, 0);

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

        let topic = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));

        let data_writer = publisher
            .create_datawriter::<Foo>(&topic, None, None, 0)
            .unwrap();

        assert_eq!(1, publisher.data_writer_list.read_lock().len());

        publisher.delete_datawriter::<Foo>(&data_writer).unwrap();

        assert_eq!(0, publisher.data_writer_list.read_lock().len());
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

        let publisher2_attributes: PublisherAttributes<MockRtps> = PublisherAttributes {
            _qos: PublisherQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_writer_list: DdsRwLock::new(Vec::new()),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
            parent_participant: domain_participant.downgrade(),
        };
        let publisher2 = DdsShared::new(publisher2_attributes);

        let topic = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));

        let data_writer = publisher
            .create_datawriter::<Foo>(&topic, None, None, 0)
            .unwrap();

        assert_eq!(1, publisher.data_writer_list.read_lock().len());
        assert_eq!(0, publisher2.data_writer_list.read_lock().len());

        assert!(matches!(
            publisher2.delete_datawriter::<Foo>(&data_writer),
            Err(DdsError::PreconditionNotMet(_))
        ));
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

        let topic = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));

        assert!(publisher.lookup_datawriter::<Foo>(&topic).is_err());
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

        let topic = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));

        let data_writer = publisher
            .create_datawriter::<Foo>(&topic, None, None, 0)
            .unwrap();

        assert!(publisher.lookup_datawriter::<Foo>(&topic).unwrap() == data_writer);
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

        let topic_foo = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));

        let topic_bar = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Bar::type_name(),
            "topic",
            DdsWeak::new(),
        ));

        publisher
            .create_datawriter::<Bar>(&topic_bar, None, None, 0)
            .unwrap();

        assert!(publisher.lookup_datawriter::<Foo>(&topic_foo).is_err());
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

        let topic1 = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic1",
            DdsWeak::new(),
        ));

        let topic2 = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic2",
            DdsWeak::new(),
        ));

        publisher
            .create_datawriter::<Foo>(&topic2, None, None, 0)
            .unwrap();

        assert!(publisher.lookup_datawriter::<Foo>(&topic1).is_err());
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

        let topic_foo = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));

        let topic_bar = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Bar::type_name(),
            "topic",
            DdsWeak::new(),
        ));

        let data_writer_foo = publisher
            .create_datawriter::<Foo>(&topic_foo, None, None, 0)
            .unwrap();
        let data_writer_bar = publisher
            .create_datawriter::<Bar>(&topic_bar, None, None, 0)
            .unwrap();

        assert!(publisher.lookup_datawriter::<Foo>(&topic_foo).unwrap() == data_writer_foo);

        assert!(publisher.lookup_datawriter::<Bar>(&topic_bar).unwrap() == data_writer_bar);
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

        let topic1 = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic1",
            DdsWeak::new(),
        ));

        let topic2 = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic2",
            DdsWeak::new(),
        ));

        let data_writer1 = publisher
            .create_datawriter::<Foo>(&topic1, None, None, 0)
            .unwrap();
        let data_writer2 = publisher
            .create_datawriter::<Foo>(&topic2, None, None, 0)
            .unwrap();

        assert!(publisher.lookup_datawriter::<Foo>(&topic1).unwrap() == data_writer1);

        assert!(publisher.lookup_datawriter::<Foo>(&topic2).unwrap() == data_writer2);
    }
}
