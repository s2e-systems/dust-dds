use crate::{
    builtin_topics::{BuiltInTopicKey, ParticipantBuiltinTopicData},
    domain::domain_participant_factory::DomainId,
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, DCPS_SUBSCRIPTION},
            discovered_topic_data::{DiscoveredTopicData, DCPS_TOPIC},
            discovered_writer_data::{DiscoveredWriterData, DCPS_PUBLICATION},
            spdp_discovered_participant_data::{
                ParticipantProxy, SpdpDiscoveredParticipantData, DCPS_PARTICIPANT,
            },
        },
        dds::{dds_data_reader::DdsDataReader, dds_subscriber::DdsSubscriber, dds_topic::DdsTopic},
        rtps::{
            discovery_types::{BuiltinEndpointQos, BuiltinEndpointSet},
            endpoint::RtpsEndpoint,
            group::RtpsGroup,
            messages::types::Count,
            participant::RtpsParticipant,
            reader::RtpsReader,
            reader_locator::RtpsReaderLocator,
            stateful_reader::{
                RtpsStatefulReader, DEFAULT_HEARTBEAT_RESPONSE_DELAY,
                DEFAULT_HEARTBEAT_SUPPRESSION_DURATION,
            },
            stateful_writer::{
                RtpsStatefulWriter, DEFAULT_HEARTBEAT_PERIOD, DEFAULT_NACK_RESPONSE_DELAY,
                DEFAULT_NACK_SUPPRESSION_DURATION,
            },
            stateless_reader::RtpsStatelessReader,
            stateless_writer::RtpsStatelessWriter,
            types::{
                EntityId, EntityKey, Guid, Locator, ProtocolVersion, TopicKind, VendorId,
                BUILT_IN_READER_GROUP, BUILT_IN_READER_WITH_KEY, BUILT_IN_TOPIC,
                BUILT_IN_WRITER_GROUP, BUILT_IN_WRITER_WITH_KEY,
            },
            writer::RtpsWriter,
        },
        rtps_udp_psm::udp_transport::UdpTransportWrite,
        utils::actor::{actor_interface, spawn_actor, Actor, ActorAddress},
    },
    infrastructure::{
        instance::InstanceHandle,
        qos::{DataReaderQos, DataWriterQos},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            QosPolicyId, ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        time::{DurationKind, DURATION_ZERO},
    },
    topic_definition::type_support::DdsType,
    {
        builtin_topics::TopicBuiltinTopicData,
        infrastructure::{
            error::{DdsError, DdsResult},
            qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos},
            time::{Duration, Time},
        },
    },
};

use std::{
    collections::{HashMap, HashSet},
    time::{SystemTime, UNIX_EPOCH},
};

use super::{
    dds_data_writer::DdsDataWriter, dds_publisher::DdsPublisher,
    status_listener::ListenerTriggerKind,
};

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER: EntityId =
    EntityId::new(EntityKey::new([0x00, 0x01, 0x00]), BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER: EntityId =
    EntityId::new(EntityKey::new([0x00, 0x01, 0x00]), BUILT_IN_READER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId =
    EntityId::new(EntityKey::new([0, 0, 0x02]), BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId =
    EntityId::new(EntityKey::new([0, 0, 0x02]), BUILT_IN_READER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId =
    EntityId::new(EntityKey::new([0, 0, 0x03]), BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId =
    EntityId::new(EntityKey::new([0, 0, 0x03]), BUILT_IN_READER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId =
    EntityId::new(EntityKey::new([0, 0, 0x04]), BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId =
    EntityId::new(EntityKey::new([0, 0, 0x04]), BUILT_IN_READER_WITH_KEY);

pub struct DdsDomainParticipant {
    rtps_participant: RtpsParticipant,
    domain_id: DomainId,
    domain_tag: String,
    qos: DomainParticipantQos,
    builtin_subscriber: Actor<DdsSubscriber>,
    builtin_publisher: Actor<DdsPublisher>,
    user_defined_subscriber_list: Vec<Actor<DdsSubscriber>>,
    user_defined_subscriber_counter: u8,
    default_subscriber_qos: SubscriberQos,
    user_defined_publisher_list: Vec<Actor<DdsPublisher>>,
    user_defined_publisher_counter: u8,
    default_publisher_qos: PublisherQos,
    topic_list: Vec<Actor<DdsTopic>>,
    user_defined_topic_counter: u8,
    default_topic_qos: TopicQos,
    manual_liveliness_count: Count,
    lease_duration: Duration,
    discovered_participant_list: HashMap<InstanceHandle, SpdpDiscoveredParticipantData>,
    discovered_topic_list: HashMap<InstanceHandle, TopicBuiltinTopicData>,
    enabled: bool,
    ignored_participants: HashSet<InstanceHandle>,
    ignored_publications: HashSet<InstanceHandle>,
    ignored_subcriptions: HashSet<InstanceHandle>,
    data_max_size_serialized: usize,
    udp_transport_write: Actor<UdpTransportWrite>,
}

impl DdsDomainParticipant {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rtps_participant: RtpsParticipant,
        domain_id: DomainId,
        domain_tag: String,
        domain_participant_qos: DomainParticipantQos,
        spdp_discovery_locator_list: &[Locator],
        data_max_size_serialized: usize,
        udp_transport_write: Actor<UdpTransportWrite>,
    ) -> Self {
        let lease_duration = Duration::new(100, 0);
        let guid_prefix = rtps_participant.guid().prefix();

        let spdp_topic_entity_id = EntityId::new(EntityKey::new([0, 0, 0]), BUILT_IN_TOPIC);
        let spdp_topic_guid = Guid::new(guid_prefix, spdp_topic_entity_id);
        let _spdp_topic_participant = DdsTopic::new(
            spdp_topic_guid,
            TopicQos::default(),
            SpdpDiscoveredParticipantData::type_name(),
            DCPS_PARTICIPANT,
        );

        let sedp_topics_entity_id = EntityId::new(EntityKey::new([0, 0, 1]), BUILT_IN_TOPIC);
        let sedp_topics_guid = Guid::new(guid_prefix, sedp_topics_entity_id);
        let _sedp_topic_topics = DdsTopic::new(
            sedp_topics_guid,
            TopicQos::default(),
            DiscoveredTopicData::type_name(),
            DCPS_TOPIC,
        );

        let sedp_publications_entity_id = EntityId::new(EntityKey::new([0, 0, 2]), BUILT_IN_TOPIC);
        let sedp_publications_guid = Guid::new(guid_prefix, sedp_publications_entity_id);
        let _sedp_topic_publications = DdsTopic::new(
            sedp_publications_guid,
            TopicQos::default(),
            DiscoveredWriterData::type_name(),
            DCPS_PUBLICATION,
        );

        let sedp_subscriptions_entity_id = EntityId::new(EntityKey::new([0, 0, 2]), BUILT_IN_TOPIC);
        let sedp_subscriptions_guid = Guid::new(guid_prefix, sedp_subscriptions_entity_id);
        let _sedp_topic_subscriptions = DdsTopic::new(
            sedp_subscriptions_guid,
            TopicQos::default(),
            DiscoveredReaderData::type_name(),
            DCPS_SUBSCRIPTION,
        );

        // Built-in subscriber creation
        let spdp_builtin_participant_reader = spawn_actor(DdsDataReader::new(
            create_builtin_stateless_reader::<SpdpDiscoveredParticipantData>(Guid::new(
                guid_prefix,
                ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
            )),
            SpdpDiscoveredParticipantData::type_name(),
            String::from(DCPS_PARTICIPANT),
            None,
            vec![],
        ));

        let sedp_builtin_topics_reader = spawn_actor(DdsDataReader::new(
            create_builtin_stateful_reader::<DiscoveredTopicData>(Guid::new(
                guid_prefix,
                ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
            )),
            DiscoveredTopicData::type_name(),
            String::from(DCPS_TOPIC),
            None,
            vec![],
        ));

        let sedp_builtin_publications_reader = spawn_actor(DdsDataReader::new(
            create_builtin_stateful_reader::<DiscoveredWriterData>(Guid::new(
                guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            )),
            DiscoveredWriterData::type_name(),
            String::from(DCPS_PUBLICATION),
            None,
            vec![],
        ));

        let sedp_builtin_subscriptions_reader = spawn_actor(DdsDataReader::new(
            create_builtin_stateful_reader::<DiscoveredReaderData>(Guid::new(
                guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
            )),
            DiscoveredReaderData::type_name(),
            String::from(DCPS_SUBSCRIPTION),
            None,
            vec![],
        ));

        let builtin_subscriber = spawn_actor(DdsSubscriber::new(
            SubscriberQos::default(),
            RtpsGroup::new(Guid::new(
                guid_prefix,
                EntityId::new(EntityKey::new([0, 0, 0]), BUILT_IN_READER_GROUP),
            )),
        ));

        builtin_subscriber
            .address()
            .stateless_data_reader_add(spdp_builtin_participant_reader)
            .unwrap();
        builtin_subscriber
            .address()
            .stateful_data_reader_add(sedp_builtin_topics_reader)
            .unwrap();
        builtin_subscriber
            .address()
            .stateful_data_reader_add(sedp_builtin_publications_reader)
            .unwrap();
        builtin_subscriber
            .address()
            .stateful_data_reader_add(sedp_builtin_subscriptions_reader)
            .unwrap();

        // Built-in publisher creation
        let spdp_builtin_participant_writer = spawn_actor(DdsDataWriter::new(
            create_builtin_stateless_writer(Guid::new(
                guid_prefix,
                ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
            )),
            SpdpDiscoveredParticipantData::type_name(),
            String::from(DCPS_PARTICIPANT),
        ));

        for reader_locator in spdp_discovery_locator_list
            .iter()
            .map(|&locator| RtpsReaderLocator::new(locator, false))
        {
            spdp_builtin_participant_writer
                .address()
                .reader_locator_add(reader_locator)
                .unwrap();
        }

        let sedp_builtin_topics_writer = DdsDataWriter::new(
            create_builtin_stateful_writer(Guid::new(
                guid_prefix,
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            )),
            DiscoveredTopicData::type_name(),
            String::from(DCPS_TOPIC),
        );
        let sedp_builtin_topics_writer_actor = spawn_actor(sedp_builtin_topics_writer);

        let sedp_builtin_publications_writer = DdsDataWriter::new(
            create_builtin_stateful_writer(Guid::new(
                guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            )),
            DiscoveredWriterData::type_name(),
            String::from(DCPS_PUBLICATION),
        );
        let sedp_builtin_publications_writer_actor = spawn_actor(sedp_builtin_publications_writer);

        let sedp_builtin_subscriptions_writer = DdsDataWriter::new(
            create_builtin_stateful_writer(Guid::new(
                guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            )),
            DiscoveredReaderData::type_name(),
            String::from(DCPS_SUBSCRIPTION),
        );
        let sedp_builtin_subscriptions_writer_actor =
            spawn_actor(sedp_builtin_subscriptions_writer);

        let builtin_publisher = spawn_actor(DdsPublisher::new(
            PublisherQos::default(),
            RtpsGroup::new(Guid::new(
                guid_prefix,
                EntityId::new(EntityKey::new([0, 0, 0]), BUILT_IN_WRITER_GROUP),
            )),
        ));

        builtin_publisher
            .address()
            .stateless_datawriter_add(spdp_builtin_participant_writer)
            .unwrap();
        builtin_publisher
            .address()
            .stateful_datawriter_add(sedp_builtin_topics_writer_actor)
            .unwrap();
        builtin_publisher
            .address()
            .stateful_datawriter_add(sedp_builtin_publications_writer_actor)
            .unwrap();
        builtin_publisher
            .address()
            .stateful_datawriter_add(sedp_builtin_subscriptions_writer_actor)
            .unwrap();

        Self {
            rtps_participant,
            domain_id,
            domain_tag,
            qos: domain_participant_qos,
            builtin_subscriber,
            builtin_publisher,
            user_defined_subscriber_list: Vec::new(),
            user_defined_subscriber_counter: 0,
            default_subscriber_qos: SubscriberQos::default(),
            user_defined_publisher_list: Vec::new(),
            user_defined_publisher_counter: 0,
            default_publisher_qos: PublisherQos::default(),
            topic_list: Vec::new(),
            user_defined_topic_counter: 0,
            default_topic_qos: TopicQos::default(),
            manual_liveliness_count: Count::new(0),
            lease_duration,
            discovered_participant_list: HashMap::new(),
            discovered_topic_list: HashMap::new(),
            enabled: false,
            ignored_participants: HashSet::new(),
            ignored_publications: HashSet::new(),
            ignored_subcriptions: HashSet::new(),
            data_max_size_serialized,
            udp_transport_write,
        }
    }
}

actor_interface! {
// Rtps Entity methods
impl DdsDomainParticipant {
        pub fn get_guid(&self) -> Guid {
            self.rtps_participant.guid()
        }
}
}

actor_interface! {
// Rtps Participant methods
impl DdsDomainParticipant {
    pub fn get_default_unicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_participant
            .default_unicast_locator_list()
            .to_vec()
    }

    pub fn get_default_multicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_participant
            .default_multicast_locator_list()
            .to_vec()
    }

    pub fn get_protocol_version(&self) -> ProtocolVersion {
        self.rtps_participant.protocol_version()
    }

    pub fn get_vendor_id(&self) -> VendorId {
        self.rtps_participant.vendor_id()
    }
}
}

actor_interface! {
impl DdsDomainParticipant {
    pub fn get_metatraffic_unicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_participant
            .metatraffic_unicast_locator_list()
            .to_vec()
    }

    pub fn get_metatraffic_multicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_participant
            .metatraffic_multicast_locator_list()
            .to_vec()
    }

    pub fn get_builtin_subscriber(&self) -> ActorAddress<DdsSubscriber> {
        self.builtin_subscriber.address()
    }

    pub fn get_builtin_publisher(&self) -> ActorAddress<DdsPublisher> {
        self.builtin_publisher.address()
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_participant.guid().into()
    }

    pub fn get_domain_id(&self) -> DomainId {
        self.domain_id
    }

    pub fn get_domain_tag(&self) -> String {
        self.domain_tag.clone()
    }

    pub fn get_current_time(&self) -> Time {
        let now_system_time = SystemTime::now();
        let unix_time = now_system_time
            .duration_since(UNIX_EPOCH)
            .expect("Clock time is before Unix epoch start");
        Time::new(unix_time.as_secs() as i32, unix_time.subsec_nanos())
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn ignore_participant(&mut self, handle: InstanceHandle) {
        self.ignored_participants.insert(handle);
    }

    pub fn is_participant_ignored(&self, handle: InstanceHandle) -> bool {
        self.ignored_participants.contains(&handle)
    }

    pub fn ignore_subscription(&mut self, handle: InstanceHandle) {
        self.ignored_subcriptions.insert(handle);
    }

    pub fn is_subscription_ignored(&self, handle: InstanceHandle) -> bool {
        self.ignored_subcriptions.contains(&handle)
    }

    pub fn ignore_publication(&mut self, handle: InstanceHandle) {
        self.ignored_publications.insert(handle);
    }

    pub fn is_publication_ignored(&self, handle: InstanceHandle) -> bool {
        self.ignored_publications.contains(&handle)
    }

    pub fn ignore_topic(&self, _handle: InstanceHandle) {
        todo!()
    }

    pub fn is_topic_ignored(&self, _handle: InstanceHandle) -> bool {
        todo!()
    }

    pub fn discovered_participant_add(
        &mut self,
        handle: InstanceHandle,
        discovered_participant_data: SpdpDiscoveredParticipantData,
    ) {
        self.discovered_participant_list
            .insert(handle, discovered_participant_data);
    }

    pub fn discovered_participant_get(&self, handle: InstanceHandle) -> Option<SpdpDiscoveredParticipantData> {
        self.discovered_participant_list.get(&handle).cloned()
    }

    pub fn _discovered_participant_remove(&mut self, handle: InstanceHandle) {
        self.discovered_participant_list.remove(&handle);
    }

    pub fn create_unique_publisher_id(&mut self) -> u8 {
        let counter = self.user_defined_publisher_counter;
        self.user_defined_publisher_counter += 1;
        counter
    }

    pub fn add_user_defined_publisher(&mut self, publisher: Actor<DdsPublisher>) {
        self.user_defined_publisher_list.push(publisher)
    }

    pub fn get_user_defined_publisher_list(&self) -> Vec<ActorAddress<DdsPublisher>> {
        self.user_defined_publisher_list.iter().map(|a| a.address()).collect()
    }

    pub fn delete_user_defined_publisher(&mut self, handle: InstanceHandle) {
        self.user_defined_publisher_list
            .retain(|p|
                if let Ok(h) = p.address()
                    .get_instance_handle() {
                        h != handle
                    } else {
                        false
                    });
    }

    pub fn create_unique_subscriber_id(&mut self) -> u8 {
        let counter = self.user_defined_subscriber_counter;
        self.user_defined_subscriber_counter += 1;
        counter
    }

    pub fn add_user_defined_subscriber(&mut self, subscriber: Actor<DdsSubscriber>) {
        self.user_defined_subscriber_list.push(subscriber)
    }

    pub fn get_user_defined_subscriber_list(&self) -> Vec<ActorAddress<DdsSubscriber>> {
        self.user_defined_subscriber_list.iter().map(|a| a.address()).collect()
    }

    pub fn delete_user_defined_subscriber(&mut self, handle: InstanceHandle) {
        self.user_defined_subscriber_list
            .retain(|p|
                if let Ok(h) = p.address()
                    .get_instance_handle() {
                        h != handle
                    } else {
                        false
                    });
    }

    pub fn create_unique_topic_id(&mut self) -> u8 {
        let counter = self.user_defined_topic_counter;
        self.user_defined_topic_counter += 1;
        counter
    }

    pub fn add_user_defined_topic(&mut self, topic: Actor<DdsTopic>) {
        self.topic_list.push(topic)
    }

    pub fn get_user_defined_topic_list(&self) -> Vec<ActorAddress<DdsTopic>> {
        self.topic_list.iter().map(|a| a.address()).collect()
    }

    pub fn delete_user_defined_topic(&mut self, handle: InstanceHandle) {
        self.topic_list
            .retain(|p|
                if let Ok(h) = p.address()
                    .get_instance_handle() {
                        h != handle
                    } else {
                        false
                    });
    }

    pub fn is_empty(&self) -> bool {
        self.user_defined_publisher_list.iter().count() == 0
            && self.user_defined_subscriber_list.iter().count() == 0
            && self.topic_list.iter().count() == 0
    }

    pub fn delete_topic(&mut self, handle: InstanceHandle) {
        self.topic_list
            .retain(|t|
                if let Ok(h) = t.address()
                    .get_instance_handle() {
                        h != handle
                    } else {
                        false
                    });
    }

    pub fn get_qos(&self) -> DomainParticipantQos {
        self.qos.clone()
    }

    pub fn data_max_size_serialized(&self) -> usize {
        self.data_max_size_serialized
    }

    pub fn find_topic(
        &mut self,
        _topic_name: String,
        _type_name: &'static str,
    ) -> Option<ActorAddress<DdsTopic>> {
        todo!()
        // // Check if a topic exists locally. If topic doesn't exist locally check if it has already been
        // // discovered and, if so, create a new local topic representing the discovered topic
        // if let Some(topic) = self
        //     .topic_list
        //     .iter()
        //     .find(|topic| topic.get_name() == topic_name && topic.get_type_name() == type_name)
        // {
        //     Some(topic.guid())
        // } else if let Some(discovered_topic_info) = self
        //     .discovered_topic_list
        //     .values()
        //     .find(|t| t.name() == topic_name && t.get_type_name() == type_name)
        //     .cloned()
        // {
        //     let qos = TopicQos {
        //         topic_data: discovered_topic_info.topic_data().clone(),
        //         durability: discovered_topic_info.durability().clone(),
        //         deadline: discovered_topic_info.deadline().clone(),
        //         latency_budget: discovered_topic_info.latency_budget().clone(),
        //         liveliness: discovered_topic_info.liveliness().clone(),
        //         reliability: discovered_topic_info.reliability().clone(),
        //         destination_order: discovered_topic_info.destination_order().clone(),
        //         history: discovered_topic_info.history().clone(),
        //         resource_limits: discovered_topic_info.resource_limits().clone(),
        //         transport_priority: discovered_topic_info.transport_priority().clone(),
        //         lifespan: discovered_topic_info.lifespan().clone(),
        //         ownership: discovered_topic_info.ownership().clone(),
        //     };
        //     Some(
        //         self.create_topic(
        //             discovered_topic_info.name(),
        //             type_name,
        //             QosKind::Specific(qos),
        //         )
        //         .unwrap(),
        //     )
        // } else {
        //     None
        // }
    }

    pub fn delete_contained_entities(&mut self) -> DdsResult<()> {
        for user_defined_publisher in self.user_defined_publisher_list.drain(..) {
            user_defined_publisher
                .address()
                .delete_contained_entities()?;
        }

        for user_defined_subscriber in self.user_defined_subscriber_list.drain(..) {
            user_defined_subscriber
                .address()
                .delete_contained_entities()?;
        }

        self.topic_list.clear();

        Ok(())
    }

    pub fn set_default_publisher_qos(&mut self, qos: PublisherQos){
        self.default_publisher_qos = qos;
    }

    pub fn default_publisher_qos(&self) -> PublisherQos {
        self.default_publisher_qos.clone()
    }

    pub fn set_default_subscriber_qos(&mut self, qos: SubscriberQos) {
        self.default_subscriber_qos = qos;
    }

    pub fn default_subscriber_qos(&self) -> SubscriberQos {
        self.default_subscriber_qos.clone()
    }

    pub fn set_default_topic_qos(&mut self, qos: TopicQos) {
        self.default_topic_qos = qos;
    }

    pub fn default_topic_qos(&self) -> TopicQos {
        self.default_topic_qos.clone()
    }

    pub fn discovered_topic_list(&self) -> Vec<InstanceHandle> {
        self.discovered_topic_list.keys().cloned().collect()
    }

    pub fn discovered_topic_data(
        &self,
        topic_handle: InstanceHandle,
    ) -> DdsResult<TopicBuiltinTopicData> {
        self.discovered_topic_list
            .get(&topic_handle)
            .cloned()
            .ok_or(DdsError::BadParameter)
    }

    pub fn set_qos(&mut self, qos: DomainParticipantQos) {
        self.qos = qos;
    }

    pub fn as_spdp_discovered_participant_data(&self) -> SpdpDiscoveredParticipantData {
        SpdpDiscoveredParticipantData::new(
            ParticipantBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: self.rtps_participant.guid().into(),
                },
                self.qos.user_data.clone(),
            ),
            ParticipantProxy::new(
                self.domain_id,
                self.domain_tag.clone(),
                self.rtps_participant.protocol_version(),
                self.rtps_participant.guid().prefix(),
                self.rtps_participant.vendor_id(),
                false,
                self.rtps_participant
                    .metatraffic_unicast_locator_list()
                    .to_vec(),
                self.rtps_participant
                    .metatraffic_multicast_locator_list()
                    .to_vec(),
                self.rtps_participant
                    .default_unicast_locator_list()
                    .to_vec(),
                self.rtps_participant
                    .default_multicast_locator_list()
                    .to_vec(),
                BuiltinEndpointSet::default(),
                self.manual_liveliness_count,
                BuiltinEndpointQos::default(),
            ),
            self.lease_duration,
        )
    }

    pub fn get_udp_transport_write(&self) -> ActorAddress<UdpTransportWrite> {
        self.udp_transport_write.address()
    }

    // pub fn remove_discovered_participant(&self, participant_handle: InstanceHandle) {
    //     if let Some((_, discovered_participant_data)) = self
    //         .discovered_participant_list()
    //         .into_iter()
    //         .find(|&(h, _)| h == &participant_handle)
    //     {
    //         let participant_guid_prefix = discovered_participant_data.guid_prefix();
    //         self.get_builtin_subscriber()
    //             .sedp_builtin_publications_reader()
    //             .remove_matched_participant(participant_guid_prefix);
    //         self.get_builtin_subscriber()
    //             .sedp_builtin_subscriptions_reader()
    //             .remove_matched_participant(participant_guid_prefix);
    //         self.get_builtin_subscriber()
    //             .sedp_builtin_topics_reader()
    //             .remove_matched_participant(participant_guid_prefix);
    //         self.get_builtin_publisher()
    //             .sedp_builtin_publications_writer()
    //             .remove_matched_participant(participant_guid_prefix);
    //         self.get_builtin_publisher()
    //             .sedp_builtin_subscriptions_writer()
    //             .remove_matched_participant(participant_guid_prefix);
    //         self.get_builtin_publisher()
    //             .sedp_builtin_topics_writer()
    //             .remove_matched_participant(participant_guid_prefix);
    //     }

    //     self.discovered_participant_remove(participant_handle);
    // }



    pub fn discover_matched_topics(
        &mut self,
        _listener_sender: tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) -> DdsResult<()> {
        todo!()
        // while let Ok(samples) = self
        //     .get_builtin_subscriber_mut()
        //     .stateful_data_reader_list_mut()
        //     .iter_mut()
        //     .find(|x| x.get_topic_name() == DCPS_TOPIC)
        //     .unwrap()
        //     .read::<DiscoveredTopicData>(
        //         1,
        //         &[SampleStateKind::NotRead],
        //         ANY_VIEW_STATE,
        //         ANY_INSTANCE_STATE,
        //         None,
        //     )
        // {
        //     let guid = self.guid();
        //     for sample in samples {
        //         if let Some(topic_data) = sample.data.as_ref() {
        //             for topic in self.topic_list_mut() {
        //                 topic.process_discovered_topic(topic_data, guid, listener_sender);
        //             }

        //             self.discovered_topic_list.insert(
        //                 topic_data.get_serialized_key().into(),
        //                 topic_data.topic_builtin_topic_data().clone(),
        //             );
        //         }
        //     }
        // }

        // Ok(())
    }

    pub fn update_communication_status(
        &mut self,
        _listener_sender: tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) -> DdsResult<()> {
        let _now = self.get_current_time();
        let _guid = self.get_guid();
        for _subscriber in self.user_defined_subscriber_list.iter_mut() {
            todo!()
            // subscriber.update_communication_status(now, guid, listener_sender);
        }

        Ok(())
    }
}
}

fn _on_writer_publication_matched(
    _writer: &DdsDataWriter<RtpsStatefulWriter>,
    _parent_publisher_guid: Guid,
    _parent_participant_guid: Guid,
    _listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
) {
    todo!()
    // listener_sender
    //     .try_send(ListenerTriggerKind::PublicationMatched(
    //         DataWriterNode::new(
    //             writer.guid(),
    //             parent_publisher_guid,
    //             parent_participant_guid,
    //         ),
    //     ))
    //     .ok();
}

pub fn _remove_writer_matched_reader(
    writer: &mut DdsDataWriter<RtpsStatefulWriter>,
    discovered_reader_handle: InstanceHandle,
    parent_publisher_guid: Guid,
    parent_participant_guid: Guid,
    listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
) {
    if let Some(r) = writer.get_matched_subscription_data(discovered_reader_handle) {
        let handle = r.key().value.into();
        writer.matched_reader_remove(handle);
        writer.remove_matched_subscription(handle.into());

        _on_writer_publication_matched(
            writer,
            parent_publisher_guid,
            parent_participant_guid,
            listener_sender,
        )
    }
}

fn _writer_on_offered_incompatible_qos(
    _writer: &mut DdsDataWriter<RtpsStatefulWriter>,
    _handle: InstanceHandle,
    _incompatible_qos_policy_list: Vec<QosPolicyId>,
    _parent_publisher_guid: Guid,
    _parent_participant_guid: Guid,
    _listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
) {
    todo!()
    // if !writer.get_incompatible_subscriptions().contains(&handle) {
    //     writer.add_offered_incompatible_qos(handle, incompatible_qos_policy_list);
    //     listener_sender
    //         .try_send(ListenerTriggerKind::OfferedIncompatibleQos(
    //             DataWriterNode::new(
    //                 writer.guid(),
    //                 parent_publisher_guid,
    //                 parent_participant_guid,
    //             ),
    //         ))
    //         .ok();
    // }
}

fn create_builtin_stateful_writer(guid: Guid) -> RtpsStatefulWriter {
    let unicast_locator_list = &[];
    let multicast_locator_list = &[];
    let topic_kind = TopicKind::WithKey;
    let push_mode = true;
    let heartbeat_period = DEFAULT_HEARTBEAT_PERIOD;
    let nack_response_delay = DEFAULT_NACK_RESPONSE_DELAY;
    let nack_suppression_duration = DEFAULT_NACK_SUPPRESSION_DURATION;
    let data_max_size_serialized = usize::MAX;
    let qos = DataWriterQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepLast(1),
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(DURATION_ZERO),
        },
        ..Default::default()
    };
    RtpsStatefulWriter::new(RtpsWriter::new(
        RtpsEndpoint::new(
            guid,
            topic_kind,
            unicast_locator_list,
            multicast_locator_list,
        ),
        push_mode,
        heartbeat_period,
        nack_response_delay,
        nack_suppression_duration,
        data_max_size_serialized,
        qos,
    ))
}

fn create_builtin_stateless_writer(guid: Guid) -> RtpsStatelessWriter {
    let unicast_locator_list = &[];
    let multicast_locator_list = &[];
    let qos = DataWriterQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepLast(1),
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(DURATION_ZERO),
        },
        ..Default::default()
    };
    RtpsStatelessWriter::new(RtpsWriter::new(
        RtpsEndpoint::new(
            guid,
            TopicKind::WithKey,
            unicast_locator_list,
            multicast_locator_list,
        ),
        true,
        DURATION_ZERO,
        DURATION_ZERO,
        DURATION_ZERO,
        usize::MAX,
        qos,
    ))
}

fn create_builtin_stateless_reader<Foo>(guid: Guid) -> RtpsStatelessReader
where
    Foo: DdsType + for<'de> serde::Deserialize<'de>,
{
    let unicast_locator_list = &[];
    let multicast_locator_list = &[];
    let qos = DataReaderQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepLast(1),
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(DURATION_ZERO),
        },
        ..Default::default()
    };
    let reader = RtpsReader::new::<Foo>(
        RtpsEndpoint::new(
            guid,
            TopicKind::WithKey,
            unicast_locator_list,
            multicast_locator_list,
        ),
        DURATION_ZERO,
        DURATION_ZERO,
        false,
        qos,
    );
    RtpsStatelessReader::new(reader)
}

fn create_builtin_stateful_reader<Foo>(guid: Guid) -> RtpsStatefulReader
where
    Foo: DdsType + for<'de> serde::Deserialize<'de>,
{
    let qos = DataReaderQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepLast(1),
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(DURATION_ZERO),
        },
        ..Default::default()
    };
    let topic_kind = TopicKind::WithKey;
    let heartbeat_response_delay = DEFAULT_HEARTBEAT_RESPONSE_DELAY;
    let heartbeat_suppression_duration = DEFAULT_HEARTBEAT_SUPPRESSION_DURATION;
    let expects_inline_qos = false;
    let unicast_locator_list = &[];
    let multicast_locator_list = &[];

    RtpsStatefulReader::new(RtpsReader::new::<Foo>(
        RtpsEndpoint::new(
            guid,
            topic_kind,
            unicast_locator_list,
            multicast_locator_list,
        ),
        heartbeat_response_delay,
        heartbeat_suppression_duration,
        expects_inline_qos,
        qos,
    ))
}
