use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, Condvar,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    builtin_topics::BuiltInTopicKey,
    domain::domain_participant_factory::DomainId,
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::ReaderProxy, discovered_writer_data::WriterProxy,
        },
        rtps::{
            discovery_types::{BuiltinEndpointQos, BuiltinEndpointSet},
            endpoint::RtpsEndpoint,
            messages::RtpsMessage,
            reader::RtpsReader,
            reader_locator::RtpsReaderLocator,
            stateful_reader::RtpsStatefulReader,
            stateful_writer::RtpsStatefulWriter,
            stateless_reader::RtpsStatelessReader,
            transport::TransportWrite,
            types::{
                Count, EntityId, EntityKind, Guid, Locator, ProtocolVersion, TopicKind, VendorId,
                BUILT_IN_READER_GROUP, BUILT_IN_READER_WITH_KEY, BUILT_IN_WRITER_GROUP,
                BUILT_IN_WRITER_WITH_KEY, USER_DEFINED_READER_GROUP, USER_DEFINED_WRITER_GROUP,
            },
            writer::RtpsWriter,
        },
    },
    infrastructure::{
        instance::InstanceHandle,
        qos::QosKind,
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind},
        status::{StatusKind, NO_STATUS},
    },
    publication::publisher_listener::PublisherListener,
    subscription::{
        data_reader::DataReader,
        data_reader_listener::DataReaderListener,
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        subscriber_listener::SubscriberListener,
    },
    topic_definition::topic_listener::TopicListener,
    topic_definition::type_support::DdsType,
    {
        builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
        infrastructure::{
            error::{DdsError, DdsResult},
            qos::{
                DataReaderQos, DataWriterQos, DomainParticipantQos, PublisherQos, SubscriberQos,
                TopicQos,
            },
            qos_policy::{HistoryQosPolicy, HistoryQosPolicyKind},
            time::{Duration, Time},
        },
    },
};
use crate::{
    implementation::rtps::{
        group::RtpsGroupImpl, participant::RtpsParticipant, stateless_writer::RtpsStatelessWriter,
    },
    infrastructure::time::DURATION_ZERO,
};

use crate::implementation::{
    data_representation_builtin_endpoints::{
        discovered_reader_data::{DiscoveredReaderData, DCPS_SUBSCRIPTION},
        discovered_topic_data::{DiscoveredTopicData, DCPS_TOPIC},
        discovered_writer_data::{DiscoveredWriterData, DCPS_PUBLICATION},
        spdp_discovered_participant_data::{
            ParticipantProxy, SpdpDiscoveredParticipantData, DCPS_PARTICIPANT,
        },
    },
    utils::shared_object::{DdsRwLock, DdsShared},
};

use super::{
    data_reader_impl::{DataReaderImpl, RtpsReaderKind},
    data_writer_impl::{DataWriterImpl, RtpsWriterKind},
    message_receiver::MessageReceiver,
    participant_discovery::ParticipantDiscovery,
    publisher_impl::PublisherImpl,
    subscriber_impl::SubscriberImpl,
    topic_impl::TopicImpl,
};

use crate::domain::domain_participant_listener::DomainParticipantListener;

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

const DEFAULT_HEARTBEAT_PERIOD: Duration = Duration::new(2, 0);

const DEFAULT_NACK_RESPONSE_DELAY: Duration = Duration::new(0, 200);

const DEFAULT_NACK_SUPPRESSION_DURATION: Duration = DURATION_ZERO;

const DEFAULT_HEARTBEAT_RESPONSE_DELAY: Duration = Duration::new(0, 500);

const DEFAULT_HEARTBEAT_SUPPRESSION_DURATION: Duration = DURATION_ZERO;

pub const USER_DEFINED_TOPIC: EntityKind = 0x0a;

pub struct DomainParticipantImpl {
    rtps_participant: RtpsParticipant,
    domain_id: DomainId,
    domain_tag: String,
    qos: DdsRwLock<DomainParticipantQos>,
    builtin_subscriber: DdsShared<SubscriberImpl>,
    builtin_publisher: DdsShared<PublisherImpl>,
    user_defined_subscriber_list: DdsRwLock<Vec<DdsShared<SubscriberImpl>>>,
    user_defined_subscriber_counter: AtomicU8,
    default_subscriber_qos: SubscriberQos,
    user_defined_publisher_list: DdsRwLock<Vec<DdsShared<PublisherImpl>>>,
    user_defined_publisher_counter: AtomicU8,
    default_publisher_qos: PublisherQos,
    topic_list: DdsRwLock<Vec<DdsShared<TopicImpl>>>,
    user_defined_topic_counter: AtomicU8,
    default_topic_qos: TopicQos,
    manual_liveliness_count: Count,
    lease_duration: Duration,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    discovered_participant_list: DdsRwLock<HashMap<InstanceHandle, ParticipantBuiltinTopicData>>,
    discovered_topic_list: DdsShared<DdsRwLock<HashMap<InstanceHandle, TopicBuiltinTopicData>>>,
    enabled: DdsRwLock<bool>,
    announce_condvar: Arc<Condvar>,
}

impl DomainParticipantImpl {
    pub fn new(
        rtps_participant: RtpsParticipant,
        domain_id: DomainId,
        domain_tag: String,
        domain_participant_qos: DomainParticipantQos,
        metatraffic_unicast_locator_list: Vec<Locator>,
        metatraffic_multicast_locator_list: Vec<Locator>,
        announce_condvar: Arc<Condvar>,
    ) -> DdsShared<Self> {
        let lease_duration = Duration::new(100, 0);
        let guid_prefix = rtps_participant.guid().prefix();

        let builtin_subscriber = SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(Guid::new(
                guid_prefix,
                EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP),
            )),
        );

        let builtin_publisher = PublisherImpl::new(
            PublisherQos::default(),
            RtpsGroupImpl::new(Guid::new(
                guid_prefix,
                EntityId::new([0, 0, 0], BUILT_IN_WRITER_GROUP),
            )),
        );

        DdsShared::new(DomainParticipantImpl {
            rtps_participant,
            domain_id,
            domain_tag,
            qos: DdsRwLock::new(domain_participant_qos),
            builtin_subscriber,
            builtin_publisher,
            user_defined_subscriber_list: DdsRwLock::new(Vec::new()),
            user_defined_subscriber_counter: AtomicU8::new(0),
            default_subscriber_qos: SubscriberQos::default(),
            user_defined_publisher_list: DdsRwLock::new(Vec::new()),
            user_defined_publisher_counter: AtomicU8::new(0),
            default_publisher_qos: PublisherQos::default(),
            topic_list: DdsRwLock::new(Vec::new()),
            user_defined_topic_counter: AtomicU8::new(0),
            default_topic_qos: TopicQos::default(),
            manual_liveliness_count: Count(0),
            lease_duration,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            discovered_participant_list: DdsRwLock::new(HashMap::new()),
            discovered_topic_list: DdsShared::new(DdsRwLock::new(HashMap::new())),
            enabled: DdsRwLock::new(false),
            announce_condvar,
        })
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read_lock()
    }

    pub fn is_parent(&self, other: InstanceHandle) -> bool {
        self.rtps_participant.guid().prefix() == Guid::from(other).prefix()
    }
}

struct RegisterDiscoveredTopicsListener {
    discovered_topic_list: DdsShared<DdsRwLock<HashMap<InstanceHandle, TopicBuiltinTopicData>>>,
}

impl DataReaderListener for RegisterDiscoveredTopicsListener {
    type Foo = DiscoveredTopicData;

    fn on_data_available(&mut self, the_reader: &DataReader<Self::Foo>) {
        let topic_data = the_reader
            .take(
                i32::MAX,
                ANY_SAMPLE_STATE,
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
            )
            .unwrap()[0]
            .data
            .as_ref()
            .unwrap()
            .topic_builtin_topic_data
            .clone();

        self.discovered_topic_list
            .write_lock()
            .insert(topic_data.key.value.into(), topic_data);
    }
}

impl DdsShared<DomainParticipantImpl> {
    pub fn announce_topic(&self, sedp_discovered_topic_data: DiscoveredTopicData) {
        if let Some(topic_creation_topic) =
            self.lookup_topicdescription::<DiscoveredTopicData>(DCPS_TOPIC)
        {
            if let Ok(sedp_builtin_topic_announcer) = self
                .builtin_publisher
                .lookup_datawriter::<DiscoveredTopicData>(&topic_creation_topic)
            {
                sedp_builtin_topic_announcer
                    .write_w_timestamp(
                        &sedp_discovered_topic_data,
                        None,
                        self.get_current_time().unwrap(),
                    )
                    .unwrap();
            }
        }
    }
}

impl DdsShared<DomainParticipantImpl> {
    pub fn create_publisher(
        &self,
        qos: QosKind<PublisherQos>,
        _a_listener: Option<Box<dyn PublisherListener>>,
        _mask: &[StatusKind],
    ) -> DdsResult<DdsShared<PublisherImpl>> {
        let publisher_qos = match qos {
            QosKind::Default => self.default_publisher_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let publisher_counter = self
            .user_defined_publisher_counter
            .fetch_add(1, Ordering::Relaxed);
        let entity_id = EntityId::new([publisher_counter, 0, 0], USER_DEFINED_WRITER_GROUP);
        let guid = Guid::new(self.rtps_participant.guid().prefix(), entity_id);
        let rtps_group = RtpsGroupImpl::new(guid);
        let publisher_impl_shared = PublisherImpl::new(publisher_qos, rtps_group);
        if *self.enabled.read_lock()
            && self
                .qos
                .read_lock()
                .entity_factory
                .autoenable_created_entities
        {
            publisher_impl_shared.enable(self)?;
        }

        self.user_defined_publisher_list
            .write_lock()
            .push(publisher_impl_shared.clone());

        Ok(publisher_impl_shared)
    }

    pub fn delete_publisher(&self, a_publisher_handle: InstanceHandle) -> DdsResult<()> {
        let mut publisher_list = self.user_defined_publisher_list.write_lock();

        let publisher = publisher_list
            .iter()
            .find(|&x| x.get_instance_handle() == a_publisher_handle)
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(
                    "Publisher can only be deleted from its parent participant".to_string(),
                )
            })?;

        if !publisher.is_empty() {
            return Err(DdsError::PreconditionNotMet(
                "Publisher still contains data writers".to_string(),
            ));
        }

        publisher_list.retain(|x| x.get_instance_handle() != a_publisher_handle);

        Ok(())
    }

    pub fn create_subscriber(
        &self,
        qos: QosKind<SubscriberQos>,
        _a_listener: Option<Box<dyn SubscriberListener>>,
        _mask: &[StatusKind],
    ) -> DdsResult<DdsShared<SubscriberImpl>> {
        let subscriber_qos = match qos {
            QosKind::Default => self.default_subscriber_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let subcriber_counter = self
            .user_defined_subscriber_counter
            .fetch_add(1, Ordering::Relaxed);
        let entity_id = EntityId::new([subcriber_counter, 0, 0], USER_DEFINED_READER_GROUP);
        let guid = Guid::new(self.rtps_participant.guid().prefix(), entity_id);
        let rtps_group = RtpsGroupImpl::new(guid);
        let subscriber_shared = SubscriberImpl::new(subscriber_qos, rtps_group);
        if *self.enabled.read_lock()
            && self
                .qos
                .read_lock()
                .entity_factory
                .autoenable_created_entities
        {
            subscriber_shared.enable(self)?;
        }

        self.user_defined_subscriber_list
            .write_lock()
            .push(subscriber_shared.clone());

        Ok(subscriber_shared)
    }

    pub fn delete_subscriber(&self, a_subscriber_handle: InstanceHandle) -> DdsResult<()> {
        let mut subscriber_list = self.user_defined_subscriber_list.write_lock();

        let subscriber = subscriber_list
            .iter()
            .find(|&x| x.get_instance_handle() == a_subscriber_handle)
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(
                    "Subscriber can only be deleted from its parent participant".to_string(),
                )
            })?;

        if !subscriber.is_empty() {
            return Err(DdsError::PreconditionNotMet(
                "Subscriber still contains data readers".to_string(),
            ));
        }

        subscriber_list.retain(|x| x.get_instance_handle() != a_subscriber_handle);

        Ok(())
    }

    pub fn create_topic<Foo>(
        &self,
        topic_name: &str,
        qos: QosKind<TopicQos>,
        _a_listener: Option<Box<dyn TopicListener<Foo = Foo>>>,
        _mask: &[StatusKind],
    ) -> DdsResult<DdsShared<TopicImpl>>
    where
        Foo: DdsType,
    {
        let topic_counter = self
            .user_defined_topic_counter
            .fetch_add(1, Ordering::Relaxed);
        let topic_guid = Guid::new(
            self.rtps_participant.guid().prefix(),
            EntityId::new([topic_counter, 0, 0], USER_DEFINED_TOPIC),
        );
        let qos = match qos {
            QosKind::Default => self.default_topic_qos.clone(),
            QosKind::Specific(q) => q,
        };

        // /////// Create topic
        let topic_shared = TopicImpl::new(
            topic_guid,
            qos,
            Foo::type_name(),
            topic_name,
            self.downgrade(),
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

        self.topic_list.write_lock().push(topic_shared.clone());

        Ok(topic_shared)
    }

    pub fn delete_topic<Foo>(&self, a_topic_handle: InstanceHandle) -> DdsResult<()> {
        let mut topic_list = self.topic_list.write_lock();
        let topic = topic_list
            .iter()
            .find(|&topic| topic.get_instance_handle() == a_topic_handle)
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(
                    "Topic can only be deleted from its parent publisher".to_string(),
                )
            })?;

        if topic.strong_count() > 1 {
            return Err(DdsError::PreconditionNotMet(
                "Topic still attached to some data reader or data writer".to_string(),
            ));
        }

        topic_list.retain(|x| x.get_instance_handle() != a_topic_handle);

        Ok(())
    }

    pub fn find_topic<Foo>(
        &self,
        topic_name: &str,
        _timeout: Duration,
    ) -> DdsResult<DdsShared<TopicImpl>>
    where
        Foo: DdsType,
    {
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
            .ok_or_else(|| DdsError::PreconditionNotMet("Not found".to_string()))
    }

    pub fn lookup_topicdescription<Foo>(&self, topic_name: &str) -> Option<DdsShared<TopicImpl>>
    where
        Foo: DdsType,
    {
        self.topic_list.read_lock().iter().find_map(|topic| {
            if topic.get_name().unwrap() == topic_name
                && topic.get_type_name().unwrap() == Foo::type_name()
            {
                Some(topic.clone())
            } else {
                None
            }
        })
    }

    pub fn get_builtin_subscriber(&self) -> DdsResult<DdsShared<SubscriberImpl>> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        Ok(self.builtin_subscriber.clone())
    }

    pub fn ignore_participant(&self, _handle: InstanceHandle) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn ignore_topic(&self, _handle: InstanceHandle) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn ignore_publication(&self, _handle: InstanceHandle) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn ignore_subscription(&self, _handle: InstanceHandle) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn get_domain_id(&self) -> DdsResult<DomainId> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn delete_contained_entities(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn assert_liveliness(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn set_default_publisher_qos(&self, _qos: QosKind<PublisherQos>) -> DdsResult<()> {
        todo!()
    }

    pub fn get_default_publisher_qos(&self) -> DdsResult<PublisherQos> {
        todo!()
    }

    pub fn set_default_subscriber_qos(&self, _qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        todo!()
    }

    pub fn get_default_subscriber_qos(&self) -> DdsResult<SubscriberQos> {
        todo!()
    }

    pub fn set_default_topic_qos(&self, _qos: QosKind<TopicQos>) -> DdsResult<()> {
        todo!()
    }

    pub fn get_default_topic_qos(&self) -> DdsResult<TopicQos> {
        Ok(self.default_topic_qos.clone())
    }

    pub fn get_discovered_participants(&self) -> DdsResult<Vec<InstanceHandle>> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        Ok(self
            .discovered_participant_list
            .read_lock()
            .iter()
            .map(|(&key, _)| key)
            .collect())
    }

    pub fn get_discovered_participant_data(
        &self,
        participant_handle: InstanceHandle,
    ) -> DdsResult<ParticipantBuiltinTopicData> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.discovered_participant_list
            .read_lock()
            .get(&participant_handle)
            .cloned()
            .ok_or(DdsError::BadParameter)
    }

    pub fn get_discovered_topics(&self) -> DdsResult<Vec<InstanceHandle>> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        Ok(self
            .discovered_topic_list
            .read_lock()
            .keys()
            .cloned()
            .collect())
    }

    pub fn get_discovered_topic_data(
        &self,
        topic_handle: InstanceHandle,
    ) -> DdsResult<TopicBuiltinTopicData> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.discovered_topic_list
            .read_lock()
            .get(&topic_handle)
            .cloned()
            .ok_or(DdsError::BadParameter)
    }

    pub fn contains_entity(&self, _a_handle: InstanceHandle) -> DdsResult<bool> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn get_current_time(&self) -> DdsResult<Time> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

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

impl DdsShared<DomainParticipantImpl> {
    pub fn set_qos(&self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        *self.qos.write_lock() = match qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };
        self.announce_condvar.notify_all();

        Ok(())
    }

    pub fn get_qos(&self) -> DomainParticipantQos {
        self.qos.read_lock().clone()
    }

    pub fn set_listener(
        &self,
        _a_listener: Option<Box<dyn DomainParticipantListener>>,
        _mask: &[StatusKind],
    ) -> DdsResult<()> {
        todo!()
    }

    pub fn get_listener(&self) -> DdsResult<Option<Box<dyn DomainParticipantListener>>> {
        todo!()
    }

    pub fn get_statuscondition(
        &self,
    ) -> DdsResult<crate::infrastructure::condition::StatusCondition> {
        todo!()
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    pub fn enable(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            *self.enabled.write_lock() = true;

            self.builtin_subscriber.enable(self)?;
            self.builtin_publisher.enable(self)?;

            if self
                .qos
                .read_lock()
                .entity_factory
                .autoenable_created_entities
            {
                for publisher in self.user_defined_publisher_list.read_lock().iter() {
                    publisher.enable(self)?;
                }

                for subscriber in self.user_defined_subscriber_list.read_lock().iter() {
                    subscriber.enable(self)?;
                }

                for topic in self.topic_list.read_lock().iter() {
                    topic.enable()?;
                }
            }
            self.announce_condvar.notify_all();
        }
        Ok(())
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        Ok(self.rtps_participant.guid().into())
    }
}

impl DdsShared<DomainParticipantImpl> {
    pub fn announce_participant(&self) -> DdsResult<()> {
        let dcps_topic_participant = self
            .lookup_topicdescription::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT)
            .ok_or_else(|| DdsError::PreconditionNotMet("Topic not found".to_string()))?;

        let spdp_participant_writer = self
            .builtin_publisher
            .lookup_datawriter::<SpdpDiscoveredParticipantData>(&dcps_topic_participant)?;

        let spdp_discovered_participant_data = SpdpDiscoveredParticipantData {
            dds_participant_data: ParticipantBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: self.rtps_participant.guid().into(),
                },
                user_data: self.qos.read_lock().user_data.clone(),
            },
            participant_proxy: ParticipantProxy {
                domain_id: self.domain_id,
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
            lease_duration: self.lease_duration.into(),
        };
        spdp_participant_writer.write_w_timestamp(
            &spdp_discovered_participant_data,
            None,
            self.get_current_time().unwrap(),
        )
    }
}

impl DdsShared<DomainParticipantImpl> {
    pub fn add_discovered_participant(
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
            self.domain_id as i32,
            &self.domain_tag,
        ) {
            let sedp_builtin_publication_writer_shared = self
                .builtin_publisher
                .lookup_datawriter::<DiscoveredWriterData>(&dcps_publication_topic)
                .unwrap();
            sedp_builtin_publication_writer_shared.add_matched_participant(&participant_discovery);

            let sedp_builtin_publication_reader_shared = self
                .builtin_subscriber
                .lookup_datareader::<DiscoveredWriterData>(&dcps_publication_topic)
                .unwrap();
            sedp_builtin_publication_reader_shared.add_matched_participant(&participant_discovery);

            let sedp_builtin_subscription_writer_shared = self
                .builtin_publisher
                .lookup_datawriter::<DiscoveredReaderData>(&dcps_subscription_topic)
                .unwrap();
            sedp_builtin_subscription_writer_shared.add_matched_participant(&participant_discovery);

            let sedp_builtin_subscription_reader_shared = self
                .builtin_subscriber
                .lookup_datareader::<DiscoveredReaderData>(&dcps_subscription_topic)
                .unwrap();
            sedp_builtin_subscription_reader_shared.add_matched_participant(&participant_discovery);

            let sedp_builtin_topic_writer_shared = self
                .builtin_publisher
                .lookup_datawriter::<DiscoveredTopicData>(&dcps_topic_topic)
                .unwrap();
            sedp_builtin_topic_writer_shared.add_matched_participant(&participant_discovery);

            let sedp_builtin_topic_reader_shared = self
                .builtin_subscriber
                .lookup_datareader::<DiscoveredTopicData>(&dcps_topic_topic)
                .unwrap();
            sedp_builtin_topic_reader_shared.add_matched_participant(&participant_discovery);

            self.discovered_participant_list.write_lock().insert(
                discovered_participant_data
                    .dds_participant_data
                    .key
                    .value
                    .into(),
                discovered_participant_data.dds_participant_data.clone(),
            );
        }
    }
}

impl DdsShared<DomainParticipantImpl> {
    pub fn default_unicast_locator_list(&self) -> &[Locator] {
        self.rtps_participant.default_unicast_locator_list()
    }

    pub fn default_multicast_locator_list(&self) -> &[Locator] {
        self.rtps_participant.default_multicast_locator_list()
    }

    pub fn protocol_version(&self) -> ProtocolVersion {
        self.rtps_participant.protocol_version()
    }

    pub fn vendor_id(&self) -> VendorId {
        self.rtps_participant.vendor_id()
    }
}

impl DdsShared<DomainParticipantImpl> {
    pub fn send_built_in_data(&self, transport: &mut impl TransportWrite) {
        self.builtin_publisher.send_message(transport);
        self.builtin_subscriber.send_message(transport);
    }
}

impl DdsShared<DomainParticipantImpl> {
    pub fn receive_built_in_data(&self, source_locator: Locator, message: RtpsMessage) {
        MessageReceiver::new().process_message(
            self.rtps_participant.guid().prefix(),
            core::slice::from_ref(&self.builtin_publisher),
            core::slice::from_ref(&self.builtin_subscriber),
            source_locator,
            &message,
        );
    }
}

impl DdsShared<DomainParticipantImpl> {
    pub fn create_builtins(&self) -> DdsResult<()> {
        let guid_prefix = self.rtps_participant.guid().prefix();

        ///////// Create built-in DDS data readers and data writers

        ////////// SPDP built-in topic, reader and writer
        {
            let spdp_topic_participant = self.create_topic::<SpdpDiscoveredParticipantData>(
                DCPS_PARTICIPANT,
                QosKind::Specific(self.get_default_topic_qos()?),
                None,
                NO_STATUS,
            )?;
            spdp_topic_participant.enable()?;

            let spdp_builtin_participant_reader_guid =
                Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER);

            let unicast_locator_list = &[];
            let multicast_locator_list = &[];
            let spdp_builtin_participant_rtps_reader =
                RtpsStatelessReader::new(RtpsReader::new::<SpdpDiscoveredParticipantData>(
                    RtpsEndpoint::new(
                        spdp_builtin_participant_reader_guid,
                        TopicKind::WithKey,
                        unicast_locator_list,
                        multicast_locator_list,
                    ),
                    DURATION_ZERO,
                    DURATION_ZERO,
                    false,
                    DataReaderQos {
                        history: HistoryQosPolicy {
                            kind: HistoryQosPolicyKind::KeepAll,
                            depth: 0,
                        },
                        ..Default::default()
                    },
                ));

            let spdp_builtin_participant_data_reader = DataReaderImpl::new(
                RtpsReaderKind::Stateless(spdp_builtin_participant_rtps_reader),
                spdp_topic_participant.clone(),
                None,
                self.builtin_subscriber.downgrade(),
            );
            spdp_builtin_participant_data_reader.enable(self)?;
            self.builtin_subscriber
                .add_data_reader(spdp_builtin_participant_data_reader);

            let spdp_reader_locators: Vec<RtpsReaderLocator> = self
                .metatraffic_multicast_locator_list
                .iter()
                .map(|&locator| RtpsReaderLocator::new(locator, false))
                .collect();

            let spdp_builtin_participant_writer_guid =
                Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER);
            let unicast_locator_list = &[];
            let multicast_locator_list = &[];

            let mut spdp_builtin_participant_rtps_writer =
                RtpsStatelessWriter::new(RtpsWriter::new(
                    RtpsEndpoint::new(
                        spdp_builtin_participant_writer_guid,
                        TopicKind::WithKey,
                        unicast_locator_list,
                        multicast_locator_list,
                    ),
                    true,
                    DURATION_ZERO,
                    DURATION_ZERO,
                    DURATION_ZERO,
                    None,
                    DataWriterQos {
                        reliability: ReliabilityQosPolicy {
                            kind: ReliabilityQosPolicyKind::BestEffort,
                            max_blocking_time: DURATION_ZERO,
                        },
                        ..Default::default()
                    },
                ));

            for reader_locator in spdp_reader_locators {
                spdp_builtin_participant_rtps_writer.reader_locator_add(reader_locator);
            }

            let spdp_builtin_participant_data_writer = DataWriterImpl::new(
                RtpsWriterKind::Stateless(spdp_builtin_participant_rtps_writer),
                None,
                spdp_topic_participant,
                self.builtin_publisher.downgrade(),
            );
            spdp_builtin_participant_data_writer.enable(self)?;
            self.builtin_publisher
                .add_data_writer(spdp_builtin_participant_data_writer);
        }

        ////////// SEDP built-in publication topic, reader and writer
        {
            let sedp_topic_publication = self.create_topic::<DiscoveredWriterData>(
                DCPS_PUBLICATION,
                QosKind::Specific(self.get_default_topic_qos()?),
                None,
                NO_STATUS,
            )?;
            sedp_topic_publication.enable()?;

            let guid = Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
            let topic_kind = TopicKind::WithKey;
            let heartbeat_response_delay = DEFAULT_HEARTBEAT_RESPONSE_DELAY;
            let heartbeat_suppression_duration = DEFAULT_HEARTBEAT_SUPPRESSION_DURATION;
            let expects_inline_qos = false;
            let unicast_locator_list = &[];
            let multicast_locator_list = &[];
            let sedp_builtin_publications_rtps_reader =
                RtpsStatefulReader::new(RtpsReader::new::<DiscoveredReaderData>(
                    RtpsEndpoint::new(
                        guid,
                        topic_kind,
                        unicast_locator_list,
                        multicast_locator_list,
                    ),
                    heartbeat_response_delay,
                    heartbeat_suppression_duration,
                    expects_inline_qos,
                    DataReaderQos {
                        history: HistoryQosPolicy {
                            kind: HistoryQosPolicyKind::KeepAll,
                            depth: 0,
                        },
                        reliability: ReliabilityQosPolicy {
                            kind: ReliabilityQosPolicyKind::Reliable,
                            max_blocking_time: DURATION_ZERO,
                        },
                        ..Default::default()
                    },
                ));

            let sedp_builtin_publications_data_reader = DataReaderImpl::new(
                RtpsReaderKind::Stateful(sedp_builtin_publications_rtps_reader),
                sedp_topic_publication.clone(),
                None,
                self.builtin_subscriber.downgrade(),
            );
            sedp_builtin_publications_data_reader.enable(self)?;
            self.builtin_subscriber
                .add_data_reader(sedp_builtin_publications_data_reader);

            let guid = Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
            let topic_kind = TopicKind::WithKey;
            let push_mode = true;
            let heartbeat_period = DEFAULT_HEARTBEAT_PERIOD;
            let nack_response_delay = DEFAULT_NACK_RESPONSE_DELAY;
            let nack_suppression_duration = DEFAULT_NACK_SUPPRESSION_DURATION;
            let data_max_size_serialized = None;
            let sedp_builtin_publications_rtps_writer = RtpsStatefulWriter::new(RtpsWriter::new(
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
                DataWriterQos::default(),
            ));

            let sedp_builtin_publications_data_writer = DataWriterImpl::new(
                RtpsWriterKind::Stateful(sedp_builtin_publications_rtps_writer),
                None,
                sedp_topic_publication,
                self.builtin_publisher.downgrade(),
            );
            sedp_builtin_publications_data_writer.enable(self)?;

            self.builtin_publisher
                .add_data_writer(sedp_builtin_publications_data_writer);
        }

        ////////// SEDP built-in subcriptions topic, reader and writer
        {
            let sedp_topic_subscription = self.create_topic::<DiscoveredReaderData>(
                DCPS_SUBSCRIPTION,
                QosKind::Specific(self.get_default_topic_qos()?),
                None,
                NO_STATUS,
            )?;
            sedp_topic_subscription.enable()?;

            let guid = Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            let topic_kind = TopicKind::WithKey;
            let heartbeat_response_delay = DEFAULT_HEARTBEAT_RESPONSE_DELAY;
            let heartbeat_suppression_duration = DEFAULT_HEARTBEAT_SUPPRESSION_DURATION;
            let expects_inline_qos = false;
            let unicast_locator_list = &[];
            let multicast_locator_list = &[];
            let sedp_builtin_subscriptions_rtps_reader =
                RtpsStatefulReader::new(RtpsReader::new::<DiscoveredWriterData>(
                    RtpsEndpoint::new(
                        guid,
                        topic_kind,
                        unicast_locator_list,
                        multicast_locator_list,
                    ),
                    heartbeat_response_delay,
                    heartbeat_suppression_duration,
                    expects_inline_qos,
                    DataReaderQos {
                        history: HistoryQosPolicy {
                            kind: HistoryQosPolicyKind::KeepAll,
                            depth: 0,
                        },
                        reliability: ReliabilityQosPolicy {
                            kind: ReliabilityQosPolicyKind::Reliable,
                            max_blocking_time: DURATION_ZERO,
                        },
                        ..Default::default()
                    },
                ));

            let sedp_builtin_subscriptions_data_reader = DataReaderImpl::new(
                RtpsReaderKind::Stateful(sedp_builtin_subscriptions_rtps_reader),
                sedp_topic_subscription.clone(),
                None,
                self.builtin_subscriber.downgrade(),
            );
            sedp_builtin_subscriptions_data_reader.enable(self)?;
            self.builtin_subscriber
                .add_data_reader(sedp_builtin_subscriptions_data_reader);

            let guid = Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            let topic_kind = TopicKind::WithKey;
            let push_mode = true;
            let heartbeat_period = DEFAULT_HEARTBEAT_PERIOD;
            let nack_response_delay = DEFAULT_NACK_RESPONSE_DELAY;
            let nack_suppression_duration = DEFAULT_NACK_SUPPRESSION_DURATION;
            let data_max_size_serialized = None;
            let unicast_locator_list = &[];
            let multicast_locator_list = &[];
            let sedp_builtin_subscriptions_rtps_writer = RtpsStatefulWriter::new(RtpsWriter::new(
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
                DataWriterQos::default(),
            ));
            let sedp_builtin_subscriptions_data_writer = DataWriterImpl::new(
                RtpsWriterKind::Stateful(sedp_builtin_subscriptions_rtps_writer),
                None,
                sedp_topic_subscription,
                self.builtin_publisher.downgrade(),
            );
            sedp_builtin_subscriptions_data_writer.enable(self)?;
            self.builtin_publisher
                .add_data_writer(sedp_builtin_subscriptions_data_writer);
        }

        ////////// SEDP built-in topics topic, reader and writer
        {
            let sedp_topic_topic = self.create_topic::<DiscoveredTopicData>(
                DCPS_TOPIC,
                QosKind::Specific(self.get_default_topic_qos()?),
                None,
                NO_STATUS,
            )?;
            sedp_topic_topic.enable()?;

            let guid = Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
            let topic_kind = TopicKind::WithKey;
            let heartbeat_response_delay = DEFAULT_HEARTBEAT_RESPONSE_DELAY;
            let heartbeat_suppression_duration = DEFAULT_HEARTBEAT_SUPPRESSION_DURATION;
            let expects_inline_qos = false;
            let unicast_locator_list = &[];
            let multicast_locator_list = &[];
            let sedp_builtin_topics_rtps_reader =
                RtpsStatefulReader::new(RtpsReader::new::<DiscoveredTopicData>(
                    RtpsEndpoint::new(
                        guid,
                        topic_kind,
                        unicast_locator_list,
                        multicast_locator_list,
                    ),
                    heartbeat_response_delay,
                    heartbeat_suppression_duration,
                    expects_inline_qos,
                    DataReaderQos {
                        history: HistoryQosPolicy {
                            kind: HistoryQosPolicyKind::KeepAll,
                            depth: 0,
                        },
                        reliability: ReliabilityQosPolicy {
                            kind: ReliabilityQosPolicyKind::Reliable,
                            max_blocking_time: DURATION_ZERO,
                        },
                        ..Default::default()
                    },
                ));

            let sedp_builtin_topics_data_reader = DataReaderImpl::new(
                RtpsReaderKind::Stateful(sedp_builtin_topics_rtps_reader),
                sedp_topic_topic.clone(),
                None,
                self.builtin_subscriber.downgrade(),
            );
            sedp_builtin_topics_data_reader.enable(self)?;

            // set the topic listener
            {
                let topics_data_reader =
                    DataReader::new(sedp_builtin_topics_data_reader.downgrade());
                topics_data_reader.set_listener(
                    Some(Box::new(RegisterDiscoveredTopicsListener {
                        discovered_topic_list: self.discovered_topic_list.clone(),
                    })),
                    NO_STATUS,
                )?;
            }

            self.builtin_subscriber
                .add_data_reader(sedp_builtin_topics_data_reader);

            let guid = Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
            let topic_kind = TopicKind::WithKey;
            let push_mode = true;
            let heartbeat_period = DEFAULT_HEARTBEAT_PERIOD;
            let nack_response_delay = DEFAULT_NACK_RESPONSE_DELAY;
            let nack_suppression_duration = DEFAULT_NACK_SUPPRESSION_DURATION;
            let data_max_size_serialized = None;
            let unicast_locator_list = &[];
            let multicast_locator_list = &[];
            let sedp_builtin_topics_rtps_writer = RtpsStatefulWriter::new(RtpsWriter::new(
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
                DataWriterQos::default(),
            ));

            let sedp_builtin_topics_data_writer = DataWriterImpl::new(
                RtpsWriterKind::Stateful(sedp_builtin_topics_rtps_writer),
                None,
                sedp_topic_topic,
                self.builtin_publisher.downgrade(),
            );
            sedp_builtin_topics_data_writer.enable(self)?;
            self.builtin_publisher
                .add_data_writer(sedp_builtin_topics_data_writer);
        }

        Ok(())
    }
}

impl DdsShared<DomainParticipantImpl> {
    pub fn send_user_defined_data(&self, transport: &mut impl TransportWrite) {
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

impl DdsShared<DomainParticipantImpl> {
    pub fn receive_user_defined_data(&self, source_locator: Locator, message: RtpsMessage) {
        MessageReceiver::new().process_message(
            self.rtps_participant.guid().prefix(),
            self.user_defined_publisher_list.read_lock().as_slice(),
            self.user_defined_subscriber_list.read_lock().as_slice(),
            source_locator,
            &message,
        );
    }
}

impl DdsShared<DomainParticipantImpl> {
    pub fn discover_matched_participants(&self) -> DdsResult<()> {
        let dcps_participant_topic = self
            .lookup_topicdescription::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT)
            .ok_or_else(|| DdsError::PreconditionNotMet("Topic not found".to_string()))?;

        let spdp_builtin_participant_data_reader =
            self.builtin_subscriber
                .lookup_datareader::<SpdpDiscoveredParticipantData>(&dcps_participant_topic)?;

        if let Ok(samples) = spdp_builtin_participant_data_reader.take(
            1,
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        ) {
            for discovered_participant_data_sample in samples.iter() {
                self.add_discovered_participant(
                    discovered_participant_data_sample.data.as_ref().unwrap(),
                )
            }
        }

        Ok(())
    }
}

impl DdsShared<DomainParticipantImpl> {
    pub fn discover_matched_writers(&self) -> DdsResult<()> {
        let user_defined_subscribers = self.user_defined_subscriber_list.read_lock();

        if user_defined_subscribers.is_empty() {
            return Ok(());
        }

        let dcps_publication_topic = self
            .lookup_topicdescription::<DiscoveredWriterData>(DCPS_PUBLICATION)
            .ok_or_else(|| DdsError::PreconditionNotMet("Topic not found".to_string()))?;
        let sedp_builtin_publication_reader = self
            .builtin_subscriber
            .lookup_datareader::<DiscoveredWriterData>(&dcps_publication_topic)?;

        let samples = sedp_builtin_publication_reader.take(
            1,
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        );

        for discovered_writer_data_sample in samples.unwrap_or_else(|_| vec![]).iter() {
            for subscriber in user_defined_subscribers.iter() {
                subscriber.add_matched_writer(discovered_writer_data_sample.data.as_ref().unwrap());
            }
        }

        Ok(())
    }
}

impl DdsShared<DomainParticipantImpl> {
    pub fn discover_matched_readers(&self) -> DdsResult<()> {
        let user_defined_publishers = self.user_defined_publisher_list.read_lock();

        if user_defined_publishers.is_empty() {
            return Ok(());
        }

        let dcps_subscription_topic = self
            .lookup_topicdescription::<DiscoveredReaderData>(DCPS_SUBSCRIPTION)
            .ok_or_else(|| DdsError::PreconditionNotMet("Topic not found".to_string()))?;
        let sedp_builtin_subscription_reader = self
            .builtin_subscriber
            .lookup_datareader::<DiscoveredReaderData>(&dcps_subscription_topic)?;

        let samples = sedp_builtin_subscription_reader.take(
            1,
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        );

        for discovered_reader_data_sample in samples.unwrap_or_else(|_| vec![]).iter() {
            for publisher in user_defined_publishers.iter() {
                publisher.add_matched_reader(discovered_reader_data_sample.data.as_ref().unwrap())
            }
        }

        Ok(())
    }
}

impl DdsShared<DomainParticipantImpl> {
    pub fn announce_datawriter(&self, sedp_discovered_writer_data: DiscoveredWriterData) {
        let writer_data = &DiscoveredWriterData {
            writer_proxy: WriterProxy {
                unicast_locator_list: self.default_unicast_locator_list().to_vec(),
                multicast_locator_list: self.default_multicast_locator_list().to_vec(),
                ..sedp_discovered_writer_data.writer_proxy
            },
            ..sedp_discovered_writer_data
        };

        if let Some(publication_topic) =
            self.lookup_topicdescription::<DiscoveredWriterData>(DCPS_PUBLICATION)
        {
            if let Ok(sedp_builtin_publications_announcer) =
                self.builtin_publisher
                    .lookup_datawriter::<DiscoveredWriterData>(&publication_topic)
            {
                sedp_builtin_publications_announcer
                    .write_w_timestamp(writer_data, None, self.get_current_time().unwrap())
                    .unwrap();
            }
        }
    }

    pub fn announce_datareader(&self, sedp_discovered_reader_data: DiscoveredReaderData) {
        let reader_data = &DiscoveredReaderData {
            reader_proxy: ReaderProxy {
                unicast_locator_list: self.default_unicast_locator_list().to_vec(),
                multicast_locator_list: self.default_multicast_locator_list().to_vec(),
                ..sedp_discovered_reader_data.reader_proxy
            },
            ..sedp_discovered_reader_data
        };

        if let Some(subscription_topic) =
            self.lookup_topicdescription::<DiscoveredReaderData>(DCPS_SUBSCRIPTION)
        {
            if let Ok(sedp_builtin_subscription_announcer) =
                self.builtin_publisher
                    .lookup_datawriter::<DiscoveredReaderData>(&subscription_topic)
            {
                sedp_builtin_subscription_announcer
                    .write_w_timestamp(reader_data, None, self.get_current_time().unwrap())
                    .unwrap();
            }
        }
    }
}
