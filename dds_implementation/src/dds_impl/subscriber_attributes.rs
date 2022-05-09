use dds_api::{
    builtin_topics::SubscriptionBuiltinTopicData,
    dcps_psm::{
        BuiltInTopicKey, Duration, InstanceHandle, InstanceStateMask, SampleLostStatus,
        SampleStateMask, StatusMask, ViewStateMask,
    },
    domain::domain_participant::DomainParticipantTopicFactory,
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataReaderQos, SubscriberQos, TopicQos},
        qos_policy::{
            DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
            LatencyBudgetQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, PartitionQosPolicy,
            PresentationQosPolicy, ReliabilityQosPolicy, ReliabilityQosPolicyKind,
            TimeBasedFilterQosPolicy, TopicDataQosPolicy, UserDataQosPolicy,
        },
    },
    publication::{data_writer::DataWriter, publisher::PublisherDataWriterFactory},
    return_type::{DdsError, DdsResult},
    subscription::{
        data_reader::AnyDataReader,
        subscriber::{Subscriber, SubscriberDataReaderFactory},
        subscriber_listener::SubscriberListener,
    },
    topic::topic_description::TopicDescription,
};
use rtps_pim::{
    behavior::reader::stateful_reader::RtpsStatefulReaderConstructor,
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
    data_representation_builtin_endpoints::discovered_reader_data::{
        DiscoveredReaderData, RtpsReaderProxy, DCPS_SUBSCRIPTION,
    },
    dds_type::DdsType,
    utils::{
        rtps_structure::RtpsStructure,
        shared_object::{DdsRwLock, DdsShared, DdsWeak},
        timer::ThreadTimer,
    },
};

use super::{
    data_reader_attributes::{DataReaderAttributes, RtpsReader},
    domain_participant_attributes::DomainParticipantAttributes,
    topic_attributes::TopicAttributes,
};

pub struct SubscriberAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub qos: SubscriberQos,
    pub rtps_group: Rtps::Group,
    pub data_reader_list: DdsRwLock<Vec<DdsShared<DataReaderAttributes<Rtps, ThreadTimer>>>>,
    pub user_defined_data_reader_counter: u8,
    pub default_data_reader_qos: DataReaderQos,
    pub parent_domain_participant: DdsWeak<DomainParticipantAttributes<Rtps>>,
}

impl<Rtps> SubscriberAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn new(
        qos: SubscriberQos,
        rtps_group: Rtps::Group,
        parent_domain_participant: DdsWeak<DomainParticipantAttributes<Rtps>>,
    ) -> Self {
        Self {
            qos,
            rtps_group,
            data_reader_list: DdsRwLock::new(Vec::new()),
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
            parent_domain_participant,
        }
    }
}

impl<Rtps, Foo> SubscriberDataReaderFactory<Foo> for DdsShared<SubscriberAttributes<Rtps>>
where
    Rtps: RtpsStructure,
    Foo: DdsType,
{
    type TopicType = DdsShared<TopicAttributes<Rtps>>;
    type DataReaderType = DdsShared<DataReaderAttributes<Rtps, ThreadTimer>>;

    fn datareader_factory_create_datareader(
        &self,
        a_topic: &Self::TopicType,
        qos: Option<DataReaderQos>,
        a_listener: Option<<Self::DataReaderType as Entity>::Listener>,
        _mask: StatusMask,
    ) -> DdsResult<Self::DataReaderType>
    where
        Self::DataReaderType: Entity,
    {
        // /////// Build the GUID
        let entity_id = {
            let entity_kind = match Foo::has_key() {
                true => USER_DEFINED_WRITER_WITH_KEY,
                false => USER_DEFINED_WRITER_NO_KEY,
            };

            EntityId::new(
                [
                    self.rtps_group.guid().entity_id().entity_key()[0],
                    self.user_defined_data_reader_counter,
                    0,
                ],
                entity_kind,
            )
        };

        let guid = Guid::new(self.rtps_group.guid().prefix(), entity_id);

        // /////// Create data reader
        let data_reader_shared = {
            let qos = qos.unwrap_or(self.default_data_reader_qos.clone());
            qos.is_consistent()?;

            let topic_kind = match Foo::has_key() {
                true => TopicKind::WithKey,
                false => TopicKind::NoKey,
            };

            let reliability_level = match qos.reliability.kind {
                ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
                ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
            };

            let domain_participant = self.parent_domain_participant.upgrade()?;
            let rtps_reader = RtpsReader::Stateful(Rtps::StatefulReader::new(
                guid,
                topic_kind,
                reliability_level,
                &domain_participant
                    .rtps_participant
                    .default_unicast_locator_list(),
                &domain_participant
                    .rtps_participant
                    .default_multicast_locator_list(),
                rtps_pim::behavior::types::DURATION_ZERO,
                rtps_pim::behavior::types::DURATION_ZERO,
                false,
            ));

            let data_reader = DataReaderAttributes::new(
                qos,
                rtps_reader,
                a_topic.clone(),
                a_listener,
                self.downgrade(),
            );

            let data_reader_shared = DdsShared::new(data_reader);

            self.data_reader_list
                .write_lock()
                .push(data_reader_shared.clone());

            data_reader_shared
        };

        // /////// Announce the data reader creation
        {
            let domain_participant = self.parent_domain_participant.upgrade()?;

            let builtin_publisher_option = domain_participant.builtin_publisher.read_lock().clone();
            if let Some(builtin_publisher) = builtin_publisher_option {
                if let Ok(subscription_topic) =
                    DomainParticipantTopicFactory::<DiscoveredReaderData>::topic_factory_lookup_topicdescription(
                        &domain_participant,
                        DCPS_SUBSCRIPTION,
                    )
                {
                    if let Ok(sedp_builtin_subscription_announcer) =
                        PublisherDataWriterFactory::<DiscoveredReaderData>::datawriter_factory_lookup_datawriter(&builtin_publisher, &subscription_topic)
                    {
                        let sedp_discovered_reader_data = DiscoveredReaderData {
                            reader_proxy: RtpsReaderProxy {
                                remote_reader_guid: guid,
                                remote_group_entity_id: entity_id,
                                unicast_locator_list: domain_participant
                                    .rtps_participant
                                    .default_unicast_locator_list()
                                    .to_vec(),
                                multicast_locator_list: domain_participant
                                    .rtps_participant
                                    .default_multicast_locator_list()
                                    .to_vec(),
                                expects_inline_qos: false,
                            },

                            subscription_builtin_topic_data: SubscriptionBuiltinTopicData {
                                key: BuiltInTopicKey { value: guid.into() },
                                participant_key: BuiltInTopicKey { value: [1; 16] },
                                topic_name: a_topic.get_name().unwrap().clone(),
                                type_name: Foo::type_name().to_string(),
                                durability: DurabilityQosPolicy::default(),
                                deadline: DeadlineQosPolicy::default(),
                                latency_budget: LatencyBudgetQosPolicy::default(),
                                liveliness: LivelinessQosPolicy::default(),
                                reliability: ReliabilityQosPolicy {
                                    kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos,
                                    max_blocking_time: Duration::new(3, 0),
                                },
                                ownership: OwnershipQosPolicy::default(),
                                destination_order: DestinationOrderQosPolicy::default(),
                                user_data: UserDataQosPolicy::default(),
                                time_based_filter: TimeBasedFilterQosPolicy::default(),
                                presentation: PresentationQosPolicy::default(),
                                partition: PartitionQosPolicy::default(),
                                topic_data: TopicDataQosPolicy::default(),
                                group_data: GroupDataQosPolicy::default(),
                            },
                        };

                        sedp_builtin_subscription_announcer
                            .write(&sedp_discovered_reader_data, None)
                            .unwrap();
                    }
                }
            }
        }

        Ok(data_reader_shared)
    }

    fn datareader_factory_delete_datareader(
        &self,
        a_datareader: &Self::DataReaderType,
    ) -> DdsResult<()> {
        let data_reader_list = &mut self.data_reader_list.write_lock();
        let data_reader_list_position = data_reader_list
            .iter()
            .position(|x| x == a_datareader)
            .ok_or(DdsError::PreconditionNotMet(
                "Data reader can only be deleted from its parent subscriber".to_string(),
            ))?;
        data_reader_list.remove(data_reader_list_position);

        Ok(())
    }

    fn datareader_factory_lookup_datareader(
        &self,
        topic: &Self::TopicType,
    ) -> DdsResult<Self::DataReaderType> {
        let data_reader_list = &self.data_reader_list.write_lock();

        data_reader_list
            .iter()
            .find_map(|data_reader_shared| {
                let data_reader_topic = &data_reader_shared.topic;

                if data_reader_topic.get_name().ok()? == topic.get_name().ok()?
                    && data_reader_topic.get_type_name().ok()? == Foo::type_name()
                {
                    Some(data_reader_shared.clone())
                } else {
                    None
                }
            })
            .ok_or(DdsError::PreconditionNotMet("Not found".to_string()))
    }
}

impl<Rtps> Subscriber for DdsShared<SubscriberAttributes<Rtps>>
where
    Rtps: RtpsStructure,
{
    type DomainParticipant = DdsWeak<DomainParticipantAttributes<Rtps>>;

    fn begin_access(&self) -> DdsResult<()> {
        todo!()
    }

    fn end_access(&self) -> DdsResult<()> {
        todo!()
    }

    fn get_datareaders(
        &self,
        _readers: &mut [&mut dyn AnyDataReader],
        _sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
    ) -> DdsResult<()> {
        todo!()
    }

    fn notify_datareaders(&self) -> DdsResult<()> {
        todo!()
    }

    fn get_participant(&self) -> DdsResult<Self::DomainParticipant> {
        Ok(self.parent_domain_participant.clone())
    }

    fn get_sample_lost_status(&self, _status: &mut SampleLostStatus) -> DdsResult<()> {
        todo!()
    }

    fn delete_contained_entities(&self) -> DdsResult<()> {
        todo!()
    }

    fn set_default_datareader_qos(&self, _qos: Option<DataReaderQos>) -> DdsResult<()> {
        todo!()
    }

    fn get_default_datareader_qos(&self) -> DdsResult<DataReaderQos> {
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        todo!()
    }
}

impl<Rtps> Entity for DdsShared<SubscriberAttributes<Rtps>>
where
    Rtps: RtpsStructure,
{
    type Qos = SubscriberQos;
    type Listener = Box<dyn SubscriberListener>;

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

    fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        todo!()
    }

    fn get_status_changes(&self) -> DdsResult<StatusMask> {
        todo!()
    }

    fn enable(&self) -> DdsResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use dds_api::{
        dcps_psm::DomainId,
        infrastructure::qos::{DomainParticipantQos, PublisherQos},
        return_type::DdsError,
    };
    use rtps_pim::structure::types::{EntityId, Guid, GuidPrefix};

    use crate::{
        dds_impl::{
            publisher_attributes::PublisherAttributes, subscriber_proxy::SubscriberProxy,
            topic_proxy::TopicProxy,
        },
        dds_type::{DdsDeserialize, DdsType},
        test_utils::{mock_rtps::MockRtps, mock_rtps_group::MockRtpsGroup},
    };

    use super::*;

    macro_rules! make_empty_dds_type {
        ($type_name:ident) => {
            struct $type_name {}

            impl<'de> DdsDeserialize<'de> for $type_name {
                fn deserialize(_buf: &mut &'de [u8]) -> DdsResult<Self> {
                    Ok($type_name {})
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
    fn create_datareader() {
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

        let mut subscriber_attributes = SubscriberAttributes {
            qos: SubscriberQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_reader_list: DdsRwLock::new(Vec::new()),
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
            parent_domain_participant: domain_participant.downgrade(),
        };
        subscriber_attributes
            .rtps_group
            .expect_guid()
            .return_const(Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)));

        let subscriber = DdsShared::new(subscriber_attributes);
        let subscriber_proxy = SubscriberProxy::new(subscriber.downgrade());

        let topic = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));
        let topic_proxy = TopicProxy::<Foo, _>::new(topic.downgrade());

        let data_reader = subscriber_proxy.create_datareader(&topic_proxy, None, None, 0);

        assert!(data_reader.is_ok());
    }

    #[test]
    fn datareader_factory_delete_datareader() {
        let mut domain_participant_attributes = DomainParticipantAttributes::<MockRtps>::new(
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

        let mut subscriber_attributes = SubscriberAttributes {
            qos: SubscriberQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_reader_list: DdsRwLock::new(Vec::new()),
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
            parent_domain_participant: domain_participant.downgrade(),
        };
        subscriber_attributes
            .rtps_group
            .expect_guid()
            .return_const(Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)));
        let subscriber = DdsShared::new(subscriber_attributes);
        let subscriber_proxy = SubscriberProxy::new(subscriber.downgrade());

        let topic = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));
        let topic_proxy = TopicProxy::<Foo, _>::new(topic.downgrade());

        let data_reader = subscriber_proxy
            .datareader_factory_create_datareader(&topic_proxy, None, None, 0)
            .unwrap();

        assert_eq!(1, subscriber.data_reader_list.read_lock().len());

        subscriber_proxy
            .datareader_factory_delete_datareader(&data_reader)
            .unwrap();
        assert_eq!(0, subscriber.data_reader_list.read_lock().len());
        assert!(data_reader.as_ref().upgrade().is_err());
    }

    #[test]
    fn datareader_factory_delete_datareader_from_other_subscriber() {
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

        let mut subscriber_attributes = SubscriberAttributes {
            qos: SubscriberQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_reader_list: DdsRwLock::new(Vec::new()),
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
            parent_domain_participant: domain_participant.downgrade(),
        };
        subscriber_attributes
            .rtps_group
            .expect_guid()
            .return_const(Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)));
        let subscriber = DdsShared::new(subscriber_attributes);
        let subscriber_proxy = SubscriberProxy::new(subscriber.downgrade());

        let mut subscriber2_attributes = SubscriberAttributes {
            qos: SubscriberQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_reader_list: DdsRwLock::new(Vec::new()),
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
            parent_domain_participant: domain_participant.downgrade(),
        };
        subscriber2_attributes
            .rtps_group
            .expect_guid()
            .return_const(Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)));
        let subscriber2 = DdsShared::new(subscriber2_attributes);
        let subscriber2_proxy = SubscriberProxy::new(subscriber2.downgrade());

        let topic = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));
        let topic_proxy = TopicProxy::<Foo, _>::new(topic.downgrade());

        let data_reader = subscriber_proxy
            .datareader_factory_create_datareader(&topic_proxy, None, None, 0)
            .unwrap();

        assert_eq!(1, subscriber.data_reader_list.read_lock().len());
        assert_eq!(0, subscriber2.data_reader_list.read_lock().len());

        assert!(matches!(
            subscriber2_proxy.datareader_factory_delete_datareader(&data_reader),
            Err(DdsError::PreconditionNotMet(_))
        ));
        assert!(data_reader.as_ref().upgrade().is_ok());
    }

    #[test]
    fn datareader_factory_lookup_datareader_when_empty() {
        let domain_participant_attributes = DomainParticipantAttributes::new(
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

        let mut subscriber_attributes: SubscriberAttributes<MockRtps> = SubscriberAttributes {
            qos: SubscriberQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_reader_list: DdsRwLock::new(Vec::new()),
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
            parent_domain_participant: domain_participant.downgrade(),
        };
        subscriber_attributes
            .rtps_group
            .expect_guid()
            .return_const(Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)));
        let subscriber = DdsShared::new(subscriber_attributes);
        let subscriber_proxy = SubscriberProxy::new(subscriber.downgrade());

        let topic = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));
        let topic_proxy = TopicProxy::<Foo, _>::new(topic.downgrade());

        assert!(subscriber_proxy
            .datareader_factory_lookup_datareader(&topic_proxy)
            .is_err());
    }

    #[test]
    fn datareader_factory_lookup_datareader_when_one_datareader() {
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

        let mut subscriber_attributes = SubscriberAttributes {
            qos: SubscriberQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_reader_list: DdsRwLock::new(Vec::new()),
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
            parent_domain_participant: domain_participant.downgrade(),
        };
        subscriber_attributes
            .rtps_group
            .expect_guid()
            .return_const(Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)));
        let subscriber = DdsShared::new(subscriber_attributes);
        let subscriber_proxy = SubscriberProxy::new(subscriber.downgrade());

        let topic = DdsShared::new(TopicAttributes::new(
            TopicQos::default(),
            Foo::type_name(),
            "topic",
            DdsWeak::new(),
        ));
        let topic_proxy = TopicProxy::<Foo, _>::new(topic.downgrade());

        let data_reader = subscriber_proxy
            .datareader_factory_create_datareader(&topic_proxy, None, None, 0)
            .unwrap();

        assert!(
            subscriber_proxy
                .datareader_factory_lookup_datareader(&topic_proxy)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == data_reader.as_ref().upgrade().unwrap()
        );
    }

    make_empty_dds_type!(Bar);

    #[test]
    fn datareader_factory_lookup_datareader_when_one_datareader_with_wrong_type() {
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

        let mut subscriber_attributes = SubscriberAttributes {
            qos: SubscriberQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_reader_list: DdsRwLock::new(Vec::new()),
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
            parent_domain_participant: domain_participant.downgrade(),
        };
        subscriber_attributes
            .rtps_group
            .expect_guid()
            .return_const(Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)));
        let subscriber = DdsShared::new(subscriber_attributes);
        let subscriber_proxy = SubscriberProxy::new(subscriber.downgrade());

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

        subscriber_proxy
            .datareader_factory_create_datareader(&topic_bar_proxy, None, None, 0)
            .unwrap();

        assert!(subscriber_proxy
            .datareader_factory_lookup_datareader(&topic_foo_proxy)
            .is_err());
    }

    #[test]
    fn datareader_factory_lookup_datareader_when_one_datareader_with_wrong_topic() {
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

        let mut subscriber_attributes = SubscriberAttributes {
            qos: SubscriberQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_reader_list: DdsRwLock::new(Vec::new()),
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
            parent_domain_participant: domain_participant.downgrade(),
        };
        subscriber_attributes
            .rtps_group
            .expect_guid()
            .return_const(Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)));
        let subscriber = DdsShared::new(subscriber_attributes);
        let subscriber_proxy = SubscriberProxy::new(subscriber.downgrade());

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

        subscriber_proxy
            .datareader_factory_create_datareader(&topic2_proxy, None, None, 0)
            .unwrap();

        assert!(subscriber_proxy
            .datareader_factory_lookup_datareader(&topic1_proxy)
            .is_err());
    }

    #[test]
    fn datareader_factory_lookup_datareader_with_two_types() {
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

        let mut subscriber_attributes = SubscriberAttributes {
            qos: SubscriberQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_reader_list: DdsRwLock::new(Vec::new()),
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
            parent_domain_participant: domain_participant.downgrade(),
        };
        subscriber_attributes
            .rtps_group
            .expect_guid()
            .return_const(Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)));
        let subscriber = DdsShared::new(subscriber_attributes);
        let subscriber_proxy = SubscriberProxy::new(subscriber.downgrade());

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

        let data_reader_foo = subscriber_proxy
            .datareader_factory_create_datareader(&topic_foo_proxy, None, None, 0)
            .unwrap();
        let data_reader_bar = subscriber_proxy
            .datareader_factory_create_datareader(&topic_bar_proxy, None, None, 0)
            .unwrap();

        assert!(
            subscriber_proxy
                .datareader_factory_lookup_datareader(&topic_foo_proxy)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == data_reader_foo.as_ref().upgrade().unwrap()
        );

        assert!(
            subscriber_proxy
                .datareader_factory_lookup_datareader(&topic_bar_proxy)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == data_reader_bar.as_ref().upgrade().unwrap()
        );
    }

    #[test]
    fn datareader_factory_lookup_datareader_with_two_topics() {
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

        let mut subscriber_attributes = SubscriberAttributes {
            qos: SubscriberQos::default(),
            rtps_group: MockRtpsGroup::new(),
            data_reader_list: DdsRwLock::new(Vec::new()),
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
            parent_domain_participant: domain_participant.downgrade(),
        };
        subscriber_attributes
            .rtps_group
            .expect_guid()
            .return_const(Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)));
        let subscriber = DdsShared::new(subscriber_attributes);
        let subscriber_proxy = SubscriberProxy::new(subscriber.downgrade());

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

        let data_reader1 = subscriber_proxy
            .datareader_factory_create_datareader(&topic1_proxy, None, None, 0)
            .unwrap();
        let data_reader2 = subscriber_proxy
            .datareader_factory_create_datareader(&topic2_proxy, None, None, 0)
            .unwrap();

        assert!(
            subscriber_proxy
                .datareader_factory_lookup_datareader(&topic1_proxy)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == data_reader1.as_ref().upgrade().unwrap()
        );

        assert!(
            subscriber_proxy
                .datareader_factory_lookup_datareader(&topic2_proxy)
                .unwrap()
                .as_ref()
                .upgrade()
                .unwrap()
                == data_reader2.as_ref().upgrade().unwrap()
        );
    }
}
