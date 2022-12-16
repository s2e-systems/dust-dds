use std::{
    collections::HashMap,
    sync::atomic::{AtomicU8, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    builtin_topics::BuiltInTopicKey,
    domain::{
        domain_participant_factory::DomainId,
        domain_participant_listener::DomainParticipantListener,
    },
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::ReaderProxy,
            discovered_reader_data::{DiscoveredReaderData, DCPS_SUBSCRIPTION},
            discovered_topic_data::{DiscoveredTopicData, DCPS_TOPIC},
            discovered_writer_data::WriterProxy,
            discovered_writer_data::{DiscoveredWriterData, DCPS_PUBLICATION},
            spdp_discovered_participant_data::{
                ParticipantProxy, SpdpDiscoveredParticipantData, DCPS_PARTICIPANT,
            },
        },
        rtps::{
            discovery_types::{BuiltinEndpointQos, BuiltinEndpointSet},
            group::RtpsGroupImpl,
            messages::RtpsMessage,
            participant::RtpsParticipant,
            transport::TransportWrite,
            types::{
                Count, EntityId, Guid, Locator, BUILT_IN_READER_WITH_KEY, BUILT_IN_TOPIC,
                BUILT_IN_WRITER_WITH_KEY, USER_DEFINED_READER_GROUP, USER_DEFINED_TOPIC,
                USER_DEFINED_WRITER_GROUP,
            },
        },
        utils::{
            condvar::DdsCondvar,
            shared_object::{DdsRwLock, DdsShared, DdsWeak},
        },
    },
    infrastructure::{instance::InstanceHandle, qos::QosKind, status::StatusKind},
    publication::publisher_listener::PublisherListener,
    subscription::{
        sample_info::{InstanceStateKind, ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        subscriber_listener::SubscriberListener,
    },
    topic_definition::topic_listener::TopicListener,
    topic_definition::type_support::DdsType,
    {
        builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
        infrastructure::{
            error::{DdsError, DdsResult},
            qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos},
            time::{Duration, Time},
        },
    },
};

use super::{
    builtin_publisher::BuiltinPublisher, builtin_subscriber::BuiltInSubscriber,
    message_receiver::MessageReceiver, participant_discovery::ParticipantDiscovery,
    topic_impl::TopicImpl, user_defined_publisher::UserDefinedPublisher,
    user_defined_subscriber::UserDefinedSubscriber,
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

pub struct DomainParticipantImpl {
    rtps_participant: RtpsParticipant,
    domain_id: DomainId,
    domain_tag: String,
    qos: DdsRwLock<DomainParticipantQos>,
    builtin_subscriber: DdsShared<BuiltInSubscriber>,
    builtin_publisher: DdsShared<BuiltinPublisher>,
    user_defined_subscriber_list: DdsRwLock<Vec<DdsShared<UserDefinedSubscriber>>>,
    user_defined_subscriber_counter: AtomicU8,
    default_subscriber_qos: SubscriberQos,
    user_defined_publisher_list: DdsRwLock<Vec<DdsShared<UserDefinedPublisher>>>,
    user_defined_publisher_counter: AtomicU8,
    default_publisher_qos: PublisherQos,
    topic_list: DdsRwLock<Vec<DdsShared<TopicImpl>>>,
    user_defined_topic_counter: AtomicU8,
    default_topic_qos: TopicQos,
    manual_liveliness_count: Count,
    lease_duration: Duration,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    discovered_participant_list: DdsRwLock<HashMap<InstanceHandle, SpdpDiscoveredParticipantData>>,
    discovered_topic_list: DdsShared<DdsRwLock<HashMap<InstanceHandle, TopicBuiltinTopicData>>>,
    enabled: DdsRwLock<bool>,
    announce_condvar: DdsCondvar,
    user_defined_data_send_condvar: DdsCondvar,
}

impl DomainParticipantImpl {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rtps_participant: RtpsParticipant,
        domain_id: DomainId,
        domain_tag: String,
        domain_participant_qos: DomainParticipantQos,
        metatraffic_unicast_locator_list: Vec<Locator>,
        metatraffic_multicast_locator_list: Vec<Locator>,
        spdp_discovery_locator_list: &[Locator],
        announce_condvar: DdsCondvar,
        user_defined_data_send_condvar: DdsCondvar,
    ) -> DdsShared<Self> {
        let lease_duration = Duration::new(100, 0);
        let guid_prefix = rtps_participant.guid().prefix();

        let spdp_topic_entity_id = EntityId::new([0, 0, 0], BUILT_IN_TOPIC);
        let spdp_topic_guid = Guid::new(guid_prefix, spdp_topic_entity_id);
        let spdp_topic_participant = TopicImpl::new(
            spdp_topic_guid,
            TopicQos::default(),
            SpdpDiscoveredParticipantData::type_name(),
            DCPS_PARTICIPANT,
            DdsWeak::new(),
        );

        let sedp_topics_entity_id = EntityId::new([0, 0, 1], BUILT_IN_TOPIC);
        let sedp_topics_guid = Guid::new(guid_prefix, sedp_topics_entity_id);
        let sedp_topic_topics = TopicImpl::new(
            sedp_topics_guid,
            TopicQos::default(),
            DiscoveredTopicData::type_name(),
            DCPS_TOPIC,
            DdsWeak::new(),
        );

        let sedp_publications_entity_id = EntityId::new([0, 0, 2], BUILT_IN_TOPIC);
        let sedp_publications_guid = Guid::new(guid_prefix, sedp_publications_entity_id);
        let sedp_topic_publications = TopicImpl::new(
            sedp_publications_guid,
            TopicQos::default(),
            DiscoveredWriterData::type_name(),
            DCPS_PUBLICATION,
            DdsWeak::new(),
        );

        let sedp_subscriptions_entity_id = EntityId::new([0, 0, 2], BUILT_IN_TOPIC);
        let sedp_subscriptions_guid = Guid::new(guid_prefix, sedp_subscriptions_entity_id);
        let sedp_topic_subscriptions = TopicImpl::new(
            sedp_subscriptions_guid,
            TopicQos::default(),
            DiscoveredReaderData::type_name(),
            DCPS_SUBSCRIPTION,
            DdsWeak::new(),
        );

        let builtin_subscriber = BuiltInSubscriber::new(
            guid_prefix,
            spdp_topic_participant,
            sedp_topic_topics.clone(),
            sedp_topic_publications.clone(),
            sedp_topic_subscriptions.clone(),
        );

        let builtin_publisher = BuiltinPublisher::new(
            guid_prefix,
            sedp_topic_topics,
            sedp_topic_publications,
            sedp_topic_subscriptions,
            spdp_discovery_locator_list,
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
            manual_liveliness_count: Count::new(0),
            lease_duration,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            discovered_participant_list: DdsRwLock::new(HashMap::new()),
            discovered_topic_list: DdsShared::new(DdsRwLock::new(HashMap::new())),
            enabled: DdsRwLock::new(false),
            announce_condvar,
            user_defined_data_send_condvar,
        })
    }
}

impl DdsShared<DomainParticipantImpl> {
    pub fn is_enabled(&self) -> bool {
        *self.enabled.read_lock()
    }

    pub fn announce_topic(&self, sedp_discovered_topic_data: DiscoveredTopicData) {
        self.builtin_publisher
            .sedp_builtin_topics_writer()
            .write_w_timestamp(
                &sedp_discovered_topic_data,
                None,
                self.get_current_time().unwrap(),
            )
            .unwrap();
    }

    pub fn create_publisher(
        &self,
        qos: QosKind<PublisherQos>,
        _a_listener: Option<Box<dyn PublisherListener>>,
        _mask: &[StatusKind],
    ) -> DdsResult<DdsShared<UserDefinedPublisher>> {
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
        let publisher_impl_shared = UserDefinedPublisher::new(
            publisher_qos,
            rtps_group,
            self.downgrade(),
            self.user_defined_data_send_condvar.clone(),
        );
        if *self.enabled.read_lock()
            && self
                .qos
                .read_lock()
                .entity_factory
                .autoenable_created_entities
        {
            publisher_impl_shared.enable()?;
        }

        self.user_defined_publisher_list
            .write_lock()
            .push(publisher_impl_shared.clone());

        Ok(publisher_impl_shared)
    }

    pub fn delete_publisher(&self, a_publisher_handle: InstanceHandle) -> DdsResult<()> {
        if !self
            .user_defined_publisher_list
            .read_lock()
            .iter()
            .find(|&x| x.get_instance_handle() == a_publisher_handle)
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(
                    "Publisher can only be deleted from its parent participant".to_string(),
                )
            })?
            .is_empty()
        {
            return Err(DdsError::PreconditionNotMet(
                "Publisher still contains data writers".to_string(),
            ));
        }

        self.user_defined_publisher_list
            .write_lock()
            .retain(|x| x.get_instance_handle() != a_publisher_handle);

        Ok(())
    }

    pub fn create_subscriber(
        &self,
        qos: QosKind<SubscriberQos>,
        _a_listener: Option<Box<dyn SubscriberListener>>,
        _mask: &[StatusKind],
    ) -> DdsResult<DdsShared<UserDefinedSubscriber>> {
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
        let subscriber_shared = UserDefinedSubscriber::new(
            subscriber_qos,
            rtps_group,
            self.downgrade(),
            self.user_defined_data_send_condvar.clone(),
        );
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
        if !self
            .user_defined_subscriber_list
            .read_lock()
            .iter()
            .find(|&x| x.get_instance_handle() == a_subscriber_handle)
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(
                    "Subscriber can only be deleted from its parent participant".to_string(),
                )
            })?
            .is_empty()
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
        if self
            .topic_list
            .read_lock()
            .iter()
            .find(|&topic| topic.get_instance_handle() == a_topic_handle)
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(
                    "Topic can only be deleted from its parent publisher".to_string(),
                )
            })?
            .strong_count()
            > 1
        {
            return Err(DdsError::PreconditionNotMet(
                "Topic still attached to some data reader or data writer".to_string(),
            ));
        }

        self.topic_list
            .write_lock()
            .retain(|x| x.get_instance_handle() != a_topic_handle);

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
                if topic.get_name() == topic_name && topic.get_type_name() == Foo::type_name() {
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
            if topic.get_name() == topic_name && topic.get_type_name() == Foo::type_name() {
                Some(topic.clone())
            } else {
                None
            }
        })
    }

    pub fn get_builtin_subscriber(&self) -> DdsResult<DdsShared<BuiltInSubscriber>> {
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

        Ok(self
            .discovered_participant_list
            .read_lock()
            .get(&participant_handle)
            .ok_or(DdsError::BadParameter)?
            .dds_participant_data
            .clone())
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

            self.builtin_subscriber.enable()?;
            self.builtin_publisher.enable()?;

            if self
                .qos
                .read_lock()
                .entity_factory
                .autoenable_created_entities
            {
                for publisher in self.user_defined_publisher_list.read_lock().iter() {
                    publisher.enable()?;
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

    pub fn announce_participant(&self) -> DdsResult<()> {
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
        self.builtin_publisher
            .spdp_builtin_participant_writer()
            .write_w_timestamp(
                &spdp_discovered_participant_data,
                None,
                self.get_current_time().unwrap(),
            )
    }

    pub fn add_discovered_participant(
        &self,
        discovered_participant_data: SpdpDiscoveredParticipantData,
    ) {
        if let Ok(participant_discovery) = ParticipantDiscovery::new(
            &discovered_participant_data,
            self.domain_id as i32,
            &self.domain_tag,
        ) {
            self.builtin_publisher
                .sedp_builtin_publications_writer()
                .add_matched_participant(&participant_discovery);

            let sedp_builtin_publication_reader_shared =
                self.builtin_subscriber.sedp_builtin_publications_reader();
            sedp_builtin_publication_reader_shared.add_matched_participant(&participant_discovery);

            self.builtin_publisher
                .sedp_builtin_subscriptions_writer()
                .add_matched_participant(&participant_discovery);

            let sedp_builtin_subscription_reader_shared =
                self.builtin_subscriber.sedp_builtin_subscriptions_reader();
            sedp_builtin_subscription_reader_shared.add_matched_participant(&participant_discovery);

            self.builtin_publisher
                .sedp_builtin_topics_writer()
                .add_matched_participant(&participant_discovery);

            let sedp_builtin_topic_reader_shared =
                self.builtin_subscriber.sedp_builtin_topics_reader();
            sedp_builtin_topic_reader_shared.add_matched_participant(&participant_discovery);

            self.discovered_participant_list.write_lock().insert(
                discovered_participant_data.get_serialized_key().into(),
                discovered_participant_data,
            );
        }
    }

    pub fn default_unicast_locator_list(&self) -> &[Locator] {
        self.rtps_participant.default_unicast_locator_list()
    }

    pub fn default_multicast_locator_list(&self) -> &[Locator] {
        self.rtps_participant.default_multicast_locator_list()
    }

    pub fn send_built_in_data(&self, transport: &mut impl TransportWrite) {
        self.builtin_publisher.send_message(transport);
        self.builtin_subscriber.send_message(transport);
    }

    pub fn receive_built_in_data(
        &self,
        source_locator: Locator,
        message: RtpsMessage,
    ) -> DdsResult<()> {
        MessageReceiver::new(self.get_current_time()?).process_message(
            self.rtps_participant.guid().prefix(),
            core::slice::from_ref(&self.builtin_publisher),
            core::slice::from_ref(&self.builtin_subscriber),
            source_locator,
            &message,
        )
    }

    pub fn send_user_defined_data(&self, transport: &mut impl TransportWrite) {
        for publisher in self.user_defined_publisher_list.read_lock().iter() {
            publisher.send_message(transport)
        }

        for subscriber in self.user_defined_subscriber_list.read_lock().iter() {
            subscriber.send_message(transport)
        }
    }

    pub fn receive_user_defined_data(
        &self,
        source_locator: Locator,
        message: RtpsMessage,
    ) -> DdsResult<()> {
        MessageReceiver::new(self.get_current_time()?).process_message(
            self.rtps_participant.guid().prefix(),
            self.user_defined_publisher_list.read_lock().as_slice(),
            self.user_defined_subscriber_list.read_lock().as_slice(),
            source_locator,
            &message,
        )
    }

    pub fn discover_matched_participants(&self) -> DdsResult<()> {
        let spdp_builtin_participant_data_reader =
            self.builtin_subscriber.spdp_builtin_participant_reader();

        if let Ok(samples) = spdp_builtin_participant_data_reader.take(
            1,
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        ) {
            for discovered_participant_data_sample in samples.into_iter() {
                if let Some(discovered_participant_data) = discovered_participant_data_sample.data {
                    self.add_discovered_participant(discovered_participant_data)
                }
            }
        }

        Ok(())
    }

    pub fn discover_matched_writers(&self) -> DdsResult<()> {
        if let Ok(samples) = self
            .builtin_subscriber
            .sedp_builtin_publications_reader()
            .take(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        {
            for discovered_writer_data_sample in samples.iter() {
                match discovered_writer_data_sample.sample_info.instance_state {
                    InstanceStateKind::Alive => {
                        for subscriber in self.user_defined_subscriber_list.read_lock().iter() {
                            subscriber.add_matched_writer(
                                discovered_writer_data_sample.data.as_ref().unwrap(),
                            );
                        }
                    }
                    InstanceStateKind::NotAliveDisposed => {
                        for subscriber in self.user_defined_subscriber_list.read_lock().iter() {
                            subscriber.remove_matched_writer(
                                discovered_writer_data_sample.sample_info.instance_handle,
                            );
                        }
                    }
                    InstanceStateKind::NotAliveNoWriters => todo!(),
                }
            }
        }

        Ok(())
    }

    pub fn discover_matched_readers(&self) -> DdsResult<()> {
        if let Ok(samples) = self
            .builtin_subscriber
            .sedp_builtin_subscriptions_reader()
            .take(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        {
            for discovered_reader_data_sample in samples.iter() {
                for publisher in self.user_defined_publisher_list.read_lock().iter() {
                    match discovered_reader_data_sample.sample_info.instance_state {
                        InstanceStateKind::Alive => publisher.add_matched_reader(
                            discovered_reader_data_sample.data.as_ref().unwrap(),
                        ),
                        InstanceStateKind::NotAliveDisposed => publisher.remove_matched_reader(
                            discovered_reader_data_sample.sample_info.instance_handle,
                        ),
                        InstanceStateKind::NotAliveNoWriters => todo!(),
                    }
                }
            }
        }

        Ok(())
    }

    pub fn discover_matched_topics(&self) -> DdsResult<()> {
        if let Ok(samples) = self
            .builtin_subscriber
            .sedp_builtin_topics_reader()
            .take::<DiscoveredTopicData>(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        {
            for sample in samples {
                if let Some(topic_data) = sample.data.as_ref() {
                    self.discovered_topic_list.write_lock().insert(
                        topic_data.get_serialized_key().into(),
                        topic_data.topic_builtin_topic_data.clone(),
                    );
                }
            }
        }

        Ok(())
    }

    pub fn announce_created_datawriter(
        &self,
        sedp_discovered_writer_data: DiscoveredWriterData,
    ) -> DdsResult<()> {
        let writer_data = &DiscoveredWriterData {
            writer_proxy: WriterProxy {
                unicast_locator_list: self.default_unicast_locator_list().to_vec(),
                multicast_locator_list: self.default_multicast_locator_list().to_vec(),
                ..sedp_discovered_writer_data.writer_proxy
            },
            ..sedp_discovered_writer_data
        };

        self.builtin_publisher
            .sedp_builtin_publications_writer()
            .write_w_timestamp(writer_data, None, self.get_current_time()?)
    }

    pub fn announce_deleted_datawriter(
        &self,
        sedp_discovered_writer_data: DiscoveredWriterData,
    ) -> DdsResult<()> {
        self.builtin_publisher
            .sedp_builtin_publications_writer()
            .dispose_w_timestamp(&sedp_discovered_writer_data, None, self.get_current_time()?)
    }

    pub fn announce_created_datareader(&self, sedp_discovered_reader_data: DiscoveredReaderData) {
        let reader_data = &DiscoveredReaderData {
            reader_proxy: ReaderProxy {
                unicast_locator_list: self.default_unicast_locator_list().to_vec(),
                multicast_locator_list: self.default_multicast_locator_list().to_vec(),
                ..sedp_discovered_reader_data.reader_proxy
            },
            ..sedp_discovered_reader_data
        };

        self.builtin_publisher
            .sedp_builtin_subscriptions_writer()
            .write_w_timestamp(reader_data, None, self.get_current_time().unwrap())
            .unwrap();
    }

    pub fn announce_deleted_datareader(
        &self,
        sedp_discovered_reader_data: DiscoveredReaderData,
    ) -> DdsResult<()> {
        self.builtin_publisher
            .sedp_builtin_subscriptions_writer()
            .dispose_w_timestamp(&sedp_discovered_reader_data, None, self.get_current_time()?)
    }

    pub fn update_communication_status(&self) -> DdsResult<()> {
        let now = self.get_current_time()?;
        for subscriber in self.user_defined_subscriber_list.read_lock().iter() {
            subscriber.update_communication_status(now);
        }

        Ok(())
    }
}
