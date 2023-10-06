use dust_dds_derive::actor_interface;
use tracing::warn;

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
            reader_proxy::RtpsReaderProxy,
            types::{
                EntityId, Guid, Locator, ReliabilityKind, SequenceNumber, TopicKind,
                BUILT_IN_READER_GROUP, BUILT_IN_READER_WITH_KEY, BUILT_IN_TOPIC,
                BUILT_IN_WRITER_GROUP, BUILT_IN_WRITER_WITH_KEY, ENTITYID_PARTICIPANT,
                ENTITYID_UNKNOWN, USER_DEFINED_READER_GROUP, USER_DEFINED_TOPIC,
                USER_DEFINED_WRITER_GROUP,
            },
            writer::RtpsWriter,
            writer_proxy::RtpsWriterProxy,
        },
        rtps_udp_psm::udp_transport::UdpTransportWrite,
        utils::actor::{spawn_actor, Actor, ActorAddress},
    },
    infrastructure::{
        instance::InstanceHandle,
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            LifespanQosPolicy, ReliabilityQosPolicy, ReliabilityQosPolicyKind,
            ResourceLimitsQosPolicy, TransportPriorityQosPolicy,
        },
        status::StatusKind,
        time::{DurationKind, DURATION_ZERO},
    },
    publication::publisher_listener::PublisherListener,
    subscription::{
        sample_info::{
            InstanceStateKind, SampleStateKind, ANY_INSTANCE_STATE, ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
        },
        subscriber_listener::SubscriberListener,
    },
    topic_definition::{
        topic_listener::TopicListener,
        type_support::{dds_deserialize_from_bytes, dds_serialize_key, dds_serialize_to_bytes},
    },
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
    dds_data_reader,
    dds_data_writer::{self, DdsDataWriter},
    dds_domain_participant_listener::DdsDomainParticipantListener,
    dds_publisher::{self, DdsPublisher},
    dds_publisher_listener::DdsPublisherListener,
    dds_subscriber,
    dds_subscriber_listener::DdsSubscriberListener,
    dds_topic,
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
    ignored_topic_list: HashSet<InstanceHandle>,
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
            .send_mail_and_await_reply_blocking(dds_subscriber::data_reader_add::new(
                spdp_builtin_participant_reader_guid.into(),
                spdp_builtin_participant_reader,
            ))
            .unwrap();
        builtin_subscriber
            .address()
            .send_mail_and_await_reply_blocking(dds_subscriber::data_reader_add::new(
                sedp_builtin_topics_reader_guid.into(),
                sedp_builtin_topics_reader,
            ))
            .unwrap();
        builtin_subscriber
            .address()
            .send_mail_and_await_reply_blocking(dds_subscriber::data_reader_add::new(
                sedp_builtin_publications_reader_guid.into(),
                sedp_builtin_publications_reader,
            ))
            .unwrap();
        builtin_subscriber
            .address()
            .send_mail_and_await_reply_blocking(dds_subscriber::data_reader_add::new(
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
                .send_mail_and_await_reply_blocking(dds_data_writer::reader_locator_add::new(
                    reader_locator,
                ))
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
            .send_mail_and_await_reply_blocking(dds_publisher::datawriter_add::new(
                spdp_builtin_participant_writer_guid.into(),
                spdp_builtin_participant_writer,
            ))
            .unwrap();
        builtin_publisher
            .address()
            .send_mail_and_await_reply_blocking(dds_publisher::datawriter_add::new(
                sedp_builtin_topics_writer_guid.into(),
                sedp_builtin_topics_writer_actor,
            ))
            .unwrap();
        builtin_publisher
            .address()
            .send_mail_and_await_reply_blocking(dds_publisher::datawriter_add::new(
                sedp_builtin_publications_writer_guid.into(),
                sedp_builtin_publications_writer_actor,
            ))
            .unwrap();
        builtin_publisher
            .address()
            .send_mail_and_await_reply_blocking(dds_publisher::datawriter_add::new(
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
            ignored_topic_list: HashSet::new(),
            data_max_size_serialized,
            udp_transport_write,
            listener,
            status_kind,
        }
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
        let publisher_address = publisher_actor.address();
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
        let subscriber_address = subscriber_actor.address();

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
        let topic_address = topic_actor.address();
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
                .send_mail_and_await_reply_blocking(
                    dds_publisher::delete_contained_entities::new(),
                )?;
        }

        for (_, user_defined_subscriber) in self.user_defined_subscriber_list.drain() {
            user_defined_subscriber
                .address()
                .send_mail_and_await_reply_blocking(
                    dds_subscriber::delete_contained_entities::new(),
                )?;
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

    async fn discovered_participant_add(
        &mut self,
        handle: InstanceHandle,
        discovered_participant_data: SpdpDiscoveredParticipantData,
    ) {
        self.discovered_participant_list
            .insert(handle, discovered_participant_data);
    }

    async fn get_user_defined_topic_list(&self) -> Vec<ActorAddress<DdsTopic>> {
        self.topic_list.values().map(|a| a.address()).collect()
    }

    async fn discovered_participant_get(
        &self,
        handle: InstanceHandle,
    ) -> Option<SpdpDiscoveredParticipantData> {
        self.discovered_participant_list.get(&handle).cloned()
    }

    async fn is_publication_ignored(&self, handle: InstanceHandle) -> bool {
        self.ignored_publications.contains(&handle)
    }

    async fn is_subscription_ignored(&self, handle: InstanceHandle) -> bool {
        self.ignored_subcriptions.contains(&handle)
    }

    async fn is_participant_ignored(&self, handle: InstanceHandle) -> bool {
        self.ignored_participants.contains(&handle)
    }

    async fn get_domain_id(&self) -> DomainId {
        self.domain_id
    }

    async fn get_domain_tag(&self) -> String {
        self.domain_tag.clone()
    }

    async fn get_built_in_subscriber(&self) -> ActorAddress<DdsSubscriber> {
        self.builtin_subscriber.address()
    }

    async fn get_upd_transport_write(&self) -> ActorAddress<UdpTransportWrite> {
        self.udp_transport_write.address()
    }

    async fn get_guid(&self) -> Guid {
        self.rtps_participant.guid()
    }

    async fn get_user_defined_publisher_list(&self) -> Vec<ActorAddress<DdsPublisher>> {
        self.user_defined_publisher_list
            .values()
            .map(|a| a.address())
            .collect()
    }

    async fn get_user_defined_subscriber_list(&self) -> Vec<ActorAddress<DdsSubscriber>> {
        self.user_defined_subscriber_list
            .values()
            .map(|a| a.address())
            .collect()
    }

    async fn as_spdp_discovered_participant_data(&self) -> SpdpDiscoveredParticipantData {
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

    async fn get_listener(&self) -> Option<ActorAddress<DdsDomainParticipantListener>> {
        self.listener.as_ref().map(|l| l.address())
    }

    async fn get_status_kind(&self) -> Vec<StatusKind> {
        self.status_kind.clone()
    }

    async fn get_current_time(&self) -> Time {
        let now_system_time = SystemTime::now();
        let unix_time = now_system_time
            .duration_since(UNIX_EPOCH)
            .expect("Clock time is before Unix epoch start");
        Time::new(unix_time.as_secs() as i32, unix_time.subsec_nanos())
    }

    async fn get_builtin_publisher(&self) -> ActorAddress<DdsPublisher> {
        self.builtin_publisher.address()
    }

    async fn send_message(&self) {
        let now = self.get_current_time().await;
        let header = RtpsMessageHeader::new(
            self.rtps_participant.protocol_version(),
            self.rtps_participant.vendor_id(),
            self.rtps_participant.guid().prefix(),
        );
        self.builtin_publisher
            .send_mail(dds_publisher::send_message::new(
                header,
                self.udp_transport_write.address(),
                now,
            ))
            .await;
        self.builtin_subscriber
            .send_mail(dds_subscriber::send_message::new(
                header,
                self.udp_transport_write.address(),
            ))
            .await;

        for publisher in self.user_defined_publisher_list.values() {
            publisher
                .send_mail(dds_publisher::send_message::new(
                    header,
                    self.udp_transport_write.address(),
                    now,
                ))
                .await;
        }

        for subscriber in self.user_defined_subscriber_list.values() {
            subscriber
                .send_mail(dds_subscriber::send_message::new(
                    header,
                    self.udp_transport_write.address(),
                ))
                .await;
        }
    }

    async fn process_metatraffic_rtps_message(
        &self,
        message: RtpsMessageRead,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) -> DdsResult<()> {
        let reception_timestamp = self.get_current_time().await;
        let participant_mask_listener = (
            self.listener.as_ref().map(|l| l.address()),
            self.status_kind.clone(),
        );
        self.builtin_subscriber
            .send_mail_and_await_reply(dds_subscriber::process_rtps_message::new(
                message.clone(),
                reception_timestamp,
                participant_address.clone(),
                self.builtin_subscriber.address(),
                participant_mask_listener,
            ))
            .await?;

        self.builtin_publisher
            .send_mail_and_await_reply(dds_publisher::process_rtps_message::new(message))
            .await;

        Ok(())
    }

    async fn process_user_defined_rtps_message(
        &self,
        message: RtpsMessageRead,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) {
        let participant_mask_listener = (
            self.listener.as_ref().map(|a| a.address()),
            self.status_kind.clone(),
        );
        for user_defined_subscriber_address in self
            .user_defined_subscriber_list
            .values()
            .map(|a| a.address())
        {
            user_defined_subscriber_address
                .send_mail(dds_subscriber::process_rtps_message::new(
                    message.clone(),
                    self.get_current_time().await,
                    participant_address.clone(),
                    user_defined_subscriber_address.clone(),
                    participant_mask_listener.clone(),
                ))
                .await
                .expect("Should not fail to send command");

            user_defined_subscriber_address
                .send_mail(dds_subscriber::send_message::new(
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
                .send_mail(dds_publisher::process_rtps_message::new(message.clone()))
                .await
                .expect("Should not fail to send command");
            user_defined_publisher_address
                .send_mail(dds_publisher::send_message::new(
                    RtpsMessageHeader::new(
                        self.rtps_participant.protocol_version(),
                        self.rtps_participant.vendor_id(),
                        self.rtps_participant.guid().prefix(),
                    ),
                    self.udp_transport_write.address().clone(),
                    self.get_current_time().await,
                ))
                .await
                .expect("Should not fail to send command");
        }
    }

    async fn announce_created_or_modified_data_writer(
        &self,
        discovered_writer_data: DiscoveredWriterData,
    ) {
        if let Some(sedp_publications_announcer) = self
            .builtin_publisher
            .send_mail_and_await_reply(dds_publisher::lookup_datawriter::new(
                DCPS_PUBLICATION.to_string(),
            ))
            .await
        {
            let timestamp = self.get_current_time().await;
            let serialized_data = dds_serialize_to_bytes(&discovered_writer_data)
                .expect("Shouldn't fail to serialize builtin type");
            let instance_serialized_key = dds_serialize_key(&discovered_writer_data)
                .expect("Shouldn't fail to serialize key of builtin type");
            sedp_publications_announcer
                .send_mail_and_await_reply(dds_data_writer::write_w_timestamp::new(
                    serialized_data,
                    instance_serialized_key,
                    None,
                    timestamp,
                ))
                .await
                .expect("Shouldn't fail to send to built-in data writer")
                .expect("Shouldn't fail to write to built-in data writer");
        }
    }

    async fn process_builtin_discovery(
        &mut self,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) {
        self.process_spdp_participant_discovery().await;
        self.process_sedp_publications_discovery(participant_address.clone())
            .await;
        self.process_sedp_subscriptions_discovery(participant_address)
            .await;
        self.process_sedp_topics_discovery().await;
    }
}

impl DdsDomainParticipant {
    async fn process_spdp_participant_discovery(&mut self) {
        if let Some(spdp_participant_reader) = self
            .builtin_subscriber
            .send_mail_and_await_reply(dds_subscriber::lookup_datareader::new(
                DCPS_PARTICIPANT.to_string(),
            ))
            .await
        {
            if let Ok(spdp_data_sample_list) = spdp_participant_reader
                .send_mail_and_await_reply(dds_data_reader::read::new(
                    i32::MAX,
                    vec![SampleStateKind::NotRead],
                    ANY_VIEW_STATE.to_vec(),
                    ANY_INSTANCE_STATE.to_vec(),
                    None,
                ))
                .await
                .expect("Can not fail to send mail to builtin reader")
            {
                for (spdp_data_sample, _) in spdp_data_sample_list {
                    match dds_deserialize_from_bytes::<SpdpDiscoveredParticipantData>(
                        spdp_data_sample.expect("Should contain data").as_ref(),
                    ) {
                        Ok(discovered_participant_data) => {
                            self.process_discovered_participant_data(discovered_participant_data)
                                .await
                        }
                        Err(e) => warn!(
                            "Received invalid SpdpDiscoveredParticipantData. Error {:?}",
                            e
                        ),
                    }
                }
            }
        }
    }

    async fn process_discovered_participant_data(
        &mut self,
        discovered_participant_data: SpdpDiscoveredParticipantData,
    ) {
        // Check that the domainId of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        // AND
        // Check that the domainTag of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        // IN CASE no domain id was transmitted the a local domain id is assumed
        // (as specified in Table 9.19 - ParameterId mapping and default values)
        let is_domain_id_matching = discovered_participant_data
            .participant_proxy()
            .domain_id()
            .unwrap_or(self.domain_id)
            == self.domain_id;
        let is_domain_tag_matching =
            discovered_participant_data.participant_proxy().domain_tag() == self.domain_tag;
        let discovered_participant_handle = InstanceHandle::new(
            discovered_participant_data
                .dds_participant_data()
                .key()
                .value,
        );
        let is_participant_ignored = self
            .ignored_participants
            .contains(&discovered_participant_handle);
        if is_domain_id_matching && is_domain_tag_matching && !is_participant_ignored {
            self.add_matched_publications_detector(&discovered_participant_data)
                .await;
            self.add_matched_publications_announcer(&discovered_participant_data)
                .await;
            self.add_matched_subscriptions_detector(&discovered_participant_data)
                .await;
            self.add_matched_subscriptions_announcer(&discovered_participant_data)
                .await;
            self.add_matched_topics_detector(&discovered_participant_data)
                .await;
            self.add_matched_topics_announcer(&discovered_participant_data)
                .await;

            self.discovered_participant_list.insert(
                InstanceHandle::new(
                    discovered_participant_data
                        .dds_participant_data()
                        .key()
                        .value,
                ),
                discovered_participant_data,
            );
        }
    }

    async fn add_matched_publications_detector(
        &self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if let Some(sedp_publications_announcer) = self
            .builtin_publisher
            .send_mail_and_await_reply(dds_publisher::lookup_datawriter::new(
                DCPS_PUBLICATION.to_string(),
            ))
            .await
        {
            if discovered_participant_data
                .participant_proxy()
                .available_builtin_endpoints()
                .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR)
            {
                let remote_reader_guid = Guid::new(
                    discovered_participant_data
                        .participant_proxy()
                        .guid_prefix(),
                    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
                );
                let remote_group_entity_id = ENTITYID_UNKNOWN;
                let expects_inline_qos = false;
                let proxy = RtpsReaderProxy::new(
                    remote_reader_guid,
                    remote_group_entity_id,
                    discovered_participant_data
                        .participant_proxy()
                        .metatraffic_unicast_locator_list(),
                    discovered_participant_data
                        .participant_proxy()
                        .metatraffic_multicast_locator_list(),
                    expects_inline_qos,
                    true,
                    ReliabilityKind::Reliable,
                    SequenceNumber::from(0),
                );
                sedp_publications_announcer
                    .send_mail_and_await_reply(dds_data_writer::matched_reader_add::new(proxy))
                    .await
                    .unwrap();
            }
        }
    }

    async fn add_matched_publications_announcer(
        &self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if let Some(sedp_publications_detector) = self
            .builtin_subscriber
            .send_mail_and_await_reply(dds_subscriber::lookup_datareader::new(
                DCPS_PUBLICATION.to_string(),
            ))
            .await
        {
            if discovered_participant_data
                .participant_proxy()
                .available_builtin_endpoints()
                .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER)
            {
                let remote_writer_guid = Guid::new(
                    discovered_participant_data
                        .participant_proxy()
                        .guid_prefix(),
                    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                );
                let remote_group_entity_id = ENTITYID_UNKNOWN;
                let data_max_size_serialized = None;

                let proxy = RtpsWriterProxy::new(
                    remote_writer_guid,
                    discovered_participant_data
                        .participant_proxy()
                        .metatraffic_unicast_locator_list(),
                    discovered_participant_data
                        .participant_proxy()
                        .metatraffic_multicast_locator_list(),
                    data_max_size_serialized,
                    remote_group_entity_id,
                );

                sedp_publications_detector
                    .send_mail_and_await_reply(dds_data_reader::matched_writer_add::new(proxy))
                    .await
                    .unwrap();
            }
        }
    }

    async fn add_matched_subscriptions_detector(
        &self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if let Some(sedp_subscriptions_announcer) = self
            .builtin_publisher
            .send_mail_and_await_reply(dds_publisher::lookup_datawriter::new(
                DCPS_SUBSCRIPTION.to_string(),
            ))
            .await
        {
            if discovered_participant_data
                .participant_proxy()
                .available_builtin_endpoints()
                .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR)
            {
                let remote_reader_guid = Guid::new(
                    discovered_participant_data
                        .participant_proxy()
                        .guid_prefix(),
                    ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
                );
                let remote_group_entity_id = ENTITYID_UNKNOWN;
                let expects_inline_qos = false;
                let proxy = RtpsReaderProxy::new(
                    remote_reader_guid,
                    remote_group_entity_id,
                    discovered_participant_data
                        .participant_proxy()
                        .metatraffic_unicast_locator_list(),
                    discovered_participant_data
                        .participant_proxy()
                        .metatraffic_multicast_locator_list(),
                    expects_inline_qos,
                    true,
                    ReliabilityKind::Reliable,
                    SequenceNumber::from(0),
                );
                sedp_subscriptions_announcer
                    .send_mail_and_await_reply(dds_data_writer::matched_reader_add::new(proxy))
                    .await
                    .unwrap();
            }
        }
    }

    async fn add_matched_subscriptions_announcer(
        &self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if let Some(sedp_subscriptions_detector) = self
            .builtin_subscriber
            .send_mail_and_await_reply(dds_subscriber::lookup_datareader::new(
                DCPS_SUBSCRIPTION.to_string(),
            ))
            .await
        {
            if discovered_participant_data
                .participant_proxy()
                .available_builtin_endpoints()
                .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER)
            {
                let remote_writer_guid = Guid::new(
                    discovered_participant_data
                        .participant_proxy()
                        .guid_prefix(),
                    ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
                );
                let remote_group_entity_id = ENTITYID_UNKNOWN;
                let data_max_size_serialized = None;

                let proxy = RtpsWriterProxy::new(
                    remote_writer_guid,
                    discovered_participant_data
                        .participant_proxy()
                        .metatraffic_unicast_locator_list(),
                    discovered_participant_data
                        .participant_proxy()
                        .metatraffic_multicast_locator_list(),
                    data_max_size_serialized,
                    remote_group_entity_id,
                );
                sedp_subscriptions_detector
                    .send_mail_and_await_reply(dds_data_reader::matched_writer_add::new(proxy))
                    .await
                    .unwrap();
            }
        }
    }

    async fn add_matched_topics_detector(
        &self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if let Some(sedp_topics_announcer) = self
            .builtin_publisher
            .send_mail_and_await_reply(dds_publisher::lookup_datawriter::new(
                DCPS_TOPIC.to_string(),
            ))
            .await
        {
            if discovered_participant_data
                .participant_proxy()
                .available_builtin_endpoints()
                .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR)
            {
                let remote_reader_guid = Guid::new(
                    discovered_participant_data
                        .participant_proxy()
                        .guid_prefix(),
                    ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
                );
                let remote_group_entity_id = ENTITYID_UNKNOWN;
                let expects_inline_qos = false;
                let proxy = RtpsReaderProxy::new(
                    remote_reader_guid,
                    remote_group_entity_id,
                    discovered_participant_data
                        .participant_proxy()
                        .metatraffic_unicast_locator_list(),
                    discovered_participant_data
                        .participant_proxy()
                        .metatraffic_multicast_locator_list(),
                    expects_inline_qos,
                    true,
                    ReliabilityKind::Reliable,
                    SequenceNumber::from(0),
                );
                sedp_topics_announcer
                    .send_mail_and_await_reply(dds_data_writer::matched_reader_add::new(proxy))
                    .await
                    .unwrap();
            }
        }
    }

    async fn add_matched_topics_announcer(
        &self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if let Some(sedp_topics_detector) = self
            .builtin_subscriber
            .send_mail_and_await_reply(dds_subscriber::lookup_datareader::new(
                DCPS_TOPIC.to_string(),
            ))
            .await
        {
            if discovered_participant_data
                .participant_proxy()
                .available_builtin_endpoints()
                .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER)
            {
                let remote_writer_guid = Guid::new(
                    discovered_participant_data
                        .participant_proxy()
                        .guid_prefix(),
                    ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
                );
                let remote_group_entity_id = ENTITYID_UNKNOWN;
                let data_max_size_serialized = None;

                let proxy = RtpsWriterProxy::new(
                    remote_writer_guid,
                    discovered_participant_data
                        .participant_proxy()
                        .metatraffic_unicast_locator_list(),
                    discovered_participant_data
                        .participant_proxy()
                        .metatraffic_multicast_locator_list(),
                    data_max_size_serialized,
                    remote_group_entity_id,
                );
                sedp_topics_detector
                    .send_mail_and_await_reply(dds_data_reader::matched_writer_add::new(proxy))
                    .await
                    .unwrap();
            }
        }
    }

    async fn process_sedp_publications_discovery(
        &mut self,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) {
        if let Some(sedp_publications_detector) = self
            .builtin_subscriber
            .send_mail_and_await_reply(dds_subscriber::lookup_datareader::new(
                DCPS_PUBLICATION.to_string(),
            ))
            .await
        {
            if let Ok(mut discovered_writer_sample_list) = sedp_publications_detector
                .send_mail_and_await_reply(dds_data_reader::read::new(
                    i32::MAX,
                    ANY_SAMPLE_STATE.to_vec(),
                    ANY_VIEW_STATE.to_vec(),
                    ANY_INSTANCE_STATE.to_vec(),
                    None,
                ))
                .await
                .expect("Can not fail to send mail to builtin reader")
            {
                for (discovered_writer_data, discovered_writer_sample_info) in
                    discovered_writer_sample_list.drain(..)
                {
                    match discovered_writer_sample_info.instance_state {
                        InstanceStateKind::Alive => {
                            match dds_deserialize_from_bytes::<DiscoveredWriterData>(
                                discovered_writer_data
                                    .expect("Should contain data")
                                    .as_ref(),
                            ) {
                                Ok(discovered_writer_data) => {
                                    self.add_matched_writer(
                                        discovered_writer_data,
                                        participant_address.clone(),
                                    )
                                    .await;
                                }
                                Err(e) => warn!(
                                    "Received invalid DiscoveredWriterData sample. Error {:?}",
                                    e
                                ),
                            }
                        }
                        InstanceStateKind::NotAliveDisposed => {
                            self.remove_matched_writer(
                                discovered_writer_sample_info.instance_handle,
                                participant_address.clone(),
                            )
                            .await
                        }
                        InstanceStateKind::NotAliveNoWriters => {
                            todo!()
                        }
                    }
                }
            }
        }
    }

    async fn add_matched_writer(
        &mut self,
        discovered_writer_data: DiscoveredWriterData,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) {
        let is_participant_ignored = self.ignored_participants.contains(&InstanceHandle::new(
            Guid::new(
                discovered_writer_data
                    .writer_proxy()
                    .remote_writer_guid()
                    .prefix(),
                ENTITYID_PARTICIPANT,
            )
            .into(),
        ));
        let is_publication_ignored = self.ignored_publications.contains(&InstanceHandle::new(
            discovered_writer_data.dds_publication_data().key().value,
        ));
        if !is_publication_ignored && !is_participant_ignored {
            if let Some(discovered_participant_data) =
                self.discovered_participant_list.get(&InstanceHandle::new(
                    Guid::new(
                        discovered_writer_data
                            .writer_proxy()
                            .remote_writer_guid()
                            .prefix(),
                        ENTITYID_PARTICIPANT,
                    )
                    .into(),
                ))
            {
                let default_unicast_locator_list = discovered_participant_data
                    .participant_proxy()
                    .default_unicast_locator_list()
                    .to_vec();
                let default_multicast_locator_list = discovered_participant_data
                    .participant_proxy()
                    .default_multicast_locator_list()
                    .to_vec();
                for subscriber in self.user_defined_subscriber_list.values() {
                    let subscriber_address = subscriber.address();
                    let participant_mask_listener = (
                        self.listener.as_ref().map(|l| l.address()),
                        self.status_kind.clone(),
                    );
                    subscriber
                        .send_mail_and_await_reply(dds_subscriber::add_matched_writer::new(
                            discovered_writer_data.clone(),
                            default_unicast_locator_list.clone(),
                            default_multicast_locator_list.clone(),
                            subscriber_address,
                            participant_address.clone(),
                            participant_mask_listener,
                        ))
                        .await;
                }

                // Add writer topic to discovered topic list using the writer instance handle
                let topic_instance_handle =
                    InstanceHandle::new(discovered_writer_data.dds_publication_data().key().value);
                let writer_topic = TopicBuiltinTopicData::new(
                    BuiltInTopicKey::default(),
                    discovered_writer_data
                        .dds_publication_data()
                        .topic_name()
                        .to_string(),
                    discovered_writer_data
                        .dds_publication_data()
                        .get_type_name()
                        .to_string(),
                    discovered_writer_data
                        .dds_publication_data()
                        .durability()
                        .clone(),
                    discovered_writer_data
                        .dds_publication_data()
                        .deadline()
                        .clone(),
                    discovered_writer_data
                        .dds_publication_data()
                        .latency_budget()
                        .clone(),
                    discovered_writer_data
                        .dds_publication_data()
                        .liveliness()
                        .clone(),
                    discovered_writer_data
                        .dds_publication_data()
                        .reliability()
                        .clone(),
                    TransportPriorityQosPolicy::default(),
                    discovered_writer_data
                        .dds_publication_data()
                        .lifespan()
                        .clone(),
                    discovered_writer_data
                        .dds_publication_data()
                        .destination_order()
                        .clone(),
                    HistoryQosPolicy::default(),
                    ResourceLimitsQosPolicy::default(),
                    discovered_writer_data
                        .dds_publication_data()
                        .ownership()
                        .clone(),
                    discovered_writer_data
                        .dds_publication_data()
                        .topic_data()
                        .clone(),
                );
                self.discovered_topic_list
                    .insert(topic_instance_handle, writer_topic);
            }
        }
    }

    async fn remove_matched_writer(
        &self,
        discovered_writer_handle: InstanceHandle,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) {
        for subscriber in self.user_defined_subscriber_list.values() {
            let subscriber_address = subscriber.address();
            let participant_mask_listener = (
                self.listener.as_ref().map(|l| l.address()),
                self.status_kind.clone(),
            );
            subscriber
                .send_mail_and_await_reply(dds_subscriber::remove_matched_writer::new(
                    discovered_writer_handle,
                    subscriber_address,
                    participant_address.clone(),
                    participant_mask_listener,
                ))
                .await;
        }
    }

    async fn process_sedp_subscriptions_discovery(
        &mut self,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) {
        if let Some(sedp_subscriptions_detector) = self
            .builtin_subscriber
            .send_mail_and_await_reply(dds_subscriber::lookup_datareader::new(
                DCPS_SUBSCRIPTION.to_string(),
            ))
            .await
        {
            if let Ok(mut discovered_reader_sample_list) = sedp_subscriptions_detector
                .send_mail_and_await_reply(dds_data_reader::read::new(
                    i32::MAX,
                    ANY_SAMPLE_STATE.to_vec(),
                    ANY_VIEW_STATE.to_vec(),
                    ANY_INSTANCE_STATE.to_vec(),
                    None,
                ))
                .await
                .expect("Can not fail to send mail to builtin reader")
            {
                for (discovered_reader_data, discovered_reader_sample_info) in
                    discovered_reader_sample_list.drain(..)
                {
                    match discovered_reader_sample_info.instance_state {
                        InstanceStateKind::Alive => {
                            match dds_deserialize_from_bytes::<DiscoveredReaderData>(
                                discovered_reader_data
                                    .expect("Should contain data")
                                    .as_ref(),
                            ) {
                                Ok(discovered_reader_data) => {
                                    self.add_matched_reader(
                                        discovered_reader_data,
                                        participant_address.clone(),
                                    )
                                    .await;
                                }
                                Err(e) => warn!(
                                    "Received invalid DiscoveredReaderData sample. Error {:?}",
                                    e
                                ),
                            }
                        }
                        InstanceStateKind::NotAliveDisposed => {
                            self.remove_matched_reader(
                                discovered_reader_sample_info.instance_handle,
                                participant_address.clone(),
                            )
                            .await
                        }
                        InstanceStateKind::NotAliveNoWriters => {
                            todo!()
                        }
                    }
                }
            }
        }
    }

    async fn add_matched_reader(
        &mut self,
        discovered_reader_data: DiscoveredReaderData,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) {
        let is_participant_ignored = self.ignored_participants.contains(&InstanceHandle::new(
            Guid::new(
                discovered_reader_data
                    .reader_proxy()
                    .remote_reader_guid()
                    .prefix(),
                ENTITYID_PARTICIPANT,
            )
            .into(),
        ));
        let is_subscription_ignored = self.ignored_subcriptions.contains(&InstanceHandle::new(
            discovered_reader_data
                .subscription_builtin_topic_data()
                .key()
                .value,
        ));
        if !is_subscription_ignored && !is_participant_ignored {
            if let Some(discovered_participant_data) =
                self.discovered_participant_list.get(&InstanceHandle::new(
                    Guid::new(
                        discovered_reader_data
                            .reader_proxy()
                            .remote_reader_guid()
                            .prefix(),
                        ENTITYID_PARTICIPANT,
                    )
                    .into(),
                ))
            {
                let default_unicast_locator_list = discovered_participant_data
                    .participant_proxy()
                    .default_unicast_locator_list()
                    .to_vec();
                let default_multicast_locator_list = discovered_participant_data
                    .participant_proxy()
                    .default_multicast_locator_list()
                    .to_vec();

                for publisher in self.user_defined_publisher_list.values() {
                    let publisher_address = publisher.address();

                    let participant_publication_matched_listener =
                        if self.status_kind.contains(&StatusKind::PublicationMatched) {
                            self.listener.as_ref().map(|l| l.address())
                        } else {
                            None
                        };
                    let offered_incompatible_qos_participant_listener = if self
                        .status_kind
                        .contains(&StatusKind::OfferedIncompatibleQos)
                    {
                        self.listener.as_ref().map(|l| l.address())
                    } else {
                        None
                    };
                    publisher
                        .send_mail_and_await_reply(dds_publisher::add_matched_reader::new(
                            discovered_reader_data.clone(),
                            default_unicast_locator_list.clone(),
                            default_multicast_locator_list.clone(),
                            publisher_address,
                            participant_address.clone(),
                            participant_publication_matched_listener,
                            offered_incompatible_qos_participant_listener,
                        ))
                        .await;
                }

                // Add reader topic to discovered topic list using the reader instance handle
                let topic_instance_handle = InstanceHandle::new(
                    discovered_reader_data
                        .subscription_builtin_topic_data()
                        .key()
                        .value,
                );
                let reader_topic = TopicBuiltinTopicData::new(
                    BuiltInTopicKey::default(),
                    discovered_reader_data
                        .subscription_builtin_topic_data()
                        .topic_name()
                        .to_string(),
                    discovered_reader_data
                        .subscription_builtin_topic_data()
                        .get_type_name()
                        .to_string(),
                    discovered_reader_data
                        .subscription_builtin_topic_data()
                        .durability()
                        .clone(),
                    discovered_reader_data
                        .subscription_builtin_topic_data()
                        .deadline()
                        .clone(),
                    discovered_reader_data
                        .subscription_builtin_topic_data()
                        .latency_budget()
                        .clone(),
                    discovered_reader_data
                        .subscription_builtin_topic_data()
                        .liveliness()
                        .clone(),
                    discovered_reader_data
                        .subscription_builtin_topic_data()
                        .reliability()
                        .clone(),
                    TransportPriorityQosPolicy::default(),
                    LifespanQosPolicy::default(),
                    discovered_reader_data
                        .subscription_builtin_topic_data()
                        .destination_order()
                        .clone(),
                    HistoryQosPolicy::default(),
                    ResourceLimitsQosPolicy::default(),
                    discovered_reader_data
                        .subscription_builtin_topic_data()
                        .ownership()
                        .clone(),
                    discovered_reader_data
                        .subscription_builtin_topic_data()
                        .topic_data()
                        .clone(),
                );
                self.discovered_topic_list
                    .insert(topic_instance_handle, reader_topic);
            }
        }
    }

    async fn remove_matched_reader(
        &self,
        discovered_reader_handle: InstanceHandle,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) {
        for publisher in self.user_defined_publisher_list.values() {
            let publisher_address = publisher.address();
            let participant_publication_matched_listener =
                if self.status_kind.contains(&StatusKind::PublicationMatched) {
                    self.listener.as_ref().map(|l| l.address())
                } else {
                    None
                };
            publisher
                .send_mail_and_await_reply(dds_publisher::remove_matched_reader::new(
                    discovered_reader_handle,
                    publisher_address,
                    participant_address.clone(),
                    participant_publication_matched_listener,
                ))
                .await;
        }
    }

    async fn process_sedp_topics_discovery(&mut self) {
        if let Some(sedp_topics_detector) = self
            .builtin_subscriber
            .send_mail_and_await_reply(dds_subscriber::lookup_datareader::new(
                DCPS_TOPIC.to_string(),
            ))
            .await
        {
            if let Ok(mut discovered_topic_sample_list) = sedp_topics_detector
                .send_mail_and_await_reply(dds_data_reader::read::new(
                    i32::MAX,
                    ANY_SAMPLE_STATE.to_vec(),
                    ANY_VIEW_STATE.to_vec(),
                    ANY_INSTANCE_STATE.to_vec(),
                    None,
                ))
                .await
                .expect("Can not fail to send mail to builtin reader")
            {
                for (discovered_topic_data, discovered_topic_sample_info) in
                    discovered_topic_sample_list.drain(..)
                {
                    match discovered_topic_sample_info.instance_state {
                        InstanceStateKind::Alive => {
                            match dds_deserialize_from_bytes::<DiscoveredTopicData>(
                                discovered_topic_data.expect("Should contain data").as_ref(),
                            ) {
                                Ok(discovered_topic_data) => {
                                    self.add_matched_topic(discovered_topic_data).await;
                                }
                                Err(e) => warn!(
                                    "Received invalid DiscoveredTopicData sample. Error {:?}",
                                    e
                                ),
                            }
                        }
                        InstanceStateKind::NotAliveDisposed => todo!(),
                        InstanceStateKind::NotAliveNoWriters => todo!(),
                    }
                }
            }
        }
    }

    async fn add_matched_topic(&mut self, discovered_topic_data: DiscoveredTopicData) {
        let handle =
            InstanceHandle::new(discovered_topic_data.topic_builtin_topic_data().key().value);
        let is_topic_ignored = self.ignored_topic_list.contains(&handle);
        if !is_topic_ignored {
            for topic in self.topic_list.values() {
                topic
                    .send_mail_and_await_reply(dds_topic::process_discovered_topic::new(
                        discovered_topic_data.clone(),
                    ))
                    .await;
            }
            self.discovered_topic_list.insert(
                handle,
                discovered_topic_data.topic_builtin_topic_data().clone(),
            );
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
