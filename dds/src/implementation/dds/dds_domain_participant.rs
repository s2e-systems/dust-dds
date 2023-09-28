use dust_dds_derive::actor_interface;

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
            messages::{
                overall_structure::{RtpsMessageHeader, RtpsMessageRead},
                types::Count,
            },
            participant::RtpsParticipant,
            reader::RtpsReader,
            reader_locator::RtpsReaderLocator,
            types::{
                EntityId, Guid, Locator, ProtocolVersion, TopicKind, VendorId,
                BUILT_IN_READER_GROUP, BUILT_IN_READER_WITH_KEY, BUILT_IN_TOPIC,
                BUILT_IN_WRITER_GROUP, BUILT_IN_WRITER_WITH_KEY, USER_DEFINED_READER_GROUP,
                USER_DEFINED_TOPIC, USER_DEFINED_WRITER_GROUP,
            },
            writer::RtpsWriter,
        },
        rtps_udp_psm::udp_transport::UdpTransportWrite,
        utils::actor::{spawn_actor, Actor, ActorAddress, Mail, MailHandler},
    },
    infrastructure::{
        instance::InstanceHandle,
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        status::StatusKind,
        time::{DurationKind, DURATION_ZERO},
    },
    publication::publisher_listener::PublisherListener,
    subscription::subscriber_listener::SubscriberListener,
    topic_definition::topic_listener::TopicListener,
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
    dds_data_writer::{self, DdsDataWriter},
    dds_domain_participant_listener::DdsDomainParticipantListener,
    dds_publisher::{self, DdsPublisher},
    dds_publisher_listener::DdsPublisherListener,
    dds_subscriber,
    dds_subscriber_listener::DdsSubscriberListener,
};

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER: EntityId =
    EntityId::new([0x00, 0x01, 0x00], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER: EntityId =
    EntityId::new([0x00, 0x01, 0x00], BUILT_IN_READER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x02], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x02], BUILT_IN_READER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x03], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x03], BUILT_IN_READER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x04], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x04], BUILT_IN_READER_WITH_KEY);

pub const DEFAULT_HEARTBEAT_PERIOD: Duration = Duration::new(2, 0);
pub const DEFAULT_NACK_RESPONSE_DELAY: Duration = Duration::new(0, 200);
pub const DEFAULT_NACK_SUPPRESSION_DURATION: Duration = DURATION_ZERO;
pub const DEFAULT_HEARTBEAT_RESPONSE_DELAY: Duration = Duration::new(0, 500);
pub const DEFAULT_HEARTBEAT_SUPPRESSION_DURATION: Duration = DURATION_ZERO;

pub struct DdsDomainParticipant {
    rtps_participant: RtpsParticipant,
    domain_id: DomainId,
    domain_tag: String,
    qos: DomainParticipantQos,
    builtin_subscriber: Actor<DdsSubscriber>,
    builtin_publisher: Actor<DdsPublisher>,
    user_defined_subscriber_list: HashMap<InstanceHandle, Actor<DdsSubscriber>>,
    user_defined_subscriber_counter: u8,
    default_subscriber_qos: SubscriberQos,
    user_defined_publisher_list: HashMap<InstanceHandle, Actor<DdsPublisher>>,
    user_defined_publisher_counter: u8,
    default_publisher_qos: PublisherQos,
    topic_list: HashMap<InstanceHandle, Actor<DdsTopic>>,
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
    listener: Option<Actor<DdsDomainParticipantListener>>,
    status_kind: Vec<StatusKind>,
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
        listener: Option<Actor<DdsDomainParticipantListener>>,
        status_kind: Vec<StatusKind>,
    ) -> Self {
        let lease_duration = Duration::new(100, 0);
        let guid_prefix = rtps_participant.guid().prefix();

        let spdp_topic_entity_id = EntityId::new([0, 0, 0], BUILT_IN_TOPIC);
        let spdp_topic_guid = Guid::new(guid_prefix, spdp_topic_entity_id);
        let _spdp_topic_participant = DdsTopic::new(
            spdp_topic_guid,
            TopicQos::default(),
            "SpdpDiscoveredParticipantData".to_string(),
            DCPS_PARTICIPANT,
        );

        let sedp_topics_entity_id = EntityId::new([0, 0, 1], BUILT_IN_TOPIC);
        let sedp_topics_guid = Guid::new(guid_prefix, sedp_topics_entity_id);
        let _sedp_topic_topics = DdsTopic::new(
            sedp_topics_guid,
            TopicQos::default(),
            "DiscoveredTopicData".to_string(),
            DCPS_TOPIC,
        );

        let sedp_publications_entity_id = EntityId::new([0, 0, 2], BUILT_IN_TOPIC);
        let sedp_publications_guid = Guid::new(guid_prefix, sedp_publications_entity_id);
        let _sedp_topic_publications = DdsTopic::new(
            sedp_publications_guid,
            TopicQos::default(),
            "DiscoveredWriterData".to_string(),
            DCPS_PUBLICATION,
        );

        let sedp_subscriptions_entity_id = EntityId::new([0, 0, 2], BUILT_IN_TOPIC);
        let sedp_subscriptions_guid = Guid::new(guid_prefix, sedp_subscriptions_entity_id);
        let _sedp_topic_subscriptions = DdsTopic::new(
            sedp_subscriptions_guid,
            TopicQos::default(),
            "DiscoveredReaderData".to_string(),
            DCPS_SUBSCRIPTION,
        );

        // Built-in subscriber creation
        let spdp_reader_qos = DataReaderQos {
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
        let spdp_builtin_participant_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER);
        let spdp_builtin_participant_reader =
            spawn_actor(DdsDataReader::new::<SpdpDiscoveredParticipantData>(
                create_builtin_stateless_reader(spdp_builtin_participant_reader_guid),
                "SpdpDiscoveredParticipantData".to_string(),
                String::from(DCPS_PARTICIPANT),
                spdp_reader_qos,
                None,
                vec![],
            ));

        let sedp_reader_qos = DataReaderQos {
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

        let sedp_builtin_topics_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
        let sedp_builtin_topics_reader = spawn_actor(DdsDataReader::new::<DiscoveredTopicData>(
            create_builtin_stateful_reader(sedp_builtin_topics_reader_guid),
            "DiscoveredTopicData".to_string(),
            String::from(DCPS_TOPIC),
            sedp_reader_qos.clone(),
            None,
            vec![],
        ));

        let sedp_builtin_publications_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
        let sedp_builtin_publications_reader =
            spawn_actor(DdsDataReader::new::<DiscoveredWriterData>(
                create_builtin_stateful_reader(sedp_builtin_publications_reader_guid),
                "DiscoveredWriterData".to_string(),
                String::from(DCPS_PUBLICATION),
                sedp_reader_qos.clone(),
                None,
                vec![],
            ));

        let sedp_builtin_subscriptions_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let sedp_builtin_subscriptions_reader =
            spawn_actor(DdsDataReader::new::<DiscoveredReaderData>(
                create_builtin_stateful_reader(sedp_builtin_subscriptions_reader_guid),
                "DiscoveredReaderData".to_string(),
                String::from(DCPS_SUBSCRIPTION),
                sedp_reader_qos,
                None,
                vec![],
            ));

        let builtin_subscriber = spawn_actor(DdsSubscriber::new(
            SubscriberQos::default(),
            RtpsGroup::new(Guid::new(
                guid_prefix,
                EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP),
            )),
            None,
            vec![],
        ));

        builtin_subscriber
            .address()
            .send_and_reply_blocking(dds_subscriber::data_reader_add::new(
                spdp_builtin_participant_reader_guid.into(),
                spdp_builtin_participant_reader,
            ))
            .unwrap();
        builtin_subscriber
            .address()
            .send_and_reply_blocking(dds_subscriber::data_reader_add::new(
                sedp_builtin_topics_reader_guid.into(),
                sedp_builtin_topics_reader,
            ))
            .unwrap();
        builtin_subscriber
            .address()
            .send_and_reply_blocking(dds_subscriber::data_reader_add::new(
                sedp_builtin_publications_reader_guid.into(),
                sedp_builtin_publications_reader,
            ))
            .unwrap();
        builtin_subscriber
            .address()
            .send_and_reply_blocking(dds_subscriber::data_reader_add::new(
                sedp_builtin_subscriptions_reader_guid.into(),
                sedp_builtin_subscriptions_reader,
            ))
            .unwrap();

        // Built-in publisher creation
        let spdp_writer_qos = DataWriterQos {
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
        let spdp_builtin_participant_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER);
        let spdp_builtin_participant_writer = spawn_actor(DdsDataWriter::new(
            create_builtin_stateless_writer(spdp_builtin_participant_writer_guid),
            "SpdpDiscoveredParticipantData".to_string(),
            String::from(DCPS_PARTICIPANT),
            None,
            vec![],
            spdp_writer_qos,
        ));

        for reader_locator in spdp_discovery_locator_list
            .iter()
            .map(|&locator| RtpsReaderLocator::new(locator, false))
        {
            spdp_builtin_participant_writer
                .address()
                .send_and_reply_blocking(dds_data_writer::reader_locator_add::new(reader_locator))
                .unwrap();
        }

        let sedp_writer_qos = DataWriterQos {
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
        let sedp_builtin_topics_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
        let sedp_builtin_topics_writer = DdsDataWriter::new(
            create_builtin_stateful_writer(sedp_builtin_topics_writer_guid),
            "DiscoveredTopicData".to_string(),
            String::from(DCPS_TOPIC),
            None,
            vec![],
            sedp_writer_qos.clone(),
        );
        let sedp_builtin_topics_writer_actor = spawn_actor(sedp_builtin_topics_writer);

        let sedp_builtin_publications_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
        let sedp_builtin_publications_writer = DdsDataWriter::new(
            create_builtin_stateful_writer(sedp_builtin_publications_writer_guid),
            "DiscoveredWriterData".to_string(),
            String::from(DCPS_PUBLICATION),
            None,
            vec![],
            sedp_writer_qos.clone(),
        );
        let sedp_builtin_publications_writer_actor = spawn_actor(sedp_builtin_publications_writer);

        let sedp_builtin_subscriptions_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let sedp_builtin_subscriptions_writer = DdsDataWriter::new(
            create_builtin_stateful_writer(sedp_builtin_subscriptions_writer_guid),
            "DiscoveredReaderData".to_string(),
            String::from(DCPS_SUBSCRIPTION),
            None,
            vec![],
            sedp_writer_qos,
        );
        let sedp_builtin_subscriptions_writer_actor =
            spawn_actor(sedp_builtin_subscriptions_writer);

        let builtin_publisher = spawn_actor(DdsPublisher::new(
            PublisherQos::default(),
            RtpsGroup::new(Guid::new(
                guid_prefix,
                EntityId::new([0, 0, 0], BUILT_IN_WRITER_GROUP),
            )),
            None,
            vec![],
        ));

        builtin_publisher
            .address()
            .send_and_reply_blocking(dds_publisher::datawriter_add::new(
                spdp_builtin_participant_writer_guid.into(),
                spdp_builtin_participant_writer,
            ))
            .unwrap();
        builtin_publisher
            .address()
            .send_and_reply_blocking(dds_publisher::datawriter_add::new(
                sedp_builtin_topics_writer_guid.into(),
                sedp_builtin_topics_writer_actor,
            ))
            .unwrap();
        builtin_publisher
            .address()
            .send_and_reply_blocking(dds_publisher::datawriter_add::new(
                sedp_builtin_publications_writer_guid.into(),
                sedp_builtin_publications_writer_actor,
            ))
            .unwrap();
        builtin_publisher
            .address()
            .send_and_reply_blocking(dds_publisher::datawriter_add::new(
                sedp_builtin_subscriptions_writer_guid.into(),
                sedp_builtin_subscriptions_writer_actor,
            ))
            .unwrap();

        Self {
            rtps_participant,
            domain_id,
            domain_tag,
            qos: domain_participant_qos,
            builtin_subscriber,
            builtin_publisher,
            user_defined_subscriber_list: HashMap::new(),
            user_defined_subscriber_counter: 0,
            default_subscriber_qos: SubscriberQos::default(),
            user_defined_publisher_list: HashMap::new(),
            user_defined_publisher_counter: 0,
            default_publisher_qos: PublisherQos::default(),
            topic_list: HashMap::new(),
            user_defined_topic_counter: 0,
            default_topic_qos: TopicQos::default(),
            manual_liveliness_count: 0,
            lease_duration,
            discovered_participant_list: HashMap::new(),
            discovered_topic_list: HashMap::new(),
            enabled: false,
            ignored_participants: HashSet::new(),
            ignored_publications: HashSet::new(),
            ignored_subcriptions: HashSet::new(),
            data_max_size_serialized,
            udp_transport_write,
            listener,
            status_kind,
        }
    }

    fn get_current_time(&self) -> Time {
        let now_system_time = SystemTime::now();
        let unix_time = now_system_time
            .duration_since(UNIX_EPOCH)
            .expect("Clock time is before Unix epoch start");
        Time::new(unix_time.as_secs() as i32, unix_time.subsec_nanos())
    }
}

#[actor_interface]
impl DdsDomainParticipant {
    async fn create_publisher(
        &mut self,
        qos: QosKind<PublisherQos>,
        a_listener: Option<Box<dyn PublisherListener + Send + Sync>>,
        mask: Vec<StatusKind>,
    ) -> ActorAddress<DdsPublisher> {
        let publisher_qos = match qos {
            QosKind::Default => self.default_publisher_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let publisher_counter = self.create_unique_publisher_id().await;
        let entity_id = EntityId::new([publisher_counter, 0, 0], USER_DEFINED_WRITER_GROUP);
        let guid = Guid::new(self.rtps_participant.guid().prefix(), entity_id);
        let rtps_group = RtpsGroup::new(guid);
        let listener = a_listener.map(|l| spawn_actor(DdsPublisherListener::new(l)));
        let status_kind = mask.to_vec();
        let publisher = DdsPublisher::new(publisher_qos, rtps_group, listener, status_kind);

        let publisher_actor = spawn_actor(publisher);
        let publisher_address = publisher_actor.address().clone();
        self.user_defined_publisher_list
            .insert(guid.into(), publisher_actor);

        publisher_address
    }

    async fn create_subscriber(
        &mut self,
        qos: QosKind<SubscriberQos>,
        a_listener: Option<Box<dyn SubscriberListener + Send + Sync>>,
        mask: Vec<StatusKind>,
    ) -> ActorAddress<DdsSubscriber> {
        let subscriber_qos = match qos {
            QosKind::Default => self.default_subscriber_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let subcriber_counter = self.create_unique_subscriber_id().await;
        let entity_id = EntityId::new([subcriber_counter, 0, 0], USER_DEFINED_READER_GROUP);
        let guid = Guid::new(self.rtps_participant.guid().prefix(), entity_id);
        let rtps_group = RtpsGroup::new(guid);
        let listener = a_listener.map(|l| spawn_actor(DdsSubscriberListener::new(l)));
        let status_kind = mask.to_vec();

        let subscriber = DdsSubscriber::new(subscriber_qos, rtps_group, listener, status_kind);

        let subscriber_actor = spawn_actor(subscriber);
        let subscriber_address = subscriber_actor.address().clone();

        self.user_defined_subscriber_list
            .insert(guid.into(), subscriber_actor);

        subscriber_address
    }

    async fn create_topic(
        &mut self,
        topic_name: String,
        type_name: String,
        qos: QosKind<TopicQos>,
        _a_listener: Option<Box<dyn TopicListener + Send + Sync>>,
        _mask: Vec<StatusKind>,
    ) -> ActorAddress<DdsTopic> {
        let qos = match qos {
            QosKind::Default => self.default_topic_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let topic_counter = self.create_unique_topic_id().await;
        let entity_id = EntityId::new([topic_counter, 0, 0], USER_DEFINED_TOPIC);
        let guid = Guid::new(self.rtps_participant.guid().prefix(), entity_id);

        let topic = DdsTopic::new(guid, qos, type_name, &topic_name);

        let topic_actor: crate::implementation::utils::actor::Actor<DdsTopic> = spawn_actor(topic);
        let topic_address = topic_actor.address().clone();
        self.topic_list.insert(guid.into(), topic_actor);

        topic_address
    }

    async fn get_default_unicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_participant
            .default_unicast_locator_list()
            .to_vec()
    }

    async fn get_default_multicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_participant
            .default_multicast_locator_list()
            .to_vec()
    }

    async fn get_metatraffic_unicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_participant
            .metatraffic_unicast_locator_list()
            .to_vec()
    }

    async fn get_metatraffic_multicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_participant
            .metatraffic_multicast_locator_list()
            .to_vec()
    }

    async fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_participant.guid().into()
    }

    async fn enable(&mut self) {
        self.enabled = true;
    }

    async fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn ignore_participant(&mut self, handle: InstanceHandle) {
        self.ignored_participants.insert(handle);
    }

    async fn ignore_subscription(&mut self, handle: InstanceHandle) {
        self.ignored_subcriptions.insert(handle);
    }

    async fn ignore_publication(&mut self, handle: InstanceHandle) {
        self.ignored_publications.insert(handle);
    }

    async fn ignore_topic(&self, _handle: InstanceHandle) {
        todo!()
    }

    async fn is_topic_ignored(&self, _handle: InstanceHandle) -> bool {
        todo!()
    }

    async fn _discovered_participant_remove(&mut self, handle: InstanceHandle) {
        self.discovered_participant_list.remove(&handle);
    }

    async fn create_unique_publisher_id(&mut self) -> u8 {
        let counter = self.user_defined_publisher_counter;
        self.user_defined_publisher_counter += 1;
        counter
    }

    async fn delete_user_defined_publisher(&mut self, handle: InstanceHandle) {
        self.user_defined_publisher_list.remove(&handle);
    }

    async fn create_unique_subscriber_id(&mut self) -> u8 {
        let counter = self.user_defined_subscriber_counter;
        self.user_defined_subscriber_counter += 1;
        counter
    }

    async fn delete_user_defined_subscriber(&mut self, handle: InstanceHandle) {
        self.user_defined_subscriber_list.remove(&handle);
    }

    async fn create_unique_topic_id(&mut self) -> u8 {
        let counter = self.user_defined_topic_counter;
        self.user_defined_topic_counter += 1;
        counter
    }

    async fn delete_user_defined_topic(&mut self, handle: InstanceHandle) {
        self.topic_list.remove(&handle);
    }

    async fn is_empty(&self) -> bool {
        self.user_defined_publisher_list.len() == 0
            && self.user_defined_subscriber_list.len() == 0
            && self.topic_list.len() == 0
    }

    async fn delete_topic(&mut self, handle: InstanceHandle) {
        self.topic_list.remove(&handle);
    }

    async fn get_qos(&self) -> DomainParticipantQos {
        self.qos.clone()
    }

    async fn data_max_size_serialized(&self) -> usize {
        self.data_max_size_serialized
    }

    async fn delete_contained_entities(&mut self) -> DdsResult<()> {
        for (_, user_defined_publisher) in self.user_defined_publisher_list.drain() {
            user_defined_publisher
                .address()
                .send_and_reply_blocking(dds_publisher::delete_contained_entities::new())?;
        }

        for (_, user_defined_subscriber) in self.user_defined_subscriber_list.drain() {
            user_defined_subscriber
                .address()
                .send_and_reply_blocking(dds_subscriber::delete_contained_entities::new())?;
        }

        self.topic_list.clear();

        Ok(())
    }

    async fn set_default_publisher_qos(&mut self, qos: PublisherQos) {
        self.default_publisher_qos = qos;
    }

    async fn default_publisher_qos(&self) -> PublisherQos {
        self.default_publisher_qos.clone()
    }

    async fn set_default_subscriber_qos(&mut self, qos: SubscriberQos) {
        self.default_subscriber_qos = qos;
    }

    async fn default_subscriber_qos(&self) -> SubscriberQos {
        self.default_subscriber_qos.clone()
    }

    async fn set_default_topic_qos(&mut self, qos: TopicQos) {
        self.default_topic_qos = qos;
    }

    async fn default_topic_qos(&self) -> TopicQos {
        self.default_topic_qos.clone()
    }

    async fn discovered_topic_list(&self) -> Vec<InstanceHandle> {
        self.discovered_topic_list.keys().cloned().collect()
    }

    async fn discovered_topic_data(
        &self,
        topic_handle: InstanceHandle,
    ) -> DdsResult<TopicBuiltinTopicData> {
        self.discovered_topic_list
            .get(&topic_handle)
            .cloned()
            .ok_or(DdsError::BadParameter)
    }

    async fn set_qos(&mut self, qos: DomainParticipantQos) {
        self.qos = qos;
    }

    async fn get_discovered_participants(&self) -> Vec<InstanceHandle> {
        self.discovered_participant_list.keys().cloned().collect()
    }
}

pub struct DiscoveredParticipantAdd {
    handle: InstanceHandle,
    discovered_participant_data: SpdpDiscoveredParticipantData,
}

impl DiscoveredParticipantAdd {
    pub fn new(
        handle: InstanceHandle,
        discovered_participant_data: SpdpDiscoveredParticipantData,
    ) -> Self {
        Self {
            handle,
            discovered_participant_data,
        }
    }
}

impl Mail for DiscoveredParticipantAdd {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<DiscoveredParticipantAdd> for DdsDomainParticipant {
    async fn handle(
        &mut self,
        mail: DiscoveredParticipantAdd,
    ) -> <DiscoveredParticipantAdd as Mail>::Result {
        self.discovered_participant_list
            .insert(mail.handle, mail.discovered_participant_data);
    }
}

pub struct DiscoveredTopicAdd {
    handle: InstanceHandle,
    discovered_topic_data: TopicBuiltinTopicData,
}

impl DiscoveredTopicAdd {
    pub fn new(handle: InstanceHandle, discovered_topic_data: TopicBuiltinTopicData) -> Self {
        Self {
            handle,
            discovered_topic_data,
        }
    }
}

impl Mail for DiscoveredTopicAdd {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<DiscoveredTopicAdd> for DdsDomainParticipant {
    async fn handle(&mut self, mail: DiscoveredTopicAdd) -> <DiscoveredTopicAdd as Mail>::Result {
        self.discovered_topic_list
            .insert(mail.handle, mail.discovered_topic_data);
    }
}

pub struct GetUserDefinedTopicList;

impl Mail for GetUserDefinedTopicList {
    type Result = Vec<ActorAddress<DdsTopic>>;
}

#[async_trait::async_trait]
impl MailHandler<GetUserDefinedTopicList> for DdsDomainParticipant {
    async fn handle(
        &mut self,
        _mail: GetUserDefinedTopicList,
    ) -> <GetUserDefinedTopicList as Mail>::Result {
        self.topic_list
            .values()
            .map(|a| a.address().clone())
            .collect()
    }
}

pub struct DiscoveredParticipantGet {
    handle: InstanceHandle,
}

impl DiscoveredParticipantGet {
    pub fn new(handle: InstanceHandle) -> Self {
        Self { handle }
    }
}

impl Mail for DiscoveredParticipantGet {
    type Result = Option<SpdpDiscoveredParticipantData>;
}

#[async_trait::async_trait]
impl MailHandler<DiscoveredParticipantGet> for DdsDomainParticipant {
    async fn handle(
        &mut self,
        mail: DiscoveredParticipantGet,
    ) -> <DiscoveredParticipantGet as Mail>::Result {
        self.discovered_participant_list.get(&mail.handle).cloned()
    }
}

pub struct IsPublicationIgnored {
    handle: InstanceHandle,
}

impl IsPublicationIgnored {
    pub fn new(handle: InstanceHandle) -> Self {
        Self { handle }
    }
}

impl Mail for IsPublicationIgnored {
    type Result = bool;
}

#[async_trait::async_trait]
impl MailHandler<IsPublicationIgnored> for DdsDomainParticipant {
    async fn handle(
        &mut self,
        mail: IsPublicationIgnored,
    ) -> <IsPublicationIgnored as Mail>::Result {
        self.ignored_publications.contains(&mail.handle)
    }
}

pub struct IsSubscriptionIgnored {
    handle: InstanceHandle,
}

impl IsSubscriptionIgnored {
    pub fn new(handle: InstanceHandle) -> Self {
        Self { handle }
    }
}

impl Mail for IsSubscriptionIgnored {
    type Result = bool;
}

#[async_trait::async_trait]
impl MailHandler<IsSubscriptionIgnored> for DdsDomainParticipant {
    async fn handle(
        &mut self,
        mail: IsSubscriptionIgnored,
    ) -> <IsSubscriptionIgnored as Mail>::Result {
        self.ignored_subcriptions.contains(&mail.handle)
    }
}

pub struct IsParticipantIgnored {
    handle: InstanceHandle,
}

impl IsParticipantIgnored {
    pub fn new(handle: InstanceHandle) -> Self {
        Self { handle }
    }
}

impl Mail for IsParticipantIgnored {
    type Result = bool;
}

#[async_trait::async_trait]
impl MailHandler<IsParticipantIgnored> for DdsDomainParticipant {
    async fn handle(
        &mut self,
        mail: IsParticipantIgnored,
    ) -> <IsParticipantIgnored as Mail>::Result {
        self.ignored_participants.contains(&mail.handle)
    }
}

pub struct GetDomainId;

impl Mail for GetDomainId {
    type Result = DomainId;
}

#[async_trait::async_trait]
impl MailHandler<GetDomainId> for DdsDomainParticipant {
    async fn handle(&mut self, _mail: GetDomainId) -> <GetDomainId as Mail>::Result {
        self.domain_id
    }
}

pub struct GetDomainTag;

impl Mail for GetDomainTag {
    type Result = String;
}

#[async_trait::async_trait]
impl MailHandler<GetDomainTag> for DdsDomainParticipant {
    async fn handle(&mut self, _mail: GetDomainTag) -> <GetDomainTag as Mail>::Result {
        self.domain_tag.clone()
    }
}
pub struct GetBuiltInSubscriber;

impl Mail for GetBuiltInSubscriber {
    type Result = ActorAddress<DdsSubscriber>;
}

#[async_trait::async_trait]
impl MailHandler<GetBuiltInSubscriber> for DdsDomainParticipant {
    async fn handle(
        &mut self,
        _mail: GetBuiltInSubscriber,
    ) -> <GetBuiltInSubscriber as Mail>::Result {
        self.builtin_subscriber.address().clone()
    }
}

pub struct GetUdpTransportWrite;

impl Mail for GetUdpTransportWrite {
    type Result = ActorAddress<UdpTransportWrite>;
}

#[async_trait::async_trait]
impl MailHandler<GetUdpTransportWrite> for DdsDomainParticipant {
    async fn handle(
        &mut self,
        _mail: GetUdpTransportWrite,
    ) -> <GetUdpTransportWrite as Mail>::Result {
        self.udp_transport_write.address().clone()
    }
}

pub struct GetGuid;

impl Mail for GetGuid {
    type Result = Guid;
}

#[async_trait::async_trait]
impl MailHandler<GetGuid> for DdsDomainParticipant {
    async fn handle(&mut self, _mail: GetGuid) -> <GetGuid as Mail>::Result {
        self.rtps_participant.guid()
    }
}

pub struct GetProtocolVersion;

impl Mail for GetProtocolVersion {
    type Result = ProtocolVersion;
}

#[async_trait::async_trait]
impl MailHandler<GetProtocolVersion> for DdsDomainParticipant {
    async fn handle(&mut self, _mail: GetProtocolVersion) -> <GetProtocolVersion as Mail>::Result {
        self.rtps_participant.protocol_version()
    }
}

pub struct GetVendorId;

impl Mail for GetVendorId {
    type Result = VendorId;
}

#[async_trait::async_trait]
impl MailHandler<GetVendorId> for DdsDomainParticipant {
    async fn handle(&mut self, _mail: GetVendorId) -> <GetVendorId as Mail>::Result {
        self.rtps_participant.vendor_id()
    }
}

pub struct GetUserDefinedPublisherList;

impl Mail for GetUserDefinedPublisherList {
    type Result = Vec<ActorAddress<DdsPublisher>>;
}

#[async_trait::async_trait]
impl MailHandler<GetUserDefinedPublisherList> for DdsDomainParticipant {
    async fn handle(
        &mut self,
        _mail: GetUserDefinedPublisherList,
    ) -> <GetUserDefinedPublisherList as Mail>::Result {
        self.user_defined_publisher_list
            .values()
            .map(|a| a.address().clone())
            .collect()
    }
}

pub struct GetUserDefinedSubscriberList;

impl Mail for GetUserDefinedSubscriberList {
    type Result = Vec<ActorAddress<DdsSubscriber>>;
}

#[async_trait::async_trait]
impl MailHandler<GetUserDefinedSubscriberList> for DdsDomainParticipant {
    async fn handle(
        &mut self,
        _mail: GetUserDefinedSubscriberList,
    ) -> <GetUserDefinedSubscriberList as Mail>::Result {
        self.user_defined_subscriber_list
            .values()
            .map(|a| a.address().clone())
            .collect()
    }
}

pub struct AsSpdpDiscoveredParticipantData;

impl Mail for AsSpdpDiscoveredParticipantData {
    type Result = SpdpDiscoveredParticipantData;
}

#[async_trait::async_trait]
impl MailHandler<AsSpdpDiscoveredParticipantData> for DdsDomainParticipant {
    async fn handle(
        &mut self,
        _mail: AsSpdpDiscoveredParticipantData,
    ) -> <AsSpdpDiscoveredParticipantData as Mail>::Result {
        SpdpDiscoveredParticipantData::new(
            ParticipantBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: self.rtps_participant.guid().into(),
                },
                self.qos.user_data.clone(),
            ),
            ParticipantProxy::new(
                Some(self.domain_id),
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
}

pub struct GetListener;

impl Mail for GetListener {
    type Result = Option<ActorAddress<DdsDomainParticipantListener>>;
}

#[async_trait::async_trait]
impl MailHandler<GetListener> for DdsDomainParticipant {
    async fn handle(&mut self, _mail: GetListener) -> <GetListener as Mail>::Result {
        self.listener.as_ref().map(|l| l.address().clone())
    }
}

pub struct GetStatusKind;

impl Mail for GetStatusKind {
    type Result = Vec<StatusKind>;
}

#[async_trait::async_trait]
impl MailHandler<GetStatusKind> for DdsDomainParticipant {
    async fn handle(&mut self, _mail: GetStatusKind) -> <GetStatusKind as Mail>::Result {
        self.status_kind.clone()
    }
}

pub struct GetCurrentTime;

impl Mail for GetCurrentTime {
    type Result = Time;
}

#[async_trait::async_trait]
impl MailHandler<GetCurrentTime> for DdsDomainParticipant {
    async fn handle(&mut self, _mail: GetCurrentTime) -> <GetCurrentTime as Mail>::Result {
        let now_system_time = SystemTime::now();
        let unix_time = now_system_time
            .duration_since(UNIX_EPOCH)
            .expect("Clock time is before Unix epoch start");
        Time::new(unix_time.as_secs() as i32, unix_time.subsec_nanos())
    }
}

pub struct GetBuiltinPublisher;

impl Mail for GetBuiltinPublisher {
    type Result = ActorAddress<DdsPublisher>;
}

#[async_trait::async_trait]
impl MailHandler<GetBuiltinPublisher> for DdsDomainParticipant {
    async fn handle(
        &mut self,
        _mail: GetBuiltinPublisher,
    ) -> <GetBuiltinPublisher as Mail>::Result {
        self.builtin_publisher.address().clone()
    }
}

pub struct ProcessUserDefinedRtpsMessage {
    message: RtpsMessageRead,
    participant_address: ActorAddress<DdsDomainParticipant>,
}

impl ProcessUserDefinedRtpsMessage {
    pub fn new(
        message: RtpsMessageRead,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) -> Self {
        Self {
            message,
            participant_address,
        }
    }
}

impl Mail for ProcessUserDefinedRtpsMessage {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<ProcessUserDefinedRtpsMessage> for DdsDomainParticipant {
    async fn handle(
        &mut self,
        mail: ProcessUserDefinedRtpsMessage,
    ) -> <ProcessUserDefinedRtpsMessage as Mail>::Result {
        let participant_mask_listener = (
            self.listener.as_ref().map(|a| a.address()).cloned(),
            self.status_kind.clone(),
        );
        for user_defined_subscriber_address in self
            .user_defined_subscriber_list
            .values()
            .map(|a| a.address())
        {
            user_defined_subscriber_address
                .send_only(dds_subscriber::process_rtps_message::new(
                    mail.message.clone(),
                    self.get_current_time(),
                    mail.participant_address.clone(),
                    user_defined_subscriber_address.clone(),
                    participant_mask_listener.clone(),
                ))
                .await
                .expect("Should not fail to send command");

            user_defined_subscriber_address
                .send_only(dds_subscriber::send_message::new(
                    RtpsMessageHeader::new(
                        self.rtps_participant.protocol_version(),
                        self.rtps_participant.vendor_id(),
                        self.rtps_participant.guid().prefix(),
                    ),
                    self.udp_transport_write.address().clone(),
                ))
                .await
                .expect("Should not fail to send command");
        }

        for user_defined_publisher_address in self
            .user_defined_publisher_list
            .values()
            .map(|a| a.address())
        {
            user_defined_publisher_address
                .send_only(dds_publisher::process_rtps_message::new(
                    mail.message.clone(),
                ))
                .await
                .expect("Should not fail to send command");
            user_defined_publisher_address
                .send_only(dds_publisher::send_message::new(
                    RtpsMessageHeader::new(
                        self.rtps_participant.protocol_version(),
                        self.rtps_participant.vendor_id(),
                        self.rtps_participant.guid().prefix(),
                    ),
                    self.udp_transport_write.address().clone(),
                    self.get_current_time(),
                ))
                .await
                .expect("Should not fail to send command");
        }
    }
}

fn create_builtin_stateful_writer(guid: Guid) -> RtpsWriter {
    let unicast_locator_list = &[];
    let multicast_locator_list = &[];
    let topic_kind = TopicKind::WithKey;
    let push_mode = true;
    let heartbeat_period = DEFAULT_HEARTBEAT_PERIOD;
    let nack_response_delay = DEFAULT_NACK_RESPONSE_DELAY;
    let nack_suppression_duration = DEFAULT_NACK_SUPPRESSION_DURATION;
    let data_max_size_serialized = usize::MAX;

    RtpsWriter::new(
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
    )
}

fn create_builtin_stateless_writer(guid: Guid) -> RtpsWriter {
    let unicast_locator_list = &[];
    let multicast_locator_list = &[];

    RtpsWriter::new(
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
    )
}

fn create_builtin_stateless_reader(guid: Guid) -> RtpsReader {
    let unicast_locator_list = &[];
    let multicast_locator_list = &[];

    RtpsReader::new(
        RtpsEndpoint::new(
            guid,
            TopicKind::WithKey,
            unicast_locator_list,
            multicast_locator_list,
        ),
        DURATION_ZERO,
        DURATION_ZERO,
        false,
    )
}

fn create_builtin_stateful_reader(guid: Guid) -> RtpsReader {
    let topic_kind = TopicKind::WithKey;
    let heartbeat_response_delay = DEFAULT_HEARTBEAT_RESPONSE_DELAY;
    let heartbeat_suppression_duration = DEFAULT_HEARTBEAT_SUPPRESSION_DURATION;
    let expects_inline_qos = false;
    let unicast_locator_list = &[];
    let multicast_locator_list = &[];

    RtpsReader::new(
        RtpsEndpoint::new(
            guid,
            topic_kind,
            unicast_locator_list,
            multicast_locator_list,
        ),
        heartbeat_response_delay,
        heartbeat_suppression_duration,
        expects_inline_qos,
    )
}
