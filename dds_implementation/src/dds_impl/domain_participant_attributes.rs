use std::{
    sync::atomic::{AtomicU8, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    rtps_impl::{
        rtps_group_impl::RtpsGroupImpl,
        rtps_participant_impl::RtpsParticipantImpl,
        rtps_stateless_writer_impl::{RtpsReaderLocatorAttributesImpl, RtpsStatelessWriterImpl},
        utils::clock::StdTimer,
    },
    utils::rtps_communication_traits::SendRtpsMessage,
};
use dds_api::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    dcps_psm::{
        BuiltInTopicKey, DomainId, InstanceHandle, StatusMask, Time, ANY_INSTANCE_STATE,
        ANY_SAMPLE_STATE, ANY_VIEW_STATE,
    },
    domain::{
        domain_participant::{DomainParticipant, DomainParticipantTopicFactory},
        domain_participant_listener::DomainParticipantListener,
    },
    infrastructure::{
        entity::Entity,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantQos, PublisherQos, SubscriberQos,
            TopicQos,
        },
        qos_policy::{HistoryQosPolicy, HistoryQosPolicyKind},
    },
    publication::{data_writer::DataWriter, publisher::Publisher},
    return_type::{DdsError, DdsResult},
    subscription::{data_reader::DataReader, subscriber::Subscriber},
    topic::topic_description::TopicDescription,
};
use rtps_pim::{
    behavior::writer::reader_locator::RtpsReaderLocatorConstructor,
    discovery::{
        participant_discovery::ParticipantDiscovery,
        sedp::builtin_endpoints::{
            SedpBuiltinPublicationsReader, SedpBuiltinPublicationsWriter,
            SedpBuiltinSubscriptionsReader, SedpBuiltinSubscriptionsWriter,
            SedpBuiltinTopicsReader, SedpBuiltinTopicsWriter,
        },
        spdp::builtin_endpoints::{SpdpBuiltinParticipantReader, SpdpBuiltinParticipantWriter},
        types::{BuiltinEndpointQos, BuiltinEndpointSet},
    },
    messages::types::Count,
    structure::{
        entity::RtpsEntityAttributes,
        group::RtpsGroupConstructor,
        participant::{RtpsParticipantAttributes, RtpsParticipantConstructor},
        types::{
            EntityId, Guid, GuidPrefix, Locator, ProtocolVersion, VendorId, BUILT_IN_READER_GROUP,
            BUILT_IN_WRITER_GROUP, ENTITYID_PARTICIPANT, PROTOCOLVERSION,
            USER_DEFINED_READER_GROUP, USER_DEFINED_WRITER_GROUP, VENDOR_ID_S2E,
        },
    },
};

use crate::{
    data_representation_builtin_endpoints::{
        discovered_reader_data::{DiscoveredReaderData, DCPS_SUBSCRIPTION},
        discovered_topic_data::{DiscoveredTopicData, DCPS_TOPIC},
        discovered_writer_data::{DiscoveredWriterData, DCPS_PUBLICATION},
        spdp_discovered_participant_data::{
            ParticipantProxy, SpdpDiscoveredParticipantData, DCPS_PARTICIPANT,
        },
    },
    dds_type::DdsType,
    transport::{TransportRead, TransportWrite},
    utils::{
        discovery_traits::{AddMatchedReader, AddMatchedWriter},
        shared_object::{DdsRwLock, DdsShared},
        timer::ThreadTimer,
    },
};

use super::{
    data_reader_attributes::{DataReaderAttributes, DataReaderConstructor, RtpsReader},
    data_writer_attributes::{DataWriterAttributes, DataWriterConstructor, RtpsWriter},
    message_receiver::MessageReceiver,
    publisher_attributes::{
        AddDataWriter, PublisherAttributes, PublisherConstructor, PublisherEmpty,
    },
    subscriber_attributes::{
        AddDataReader, SubscriberAttributes, SubscriberConstructor, SubscriberEmpty,
    },
    topic_attributes::TopicAttributes,
};

pub struct DomainParticipantAttributes {
    rtps_participant: RtpsParticipantImpl,
    domain_id: DomainId,
    domain_tag: String,
    qos: DomainParticipantQos,
    builtin_subscriber: DdsRwLock<Option<DdsShared<SubscriberAttributes>>>,
    builtin_publisher: DdsRwLock<Option<DdsShared<PublisherAttributes>>>,
    user_defined_subscriber_list: DdsRwLock<Vec<DdsShared<SubscriberAttributes>>>,
    user_defined_subscriber_counter: AtomicU8,
    default_subscriber_qos: SubscriberQos,
    user_defined_publisher_list: DdsRwLock<Vec<DdsShared<PublisherAttributes>>>,
    user_defined_publisher_counter: AtomicU8,
    default_publisher_qos: PublisherQos,
    topic_list: DdsRwLock<Vec<DdsShared<TopicAttributes<DomainParticipantAttributes>>>>,
    default_topic_qos: TopicQos,
    manual_liveliness_count: Count,
    lease_duration: rtps_pim::behavior::types::Duration,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    enabled: DdsRwLock<bool>,
}

pub trait DomainParticipantConstructor {
    fn new(
        guid_prefix: GuidPrefix,
        domain_id: DomainId,
        domain_tag: String,
        domain_participant_qos: DomainParticipantQos,
        metatraffic_unicast_locator_list: Vec<Locator>,
        metatraffic_multicast_locator_list: Vec<Locator>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
    ) -> Self;
}

impl DomainParticipantConstructor for DdsShared<DomainParticipantAttributes> {
    fn new(
        guid_prefix: GuidPrefix,
        domain_id: DomainId,
        domain_tag: String,
        domain_participant_qos: DomainParticipantQos,
        metatraffic_unicast_locator_list: Vec<Locator>,
        metatraffic_multicast_locator_list: Vec<Locator>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
    ) -> Self {
        let lease_duration = rtps_pim::behavior::types::Duration::new(100, 0);
        let protocol_version = PROTOCOLVERSION;
        let vendor_id = VENDOR_ID_S2E;
        let rtps_participant = RtpsParticipantImpl::new(
            Guid::new(guid_prefix, ENTITYID_PARTICIPANT),
            &default_unicast_locator_list,
            &default_multicast_locator_list,
            protocol_version,
            vendor_id,
        );

        DdsShared::new(DomainParticipantAttributes {
            rtps_participant,
            domain_id,
            domain_tag,
            qos: domain_participant_qos,
            builtin_subscriber: DdsRwLock::new(None),
            builtin_publisher: DdsRwLock::new(None),
            user_defined_subscriber_list: DdsRwLock::new(Vec::new()),
            user_defined_subscriber_counter: AtomicU8::new(0),
            default_subscriber_qos: SubscriberQos::default(),
            user_defined_publisher_list: DdsRwLock::new(Vec::new()),
            user_defined_publisher_counter: AtomicU8::new(0),
            default_publisher_qos: PublisherQos::default(),
            topic_list: DdsRwLock::new(Vec::new()),
            default_topic_qos: TopicQos::default(),
            manual_liveliness_count: Count(0),
            lease_duration: lease_duration,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            enabled: DdsRwLock::new(false),
        })
    }
}

impl<Foo> DomainParticipantTopicFactory<Foo> for DdsShared<DomainParticipantAttributes>
where
    Foo: DdsType,
{
    type TopicType = DdsShared<TopicAttributes<DomainParticipantAttributes>>;

    fn topic_factory_create_topic(
        &self,
        topic_name: &str,
        qos: Option<TopicQos>,
        _a_listener: Option<<Self::TopicType as Entity>::Listener>,
        _mask: StatusMask,
    ) -> DdsResult<Self::TopicType>
    where
        Self::TopicType: Entity,
    {
        let qos = qos.unwrap_or(self.default_topic_qos.clone());

        // /////// Create topic
        let topic_shared =
            TopicAttributes::new(qos.clone(), Foo::type_name(), topic_name, self.downgrade());

        self.topic_list.write_lock().push(topic_shared.clone());

        // /////// Announce the topic creation
        {
            let builtin_publisher_option = self.builtin_publisher.read_lock().clone();
            if let Some(builtin_publisher) = builtin_publisher_option {
                if let Ok(topic_creation_topic) =
                    self.lookup_topicdescription::<DiscoveredTopicData>(DCPS_TOPIC)
                {
                    if let Ok(sedp_builtin_topic_announcer) = builtin_publisher
                        .lookup_datawriter::<DiscoveredTopicData>(&topic_creation_topic)
                    {
                        let sedp_discovered_topic_data = DiscoveredTopicData {
                            topic_builtin_topic_data: TopicBuiltinTopicData {
                                key: BuiltInTopicKey { value: [1; 16] },
                                name: topic_name.to_string(),
                                type_name: Foo::type_name().to_string(),
                                durability: qos.durability.clone(),
                                durability_service: qos.durability_service.clone(),
                                deadline: qos.deadline.clone(),
                                latency_budget: qos.latency_budget.clone(),
                                liveliness: qos.liveliness.clone(),
                                reliability: qos.reliability.clone(),
                                transport_priority: qos.transport_priority.clone(),
                                lifespan: qos.lifespan.clone(),
                                destination_order: qos.destination_order.clone(),
                                history: qos.history.clone(),
                                resource_limits: qos.resource_limits.clone(),
                                ownership: qos.ownership.clone(),
                                topic_data: qos.topic_data.clone(),
                            },
                        };

                        sedp_builtin_topic_announcer
                            .write(&sedp_discovered_topic_data, None)
                            .unwrap();
                    }
                }
            }
        }

        Ok(topic_shared)
    }

    fn topic_factory_delete_topic(&self, a_topic: &Self::TopicType) -> DdsResult<()> {
        let mut topic_list = self.topic_list.write_lock();
        let topic_list_position = topic_list.iter().position(|topic| topic == a_topic).ok_or(
            DdsError::PreconditionNotMet(
                "Topic can only be deleted from its parent publisher".to_string(),
            ),
        )?;

        // If topic is not attached to any reader or writer there must be no more than 2 strong counts
        // 1 strong stored in the list of the participant and 1 strong used to call this function
        if a_topic.strong_count() > 2 {
            return Err(DdsError::PreconditionNotMet(
                "Topic still attached to some data reader or data writer".to_string(),
            ));
        }

        topic_list.remove(topic_list_position);

        Ok(())
    }

    fn topic_factory_find_topic(
        &self,
        topic_name: &str,
        _timeout: dds_api::dcps_psm::Duration,
    ) -> DdsResult<Self::TopicType> {
        self.topic_list
            .read_lock()
            .iter()
            .find_map(|topic| {
                if topic.get_name().unwrap() == topic_name
                    && topic.get_type_name().unwrap() == Foo::type_name()
                {
                    Some(topic.clone())
                } else {
                    None
                }
            })
            .ok_or(DdsError::PreconditionNotMet("Not found".to_string()))
    }

    fn topic_factory_lookup_topicdescription(
        &self,
        topic_name: &str,
    ) -> DdsResult<Self::TopicType> {
        self.topic_list
            .read_lock()
            .iter()
            .find_map(|topic| {
                if topic.get_name().unwrap() == topic_name
                    && topic.get_type_name().unwrap() == Foo::type_name()
                {
                    Some(topic.clone())
                } else {
                    None
                }
            })
            .ok_or(DdsError::PreconditionNotMet("Not found".to_string()))
    }
}

impl DomainParticipant for DdsShared<DomainParticipantAttributes> {
    type PublisherType = DdsShared<PublisherAttributes>;
    type SubscriberType = DdsShared<SubscriberAttributes>;

    fn create_publisher(
        &self,
        qos: Option<PublisherQos>,
        _a_listener: Option<<Self::PublisherType as Entity>::Listener>,
        _mask: StatusMask,
    ) -> DdsResult<Self::PublisherType>
    where
        Self::PublisherType: Entity,
    {
        let publisher_qos = qos.unwrap_or(self.default_publisher_qos.clone());
        let publisher_counter = self
            .user_defined_publisher_counter
            .fetch_add(1, Ordering::Relaxed);
        let entity_id = EntityId::new([publisher_counter, 0, 0], USER_DEFINED_WRITER_GROUP);
        let guid = Guid::new(self.rtps_participant.guid().prefix(), entity_id);
        let rtps_group = RtpsGroupImpl::new(guid);
        let publisher_impl_shared: DdsShared<PublisherAttributes> =
            PublisherConstructor::new(publisher_qos, rtps_group, self.downgrade());
        self.user_defined_publisher_list
            .write_lock()
            .push(publisher_impl_shared.clone());

        Ok(publisher_impl_shared)
    }

    fn delete_publisher(&self, a_publisher: &Self::PublisherType) -> DdsResult<()> {
        if !DdsShared::ptr_eq(&a_publisher.get_participant()?.upgrade()?, self) {
            return Err(DdsError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant".to_string(),
            ));
        }

        if !a_publisher.is_empty() {
            return Err(DdsError::PreconditionNotMet(
                "Publisher still contains data writers".to_string(),
            ));
        }

        self.user_defined_publisher_list
            .write_lock()
            .retain(|x| x != a_publisher);

        Ok(())
    }

    fn create_subscriber(
        &self,
        qos: Option<SubscriberQos>,
        _a_listener: Option<<Self::SubscriberType as Entity>::Listener>,
        _mask: StatusMask,
    ) -> DdsResult<Self::SubscriberType>
    where
        Self::SubscriberType: Entity,
    {
        let subscriber_qos = qos.unwrap_or(self.default_subscriber_qos.clone());
        let subcriber_counter = self
            .user_defined_subscriber_counter
            .fetch_add(1, Ordering::Relaxed);
        let entity_id = EntityId::new([subcriber_counter, 0, 0], USER_DEFINED_READER_GROUP);
        let guid = Guid::new(self.rtps_participant.guid().prefix(), entity_id);
        let rtps_group = RtpsGroupImpl::new(guid);
        let subscriber_shared: DdsShared<SubscriberAttributes> =
            SubscriberConstructor::new(subscriber_qos, rtps_group, self.downgrade());
        self.user_defined_subscriber_list
            .write_lock()
            .push(subscriber_shared.clone());

        Ok(subscriber_shared)
    }

    fn delete_subscriber(&self, a_subscriber: &Self::SubscriberType) -> DdsResult<()> {
        if !DdsShared::ptr_eq(&a_subscriber.get_participant()?.upgrade()?, self) {
            return Err(DdsError::PreconditionNotMet(
                "Subscriber can only be deleted from its parent participant".to_string(),
            ));
        }

        if !a_subscriber.is_empty() {
            return Err(DdsError::PreconditionNotMet(
                "Subscriber still contains data readers".to_string(),
            ));
        }

        self.user_defined_subscriber_list
            .write_lock()
            .retain(|x| x != a_subscriber);
        Ok(())
    }

    fn get_builtin_subscriber(&self) -> DdsResult<Self::SubscriberType> {
        Ok(self
            .builtin_subscriber
            .read_lock()
            .as_ref()
            .unwrap()
            .clone())
    }

    fn ignore_participant(&self, _handle: InstanceHandle) -> DdsResult<()> {
        todo!()
    }

    fn ignore_topic(&self, _handle: InstanceHandle) -> DdsResult<()> {
        todo!()
    }

    fn ignore_publication(&self, _handle: InstanceHandle) -> DdsResult<()> {
        todo!()
    }

    fn ignore_subscription(&self, _handle: InstanceHandle) -> DdsResult<()> {
        todo!()
    }

    fn get_domain_id(&self) -> DdsResult<DomainId> {
        todo!()
    }

    fn delete_contained_entities(&self) -> DdsResult<()> {
        todo!()
    }

    fn assert_liveliness(&self) -> DdsResult<()> {
        todo!()
    }

    fn set_default_publisher_qos(&self, _qos: Option<PublisherQos>) -> DdsResult<()> {
        todo!()
    }

    fn get_default_publisher_qos(&self) -> DdsResult<PublisherQos> {
        todo!()
    }

    fn set_default_subscriber_qos(&self, _qos: Option<SubscriberQos>) -> DdsResult<()> {
        todo!()
    }

    fn get_default_subscriber_qos(&self) -> DdsResult<SubscriberQos> {
        todo!()
    }

    fn set_default_topic_qos(&self, _qos: Option<TopicQos>) -> DdsResult<()> {
        todo!()
    }

    fn get_default_topic_qos(&self) -> DdsResult<TopicQos> {
        Ok(self.default_topic_qos.clone())
    }

    fn get_discovered_participants(&self) -> DdsResult<Vec<InstanceHandle>> {
        todo!()
    }

    fn get_discovered_participant_data(
        &self,
        _participant_data: ParticipantBuiltinTopicData,
        _participant_handle: InstanceHandle,
    ) -> DdsResult<()> {
        todo!()
    }

    fn get_discovered_topics(&self, _topic_handles: &mut [InstanceHandle]) -> DdsResult<()> {
        todo!()
    }

    fn get_discovered_topic_data(
        &self,
        _topic_data: TopicBuiltinTopicData,
        _topic_handle: InstanceHandle,
    ) -> DdsResult<()> {
        todo!()
    }

    fn contains_entity(&self, _a_handle: InstanceHandle) -> DdsResult<bool> {
        todo!()
    }

    fn get_current_time(&self) -> DdsResult<dds_api::dcps_psm::Time> {
        let now_system_time = SystemTime::now();
        match now_system_time.duration_since(UNIX_EPOCH) {
            Ok(unix_time) => Ok(Time {
                sec: unix_time.as_secs() as i32,
                nanosec: unix_time.subsec_nanos(),
            }),
            Err(_) => Err(DdsError::Error),
        }
    }
}

impl Entity for DdsShared<DomainParticipantAttributes> {
    type Qos = DomainParticipantQos;
    type Listener = Box<dyn DomainParticipantListener>;

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
        *self.enabled.write_lock() = true;
        Ok(())
    }

    fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        todo!()
    }
}

pub trait AnnounceParticipant {
    fn announce_participant(&self) -> DdsResult<()>;
}

impl AnnounceParticipant for DdsShared<DomainParticipantAttributes> {
    fn announce_participant(&self) -> DdsResult<()> {
        let dcps_topic_participant =
            self.lookup_topicdescription::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT)?;
        let builtin_publisher = self.builtin_publisher.read_lock();

        let spdp_participant_writer =
            builtin_publisher
                .as_ref()
                .unwrap()
                .lookup_datawriter::<SpdpDiscoveredParticipantData>(&dcps_topic_participant)?;

        let spdp_discovered_participant_data = SpdpDiscoveredParticipantData {
            dds_participant_data: ParticipantBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: self.rtps_participant.guid().into(),
                },
                user_data: self.qos.user_data.clone(),
            },
            participant_proxy: ParticipantProxy {
                domain_id: self.domain_id as u32,
                domain_tag: self.domain_tag.clone(),
                protocol_version: self.rtps_participant.protocol_version(),
                guid_prefix: self.rtps_participant.guid().prefix(),
                vendor_id: self.rtps_participant.vendor_id(),
                expects_inline_qos: false,
                metatraffic_unicast_locator_list: self.metatraffic_unicast_locator_list.clone(),
                metatraffic_multicast_locator_list: self.metatraffic_multicast_locator_list.clone(),
                default_unicast_locator_list: self
                    .rtps_participant
                    .default_unicast_locator_list()
                    .to_vec(),
                default_multicast_locator_list: self
                    .rtps_participant
                    .default_multicast_locator_list()
                    .to_vec(),
                available_builtin_endpoints: BuiltinEndpointSet::default(),
                manual_liveliness_count: self.manual_liveliness_count,
                builtin_endpoint_qos: BuiltinEndpointQos::default(),
            },
            lease_duration: self.lease_duration,
        };
        spdp_participant_writer.write(&spdp_discovered_participant_data, None)
    }
}

pub trait AddDiscoveredParticipant {
    fn add_discovered_participant(
        &self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    );
}

impl AddDiscoveredParticipant for DdsShared<DomainParticipantAttributes> {
    fn add_discovered_participant(
        &self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        let dcps_publication_topic = self
            .lookup_topicdescription::<DiscoveredWriterData>(DCPS_PUBLICATION)
            .unwrap();
        let dcps_subscription_topic = self
            .lookup_topicdescription::<DiscoveredReaderData>(DCPS_SUBSCRIPTION)
            .unwrap();
        let dcps_topic_topic = self
            .lookup_topicdescription::<DiscoveredTopicData>(DCPS_TOPIC)
            .unwrap();

        if let Ok(participant_discovery) = ParticipantDiscovery::new(
            discovered_participant_data,
            self.domain_id as u32,
            &self.domain_tag,
        ) {
            let builtin_publisher_lock = self.builtin_publisher.read_lock();
            let builtin_subscriber_lock = self.builtin_subscriber.read_lock();
            let builtin_publisher = builtin_publisher_lock.as_ref().unwrap();
            let builtin_subscriber = builtin_subscriber_lock.as_ref().unwrap();
            let sedp_builtin_publication_writer_shared = builtin_publisher
                .lookup_datawriter::<DiscoveredWriterData>(&dcps_publication_topic)
                .unwrap();
            sedp_builtin_publication_writer_shared.add_matched_participant(&participant_discovery);

            let sedp_builtin_publication_reader_shared = builtin_subscriber
                .lookup_datareader::<DiscoveredWriterData>(&dcps_publication_topic)
                .unwrap();
            sedp_builtin_publication_reader_shared.add_matched_participant(&participant_discovery);

            let sedp_builtin_subscription_writer_shared = builtin_publisher
                .lookup_datawriter::<DiscoveredReaderData>(&dcps_subscription_topic)
                .unwrap();
            sedp_builtin_subscription_writer_shared.add_matched_participant(&participant_discovery);

            let sedp_builtin_subscription_reader_shared = builtin_subscriber
                .lookup_datareader::<DiscoveredReaderData>(&dcps_subscription_topic)
                .unwrap();
            sedp_builtin_subscription_reader_shared.add_matched_participant(&participant_discovery);

            let sedp_builtin_topic_writer_shared = builtin_publisher
                .lookup_datawriter::<DiscoveredTopicData>(&dcps_topic_topic)
                .unwrap();
            sedp_builtin_topic_writer_shared.add_matched_participant(&participant_discovery);

            let sedp_builtin_topic_reader_shared = builtin_subscriber
                .lookup_datareader::<DiscoveredTopicData>(&dcps_topic_topic)
                .unwrap();
            sedp_builtin_topic_reader_shared.add_matched_participant(&participant_discovery);
        }
    }
}

impl RtpsParticipantAttributes for DdsShared<DomainParticipantAttributes> {
    fn default_unicast_locator_list(&self) -> &[Locator] {
        self.rtps_participant.default_unicast_locator_list()
    }

    fn default_multicast_locator_list(&self) -> &[Locator] {
        self.rtps_participant.default_multicast_locator_list()
    }

    fn protocol_version(&self) -> ProtocolVersion {
        self.rtps_participant.protocol_version()
    }

    fn vendor_id(&self) -> VendorId {
        self.rtps_participant.vendor_id()
    }
}

pub trait DataWriterDiscovery {
    fn add_created_data_writer(&self, writer_data: &DiscoveredWriterData);
}

impl DataWriterDiscovery for DdsShared<DomainParticipantAttributes> {
    fn add_created_data_writer(&self, writer_data: &DiscoveredWriterData) {
        let builtin_publisher = self.builtin_publisher.read_lock();
        if let Some(builtin_publisher) = builtin_publisher.as_ref() {
            if let Ok(publication_topic) =
                self.lookup_topicdescription::<DiscoveredWriterData>(DCPS_PUBLICATION)
            {
                if let Ok(sedp_builtin_publications_announcer) =
                    builtin_publisher.lookup_datawriter::<DiscoveredWriterData>(&publication_topic)
                {
                    sedp_builtin_publications_announcer
                        .write(writer_data, None)
                        .unwrap();
                }
            }
        }
    }
}

pub trait DataReaderDiscovery {
    fn add_created_data_reader(&self, reader_data: &DiscoveredReaderData);
}

impl DataReaderDiscovery for DdsShared<DomainParticipantAttributes> {
    fn add_created_data_reader(&self, reader_data: &DiscoveredReaderData) {
        let builtin_publisher = self.builtin_publisher.read_lock();
        if let Some(builtin_publisher) = builtin_publisher.as_ref() {
            if let Ok(subscription_topic) =
                self.lookup_topicdescription::<DiscoveredReaderData>(DCPS_SUBSCRIPTION)
            {
                if let Ok(sedp_builtin_subscription_announcer) =
                    builtin_publisher.lookup_datawriter::<DiscoveredReaderData>(&subscription_topic)
                {
                    sedp_builtin_subscription_announcer
                        .write(reader_data, None)
                        .unwrap();
                }
            }
        }
    }
}

pub trait SendBuiltInData {
    fn send_built_in_data(&self, transport: &mut impl TransportWrite);
}

impl SendBuiltInData for DdsShared<DomainParticipantAttributes> {
    fn send_built_in_data(&self, transport: &mut impl TransportWrite) {
        let builtin_publisher = self.builtin_publisher.read_lock();
        let builtin_subscriber = self.builtin_subscriber.read_lock();
        if let (Some(builtin_publisher), Some(builtin_subscriber)) =
            (builtin_publisher.as_ref(), builtin_subscriber.as_ref())
        {
            builtin_publisher.send_message(transport);
            builtin_subscriber.send_message(transport);
        } else {
            println!("/!\\ Participant doesn't have a builtin publisher and a builtin subscriber");
        }
    }
}

pub trait ReceiveBuiltInData {
    fn receive_built_in_data(&self, transport: &mut impl for<'a> TransportRead<'a>);
}

impl ReceiveBuiltInData for DdsShared<DomainParticipantAttributes> {
    fn receive_built_in_data(&self, transport: &mut impl for<'a> TransportRead<'a>) {
        let publisher_list = self.builtin_publisher.read_lock();
        let subscriber_list = self.builtin_subscriber.read_lock();
        while let Some((source_locator, message)) = transport.read() {
            MessageReceiver::new().process_message(
                self.rtps_participant.guid().prefix,
                core::slice::from_ref(publisher_list.as_ref().unwrap()),
                core::slice::from_ref(subscriber_list.as_ref().unwrap()),
                source_locator,
                &message,
            );
        }
    }
}

pub trait CreateBuiltIns {
    fn create_builtins(&self) -> DdsResult<()>;
}

impl CreateBuiltIns for DdsShared<DomainParticipantAttributes> {
    fn create_builtins(&self) -> DdsResult<()> {
        let guid_prefix = self.rtps_participant.guid().prefix;
        ///////// Create the built-in publisher and subcriber

        let builtin_subscriber: DdsShared<SubscriberAttributes> = SubscriberConstructor::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(Guid::new(
                guid_prefix,
                EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP),
            )),
            self.downgrade(),
        );

        *self.builtin_subscriber.write_lock() = Some(builtin_subscriber);

        let builtin_publisher: DdsShared<PublisherAttributes> = PublisherConstructor::new(
            PublisherQos::default(),
            RtpsGroupImpl::new(Guid::new(
                guid_prefix,
                EntityId::new([0, 0, 0], BUILT_IN_WRITER_GROUP),
            )),
            self.downgrade(),
        );

        *self.builtin_publisher.write_lock() = Some(builtin_publisher);

        ///////// Create built-in DDS data readers and data writers

        let builtin_reader_qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAllHistoryQos,
                depth: 0,
            },
            ..Default::default()
        };

        ////////// SPDP built-in topic, reader and writer
        {
            let spdp_topic_participant = self.create_topic::<SpdpDiscoveredParticipantData>(
                DCPS_PARTICIPANT,
                Some(self.get_default_topic_qos()?),
                None,
                0,
            )?;

            let spdp_builtin_participant_rtps_reader =
                SpdpBuiltinParticipantReader::create(guid_prefix, &[], &[]);

            let spdp_builtin_participant_data_reader: DdsShared<DataReaderAttributes<ThreadTimer>> =
                DataReaderConstructor::new(
                    builtin_reader_qos.clone(),
                    RtpsReader::Stateless(spdp_builtin_participant_rtps_reader),
                    spdp_topic_participant.clone(),
                    None,
                    self.builtin_subscriber
                        .read_lock()
                        .clone()
                        .unwrap()
                        .downgrade(),
                );
            self.builtin_subscriber
                .write_lock()
                .as_ref()
                .unwrap()
                .add_data_reader(spdp_builtin_participant_data_reader);

            let spdp_reader_locators: Vec<RtpsReaderLocatorAttributesImpl> = self
                .metatraffic_multicast_locator_list
                .iter()
                .map(|locator| RtpsReaderLocatorAttributesImpl::new(locator.clone(), false))
                .collect();

            let spdp_builtin_participant_rtps_writer =
                SpdpBuiltinParticipantWriter::create::<RtpsStatelessWriterImpl<StdTimer>, _>(
                    guid_prefix,
                    &[],
                    &[],
                    spdp_reader_locators,
                );

            let spdp_builtin_participant_data_writer: DdsShared<DataWriterAttributes> =
                DataWriterConstructor::new(
                    DataWriterQos::default(),
                    RtpsWriter::Stateless(spdp_builtin_participant_rtps_writer),
                    None,
                    spdp_topic_participant.clone(),
                    self.builtin_publisher
                        .read_lock()
                        .clone()
                        .unwrap()
                        .downgrade(),
                );
            self.builtin_publisher
                .write_lock()
                .as_ref()
                .unwrap()
                .add_data_writer(spdp_builtin_participant_data_writer);
        }

        ////////// SEDP built-in publication topic, reader and writer
        {
            let sedp_topic_publication = self.create_topic::<DiscoveredWriterData>(
                DCPS_PUBLICATION,
                Some(self.get_default_topic_qos()?),
                None,
                0,
            )?;

            let sedp_builtin_publications_rtps_reader =
                SedpBuiltinPublicationsReader::create(guid_prefix, &[], &[]);
            let sedp_builtin_publications_data_reader: DdsShared<
                DataReaderAttributes<ThreadTimer>,
            > = DataReaderConstructor::new(
                builtin_reader_qos.clone(),
                RtpsReader::Stateful(sedp_builtin_publications_rtps_reader),
                sedp_topic_publication.clone(),
                None,
                self.builtin_subscriber
                    .read_lock()
                    .clone()
                    .unwrap()
                    .downgrade(),
            );
            self.builtin_subscriber
                .write_lock()
                .as_ref()
                .unwrap()
                .add_data_reader(sedp_builtin_publications_data_reader);

            let sedp_builtin_publications_rtps_writer =
                SedpBuiltinPublicationsWriter::create(guid_prefix, &[], &[]);
            let sedp_builtin_publications_data_writer: DdsShared<DataWriterAttributes> =
                DataWriterConstructor::new(
                    DataWriterQos::default(),
                    RtpsWriter::Stateful(sedp_builtin_publications_rtps_writer),
                    None,
                    sedp_topic_publication.clone(),
                    self.builtin_publisher
                        .read_lock()
                        .clone()
                        .unwrap()
                        .downgrade(),
                );

            self.builtin_publisher
                .write_lock()
                .as_ref()
                .unwrap()
                .add_data_writer(sedp_builtin_publications_data_writer);
        }

        ////////// SEDP built-in subcriptions topic, reader and writer
        {
            let sedp_topic_subscription = self.create_topic::<DiscoveredReaderData>(
                DCPS_SUBSCRIPTION,
                Some(self.get_default_topic_qos()?),
                None,
                0,
            )?;

            let sedp_builtin_subscriptions_rtps_reader =
                SedpBuiltinSubscriptionsReader::create(guid_prefix, &[], &[]);
            let sedp_builtin_subscriptions_data_reader: DdsShared<
                DataReaderAttributes<ThreadTimer>,
            > = DataReaderConstructor::new(
                builtin_reader_qos.clone(),
                RtpsReader::Stateful(sedp_builtin_subscriptions_rtps_reader),
                sedp_topic_subscription.clone(),
                None,
                self.builtin_subscriber
                    .read_lock()
                    .clone()
                    .unwrap()
                    .downgrade(),
            );
            self.builtin_subscriber
                .write_lock()
                .as_ref()
                .unwrap()
                .add_data_reader(sedp_builtin_subscriptions_data_reader);

            let sedp_builtin_subscriptions_rtps_writer =
                SedpBuiltinSubscriptionsWriter::create(guid_prefix, &[], &[]);
            let sedp_builtin_subscriptions_data_writer: DdsShared<DataWriterAttributes> =
                DataWriterConstructor::new(
                    DataWriterQos::default(),
                    RtpsWriter::Stateful(sedp_builtin_subscriptions_rtps_writer),
                    None,
                    sedp_topic_subscription.clone(),
                    self.builtin_publisher
                        .read_lock()
                        .clone()
                        .unwrap()
                        .downgrade(),
                );
            self.builtin_publisher
                .write_lock()
                .as_ref()
                .unwrap()
                .add_data_writer(sedp_builtin_subscriptions_data_writer);
        }

        ////////// SEDP built-in topics topic, reader and writer
        {
            let sedp_topic_topic = self.create_topic::<DiscoveredTopicData>(
                DCPS_TOPIC,
                Some(self.get_default_topic_qos()?),
                None,
                0,
            )?;

            let sedp_builtin_topics_rtps_reader =
                SedpBuiltinTopicsReader::create(guid_prefix, &[], &[]);
            let sedp_builtin_topics_data_reader: DdsShared<DataReaderAttributes<ThreadTimer>> =
                DataReaderConstructor::new(
                    builtin_reader_qos.clone(),
                    RtpsReader::Stateful(sedp_builtin_topics_rtps_reader),
                    sedp_topic_topic.clone(),
                    None,
                    self.builtin_subscriber
                        .read_lock()
                        .clone()
                        .unwrap()
                        .downgrade(),
                );
            self.builtin_subscriber
                .write_lock()
                .as_ref()
                .unwrap()
                .add_data_reader(sedp_builtin_topics_data_reader);

            let sedp_builtin_topics_rtps_writer =
                SedpBuiltinTopicsWriter::create(guid_prefix, &[], &[]);
            let sedp_builtin_topics_data_writer: DdsShared<DataWriterAttributes> =
                DataWriterConstructor::new(
                    DataWriterQos::default(),
                    RtpsWriter::Stateful(sedp_builtin_topics_rtps_writer),
                    None,
                    sedp_topic_topic.clone(),
                    self.builtin_publisher
                        .read_lock()
                        .clone()
                        .unwrap()
                        .downgrade(),
                );
            self.builtin_publisher
                .write_lock()
                .as_ref()
                .unwrap()
                .add_data_writer(sedp_builtin_topics_data_writer);
        }

        Ok(())
    }
}

pub trait SendUserDefinedData {
    fn send_user_defined_data(&self, transport: &mut impl TransportWrite);
}

impl SendUserDefinedData for DdsShared<DomainParticipantAttributes> {
    fn send_user_defined_data(&self, transport: &mut impl TransportWrite) {
        let user_defined_publisher_list = self.user_defined_publisher_list.read_lock();
        let user_defined_subscriber_list = self.user_defined_subscriber_list.read_lock();

        for publisher in user_defined_publisher_list.iter() {
            publisher.send_message(transport)
        }

        for subscriber in user_defined_subscriber_list.iter() {
            subscriber.send_message(transport)
        }
    }
}

pub trait ReceiveUserDefinedData {
    fn receive_user_defined_data(&self, transport: &mut impl for<'a> TransportRead<'a>);
}

impl ReceiveUserDefinedData for DdsShared<DomainParticipantAttributes> {
    fn receive_user_defined_data(&self, transport: &mut impl for<'a> TransportRead<'a>) {
        let user_defined_publisher_list = self.user_defined_publisher_list.read_lock();
        let user_defined_subscriber_list = self.user_defined_subscriber_list.read_lock();
        while let Some((source_locator, message)) = transport.read() {
            MessageReceiver::new().process_message(
                self.rtps_participant.guid().prefix,
                user_defined_publisher_list.as_slice(),
                user_defined_subscriber_list.as_slice(),
                source_locator,
                &message,
            );
        }
    }
}

pub trait SpdpParticipantDiscovery {
    fn discover_matched_participants(&self) -> DdsResult<()>;
}

impl SpdpParticipantDiscovery for DdsShared<DomainParticipantAttributes> {
    fn discover_matched_participants(&self) -> DdsResult<()> {
        let builtin_subscriber = self.builtin_subscriber.read_lock();

        let dcps_participant_topic =
            self.lookup_topicdescription::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT)?;

        let spdp_builtin_participant_data_reader =
            builtin_subscriber
                .as_ref()
                .unwrap()
                .lookup_datareader::<SpdpDiscoveredParticipantData>(&dcps_participant_topic)?;

        if let Ok(samples) = spdp_builtin_participant_data_reader.take(
            1,
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        ) {
            for (discovered_participant_data, _) in samples.iter() {
                self.add_discovered_participant(discovered_participant_data)
            }
        }

        Ok(())
    }
}

pub trait SedpWriterDiscovery {
    fn discover_matched_writers(&self) -> DdsResult<()>;
}

impl SedpWriterDiscovery for DdsShared<DomainParticipantAttributes> {
    fn discover_matched_writers(&self) -> DdsResult<()> {
        let user_defined_subscribers = self.user_defined_subscriber_list.read_lock();

        if user_defined_subscribers.is_empty() {
            return Ok(());
        }

        let builtin_subscriber = self.builtin_subscriber.read_lock();

        let dcps_publication_topic =
            self.lookup_topicdescription::<DiscoveredWriterData>(DCPS_PUBLICATION)?;
        let sedp_builtin_publication_reader =
            builtin_subscriber
                .as_ref()
                .unwrap()
                .lookup_datareader::<DiscoveredWriterData>(&dcps_publication_topic)?;

        let samples = sedp_builtin_publication_reader.take(
            1,
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        );

        for (sample, _) in samples.unwrap_or(vec![]).iter() {
            for subscriber in user_defined_subscribers.iter() {
                subscriber.add_matched_writer(&sample);
            }
        }

        Ok(())
    }
}

pub trait SedpReaderDiscovery {
    fn discover_matched_readers(&self) -> DdsResult<()>;
}

impl SedpReaderDiscovery for DdsShared<DomainParticipantAttributes> {
    fn discover_matched_readers(&self) -> DdsResult<()> {
        let user_defined_publishers = self.user_defined_publisher_list.read_lock();

        if user_defined_publishers.is_empty() {
            return Ok(());
        }

        let builtin_subscriber = self.builtin_subscriber.read_lock();

        let dcps_subscription_topic =
            self.lookup_topicdescription::<DiscoveredReaderData>(DCPS_SUBSCRIPTION)?;
        let sedp_builtin_subscription_reader =
            builtin_subscriber
                .as_ref()
                .unwrap()
                .lookup_datareader::<DiscoveredReaderData>(&dcps_subscription_topic)?;

        let samples = sedp_builtin_subscription_reader.take(
            1,
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        );

        for (sample, _) in samples.unwrap_or(vec![]).iter() {
            for publisher in user_defined_publishers.iter() {
                publisher.add_matched_reader(sample)
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use dds_api::return_type::{DdsError, DdsResult};

    use super::*;
    use crate::dds_type::{DdsSerialize, DdsType, Endianness};
    use std::io::Write;

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
    fn topic_factory_create_topic() {
        let domain_participant: DdsShared<DomainParticipantAttributes> =
            DomainParticipantConstructor::new(
                GuidPrefix([1; 12]),
                DomainId::default(),
                "".to_string(),
                DomainParticipantQos::default(),
                vec![],
                vec![],
                vec![],
                vec![],
            );

        let len_before = domain_participant.topic_list.read_lock().len();

        let topic = domain_participant.create_topic::<Foo>("topic", None, None, 0);

        assert!(topic.is_ok());
        assert_eq!(
            len_before + 1,
            domain_participant.topic_list.read_lock().len()
        );
    }

    #[test]
    fn topic_factory_delete_topic() {
        let domain_participant: DdsShared<DomainParticipantAttributes> =
            DomainParticipantConstructor::new(
                GuidPrefix([1; 12]),
                DomainId::default(),
                "".to_string(),
                DomainParticipantQos::default(),
                vec![],
                vec![],
                vec![],
                vec![],
            );

        let len_before = domain_participant.topic_list.read_lock().len();

        let topic = domain_participant
            .create_topic::<Foo>("topic", None, None, 0)
            .unwrap();

        assert_eq!(
            len_before + 1,
            domain_participant.topic_list.read_lock().len()
        );

        domain_participant.delete_topic::<Foo>(&topic).unwrap();

        assert_eq!(len_before, domain_participant.topic_list.read_lock().len());
    }

    #[test]
    fn topic_factory_delete_topic_from_other_participant() {
        let domain_participant: DdsShared<DomainParticipantAttributes> =
            DomainParticipantConstructor::new(
                GuidPrefix([1; 12]),
                DomainId::default(),
                "".to_string(),
                DomainParticipantQos::default(),
                vec![],
                vec![],
                vec![],
                vec![],
            );

        let domain_participant2: DdsShared<DomainParticipantAttributes> =
            DomainParticipantConstructor::new(
                GuidPrefix([1; 12]),
                DomainId::default(),
                "".to_string(),
                DomainParticipantQos::default(),
                vec![],
                vec![],
                vec![],
                vec![],
            );

        let len_before = domain_participant.topic_list.read_lock().len();
        let len_before2 = domain_participant2.topic_list.read_lock().len();

        let topic = domain_participant
            .create_topic::<Foo>("topic", None, None, 0)
            .unwrap();

        assert_eq!(
            len_before + 1,
            domain_participant.topic_list.read_lock().len()
        );
        assert_eq!(
            len_before2,
            domain_participant2.topic_list.read_lock().len()
        );

        assert!(matches!(
            domain_participant2.delete_topic::<Foo>(&topic),
            Err(DdsError::PreconditionNotMet(_))
        ));
    }

    #[test]
    fn topic_factory_lookup_topic_with_no_topic() {
        let domain_participant: DdsShared<DomainParticipantAttributes> =
            DomainParticipantConstructor::new(
                GuidPrefix([1; 12]),
                DomainId::default(),
                "".to_string(),
                DomainParticipantQos::default(),
                vec![],
                vec![],
                vec![],
                vec![],
            );

        assert!(domain_participant
            .lookup_topicdescription::<Foo>("topic")
            .is_err());
    }

    #[test]
    fn topic_factory_lookup_topic_with_one_topic() {
        let domain_participant: DdsShared<DomainParticipantAttributes> =
            DomainParticipantConstructor::new(
                GuidPrefix([1; 12]),
                DomainId::default(),
                "".to_string(),
                DomainParticipantQos::default(),
                vec![],
                vec![],
                vec![],
                vec![],
            );

        let topic = domain_participant
            .create_topic::<Foo>("topic", None, None, 0)
            .unwrap();

        assert!(
            domain_participant
                .lookup_topicdescription::<Foo>("topic")
                .unwrap()
                == topic
        );
    }

    make_empty_dds_type!(Bar);

    #[test]
    fn topic_factory_lookup_topic_with_one_topic_with_wrong_type() {
        let domain_participant: DdsShared<DomainParticipantAttributes> =
            DomainParticipantConstructor::new(
                GuidPrefix([1; 12]),
                DomainId::default(),
                "".to_string(),
                DomainParticipantQos::default(),
                vec![],
                vec![],
                vec![],
                vec![],
            );

        domain_participant
            .create_topic::<Bar>("topic", None, None, 0)
            .unwrap();

        assert!(domain_participant
            .lookup_topicdescription::<Foo>("topic")
            .is_err());
    }

    #[test]
    fn topic_factory_lookup_topic_with_one_topic_with_wrong_name() {
        let domain_participant: DdsShared<DomainParticipantAttributes> =
            DomainParticipantConstructor::new(
                GuidPrefix([1; 12]),
                DomainId::default(),
                "".to_string(),
                DomainParticipantQos::default(),
                vec![],
                vec![],
                vec![],
                vec![],
            );

        domain_participant
            .create_topic::<Foo>("other_topic", None, None, 0)
            .unwrap();

        assert!(domain_participant
            .lookup_topicdescription::<Foo>("topic")
            .is_err());
    }

    #[test]
    fn topic_factory_lookup_topic_with_two_types() {
        let domain_participant: DdsShared<DomainParticipantAttributes> =
            DomainParticipantConstructor::new(
                GuidPrefix([1; 12]),
                DomainId::default(),
                "".to_string(),
                DomainParticipantQos::default(),
                vec![],
                vec![],
                vec![],
                vec![],
            );

        let topic_foo = domain_participant
            .create_topic::<Foo>("topic", None, None, 0)
            .unwrap();
        let topic_bar = domain_participant
            .create_topic::<Bar>("topic", None, None, 0)
            .unwrap();

        assert!(
            domain_participant
                .lookup_topicdescription::<Foo>("topic")
                .unwrap()
                == topic_foo
        );

        assert!(
            domain_participant
                .lookup_topicdescription::<Bar>("topic")
                .unwrap()
                == topic_bar
        );
    }

    #[test]
    fn topic_factory_lookup_topic_with_two_topics() {
        let domain_participant: DdsShared<DomainParticipantAttributes> =
            DomainParticipantConstructor::new(
                GuidPrefix([1; 12]),
                DomainId::default(),
                "".to_string(),
                DomainParticipantQos::default(),
                vec![],
                vec![],
                vec![],
                vec![],
            );

        let topic1 = domain_participant
            .create_topic::<Foo>("topic1", None, None, 0)
            .unwrap();
        let topic2 = domain_participant
            .create_topic::<Foo>("topic2", None, None, 0)
            .unwrap();

        assert!(
            domain_participant
                .lookup_topicdescription::<Foo>("topic1")
                .unwrap()
                == topic1
        );

        assert!(
            domain_participant
                .lookup_topicdescription::<Foo>("topic2")
                .unwrap()
                == topic2
        );
    }
}
