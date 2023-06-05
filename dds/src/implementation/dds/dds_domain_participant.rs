use fnmatch_regex::glob_to_regex;
use tokio::sync::mpsc::Sender;

use crate::{
    builtin_topics::{
        BuiltInTopicKey, ParticipantBuiltinTopicData, PublicationBuiltinTopicData,
        SubscriptionBuiltinTopicData,
    },
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
        dds_actor,
        rtps::{
            discovery_types::{BuiltinEndpointQos, BuiltinEndpointSet},
            endpoint::RtpsEndpoint,
            group::RtpsGroup,
            messages::{
                overall_structure::{RtpsMessageRead, RtpsMessageWrite},
                types::Count,
            },
            participant::RtpsParticipant,
            reader::RtpsReader,
            reader_locator::RtpsReaderLocator,
            reader_proxy::RtpsReaderProxy,
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
                DurabilityKind, EntityId, EntityKey, Guid, Locator, ProtocolVersion,
                ReliabilityKind, TopicKind, VendorId, BUILT_IN_READER_GROUP,
                BUILT_IN_READER_WITH_KEY, BUILT_IN_TOPIC, BUILT_IN_WRITER_GROUP,
                BUILT_IN_WRITER_WITH_KEY, ENTITYID_PARTICIPANT, USER_DEFINED_READER_GROUP,
                USER_DEFINED_TOPIC, USER_DEFINED_WRITER_GROUP,
            },
            writer::RtpsWriter,
        },
        utils::{
            actor::{spawn_actor, ActorAddress, Actor},
            condvar::DdsCondvar,
        },
    },
    infrastructure::{
        instance::InstanceHandle,
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            QosPolicyId, ReliabilityQosPolicy, ReliabilityQosPolicyKind, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID,
            LIVELINESS_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID, RELIABILITY_QOS_POLICY_ID,
        },
        time::{DurationKind, DURATION_ZERO},
    },
    subscription::sample_info::{
        InstanceStateKind, SampleStateKind, ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE,
    },
    topic_definition::type_support::{dds_serialize, DdsType},
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
    sync::atomic::{AtomicU8, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use super::{
    dds_data_writer::DdsDataWriter, dds_publisher::DdsPublisher, message_receiver::MessageReceiver,
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

#[derive(Debug)]
pub enum AnnounceKind {
    CreatedDataReader(DiscoveredReaderData),
    CreatedDataWriter(DiscoveredWriterData),
    CratedTopic(DiscoveredTopicData),
    DeletedDataReader(InstanceHandle),
    DeletedDataWriter(InstanceHandle),
    DeletedParticipant,
}

pub struct DdsDomainParticipant {
    rtps_participant: RtpsParticipant,
    domain_id: DomainId,
    domain_tag: String,
    qos: DomainParticipantQos,
    builtin_subscriber: DdsSubscriber,
    builtin_publisher: DdsPublisher,
    user_defined_subscriber_list: Vec<Actor<DdsSubscriber>>,
    user_defined_subscriber_counter: AtomicU8,
    default_subscriber_qos: SubscriberQos,
    user_defined_publisher_list: Vec<Actor<DdsPublisher>>,
    user_defined_publisher_counter: AtomicU8,
    default_publisher_qos: PublisherQos,
    topic_list: Vec<Actor<DdsTopic>>,
    user_defined_topic_counter: AtomicU8,
    default_topic_qos: TopicQos,
    manual_liveliness_count: Count,
    lease_duration: Duration,
    discovered_participant_list: HashMap<InstanceHandle, SpdpDiscoveredParticipantData>,
    discovered_topic_list: HashMap<InstanceHandle, TopicBuiltinTopicData>,
    enabled: bool,
    user_defined_data_send_condvar: DdsCondvar,
    ignored_participants: HashSet<InstanceHandle>,
    ignored_publications: HashSet<InstanceHandle>,
    ignored_subcriptions: HashSet<InstanceHandle>,
    data_max_size_serialized: usize,
    announce_sender: Sender<AnnounceKind>,
    sedp_condvar: DdsCondvar,
    user_defined_rtps_message_channel_sender: Sender<(RtpsMessageWrite, Vec<Locator>)>,
}

impl DdsDomainParticipant {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rtps_participant: RtpsParticipant,
        domain_id: DomainId,
        domain_tag: String,
        domain_participant_qos: DomainParticipantQos,
        spdp_discovery_locator_list: &[Locator],
        user_defined_data_send_condvar: DdsCondvar,
        data_max_size_serialized: usize,
        announce_sender: Sender<AnnounceKind>,
        sedp_condvar: DdsCondvar,
        builtin_rtps_message_channel_sender: Sender<(RtpsMessageWrite, Vec<Locator>)>,
        user_defined_rtps_message_channel_sender: Sender<(RtpsMessageWrite, Vec<Locator>)>,
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
            announce_sender.clone(),
        );

        let sedp_topics_entity_id = EntityId::new(EntityKey::new([0, 0, 1]), BUILT_IN_TOPIC);
        let sedp_topics_guid = Guid::new(guid_prefix, sedp_topics_entity_id);
        let _sedp_topic_topics = DdsTopic::new(
            sedp_topics_guid,
            TopicQos::default(),
            DiscoveredTopicData::type_name(),
            DCPS_TOPIC,
            announce_sender.clone(),
        );

        let sedp_publications_entity_id = EntityId::new(EntityKey::new([0, 0, 2]), BUILT_IN_TOPIC);
        let sedp_publications_guid = Guid::new(guid_prefix, sedp_publications_entity_id);
        let _sedp_topic_publications = DdsTopic::new(
            sedp_publications_guid,
            TopicQos::default(),
            DiscoveredWriterData::type_name(),
            DCPS_PUBLICATION,
            announce_sender.clone(),
        );

        let sedp_subscriptions_entity_id = EntityId::new(EntityKey::new([0, 0, 2]), BUILT_IN_TOPIC);
        let sedp_subscriptions_guid = Guid::new(guid_prefix, sedp_subscriptions_entity_id);
        let _sedp_topic_subscriptions = DdsTopic::new(
            sedp_subscriptions_guid,
            TopicQos::default(),
            DiscoveredReaderData::type_name(),
            DCPS_SUBSCRIPTION,
            announce_sender.clone(),
        );

        // Built-in subscriber creation
        let spdp_builtin_participant_reader = DdsDataReader::new(
            create_builtin_stateless_reader::<SpdpDiscoveredParticipantData>(Guid::new(
                guid_prefix,
                ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
            )),
            ParticipantBuiltinTopicData::type_name(),
            String::from(DCPS_PARTICIPANT),
        );

        let sedp_builtin_topics_reader = spawn_actor(DdsDataReader::new(
            create_builtin_stateful_reader::<DiscoveredTopicData>(Guid::new(
                guid_prefix,
                ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
            )),
            TopicBuiltinTopicData::type_name(),
            String::from(DCPS_TOPIC),
        ));

        let sedp_builtin_publications_reader = spawn_actor(DdsDataReader::new(
            create_builtin_stateful_reader::<DiscoveredWriterData>(Guid::new(
                guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            )),
            PublicationBuiltinTopicData::type_name(),
            String::from(DCPS_PUBLICATION),
        ));

        let sedp_builtin_subscriptions_reader = spawn_actor(DdsDataReader::new(
            create_builtin_stateful_reader::<DiscoveredReaderData>(Guid::new(
                guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
            )),
            SubscriptionBuiltinTopicData::type_name(),
            String::from(DCPS_SUBSCRIPTION),
        ));

        let mut builtin_subscriber = DdsSubscriber::new(
            SubscriberQos::default(),
            RtpsGroup::new(Guid::new(
                guid_prefix,
                EntityId::new(EntityKey::new([0, 0, 0]), BUILT_IN_READER_GROUP),
            )),
        );
        builtin_subscriber.stateless_data_reader_add(spdp_builtin_participant_reader);
        builtin_subscriber.stateful_data_reader_add(sedp_builtin_topics_reader);
        builtin_subscriber.stateful_data_reader_add(sedp_builtin_publications_reader);
        builtin_subscriber.stateful_data_reader_add(sedp_builtin_subscriptions_reader);

        // Built-in publisher creation
        let mut spdp_builtin_participant_writer = DdsDataWriter::new(
            create_builtin_stateless_writer(Guid::new(
                guid_prefix,
                ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
            )),
            SpdpDiscoveredParticipantData::type_name(),
            String::from(DCPS_PARTICIPANT),
            builtin_rtps_message_channel_sender.clone(),
        );

        for reader_locator in spdp_discovery_locator_list
            .iter()
            .map(|&locator| RtpsReaderLocator::new(locator, false))
        {
            spdp_builtin_participant_writer.reader_locator_add(reader_locator);
        }

        let sedp_builtin_topics_writer = DdsDataWriter::new(
            create_builtin_stateful_writer(Guid::new(
                guid_prefix,
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            )),
            DiscoveredTopicData::type_name(),
            String::from(DCPS_TOPIC),
            builtin_rtps_message_channel_sender.clone(),
        );
        let sedp_builtin_topics_writer_actor = spawn_actor(sedp_builtin_topics_writer);

        let sedp_builtin_publications_writer = DdsDataWriter::new(
            create_builtin_stateful_writer(Guid::new(
                guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            )),
            DiscoveredWriterData::type_name(),
            String::from(DCPS_PUBLICATION),
            builtin_rtps_message_channel_sender.clone(),
        );
        let sedp_builtin_publications_writer_actor = spawn_actor(sedp_builtin_publications_writer);

        let sedp_builtin_subscriptions_writer = DdsDataWriter::new(
            create_builtin_stateful_writer(Guid::new(
                guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            )),
            DiscoveredReaderData::type_name(),
            String::from(DCPS_SUBSCRIPTION),
            builtin_rtps_message_channel_sender,
        );
        let sedp_builtin_subscriptions_writer_actor =
            spawn_actor(sedp_builtin_subscriptions_writer);

        let mut builtin_publisher = DdsPublisher::new(
            PublisherQos::default(),
            RtpsGroup::new(Guid::new(
                guid_prefix,
                EntityId::new(EntityKey::new([0, 0, 0]), BUILT_IN_WRITER_GROUP),
            )),
        );
        builtin_publisher.stateless_datawriter_add(spdp_builtin_participant_writer);
        builtin_publisher.stateful_datawriter_add(sedp_builtin_topics_writer_actor);
        builtin_publisher.stateful_datawriter_add(sedp_builtin_publications_writer_actor);
        builtin_publisher.stateful_datawriter_add(sedp_builtin_subscriptions_writer_actor);

        Self {
            rtps_participant,
            domain_id,
            domain_tag,
            qos: domain_participant_qos,
            builtin_subscriber,
            builtin_publisher,
            user_defined_subscriber_list: Vec::new(),
            user_defined_subscriber_counter: AtomicU8::new(0),
            default_subscriber_qos: SubscriberQos::default(),
            user_defined_publisher_list: Vec::new(),
            user_defined_publisher_counter: AtomicU8::new(0),
            default_publisher_qos: PublisherQos::default(),
            topic_list: Vec::new(),
            user_defined_topic_counter: AtomicU8::new(0),
            default_topic_qos: TopicQos::default(),
            manual_liveliness_count: Count::new(0),
            lease_duration,
            discovered_participant_list: HashMap::new(),
            discovered_topic_list: HashMap::new(),
            enabled: false,
            user_defined_data_send_condvar,
            ignored_participants: HashSet::new(),
            ignored_publications: HashSet::new(),
            ignored_subcriptions: HashSet::new(),
            data_max_size_serialized,
            announce_sender,
            sedp_condvar,
            user_defined_rtps_message_channel_sender,
        }
    }

    pub fn guid(&self) -> Guid {
        self.rtps_participant.guid()
    }

    pub fn vendor_id(&self) -> VendorId {
        self.rtps_participant.vendor_id()
    }

    pub fn domain_id(&self) -> DomainId {
        self.domain_id
    }

    pub fn protocol_version(&self) -> ProtocolVersion {
        self.rtps_participant.protocol_version()
    }

    pub fn default_unicast_locator_list(&self) -> &[Locator] {
        self.rtps_participant.default_unicast_locator_list()
    }

    pub fn get_user_defined_rtps_message_channel_sender(
        &self,
    ) -> Sender<(RtpsMessageWrite, Vec<Locator>)> {
        self.user_defined_rtps_message_channel_sender.clone()
    }

    pub fn default_multicast_locator_list(&self) -> &[Locator] {
        self.rtps_participant.default_multicast_locator_list()
    }

    pub fn metatraffic_unicast_locator_list(&self) -> &[Locator] {
        self.rtps_participant.metatraffic_unicast_locator_list()
    }

    pub fn metatraffic_multicast_locator_list(&self) -> &[Locator] {
        self.rtps_participant.metatraffic_multicast_locator_list()
    }

    pub fn get_builtin_subscriber(&self) -> &DdsSubscriber {
        &self.builtin_subscriber
    }

    pub fn get_builtin_subscriber_mut(&mut self) -> &mut DdsSubscriber {
        &mut self.builtin_subscriber
    }

    pub fn get_builtin_publisher(&self) -> &DdsPublisher {
        &self.builtin_publisher
    }

    pub fn get_builtin_publisher_mut(&mut self) -> &mut DdsPublisher {
        &mut self.builtin_publisher
    }

    pub fn get_current_time(&self) -> Time {
        let now_system_time = SystemTime::now();
        let unix_time = now_system_time
            .duration_since(UNIX_EPOCH)
            .expect("Clock time is before Unix epoch start");
        Time::new(unix_time.as_secs() as i32, unix_time.subsec_nanos())
    }

    pub fn enable(&mut self) {
        if !self.enabled {
            self.enabled = true;

            self.builtin_subscriber.enable().ok();
            self.builtin_publisher.enable();

            for builtin_stateless_writer in self
                .builtin_publisher
                .stateless_data_writer_list_mut()
                .iter_mut()
            {
                builtin_stateless_writer.enable();
            }

            for builtin_stateful_writer in self
                .builtin_publisher
                .stateful_data_writer_list()
                .iter_mut()
            {
                builtin_stateful_writer
                    .send_blocking(dds_actor::data_writer::Enable)
                    .ok();
            }

            if self.qos.entity_factory.autoenable_created_entities {
                for publisher in self.user_defined_publisher_list.iter_mut() {
                    publisher
                        .address()
                        .send_blocking(dds_actor::publisher::Enable)
                        .ok();
                }

                for subscriber in self.user_defined_subscriber_list.iter_mut() {
                    subscriber
                        .address()
                        .send_blocking(dds_actor::subscriber::Enable)
                        .ok();
                }

                for topic in self.topic_list.iter_mut() {
                    topic.address().send_blocking(dds_actor::topic::Enable).ok();
                }
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_participant_ignored(&self, handle: InstanceHandle) -> bool {
        self.ignored_participants.contains(&handle)
    }

    pub fn is_subscription_ignored(&self, handle: InstanceHandle) -> bool {
        self.ignored_subcriptions.contains(&handle)
    }

    pub fn is_publication_ignored(&self, handle: InstanceHandle) -> bool {
        self.ignored_publications.contains(&handle)
    }

    pub fn get_domain_tag(&self) -> &str {
        &self.domain_tag
    }

    pub fn discovered_participant_add(
        &mut self,
        handle: InstanceHandle,
        discovered_participant_data: SpdpDiscoveredParticipantData,
    ) {
        self.discovered_participant_list
            .insert(handle, discovered_participant_data);
    }

    pub fn _discovered_participant_remove(&mut self, handle: InstanceHandle) {
        self.discovered_participant_list.remove(&handle);
    }

    pub fn discovered_participant_list(
        &self,
    ) -> std::collections::hash_map::Iter<InstanceHandle, SpdpDiscoveredParticipantData> {
        self.discovered_participant_list.iter()
    }

    pub fn create_publisher(
        &mut self,
        qos: QosKind<PublisherQos>,
    ) -> DdsResult<ActorAddress<DdsPublisher>> {
        let publisher_qos = match qos {
            QosKind::Default => self.default_publisher_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let publisher_counter = self
            .user_defined_publisher_counter
            .fetch_add(1, Ordering::Relaxed);
        let entity_id = EntityId::new(
            EntityKey::new([publisher_counter, 0, 0]),
            USER_DEFINED_WRITER_GROUP,
        );
        let guid = Guid::new(self.rtps_participant.guid().prefix(), entity_id);
        let rtps_group = RtpsGroup::new(guid);
        let mut publisher = DdsPublisher::new(publisher_qos, rtps_group);
        if self.is_enabled() && self.get_qos().entity_factory.autoenable_created_entities {
            publisher.enable();
        }

        let publisher_actor = spawn_actor(publisher);
        let publisher_address = publisher_actor.address();
        self.user_defined_publisher_list.push(publisher_actor);

        Ok(publisher_address)
    }

    pub fn delete_publisher(&mut self, publisher_guid: Guid) -> DdsResult<()> {
        todo!()
        // if self.rtps_participant.guid().prefix() != publisher_guid.prefix() {
        //     return Err(DdsError::PreconditionNotMet(
        //         "Publisher can only be deleted from its parent participant".to_string(),
        //     ));
        // }

        // if self
        //     .user_defined_publisher_list()
        //     .iter()
        //     .find(|x| x.guid() == publisher_guid)
        //     .ok_or(DdsError::AlreadyDeleted)?
        //     .stateful_data_writer_list()
        //     .iter()
        //     .count()
        //     > 0
        // {
        //     return Err(DdsError::PreconditionNotMet(
        //         "Publisher still contains data writers".to_string(),
        //     ));
        // }

        // self.user_defined_publisher_list
        //     .retain(|x| x.guid() != publisher_guid);

        // Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.user_defined_publisher_list.iter().count() == 0
            && self.user_defined_subscriber_list.iter().count() == 0
            && self.topic_list.iter().count() == 0
    }

    pub fn user_defined_publisher_list(&self) -> &[DdsPublisher] {
        todo!()
        // self.user_defined_publisher_list.as_slice()
    }

    pub fn user_defined_publisher_list_mut(&mut self) -> &mut [DdsPublisher] {
        todo!()
        // self.user_defined_publisher_list.as_mut_slice()
    }

    pub fn get_publisher(&self, publisher_guid: Guid) -> Option<&DdsPublisher> {
        todo!()
    }

    pub fn get_publisher_mut(&mut self, publisher_guid: Guid) -> Option<&mut DdsPublisher> {
        self.user_defined_publisher_list_mut()
            .iter_mut()
            .find(|p| p.guid() == publisher_guid)
    }

    pub fn create_subscriber(
        &mut self,
        qos: QosKind<SubscriberQos>,
    ) -> DdsResult<ActorAddress<DdsSubscriber>> {
        let subscriber_qos = match qos {
            QosKind::Default => self.default_subscriber_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let subcriber_counter = self
            .user_defined_subscriber_counter
            .fetch_add(1, Ordering::Relaxed);
        let entity_id = EntityId::new(
            EntityKey::new([subcriber_counter, 0, 0]),
            USER_DEFINED_READER_GROUP,
        );
        let guid = Guid::new(self.rtps_participant.guid().prefix(), entity_id);
        let rtps_group = RtpsGroup::new(guid);
        let mut subscriber = DdsSubscriber::new(subscriber_qos, rtps_group);
        if self.enabled && self.qos.entity_factory.autoenable_created_entities {
            subscriber.enable()?;
        }

        let subscriber_actor = spawn_actor(subscriber);
        let subscriber_address = subscriber_actor.address();
        self.user_defined_subscriber_list.push(subscriber_actor);

        Ok(subscriber_address)
    }

    pub fn delete_subscriber(&mut self, subscriber_guid: Guid) -> DdsResult<()> {
        todo!()
        // if self.rtps_participant.guid().prefix() != subscriber_guid.prefix() {
        //     return Err(DdsError::PreconditionNotMet(
        //         "Subscriber can only be deleted from its parent participant".to_string(),
        //     ));
        // }

        // if self
        //     .user_defined_subscriber_list()
        //     .iter()
        //     .find(|&x| x.guid() == subscriber_guid)
        //     .ok_or(DdsError::AlreadyDeleted)?
        //     .stateful_data_reader_list()
        //     .iter()
        //     .count()
        //     > 0
        // {
        //     return Err(DdsError::PreconditionNotMet(
        //         "Subscriber still contains data readers".to_string(),
        //     ));
        // }

        // self.user_defined_subscriber_list
        //     .retain(|x| x.guid() != subscriber_guid);

        // Ok(())
    }

    pub fn user_defined_subscriber_list(&self) -> &[DdsSubscriber] {
        todo!()
        // &self.user_defined_subscriber_list
    }

    pub fn user_defined_subscriber_list_mut(&mut self) -> &mut [DdsSubscriber] {
        todo!()
        // &mut self.user_defined_subscriber_list
    }

    pub fn get_subscriber(&self, subscriber_guid: Guid) -> Option<&DdsSubscriber> {
        todo!()
        // self.user_defined_subscriber_list
        //     .iter()
        //     .find(|s| s.guid() == subscriber_guid)
    }

    pub fn get_subscriber_mut(&mut self, subscriber_guid: Guid) -> Option<&mut DdsSubscriber> {
        todo!()
        // self.user_defined_subscriber_list
        //     .iter_mut()
        //     .find(|s| s.guid() == subscriber_guid)
    }

    pub fn create_topic(
        &mut self,
        topic_name: &str,
        type_name: &'static str,
        qos: QosKind<TopicQos>,
    ) -> DdsResult<ActorAddress<DdsTopic>> {
        let topic_counter = self
            .user_defined_topic_counter
            .fetch_add(1, Ordering::Relaxed);
        let topic_guid = Guid::new(
            self.rtps_participant.guid().prefix(),
            EntityId::new(EntityKey::new([topic_counter, 0, 0]), USER_DEFINED_TOPIC),
        );
        let qos = match qos {
            QosKind::Default => self.default_topic_qos.clone(),
            QosKind::Specific(q) => q,
        };

        // /////// Create topic
        let mut topic = DdsTopic::new(
            topic_guid,
            qos,
            type_name,
            topic_name,
            self.announce_sender.clone(),
        );

        if self.enabled && self.qos.entity_factory.autoenable_created_entities {
            topic.enable()?;
        }

        let topic_actor = spawn_actor(topic);
        let topic_address = topic_actor.address();
        self.topic_list.push(topic_actor);

        Ok(topic_address)
    }

    pub fn delete_topic(&mut self, topic_guid: Guid) -> DdsResult<()> {
        // if self.rtps_participant.guid().prefix() != topic_guid.prefix() {
        //     return Err(DdsError::PreconditionNotMet(
        //         "Topic can only be deleted from its parent participant".to_string(),
        //     ));
        // }

        // let topic = self
        //     .topic_list
        //     .iter()
        //     .find(|&topic| topic.guid() == topic_guid)
        //     .ok_or(DdsError::AlreadyDeleted)?;

        // for publisher in self.user_defined_publisher_list() {
        //     if publisher.stateful_data_writer_list().iter().any(|w| {
        //         w.send_blocking(dds_actor::data_writer::GetTypeName).unwrap() == topic.get_type_name()
        //             && w.send_blocking(dds_actor::data_writer::GetTopicName).unwrap() == topic.get_name()
        //     }) {
        //         return Err(DdsError::PreconditionNotMet(
        //             "Topic still attached to some data writer".to_string(),
        //         ));
        //     }
        // }

        // for subscriber in self.user_defined_subscriber_list() {
        //     if subscriber.stateful_data_reader_list().iter().any(|r| {
        //         r.get_type_name() == topic.get_type_name() && r.get_topic_name() == topic.get_name()
        //     }) {
        //         return Err(DdsError::PreconditionNotMet(
        //             "Topic still attached to some data reader".to_string(),
        //         ));
        //     }
        // }

        // self.topic_list.retain(|x| x.guid() != topic_guid);
        // Ok(())
        todo!()
    }

    pub fn topic_list(&self) -> &[DdsTopic] {
        todo!()
        // &self.topic_list
    }

    pub fn topic_list_mut(&mut self) -> &mut [DdsTopic] {
        todo!()
        // &mut self.topic_list
    }

    pub fn get_topic(&self, topic_name: &str, type_name: &str) -> Option<&DdsTopic> {
        self.topic_list()
            .iter()
            .find(|t| t.get_name() == topic_name && t.get_type_name() == type_name)
    }

    pub fn get_qos(&self) -> DomainParticipantQos {
        self.qos.clone()
    }

    pub fn data_max_size_serialized(&self) -> usize {
        self.data_max_size_serialized
    }

    pub fn find_topic(
        &mut self,
        topic_name: &str,
        type_name: &'static str,
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

    pub fn ignore_participant(&mut self, handle: InstanceHandle) {
        self.ignored_participants.insert(handle);
    }

    pub fn ignore_topic(&self, _handle: InstanceHandle) {
        // todo!()
    }

    pub fn ignore_publication(&mut self, handle: InstanceHandle) {
        self.ignored_publications.insert(handle);

        // for subscriber in self.user_defined_subscriber_list() {
        //     for data_reader in subscriber.stateful_data_reader_list() {
        //         data_reader.remove_matched_writer(
        //             handle,
        //             &mut subscriber.get_status_listener_lock(),
        //             &mut self.get_status_listener_lock(),
        //         )
        //     }
        // }
    }

    pub fn ignore_subscription(&mut self, handle: InstanceHandle) {
        self.ignored_subcriptions.insert(handle);
        // for publisher in self.user_defined_publisher_list() {
        //     for data_writer in publisher.stateful_data_writer_list() {
        //         remove_writer_matched_reader(
        //             data_writer,
        //             handle,
        //             &mut publisher.get_status_listener_lock(),
        //             &mut self.status_listener.write_lock(),
        //         )
        //     }
        // }
    }

    pub fn delete_contained_entities(&mut self) -> DdsResult<()> {
        for user_defined_publisher in self.user_defined_publisher_list.drain(..) {
            user_defined_publisher
                .address()
                .send_blocking(dds_actor::publisher::DeleteContainedEntities)?;
        }

        for user_defined_subscriber in self.user_defined_subscriber_list.drain(..) {
            user_defined_subscriber
                .address()
                .send_blocking(dds_actor::subscriber::DeleteContainedEntities)?;
        }

        self.topic_list.clear();

        Ok(())
    }

    pub fn assert_liveliness(&self) -> DdsResult<()> {
        todo!()
    }

    pub fn set_default_publisher_qos(&mut self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => self.default_publisher_qos = PublisherQos::default(),
            QosKind::Specific(q) => self.default_publisher_qos = q,
        }

        Ok(())
    }

    pub fn get_default_publisher_qos(&self) -> PublisherQos {
        self.default_publisher_qos.clone()
    }

    pub fn set_default_subscriber_qos(&mut self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => self.default_subscriber_qos = SubscriberQos::default(),
            QosKind::Specific(q) => self.default_subscriber_qos = q,
        }

        Ok(())
    }

    pub fn get_default_subscriber_qos(&self) -> SubscriberQos {
        self.default_subscriber_qos.clone()
    }

    pub fn set_default_topic_qos(&mut self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => self.default_topic_qos = TopicQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                self.default_topic_qos = q;
            }
        }

        Ok(())
    }

    pub fn get_default_topic_qos(&self) -> TopicQos {
        self.default_topic_qos.clone()
    }

    pub fn get_discovered_topics(&self) -> DdsResult<Vec<InstanceHandle>> {
        Ok(self.discovered_topic_list.keys().cloned().collect())
    }

    pub fn get_discovered_topic_data(
        &self,
        topic_handle: InstanceHandle,
    ) -> DdsResult<TopicBuiltinTopicData> {
        self.discovered_topic_list
            .get(&topic_handle)
            .cloned()
            .ok_or(DdsError::BadParameter)
    }

    pub fn contains_entity(&self, _a_handle: InstanceHandle) -> DdsResult<bool> {
        todo!()
    }

    pub fn set_qos(&mut self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        self.qos = match qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };
        self.announce_participant().ok();

        Ok(())
    }

    pub fn announce_participant(&mut self) -> DdsResult<()> {
        let spdp_discovered_participant_data = SpdpDiscoveredParticipantData::new(
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
        );
        let serialized_data =
            dds_serialize(&spdp_discovered_participant_data).map_err(|_err| DdsError::Error)?;

        let current_time = self.get_current_time();
        self.builtin_publisher
            .stateless_data_writer_list_mut()
            .iter_mut()
            .find(|x| x.get_type_name() == SpdpDiscoveredParticipantData::type_name())
            .unwrap()
            .write_w_timestamp(
                serialized_data,
                spdp_discovered_participant_data.get_serialized_key(),
                None,
                current_time,
            )
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

    pub fn receive_builtin_data(
        &mut self,
        source_locator: Locator,
        message: RtpsMessageRead,
        listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) -> DdsResult<()> {
        MessageReceiver::new(self.get_current_time()).process_message(
            self.rtps_participant.guid(),
            core::slice::from_mut(&mut self.builtin_publisher),
            core::slice::from_mut(&mut self.builtin_subscriber),
            source_locator,
            &message,
            listener_sender,
        )?;
        self.user_defined_data_send_condvar.notify_all();
        Ok(())
    }

    pub fn receive_user_defined_data(
        &mut self,
        source_locator: Locator,
        message: RtpsMessageRead,
        listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) -> DdsResult<()> {
        MessageReceiver::new(self.get_current_time()).process_message(
            self.rtps_participant.guid(),
            todo!(),
            // self.user_defined_publisher_list.as_mut_slice(),
            todo!(),
            // self.user_defined_subscriber_list.as_mut_slice(),
            source_locator,
            &message,
            listener_sender,
        )?;
        self.user_defined_data_send_condvar.notify_all();
        Ok(())
    }

    pub fn discover_matched_readers(
        &mut self,
        listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) -> DdsResult<()> {
        let samples = self
            .get_builtin_subscriber_mut()
            .stateful_data_reader_list_mut()
            .iter_mut()
            .find(|x| x.get_topic_name() == DCPS_SUBSCRIPTION)
            .unwrap()
            .read::<DiscoveredReaderData>(
                i32::MAX,
                ANY_SAMPLE_STATE,
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
                None,
            )?;

        for discovered_reader_data_sample in samples.into_iter() {
            match discovered_reader_data_sample.sample_info.instance_state {
                InstanceStateKind::Alive => {
                    if let Some(discovered_reader_data) = discovered_reader_data_sample.data {
                        if !self.is_subscription_ignored(
                            discovered_reader_data
                                .reader_proxy()
                                .remote_reader_guid()
                                .into(),
                        ) {
                            let remote_reader_guid_prefix = discovered_reader_data
                                .reader_proxy()
                                .remote_reader_guid()
                                .prefix();
                            let reader_parent_participant_guid =
                                Guid::new(remote_reader_guid_prefix, ENTITYID_PARTICIPANT);

                            let participant_guid = self.guid();
                            if let Some((
                                default_unicast_locator_list,
                                default_multicast_locator_list,
                            )) = self
                                .discovered_participant_list
                                .get(&reader_parent_participant_guid.into())
                                .map(|discovered_participant_data| {
                                    (
                                        discovered_participant_data
                                            .participant_proxy()
                                            .default_unicast_locator_list()
                                            .to_vec(),
                                        discovered_participant_data
                                            .participant_proxy()
                                            .default_multicast_locator_list()
                                            .to_vec(),
                                    )
                                })
                            {
                                for publisher in self.user_defined_publisher_list_mut() {
                                    let publisher_qos = publisher.get_qos();
                                    let publisher_guid = publisher.guid();
                                    let is_discovered_reader_regex_matched_to_publisher =
                                        if let Ok(d) = glob_to_regex(
                                            &discovered_reader_data
                                                .subscription_builtin_topic_data()
                                                .partition()
                                                .name,
                                        ) {
                                            d.is_match(&publisher.get_qos().partition.name)
                                        } else {
                                            false
                                        };

                                    let is_publisher_regex_matched_to_discovered_reader =
                                        if let Ok(d) =
                                            glob_to_regex(&publisher.get_qos().partition.name)
                                        {
                                            d.is_match(
                                                &discovered_reader_data
                                                    .subscription_builtin_topic_data()
                                                    .partition()
                                                    .name,
                                            )
                                        } else {
                                            false
                                        };

                                    let is_partition_string_matched = discovered_reader_data
                                        .subscription_builtin_topic_data()
                                        .partition()
                                        .name
                                        == publisher.get_qos().partition.name;

                                    if is_discovered_reader_regex_matched_to_publisher
                                        || is_publisher_regex_matched_to_discovered_reader
                                        || is_partition_string_matched
                                    {
                                        todo!()
                                        // for data_writer in publisher.stateful_data_writer_list_mut()
                                        // {
                                        //     add_matched_reader(
                                        //         data_writer,
                                        //         &discovered_reader_data,
                                        //         &default_unicast_locator_list,
                                        //         &default_multicast_locator_list,
                                        //         &publisher_qos,
                                        //         publisher_guid,
                                        //         participant_guid,
                                        //         listener_sender,
                                        //     )
                                        // }
                                    }
                                }
                            }
                        }
                    }
                }
                InstanceStateKind::NotAliveDisposed => {
                    let participant_guid = self.guid();
                    for publisher in self.user_defined_publisher_list_mut() {
                        let publisher_guid = publisher.guid();
                        todo!()
                        // for data_writer in publisher.stateful_data_writer_list_mut() {
                        //     remove_writer_matched_reader(
                        //         data_writer,
                        //         discovered_reader_data_sample.sample_info.instance_handle,
                        //         publisher_guid,
                        //         participant_guid,
                        //         listener_sender,
                        //     )
                        // }
                    }
                }

                InstanceStateKind::NotAliveNoWriters => todo!(),
            }
        }

        Ok(())
    }

    pub fn discover_matched_topics(
        &mut self,
        listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) -> DdsResult<()> {
        while let Ok(samples) = self
            .get_builtin_subscriber_mut()
            .stateful_data_reader_list_mut()
            .iter_mut()
            .find(|x| x.get_topic_name() == DCPS_TOPIC)
            .unwrap()
            .read::<DiscoveredTopicData>(
                1,
                &[SampleStateKind::NotRead],
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
                None,
            )
        {
            let guid = self.guid();
            for sample in samples {
                if let Some(topic_data) = sample.data.as_ref() {
                    for topic in self.topic_list_mut() {
                        topic.process_discovered_topic(topic_data, guid, listener_sender);
                    }

                    self.discovered_topic_list.insert(
                        topic_data.get_serialized_key().into(),
                        topic_data.topic_builtin_topic_data().clone(),
                    );
                }
            }
        }

        Ok(())
    }

    pub fn update_communication_status(
        &mut self,
        listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) -> DdsResult<()> {
        let now = self.get_current_time();
        let guid = self.guid();
        for subscriber in self.user_defined_subscriber_list.iter_mut() {
            todo!()
            // subscriber.update_communication_status(now, guid, listener_sender);
        }

        Ok(())
    }

    pub fn announce_sender(&self) -> &Sender<AnnounceKind> {
        &self.announce_sender
    }

    pub fn user_defined_data_send_condvar(&self) -> &DdsCondvar {
        &self.user_defined_data_send_condvar
    }

    pub fn sedp_condvar(&self) -> &DdsCondvar {
        &self.sedp_condvar
    }
}

#[allow(clippy::too_many_arguments)]
fn add_matched_reader(
    writer: &mut DdsDataWriter<RtpsStatefulWriter>,
    discovered_reader_data: &DiscoveredReaderData,
    default_unicast_locator_list: &[Locator],
    default_multicast_locator_list: &[Locator],
    publisher_qos: &PublisherQos,
    parent_publisher_guid: Guid,
    parent_participant_guid: Guid,
    listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
) {
    let is_matched_topic_name = discovered_reader_data
        .subscription_builtin_topic_data()
        .topic_name()
        == writer.get_topic_name();
    let is_matched_type_name = discovered_reader_data
        .subscription_builtin_topic_data()
        .get_type_name()
        == writer.get_type_name();

    if is_matched_topic_name && is_matched_type_name {
        let incompatible_qos_policy_list = get_discovered_reader_incompatible_qos_policy_list(
            &writer.get_qos(),
            discovered_reader_data.subscription_builtin_topic_data(),
            publisher_qos,
        );
        let instance_handle = discovered_reader_data.get_serialized_key().into();

        if incompatible_qos_policy_list.is_empty() {
            let unicast_locator_list = if discovered_reader_data
                .reader_proxy()
                .unicast_locator_list()
                .is_empty()
            {
                default_unicast_locator_list
            } else {
                discovered_reader_data.reader_proxy().unicast_locator_list()
            };

            let multicast_locator_list = if discovered_reader_data
                .reader_proxy()
                .multicast_locator_list()
                .is_empty()
            {
                default_multicast_locator_list
            } else {
                discovered_reader_data
                    .reader_proxy()
                    .multicast_locator_list()
            };

            let proxy_reliability = match discovered_reader_data
                .subscription_builtin_topic_data()
                .reliability()
                .kind
            {
                ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
                ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
            };

            let proxy_durability = match discovered_reader_data
                .subscription_builtin_topic_data()
                .durability()
                .kind
            {
                DurabilityQosPolicyKind::Volatile => DurabilityKind::Volatile,
                DurabilityQosPolicyKind::TransientLocal => DurabilityKind::TransientLocal,
            };

            let reader_proxy = RtpsReaderProxy::new(
                discovered_reader_data.reader_proxy().remote_reader_guid(),
                discovered_reader_data
                    .reader_proxy()
                    .remote_group_entity_id(),
                unicast_locator_list,
                multicast_locator_list,
                discovered_reader_data.reader_proxy().expects_inline_qos(),
                true,
                proxy_reliability,
                proxy_durability,
            );

            writer.matched_reader_add(reader_proxy);

            if !writer
                .get_matched_subscriptions()
                .contains(&instance_handle)
                || writer
                    .get_matched_subscription_data(instance_handle)
                    .as_ref()
                    != Some(discovered_reader_data.subscription_builtin_topic_data())
            {
                writer.add_matched_publication(
                    instance_handle,
                    discovered_reader_data
                        .subscription_builtin_topic_data()
                        .clone(),
                );
                on_writer_publication_matched(
                    writer,
                    parent_publisher_guid,
                    parent_participant_guid,
                    listener_sender,
                )
            }
        } else {
            writer_on_offered_incompatible_qos(
                writer,
                instance_handle,
                incompatible_qos_policy_list,
                parent_publisher_guid,
                parent_participant_guid,
                listener_sender,
            );
        }
    }
}

fn get_discovered_reader_incompatible_qos_policy_list(
    writer_qos: &DataWriterQos,
    discovered_reader_data: &SubscriptionBuiltinTopicData,
    publisher_qos: &PublisherQos,
) -> Vec<QosPolicyId> {
    let mut incompatible_qos_policy_list = Vec::new();
    if &writer_qos.durability < discovered_reader_data.durability() {
        incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
    }
    if publisher_qos.presentation.access_scope < discovered_reader_data.presentation().access_scope
        || publisher_qos.presentation.coherent_access
            != discovered_reader_data.presentation().coherent_access
        || publisher_qos.presentation.ordered_access
            != discovered_reader_data.presentation().ordered_access
    {
        incompatible_qos_policy_list.push(PRESENTATION_QOS_POLICY_ID);
    }
    if &writer_qos.deadline < discovered_reader_data.deadline() {
        incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
    }
    if &writer_qos.latency_budget < discovered_reader_data.latency_budget() {
        incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
    }
    if &writer_qos.liveliness < discovered_reader_data.liveliness() {
        incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
    }
    if writer_qos.reliability.kind < discovered_reader_data.reliability().kind {
        incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
    }
    if &writer_qos.destination_order < discovered_reader_data.destination_order() {
        incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
    }
    incompatible_qos_policy_list
}

fn on_writer_publication_matched(
    writer: &DdsDataWriter<RtpsStatefulWriter>,
    parent_publisher_guid: Guid,
    parent_participant_guid: Guid,
    listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
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

pub fn remove_writer_matched_reader(
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

        on_writer_publication_matched(
            writer,
            parent_publisher_guid,
            parent_participant_guid,
            listener_sender,
        )
    }
}

fn writer_on_offered_incompatible_qos(
    writer: &mut DdsDataWriter<RtpsStatefulWriter>,
    handle: InstanceHandle,
    incompatible_qos_policy_list: Vec<QosPolicyId>,
    parent_publisher_guid: Guid,
    parent_participant_guid: Guid,
    listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
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
