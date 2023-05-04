use fnmatch_regex::glob_to_regex;

use crate::{
    builtin_topics::{
        BuiltInTopicKey, ParticipantBuiltinTopicData, PublicationBuiltinTopicData,
        SubscriptionBuiltinTopicData,
    },
    domain::{
        domain_participant_factory::DomainId,
        domain_participant_listener::DomainParticipantListener,
    },
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, DCPS_SUBSCRIPTION},
            discovered_topic_data::{DiscoveredTopicData, DCPS_TOPIC},
            discovered_writer_data::{DiscoveredWriterData, DCPS_PUBLICATION},
            spdp_discovered_participant_data::{
                ParticipantProxy, SpdpDiscoveredParticipantData, DCPS_PARTICIPANT,
            },
        },
        dds_impl::{
            dds_data_reader::DdsDataReader,
            dds_domain_participant_factory::THE_DDS_DOMAIN_PARTICIPANT_FACTORY,
            dds_subscriber::DdsSubscriber, dds_topic::DdsTopic,
        },
        rtps::{
            discovery_types::{BuiltinEndpointQos, BuiltinEndpointSet},
            endpoint::RtpsEndpoint,
            group::RtpsGroup,
            messages::RtpsMessage,
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
                Count, DurabilityKind, EntityId, EntityKey, Guid, Locator, ProtocolVersion,
                ReliabilityKind, TopicKind, VendorId, BUILT_IN_READER_GROUP,
                BUILT_IN_READER_WITH_KEY, BUILT_IN_TOPIC, BUILT_IN_WRITER_GROUP,
                BUILT_IN_WRITER_WITH_KEY, ENTITYID_PARTICIPANT, USER_DEFINED_READER_GROUP,
                USER_DEFINED_TOPIC, USER_DEFINED_WRITER_GROUP,
            },
            writer::RtpsWriter,
        },
        utils::{
            condvar::DdsCondvar,
            iterator::{DdsListIntoIterator, DdsMapIntoIterator},
            shared_object::{DdsRwLock, DdsShared},
            timer_factory::{Timer, TimerFactory},
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
        status::{StatusKind, NO_STATUS},
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
    topic_definition::type_support::{DdsDeserialize, DdsSerialize, DdsType},
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
    sync::{
        atomic::{AtomicU8, Ordering},
        mpsc::SyncSender,
        RwLockWriteGuard,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use super::{
    any_topic_listener::AnyTopicListener, dds_data_writer::DdsDataWriter,
    dds_publisher::DdsPublisher, message_receiver::MessageReceiver,
    node_listener_data_writer::ListenerDataWriterNode, status_condition_impl::StatusConditionImpl,
    status_listener::StatusListener,
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
    qos: DdsRwLock<DomainParticipantQos>,
    builtin_subscriber: DdsShared<DdsSubscriber>,
    builtin_publisher: DdsPublisher,
    user_defined_subscriber_list: DdsRwLock<Vec<DdsShared<DdsSubscriber>>>,
    user_defined_subscriber_counter: AtomicU8,
    default_subscriber_qos: DdsRwLock<SubscriberQos>,
    user_defined_publisher_list: Vec<DdsPublisher>,
    user_defined_publisher_counter: AtomicU8,
    default_publisher_qos: DdsRwLock<PublisherQos>,
    topic_list: Vec<DdsShared<DdsTopic>>,
    user_defined_topic_counter: AtomicU8,
    default_topic_qos: DdsRwLock<TopicQos>,
    manual_liveliness_count: Count,
    lease_duration: Duration,
    discovered_participant_list: DdsRwLock<HashMap<InstanceHandle, SpdpDiscoveredParticipantData>>,
    discovered_topic_list: HashMap<InstanceHandle, TopicBuiltinTopicData>,
    enabled: DdsRwLock<bool>,
    status_listener: DdsRwLock<StatusListener<dyn DomainParticipantListener + Send + Sync>>,
    user_defined_data_send_condvar: DdsCondvar,
    topic_find_condvar: DdsCondvar,
    ignored_participants: DdsRwLock<HashSet<InstanceHandle>>,
    ignored_publications: DdsRwLock<HashSet<InstanceHandle>>,
    ignored_subcriptions: DdsRwLock<HashSet<InstanceHandle>>,
    data_max_size_serialized: usize,
    _timer_factory: TimerFactory,
    timer: DdsShared<DdsRwLock<Timer>>,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
    announce_sender: SyncSender<AnnounceKind>,
}

impl DdsDomainParticipant {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rtps_participant: RtpsParticipant,
        domain_id: DomainId,
        domain_tag: String,
        domain_participant_qos: DomainParticipantQos,
        listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
        mask: &[StatusKind],
        spdp_discovery_locator_list: &[Locator],
        user_defined_data_send_condvar: DdsCondvar,
        data_max_size_serialized: usize,
        announce_sender: SyncSender<AnnounceKind>,
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
            None,
            &[],
            announce_sender.clone(),
        );

        let sedp_topics_entity_id = EntityId::new(EntityKey::new([0, 0, 1]), BUILT_IN_TOPIC);
        let sedp_topics_guid = Guid::new(guid_prefix, sedp_topics_entity_id);
        let _sedp_topic_topics = DdsTopic::new(
            sedp_topics_guid,
            TopicQos::default(),
            DiscoveredTopicData::type_name(),
            DCPS_TOPIC,
            None,
            &[],
            announce_sender.clone(),
        );

        let sedp_publications_entity_id = EntityId::new(EntityKey::new([0, 0, 2]), BUILT_IN_TOPIC);
        let sedp_publications_guid = Guid::new(guid_prefix, sedp_publications_entity_id);
        let _sedp_topic_publications = DdsTopic::new(
            sedp_publications_guid,
            TopicQos::default(),
            DiscoveredWriterData::type_name(),
            DCPS_PUBLICATION,
            None,
            &[],
            announce_sender.clone(),
        );

        let sedp_subscriptions_entity_id = EntityId::new(EntityKey::new([0, 0, 2]), BUILT_IN_TOPIC);
        let sedp_subscriptions_guid = Guid::new(guid_prefix, sedp_subscriptions_entity_id);
        let _sedp_topic_subscriptions = DdsTopic::new(
            sedp_subscriptions_guid,
            TopicQos::default(),
            DiscoveredReaderData::type_name(),
            DCPS_SUBSCRIPTION,
            None,
            &[],
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
            None,
            NO_STATUS,
        );

        let sedp_builtin_topics_reader = DdsDataReader::new(
            create_builtin_stateful_reader::<DiscoveredTopicData>(Guid::new(
                guid_prefix,
                ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
            )),
            TopicBuiltinTopicData::type_name(),
            String::from(DCPS_TOPIC),
            None,
            NO_STATUS,
        );

        let sedp_builtin_publications_reader = DdsDataReader::new(
            create_builtin_stateful_reader::<DiscoveredWriterData>(Guid::new(
                guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            )),
            PublicationBuiltinTopicData::type_name(),
            String::from(DCPS_PUBLICATION),
            None,
            NO_STATUS,
        );

        let sedp_builtin_subscriptions_reader = DdsDataReader::new(
            create_builtin_stateful_reader::<DiscoveredReaderData>(Guid::new(
                guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
            )),
            SubscriptionBuiltinTopicData::type_name(),
            String::from(DCPS_SUBSCRIPTION),
            None,
            NO_STATUS,
        );

        let builtin_subscriber = DdsSubscriber::new(
            SubscriberQos::default(),
            RtpsGroup::new(Guid::new(
                guid_prefix,
                EntityId::new(EntityKey::new([0, 0, 0]), BUILT_IN_READER_GROUP),
            )),
            None,
            NO_STATUS,
        );
        builtin_subscriber.stateless_data_reader_add(spdp_builtin_participant_reader);
        builtin_subscriber.stateful_data_reader_add(sedp_builtin_topics_reader);
        builtin_subscriber.stateful_data_reader_add(sedp_builtin_publications_reader);
        builtin_subscriber.stateful_data_reader_add(sedp_builtin_subscriptions_reader);

        // Built-in publisher creation
        let spdp_builtin_participant_writer = DdsDataWriter::new(
            create_builtin_stateless_writer(Guid::new(
                guid_prefix,
                ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
            )),
            None,
            NO_STATUS,
            SpdpDiscoveredParticipantData::type_name(),
            String::from(DCPS_PARTICIPANT),
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
            None,
            NO_STATUS,
            DiscoveredTopicData::type_name(),
            String::from(DCPS_TOPIC),
        );

        let sedp_builtin_publications_writer = DdsDataWriter::new(
            create_builtin_stateful_writer(Guid::new(
                guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            )),
            None,
            NO_STATUS,
            DiscoveredWriterData::type_name(),
            String::from(DCPS_PUBLICATION),
        );

        let sedp_builtin_subscriptions_writer = DdsDataWriter::new(
            create_builtin_stateful_writer(Guid::new(
                guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            )),
            None,
            NO_STATUS,
            DiscoveredReaderData::type_name(),
            String::from(DCPS_SUBSCRIPTION),
        );

        let mut builtin_publisher = DdsPublisher::new(
            PublisherQos::default(),
            RtpsGroup::new(Guid::new(
                guid_prefix,
                EntityId::new(EntityKey::new([0, 0, 0]), BUILT_IN_WRITER_GROUP),
            )),
            None,
            NO_STATUS,
        );
        builtin_publisher.stateless_datawriter_add(spdp_builtin_participant_writer);
        builtin_publisher.stateful_datawriter_add(sedp_builtin_topics_writer);
        builtin_publisher.stateful_datawriter_add(sedp_builtin_publications_writer);
        builtin_publisher.stateful_datawriter_add(sedp_builtin_subscriptions_writer);

        let timer_factory = TimerFactory::new();
        let timer = timer_factory.create_timer();

        Self {
            rtps_participant,
            domain_id,
            domain_tag,
            qos: DdsRwLock::new(domain_participant_qos),
            builtin_subscriber,
            builtin_publisher,
            user_defined_subscriber_list: DdsRwLock::new(Vec::new()),
            user_defined_subscriber_counter: AtomicU8::new(0),
            default_subscriber_qos: DdsRwLock::new(SubscriberQos::default()),
            user_defined_publisher_list: Vec::new(),
            user_defined_publisher_counter: AtomicU8::new(0),
            default_publisher_qos: DdsRwLock::new(PublisherQos::default()),
            topic_list: Vec::new(),
            user_defined_topic_counter: AtomicU8::new(0),
            default_topic_qos: DdsRwLock::new(TopicQos::default()),
            manual_liveliness_count: Count::new(0),
            lease_duration,
            discovered_participant_list: DdsRwLock::new(HashMap::new()),
            discovered_topic_list: HashMap::new(),
            enabled: DdsRwLock::new(false),
            user_defined_data_send_condvar,
            status_listener: DdsRwLock::new(StatusListener::new(listener, mask)),
            topic_find_condvar: DdsCondvar::new(),
            ignored_participants: DdsRwLock::new(HashSet::new()),
            ignored_publications: DdsRwLock::new(HashSet::new()),
            ignored_subcriptions: DdsRwLock::new(HashSet::new()),
            data_max_size_serialized,
            _timer_factory: timer_factory,
            timer,
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
            announce_sender,
        }
    }

    pub fn guid(&self) -> Guid {
        self.rtps_participant.guid()
    }

    pub fn vendor_id(&self) -> VendorId {
        self.rtps_participant.vendor_id()
    }

    pub fn protocol_version(&self) -> ProtocolVersion {
        self.rtps_participant.protocol_version()
    }

    pub fn default_unicast_locator_list(&self) -> &[Locator] {
        self.rtps_participant.default_unicast_locator_list()
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

    pub fn get_builtin_subscriber(&self) -> DdsShared<DdsSubscriber> {
        self.builtin_subscriber.clone()
    }

    pub fn get_builtin_publisher(&self) -> &DdsPublisher {
        &self.builtin_publisher
    }

    pub fn get_current_time(&self) -> Time {
        let now_system_time = SystemTime::now();
        let unix_time = now_system_time
            .duration_since(UNIX_EPOCH)
            .expect("Clock time is before Unix epoch start");
        Time::new(unix_time.as_secs() as i32, unix_time.subsec_nanos())
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read_lock()
    }

    pub fn is_participant_ignored(&self, handle: InstanceHandle) -> bool {
        self.ignored_participants.read_lock().contains(&handle)
    }

    pub fn is_subscription_ignored(&self, handle: InstanceHandle) -> bool {
        self.ignored_subcriptions.read_lock().contains(&handle)
    }

    pub fn is_publication_ignored(&self, handle: InstanceHandle) -> bool {
        self.ignored_publications.read_lock().contains(&handle)
    }

    pub fn get_domain_id(&self) -> DomainId {
        self.domain_id
    }

    pub fn get_domain_tag(&self) -> &str {
        &self.domain_tag
    }

    pub fn get_status_listener_lock(
        &self,
    ) -> RwLockWriteGuard<StatusListener<dyn DomainParticipantListener + Send + Sync>> {
        self.status_listener.write_lock()
    }

    pub fn discovered_participant_add(
        &self,
        handle: InstanceHandle,
        discovered_participant_data: SpdpDiscoveredParticipantData,
    ) {
        self.discovered_participant_list
            .write_lock()
            .insert(handle, discovered_participant_data);
    }

    pub fn _discovered_participant_remove(&self, handle: InstanceHandle) {
        self.discovered_participant_list
            .write_lock()
            .remove(&handle);
    }

    pub fn discovered_participant_list(
        &self,
    ) -> DdsMapIntoIterator<InstanceHandle, SpdpDiscoveredParticipantData> {
        DdsMapIntoIterator::new(self.discovered_participant_list.read_lock())
    }

    pub fn create_publisher(
        &mut self,
        qos: QosKind<PublisherQos>,
        a_listener: Option<Box<dyn PublisherListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<Guid> {
        let publisher_qos = match qos {
            QosKind::Default => self.default_publisher_qos.read_lock().clone(),
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
        let publisher_impl_shared = DdsPublisher::new(publisher_qos, rtps_group, a_listener, mask);
        if self.is_enabled() && self.get_qos().entity_factory.autoenable_created_entities {
            publisher_impl_shared.enable();
        }

        self.user_defined_publisher_list.push(publisher_impl_shared);

        Ok(guid)
    }

    pub fn delete_publisher(&mut self, a_publisher_handle: InstanceHandle) -> DdsResult<()> {
        if self
            .user_defined_publisher_list()
            .into_iter()
            .find(|x| InstanceHandle::from(x.guid()) == a_publisher_handle)
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(
                    "Publisher can only be deleted from its parent participant".to_string(),
                )
            })?
            .stateful_data_writer_list()
            .into_iter()
            .count()
            > 0
        {
            return Err(DdsError::PreconditionNotMet(
                "Publisher still contains data writers".to_string(),
            ));
        }

        self.user_defined_publisher_list
            .retain(|x| InstanceHandle::from(x.guid()) != a_publisher_handle);

        Ok(())
    }

    pub fn user_defined_publisher_list(&self) -> &[DdsPublisher] {
        self.user_defined_publisher_list.as_slice()
    }

    pub fn user_defined_publisher_list_mut(&mut self) -> &mut [DdsPublisher] {
        self.user_defined_publisher_list.as_mut_slice()
    }

    pub fn create_subscriber(
        &self,
        qos: QosKind<SubscriberQos>,
        a_listener: Option<Box<dyn SubscriberListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<DdsShared<DdsSubscriber>> {
        let subscriber_qos = match qos {
            QosKind::Default => self.default_subscriber_qos.read_lock().clone(),
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
        let subscriber_shared = DdsSubscriber::new(subscriber_qos, rtps_group, a_listener, mask);
        if *self.enabled.read_lock()
            && self
                .qos
                .read_lock()
                .entity_factory
                .autoenable_created_entities
        {
            subscriber_shared.enable()?;
        }

        self.user_defined_subscriber_list
            .write_lock()
            .push(subscriber_shared.clone());

        Ok(subscriber_shared)
    }

    pub fn delete_subscriber(&self, a_subscriber_handle: InstanceHandle) -> DdsResult<()> {
        if self
            .user_defined_subscriber_list()
            .into_iter()
            .find(|&x| x.get_instance_handle() == a_subscriber_handle)
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(
                    "Subscriber can only be deleted from its parent participant".to_string(),
                )
            })?
            .stateful_data_reader_list()
            .into_iter()
            .count()
            > 0
        {
            return Err(DdsError::PreconditionNotMet(
                "Subscriber still contains data readers".to_string(),
            ));
        }

        self.user_defined_subscriber_list
            .write_lock()
            .retain(|x| x.get_instance_handle() != a_subscriber_handle);

        Ok(())
    }

    pub fn user_defined_subscriber_list(&self) -> DdsListIntoIterator<DdsShared<DdsSubscriber>> {
        DdsListIntoIterator::new(self.user_defined_subscriber_list.read_lock())
    }

    pub fn create_topic(
        &mut self,
        topic_name: &str,
        type_name: &'static str,
        qos: QosKind<TopicQos>,
        a_listener: Option<Box<dyn AnyTopicListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<DdsShared<DdsTopic>> {
        let topic_counter = self
            .user_defined_topic_counter
            .fetch_add(1, Ordering::Relaxed);
        let topic_guid = Guid::new(
            self.rtps_participant.guid().prefix(),
            EntityId::new(EntityKey::new([topic_counter, 0, 0]), USER_DEFINED_TOPIC),
        );
        let qos = match qos {
            QosKind::Default => self.default_topic_qos.read_lock().clone(),
            QosKind::Specific(q) => q,
        };

        // /////// Create topic
        let topic_shared = DdsTopic::new(
            topic_guid,
            qos,
            type_name,
            topic_name,
            a_listener,
            mask,
            self.announce_sender.clone(),
        );
        if *self.enabled.read_lock()
            && self
                .qos
                .read_lock()
                .entity_factory
                .autoenable_created_entities
        {
            topic_shared.enable()?;
        }

        self.topic_list.push(topic_shared.clone());
        self.topic_find_condvar.notify_all();

        Ok(topic_shared)
    }

    pub fn delete_topic(&mut self, a_topic_handle: InstanceHandle) -> DdsResult<()> {
        let topic = self
            .topic_list
            .iter()
            .find(|&topic| topic.get_instance_handle() == a_topic_handle)
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(
                    "Topic can only be deleted from its parent publisher".to_string(),
                )
            })?
            .clone();

        for publisher in self.user_defined_publisher_list() {
            if publisher.stateful_data_writer_list().into_iter().any(|w| {
                w.get_type_name() == topic.get_type_name() && w.get_topic_name() == topic.get_name()
            }) {
                return Err(DdsError::PreconditionNotMet(
                    "Topic still attached to some data writer".to_string(),
                ));
            }
        }

        for subscriber in &self.user_defined_subscriber_list() {
            if subscriber.stateful_data_reader_list().into_iter().any(|r| {
                r.get_type_name() == topic.get_type_name() && r.get_topic_name() == topic.get_name()
            }) {
                return Err(DdsError::PreconditionNotMet(
                    "Topic still attached to some data reader".to_string(),
                ));
            }
        }

        self.topic_list
            .retain(|x| x.get_instance_handle() != a_topic_handle);
        Ok(())
    }

    pub fn topic_list(&self) -> &[DdsShared<DdsTopic>] {
        &self.topic_list
    }

    pub fn get_qos(&self) -> DomainParticipantQos {
        self.qos.read_lock().clone()
    }

    pub fn data_max_size_serialized(&self) -> usize {
        self.data_max_size_serialized
    }

    pub fn find_topic(
        &mut self,
        topic_name: &str,
        type_name: &'static str,
        timeout: Duration,
    ) -> DdsResult<DdsShared<DdsTopic>> {
        let start_time = self.get_current_time();

        while self.get_current_time() - start_time < timeout {
            // Check if a topic exists locally. If topic doesn't exist locally check if it has already been
            // discovered and, if so, create a new local topic representing the discovered topic
            if let Some(topic) = self
                .topic_list
                .iter()
                .find(|topic| topic.get_name() == topic_name && topic.get_type_name() == type_name)
            {
                return Ok(topic.clone());
            }

            // NOTE: Do not make this an else with the previous if because the topic_list read_lock is
            // kept and this enters a deadlock
            if let Some(discovered_topic_info) = self
                .discovered_topic_list
                .values()
                .find(|t| t.name() == topic_name && t.get_type_name() == type_name)
                .cloned()
            {
                let qos = TopicQos {
                    topic_data: discovered_topic_info.topic_data().clone(),
                    durability: discovered_topic_info.durability().clone(),
                    deadline: discovered_topic_info.deadline().clone(),
                    latency_budget: discovered_topic_info.latency_budget().clone(),
                    liveliness: discovered_topic_info.liveliness().clone(),
                    reliability: discovered_topic_info.reliability().clone(),
                    destination_order: discovered_topic_info.destination_order().clone(),
                    history: discovered_topic_info.history().clone(),
                    resource_limits: discovered_topic_info.resource_limits().clone(),
                    transport_priority: discovered_topic_info.transport_priority().clone(),
                    lifespan: discovered_topic_info.lifespan().clone(),
                    ownership: discovered_topic_info.ownership().clone(),
                };
                return self.create_topic(
                    discovered_topic_info.name(),
                    type_name,
                    QosKind::Specific(qos),
                    None,
                    NO_STATUS,
                );
            }
            // Block until timeout unless new topic is found or created
            let duration_until_timeout = (self.get_current_time() - start_time) - timeout;
            self.topic_find_condvar
                .wait_timeout(duration_until_timeout)
                .ok();
        }
        Err(DdsError::Timeout)
    }

    pub fn ignore_participant(&self, handle: InstanceHandle) {
        self.ignored_participants.write_lock().insert(handle);
    }

    pub fn ignore_topic(&self, _handle: InstanceHandle) {
        todo!()
    }

    pub fn ignore_publication(&self, handle: InstanceHandle) {
        self.ignored_publications.write_lock().insert(handle);

        for subscriber in &self.user_defined_subscriber_list() {
            for data_reader in &subscriber.stateful_data_reader_list() {
                data_reader.remove_matched_writer(
                    handle,
                    &mut subscriber.get_status_listener_lock(),
                    &mut self.get_status_listener_lock(),
                )
            }
        }
    }

    pub fn ignore_subscription(&self, handle: InstanceHandle) {
        self.ignored_subcriptions.write_lock().insert(handle);
        for publisher in self.user_defined_publisher_list() {
            for data_writer in publisher.stateful_data_writer_list() {
                remove_writer_matched_reader(
                    data_writer,
                    handle,
                    &mut publisher.get_status_listener_lock(),
                    &mut self.status_listener.write_lock(),
                )
            }
        }
    }

    pub fn delete_contained_entities(&mut self) -> DdsResult<()> {
        for mut user_defined_publisher in self.user_defined_publisher_list.drain(..) {
            for data_writer in user_defined_publisher
                .stateful_datawriter_drain()
                .into_iter()
            {
                if data_writer.is_enabled() {
                    self.announce_sender
                        .send(AnnounceKind::DeletedDataWriter(data_writer.guid().into()))
                        .ok();
                }
            }
        }

        for user_defined_subscriber in self.user_defined_subscriber_list.write_lock().drain(..) {
            for data_reader in user_defined_subscriber
                .stateful_data_reader_drain()
                .into_iter()
            {
                if data_reader.is_enabled() {
                    self.announce_sender
                        .send(AnnounceKind::DeletedDataReader(
                            data_reader.get_instance_handle(),
                        ))
                        .ok();
                }
            }
        }

        self.topic_list.clear();

        Ok(())
    }

    pub fn assert_liveliness(&self) -> DdsResult<()> {
        todo!()
    }

    pub fn set_default_publisher_qos(&self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => *self.default_publisher_qos.write_lock() = PublisherQos::default(),
            QosKind::Specific(q) => *self.default_publisher_qos.write_lock() = q,
        }

        Ok(())
    }

    pub fn get_default_publisher_qos(&self) -> PublisherQos {
        self.default_publisher_qos.read_lock().clone()
    }

    pub fn set_default_subscriber_qos(&self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => {
                *self.default_subscriber_qos.write_lock() = SubscriberQos::default()
            }
            QosKind::Specific(q) => *self.default_subscriber_qos.write_lock() = q,
        }

        Ok(())
    }

    pub fn get_default_subscriber_qos(&self) -> SubscriberQos {
        self.default_subscriber_qos.read_lock().clone()
    }

    pub fn set_default_topic_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => *self.default_topic_qos.write_lock() = TopicQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                *self.default_topic_qos.write_lock() = q;
            }
        }

        Ok(())
    }

    pub fn get_default_topic_qos(&self) -> TopicQos {
        self.default_topic_qos.read_lock().clone()
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

    pub fn set_qos(&self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        *self.qos.write_lock() = match qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };
        self.announce_participant().ok();

        Ok(())
    }

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }

    pub fn get_status_changes(&self) -> Vec<StatusKind> {
        self.status_condition.read_lock().get_status_changes()
    }

    pub fn enable(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            *self.enabled.write_lock() = true;

            self.builtin_subscriber.enable()?;
            self.builtin_publisher.enable();

            for builtin_stateless_writer in self
                .builtin_publisher
                .stateless_data_writer_list()
                .into_iter()
            {
                builtin_stateless_writer.enable();
            }

            for builtin_stateful_writer in self
                .builtin_publisher
                .stateful_data_writer_list()
                .into_iter()
            {
                builtin_stateful_writer.enable()
            }

            if self
                .qos
                .read_lock()
                .entity_factory
                .autoenable_created_entities
            {
                for publisher in self.user_defined_publisher_list() {
                    publisher.enable();
                }

                for subscriber in self.user_defined_subscriber_list.read_lock().iter() {
                    subscriber.enable()?;
                }

                for topic in self.topic_list.iter() {
                    topic.enable()?;
                }
            }

            self.announce_participant().ok();
            let participant_guid = self.guid();
            self.timer.write_lock().start_timer(
                DurationKind::Finite(Duration::new(5, 0)),
                InstanceHandle::new([0; 16]),
                move || {
                    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant(
                        &participant_guid.prefix(),
                        |dp| {
                            if let Some(dp) = dp {
                                dp.announce_participant().ok();
                            }
                        },
                    );
                },
            );
        }
        Ok(())
    }

    fn announce_participant(&self) -> DdsResult<()> {
        let spdp_discovered_participant_data = SpdpDiscoveredParticipantData::new(
            ParticipantBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: self.rtps_participant.guid().into(),
                },
                self.qos.read_lock().user_data.clone(),
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
        let mut serialized_data = Vec::new();
        spdp_discovered_participant_data.dds_serialize(&mut serialized_data)?;

        self.builtin_publisher
            .stateless_data_writer_list()
            .into_iter()
            .find(|x| x.get_type_name() == SpdpDiscoveredParticipantData::type_name())
            .unwrap()
            .write_w_timestamp(
                serialized_data,
                spdp_discovered_participant_data.get_serialized_key(),
                None,
                self.get_current_time(),
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

    pub fn receive_user_defined_data(
        &self,
        source_locator: Locator,
        message: RtpsMessage,
    ) -> DdsResult<()> {
        MessageReceiver::new(self.get_current_time()).process_message(
            self.rtps_participant.guid().prefix(),
            self.user_defined_publisher_list.as_slice(),
            self.user_defined_subscriber_list.read_lock().as_slice(),
            source_locator,
            &message,
            &mut self.status_listener.write_lock(),
        )?;
        self.user_defined_data_send_condvar.notify_all();
        Ok(())
    }

    pub fn discover_matched_readers(&self) -> DdsResult<()> {
        let samples = self
            .get_builtin_subscriber()
            .stateful_data_reader_list()
            .into_iter()
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

                            if let Some(discovered_participant_data) = self
                                .discovered_participant_list
                                .read_lock()
                                .get(&reader_parent_participant_guid.into())
                            {
                                for publisher in self.user_defined_publisher_list() {
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
                                        for data_writer in publisher.stateful_data_writer_list() {
                                            add_matched_reader(
                                                data_writer,
                                                &discovered_reader_data,
                                                discovered_participant_data
                                                    .participant_proxy()
                                                    .default_unicast_locator_list(),
                                                discovered_participant_data
                                                    .participant_proxy()
                                                    .default_multicast_locator_list(),
                                                &mut publisher.get_status_listener_lock(),
                                                &mut self.get_status_listener_lock(),
                                                &publisher.get_qos(),
                                            )
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                InstanceStateKind::NotAliveDisposed => {
                    for publisher in self.user_defined_publisher_list() {
                        for data_writer in publisher.stateful_data_writer_list() {
                            remove_writer_matched_reader(
                                data_writer,
                                discovered_reader_data_sample.sample_info.instance_handle,
                                &mut publisher.get_status_listener_lock(),
                                &mut self.status_listener.write_lock(),
                            )
                        }
                    }
                }

                InstanceStateKind::NotAliveNoWriters => todo!(),
            }
        }

        Ok(())
    }

    pub fn discover_matched_topics(&mut self) -> DdsResult<()> {
        while let Ok(samples) = self
            .get_builtin_subscriber()
            .stateful_data_reader_list()
            .into_iter()
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
            for sample in samples {
                if let Some(topic_data) = sample.data.as_ref() {
                    for topic in self.topic_list() {
                        topic.process_discovered_topic(
                            topic_data,
                            &mut self.get_status_listener_lock(),
                        );
                    }

                    self.discovered_topic_list.insert(
                        topic_data.get_serialized_key().into(),
                        topic_data.topic_builtin_topic_data().clone(),
                    );

                    self.topic_find_condvar.notify_all();
                }
            }
        }

        Ok(())
    }

    pub fn update_communication_status(&self) -> DdsResult<()> {
        let now = self.get_current_time();
        for subscriber in self.user_defined_subscriber_list.read_lock().iter() {
            subscriber.update_communication_status(now, &mut self.status_listener.write_lock());
        }

        Ok(())
    }

    pub fn cancel_timers(&self) {
        self.timer.write_lock().cancel_timers()
    }
}

fn add_matched_reader(
    writer: &DdsDataWriter<RtpsStatefulWriter>,
    discovered_reader_data: &DiscoveredReaderData,
    default_unicast_locator_list: &[Locator],
    default_multicast_locator_list: &[Locator],
    publisher_status_listener: &mut StatusListener<dyn PublisherListener + Send + Sync>,
    participant_status_listener: &mut StatusListener<dyn DomainParticipantListener + Send + Sync>,
    publisher_qos: &PublisherQos,
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
                    publisher_status_listener,
                    participant_status_listener,
                )
            }
        } else {
            writer_on_offered_incompatible_qos(
                writer,
                instance_handle,
                incompatible_qos_policy_list,
                publisher_status_listener,
                participant_status_listener,
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
    publisher_status_listener: &mut StatusListener<dyn PublisherListener + Send + Sync>,
    participant_status_listener: &mut StatusListener<dyn DomainParticipantListener + Send + Sync>,
) {
    writer.add_communication_state(StatusKind::PublicationMatched);

    let publication_matched_status_kind = &StatusKind::PublicationMatched;

    if writer.is_listener_enabled(publication_matched_status_kind) {
        writer.trigger_on_publication_matched(
            ListenerDataWriterNode::new(),
            writer.get_publication_matched_status(),
        )
    } else if publisher_status_listener.is_enabled(publication_matched_status_kind) {
        publisher_status_listener
            .listener_mut()
            .as_mut()
            .expect("Listener should be some")
            .on_publication_matched(
                &ListenerDataWriterNode::new(),
                writer.get_publication_matched_status(),
            )
    } else if participant_status_listener.is_enabled(publication_matched_status_kind) {
        participant_status_listener
            .listener_mut()
            .as_mut()
            .expect("Listener should be some")
            .on_publication_matched(
                &ListenerDataWriterNode::new(),
                writer.get_publication_matched_status(),
            )
    }
}

pub fn remove_writer_matched_reader(
    writer: &DdsDataWriter<RtpsStatefulWriter>,
    discovered_reader_handle: InstanceHandle,
    publisher_status_listener: &mut StatusListener<dyn PublisherListener + Send + Sync>,
    participant_status_listener: &mut StatusListener<dyn DomainParticipantListener + Send + Sync>,
) {
    if let Some(r) = writer.get_matched_subscription_data(discovered_reader_handle) {
        let handle = r.key().value.into();
        writer.matched_reader_remove(handle);
        writer.remove_matched_subscription(handle.into());

        on_writer_publication_matched(
            writer,
            publisher_status_listener,
            participant_status_listener,
        )
    }
}

fn writer_on_offered_incompatible_qos(
    writer: &DdsDataWriter<RtpsStatefulWriter>,
    handle: InstanceHandle,
    incompatible_qos_policy_list: Vec<QosPolicyId>,
    publisher_status_listener: &mut StatusListener<dyn PublisherListener + Send + Sync>,
    participant_status_listener: &mut StatusListener<dyn DomainParticipantListener + Send + Sync>,
) {
    if !writer.get_incompatible_subscriptions().contains(&handle) {
        writer.add_offered_incompatible_qos(handle, incompatible_qos_policy_list);

        let offerered_incompatible_qos_status_kind = &StatusKind::OfferedIncompatibleQos;
        if writer.is_listener_enabled(offerered_incompatible_qos_status_kind) {
            writer.trigger_on_offered_incompatible_qos(
                ListenerDataWriterNode::new(),
                writer.get_offered_incompatible_qos_status(),
            )
        } else if publisher_status_listener.is_enabled(offerered_incompatible_qos_status_kind) {
            publisher_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .on_offered_incompatible_qos(
                    &ListenerDataWriterNode::new(),
                    writer.get_offered_incompatible_qos_status(),
                )
        } else if participant_status_listener.is_enabled(offerered_incompatible_qos_status_kind) {
            participant_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .on_offered_incompatible_qos(
                    &ListenerDataWriterNode::new(),
                    writer.get_offered_incompatible_qos_status(),
                )
        }

        writer.add_communication_state(StatusKind::OfferedIncompatibleQos);
    }
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
    Foo: DdsType + for<'de> DdsDeserialize<'de>,
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
    Foo: DdsType + for<'de> DdsDeserialize<'de>,
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
