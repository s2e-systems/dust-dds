use dust_dds_derive::actor_interface;
use tracing::warn;

use crate::{
    builtin_topics::{
        BuiltInTopicKey, ParticipantBuiltinTopicData, PublicationBuiltinTopicData,
        SubscriptionBuiltinTopicData, TopicBuiltinTopicData,
    },
    data_representation_builtin_endpoints::{
        discovered_reader_data::{DiscoveredReaderData, ReaderProxy, DCPS_SUBSCRIPTION},
        discovered_topic_data::{DiscoveredTopicData, DCPS_TOPIC},
        discovered_writer_data::{DiscoveredWriterData, WriterProxy, DCPS_PUBLICATION},
        spdp_discovered_participant_data::{
            ParticipantProxy, SpdpDiscoveredParticipantData, DCPS_PARTICIPANT,
        },
    },
    dds::infrastructure,
    dds_async::{
        domain_participant::DomainParticipantAsync,
        domain_participant_listener::DomainParticipantListenerAsync,
        publisher_listener::PublisherListenerAsync, subscriber_listener::SubscriberListenerAsync,
        topic_listener::TopicListenerAsync,
    },
    domain::domain_participant_factory::DomainId,
    implementation::{
        actor::{Actor, ActorAddress, Mail, MailHandler, DEFAULT_ACTOR_BUFFER_SIZE},
        actors::{
            data_reader_actor::DataReaderActor, subscriber_actor::SubscriberActor,
            topic_actor::TopicActor,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantQos, PublisherQos, QosKind, SubscriberQos, TopicQos},
        qos_policy::{
            HistoryQosPolicy, LifespanQosPolicy, ResourceLimitsQosPolicy, TopicDataQosPolicy,
            TransportPriorityQosPolicy,
        },
        status::StatusKind,
        time::{Duration, Time},
    },
    rtps::{
        discovery_types::{
            BuiltinEndpointQos, BuiltinEndpointSet, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
        },
        group::RtpsGroup,
        messages::{overall_structure::RtpsMessageRead, types::Count},
        participant::RtpsParticipant,
        types::{
            EntityId, Guid, Locator, BUILT_IN_READER_GROUP, BUILT_IN_WRITER_GROUP,
            ENTITYID_PARTICIPANT, ENTITYID_UNKNOWN, USER_DEFINED_READER_GROUP, USER_DEFINED_TOPIC,
            USER_DEFINED_WRITER_GROUP,
        },
    },
    subscription::sample_info::{
        InstanceStateKind, ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE,
    },
    topic_definition::type_support::{
        deserialize_rtps_classic_cdr, serialize_rtps_classic_cdr_le, DdsDeserialize, DdsHasKey,
        DdsKey, DdsTypeXml, DynamicTypeInterface,
    },
};

use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use super::{
    data_reader_actor,
    data_writer_actor::{self, DataWriterActor},
    domain_participant_factory_actor::{sedp_data_reader_qos, sedp_data_writer_qos},
    domain_participant_listener_actor::DomainParticipantListenerActor,
    message_sender_actor::MessageSenderActor,
    publisher_actor::{self, PublisherActor},
    status_condition_actor::StatusConditionActor,
    subscriber_actor, topic_actor,
};

const BUILT_IN_TOPIC_NAME_LIST: [&str; 4] = [
    DCPS_PARTICIPANT,
    DCPS_TOPIC,
    DCPS_PUBLICATION,
    DCPS_SUBSCRIPTION,
];

pub struct FooTypeSupport {
    has_key: bool,
    get_serialized_key_from_serialized_foo: fn(&[u8]) -> DdsResult<Vec<u8>>,
    instance_handle_from_serialized_foo: fn(&[u8]) -> DdsResult<InstanceHandle>,
    instance_handle_from_serialized_key: fn(&[u8]) -> DdsResult<InstanceHandle>,
    type_xml: String,
}

impl FooTypeSupport {
    pub fn new<Foo>() -> Self
    where
        Foo: DdsKey + DdsHasKey + DdsTypeXml,
    {
        // This function is a workaround due to an issue resolving
        // lifetimes of the closure.
        // See for more details: https://github.com/rust-lang/rust/issues/41078
        fn define_function_with_correct_lifetime<F, O>(closure: F) -> F
        where
            F: for<'a> Fn(&'a [u8]) -> DdsResult<O>,
        {
            closure
        }

        let get_serialized_key_from_serialized_foo =
            define_function_with_correct_lifetime(|serialized_foo| {
                let mut writer = Vec::new();
                let foo_key = Foo::get_key_from_serialized_data(serialized_foo)?;
                serialize_rtps_classic_cdr_le(&foo_key, &mut writer)?;
                Ok(writer)
            });

        let instance_handle_from_serialized_foo =
            define_function_with_correct_lifetime(|serialized_foo| {
                let foo_key = Foo::get_key_from_serialized_data(serialized_foo)?;
                InstanceHandle::try_from_key(&foo_key)
            });

        let instance_handle_from_serialized_key =
            define_function_with_correct_lifetime(|mut serialized_key| {
                let foo_key = deserialize_rtps_classic_cdr::<Foo::Key>(&mut serialized_key)?;
                InstanceHandle::try_from_key(&foo_key)
            });

        let type_xml = Foo::get_type_xml().unwrap_or(String::new());

        Self {
            has_key: Foo::HAS_KEY,
            get_serialized_key_from_serialized_foo,
            instance_handle_from_serialized_foo,
            instance_handle_from_serialized_key,
            type_xml,
        }
    }
}

impl DynamicTypeInterface for FooTypeSupport {
    fn has_key(&self) -> bool {
        self.has_key
    }

    fn get_serialized_key_from_serialized_foo(&self, serialized_foo: &[u8]) -> DdsResult<Vec<u8>> {
        (self.get_serialized_key_from_serialized_foo)(serialized_foo)
    }

    fn instance_handle_from_serialized_foo(
        &self,
        serialized_foo: &[u8],
    ) -> DdsResult<InstanceHandle> {
        (self.instance_handle_from_serialized_foo)(serialized_foo)
    }

    fn instance_handle_from_serialized_key(
        &self,
        serialized_key: &[u8],
    ) -> DdsResult<InstanceHandle> {
        (self.instance_handle_from_serialized_key)(serialized_key)
    }

    fn xml_type(&self) -> String {
        self.type_xml.clone()
    }
}

pub struct DomainParticipantActor {
    rtps_participant: RtpsParticipant,
    domain_id: DomainId,
    domain_tag: String,
    qos: DomainParticipantQos,
    builtin_subscriber: Actor<SubscriberActor>,
    builtin_publisher: Actor<PublisherActor>,
    user_defined_subscriber_list: HashMap<InstanceHandle, Actor<SubscriberActor>>,
    user_defined_subscriber_counter: u8,
    default_subscriber_qos: SubscriberQos,
    user_defined_publisher_list: HashMap<InstanceHandle, Actor<PublisherActor>>,
    user_defined_publisher_counter: u8,
    default_publisher_qos: PublisherQos,
    topic_list: HashMap<String, Actor<TopicActor>>,
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
    listener: Actor<DomainParticipantListenerActor>,
    status_kind: Vec<StatusKind>,
    status_condition: Actor<StatusConditionActor>,
    message_sender_actor: Actor<MessageSenderActor>,
}

impl DomainParticipantActor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rtps_participant: RtpsParticipant,
        domain_id: DomainId,
        domain_tag: String,
        domain_participant_qos: DomainParticipantQos,
        data_max_size_serialized: usize,
        listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
        topic_list: HashMap<String, Actor<TopicActor>>,
        builtin_data_writer_list: Vec<DataWriterActor>,
        builtin_data_reader_list: Vec<DataReaderActor>,
        message_sender_actor: MessageSenderActor,
        handle: &tokio::runtime::Handle,
    ) -> Self {
        let lease_duration = Duration::new(100, 0);
        let guid_prefix = rtps_participant.guid().prefix();

        let builtin_subscriber = Actor::spawn(
            SubscriberActor::new(
                SubscriberQos::default(),
                RtpsGroup::new(Guid::new(
                    guid_prefix,
                    EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP),
                )),
                None,
                vec![],
                builtin_data_reader_list,
                handle,
            ),
            handle,
            DEFAULT_ACTOR_BUFFER_SIZE,
        );

        let builtin_publisher = Actor::spawn(
            PublisherActor::new(
                PublisherQos::default(),
                RtpsGroup::new(Guid::new(
                    guid_prefix,
                    EntityId::new([0, 0, 0], BUILT_IN_WRITER_GROUP),
                )),
                None,
                vec![],
                builtin_data_writer_list,
                handle,
            ),
            handle,
            DEFAULT_ACTOR_BUFFER_SIZE,
        );

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
            topic_list,
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
            listener: Actor::spawn(
                DomainParticipantListenerActor::new(listener),
                handle,
                DEFAULT_ACTOR_BUFFER_SIZE,
            ),
            status_kind,
            status_condition: Actor::spawn(
                StatusConditionActor::default(),
                handle,
                DEFAULT_ACTOR_BUFFER_SIZE,
            ),
            message_sender_actor: Actor::spawn(
                message_sender_actor,
                handle,
                128, /*Message sender is used by every writer and reader so uses a bigger buffer to prevent blocking on the full channel */
            ),
        }
    }

    async fn lookup_discovered_topic(
        &mut self,
        topic_name: String,
        type_support: Arc<dyn DynamicTypeInterface + Send + Sync>,
        runtime_handle: tokio::runtime::Handle,
    ) -> DdsResult<
        Option<(
            ActorAddress<TopicActor>,
            ActorAddress<StatusConditionActor>,
            String,
        )>,
    > {
        for discovered_topic_data in self.discovered_topic_list.values() {
            if discovered_topic_data.name() == topic_name {
                let qos = TopicQos {
                    topic_data: discovered_topic_data.topic_data().clone(),
                    durability: discovered_topic_data.durability().clone(),
                    deadline: discovered_topic_data.deadline().clone(),
                    latency_budget: discovered_topic_data.latency_budget().clone(),
                    liveliness: discovered_topic_data.liveliness().clone(),
                    reliability: discovered_topic_data.reliability().clone(),
                    destination_order: discovered_topic_data.destination_order().clone(),
                    history: discovered_topic_data.history().clone(),
                    resource_limits: discovered_topic_data.resource_limits().clone(),
                    transport_priority: discovered_topic_data.transport_priority().clone(),
                    lifespan: discovered_topic_data.lifespan().clone(),
                    ownership: discovered_topic_data.ownership().clone(),
                };
                let type_name = discovered_topic_data.get_type_name().to_owned();
                let (topic_address, status_condition_address) = self
                    .create_user_defined_topic(
                        topic_name,
                        type_name.clone(),
                        QosKind::Specific(qos),
                        None,
                        vec![],
                        type_support,
                        runtime_handle,
                    )
                    .await?;
                return Ok(Some((topic_address, status_condition_address, type_name)));
            }
        }
        Ok(None)
    }

    #[allow(clippy::too_many_arguments)]
    async fn create_user_defined_topic(
        &mut self,
        topic_name: String,
        type_name: String,
        qos: QosKind<TopicQos>,
        a_listener: Option<Box<dyn TopicListenerAsync + Send>>,
        _mask: Vec<StatusKind>,
        type_support: Arc<dyn DynamicTypeInterface + Send + Sync>,
        runtime_handle: tokio::runtime::Handle,
    ) -> DdsResult<(ActorAddress<TopicActor>, ActorAddress<StatusConditionActor>)> {
        if let Entry::Vacant(e) = self.topic_list.entry(topic_name.clone()) {
            let qos = match qos {
                QosKind::Default => self.default_topic_qos.clone(),
                QosKind::Specific(q) => q,
            };
            let topic_counter = self.user_defined_topic_counter;
            self.user_defined_topic_counter += 1;
            let entity_id = EntityId::new([topic_counter, 0, 0], USER_DEFINED_TOPIC);
            let guid = Guid::new(self.rtps_participant.guid().prefix(), entity_id);

            let topic = TopicActor::new(
                guid,
                qos,
                type_name,
                &topic_name,
                a_listener,
                type_support,
                &runtime_handle,
            );

            let topic_actor: crate::implementation::actor::Actor<TopicActor> =
                Actor::spawn(topic, &runtime_handle, DEFAULT_ACTOR_BUFFER_SIZE);
            let topic_address = topic_actor.address();
            let topic_status_condition = topic_actor
                .send_actor_mail(topic_actor::GetStatuscondition)
                .await
                .receive_reply()
                .await;
            e.insert(topic_actor);

            Ok((topic_address, topic_status_condition))
        } else {
            Err(DdsError::PreconditionNotMet(format!("Topic with name {} already exists. To access this topic call the lookup_topicdescription method.",topic_name)))
        }
    }

    async fn lookup_topicdescription(
        &self,
        topic_name: String,
    ) -> DdsResult<
        Option<(
            ActorAddress<TopicActor>,
            ActorAddress<StatusConditionActor>,
            String,
        )>,
    > {
        if let Some(topic) = self.topic_list.get(&topic_name) {
            let type_name = topic
                .send_actor_mail(topic_actor::GetTypeName)
                .await
                .receive_reply()
                .await;
            let status_condition = topic
                .send_actor_mail(topic_actor::GetStatuscondition)
                .await
                .receive_reply()
                .await;
            Ok(Some((topic.address(), status_condition, type_name)))
        } else {
            Ok(None)
        }
    }

    fn get_current_time(&self) -> infrastructure::time::Time {
        let now_system_time = SystemTime::now();
        let unix_time = now_system_time
            .duration_since(UNIX_EPOCH)
            .expect("Clock time is before Unix epoch start");
        infrastructure::time::Time::new(unix_time.as_secs() as i32, unix_time.subsec_nanos())
    }
}

pub struct CreateUserDefinedPublisher {
    pub qos: QosKind<PublisherQos>,
    pub a_listener: Option<Box<dyn PublisherListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
    pub runtime_handle: tokio::runtime::Handle,
}
impl Mail for CreateUserDefinedPublisher {
    type Result = (
        ActorAddress<PublisherActor>,
        ActorAddress<StatusConditionActor>,
    );
}
impl MailHandler<CreateUserDefinedPublisher> for DomainParticipantActor {
    async fn handle(
        &mut self,
        message: CreateUserDefinedPublisher,
    ) -> <CreateUserDefinedPublisher as Mail>::Result {
        let publisher_qos = match message.qos {
            QosKind::Default => self.default_publisher_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let publisher_counter = self.user_defined_publisher_counter;
        self.user_defined_publisher_counter += 1;
        let entity_id = EntityId::new([publisher_counter, 0, 0], USER_DEFINED_WRITER_GROUP);
        let guid = Guid::new(self.rtps_participant.guid().prefix(), entity_id);
        let rtps_group = RtpsGroup::new(guid);
        let status_kind = message.mask.to_vec();
        let publisher = PublisherActor::new(
            publisher_qos,
            rtps_group,
            message.a_listener,
            status_kind,
            vec![],
            &message.runtime_handle,
        );

        let publisher_actor = Actor::spawn(
            publisher,
            &message.runtime_handle,
            DEFAULT_ACTOR_BUFFER_SIZE,
        );
        let publisher_address = publisher_actor.address();
        let publisher_status_condition = publisher_actor
            .send_actor_mail(publisher_actor::GetStatuscondition)
            .await
            .receive_reply()
            .await;
        self.user_defined_publisher_list
            .insert(InstanceHandle::new(guid.into()), publisher_actor);

        (publisher_address, publisher_status_condition)
    }
}

pub struct DeleteUserDefinedPublisher {
    pub handle: InstanceHandle,
}
impl Mail for DeleteUserDefinedPublisher {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteUserDefinedPublisher> for DomainParticipantActor {
    async fn handle(
        &mut self,
        message: DeleteUserDefinedPublisher,
    ) -> <DeleteUserDefinedPublisher as Mail>::Result {
        if let Some(p) = self.user_defined_publisher_list.get(&message.handle) {
            if !p
                .send_actor_mail(publisher_actor::GetDataWriterList)
                .await
                .receive_reply()
                .await
                .is_empty()
            {
                Err(DdsError::PreconditionNotMet(
                    "Publisher still contains data writers".to_string(),
                ))
            } else {
                let d = self
                    .user_defined_publisher_list
                    .remove(&message.handle)
                    .expect("Publisher is guaranteed to exist");
                d.stop().await;
                Ok(())
            }
        } else {
            Err(DdsError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant".to_string(),
            ))
        }
    }
}

pub struct CreateUserDefinedSubscriber {
    pub qos: QosKind<SubscriberQos>,
    pub a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
    pub runtime_handle: tokio::runtime::Handle,
}
impl Mail for CreateUserDefinedSubscriber {
    type Result = (
        ActorAddress<SubscriberActor>,
        ActorAddress<StatusConditionActor>,
    );
}
impl MailHandler<CreateUserDefinedSubscriber> for DomainParticipantActor {
    async fn handle(
        &mut self,
        message: CreateUserDefinedSubscriber,
    ) -> <CreateUserDefinedSubscriber as Mail>::Result {
        let subscriber_qos = match message.qos {
            QosKind::Default => self.default_subscriber_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let subcriber_counter = self.user_defined_subscriber_counter;
        self.user_defined_subscriber_counter += 1;
        let entity_id = EntityId::new([subcriber_counter, 0, 0], USER_DEFINED_READER_GROUP);
        let guid = Guid::new(self.rtps_participant.guid().prefix(), entity_id);
        let rtps_group = RtpsGroup::new(guid);
        let status_kind = message.mask.to_vec();

        let subscriber = SubscriberActor::new(
            subscriber_qos,
            rtps_group,
            message.a_listener,
            status_kind,
            vec![],
            &message.runtime_handle,
        );

        let subscriber_actor = Actor::spawn(
            subscriber,
            &message.runtime_handle,
            DEFAULT_ACTOR_BUFFER_SIZE,
        );
        let subscriber_address = subscriber_actor.address();
        let subscriber_status_condition = subscriber_actor
            .send_actor_mail(subscriber_actor::GetStatuscondition)
            .await
            .receive_reply()
            .await;

        self.user_defined_subscriber_list
            .insert(InstanceHandle::new(guid.into()), subscriber_actor);

        (subscriber_address, subscriber_status_condition)
    }
}

pub struct DeleteUserDefinedSubscriber {
    pub handle: InstanceHandle,
}
impl Mail for DeleteUserDefinedSubscriber {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteUserDefinedSubscriber> for DomainParticipantActor {
    async fn handle(
        &mut self,
        message: DeleteUserDefinedSubscriber,
    ) -> <DeleteUserDefinedSubscriber as Mail>::Result {
        if let Some(subscriber) = self.user_defined_subscriber_list.get(&message.handle) {
            if !subscriber
                .send_actor_mail(subscriber_actor::IsEmpty)
                .await
                .receive_reply()
                .await
            {
                Err(DdsError::PreconditionNotMet(
                    "Subscriber still contains data readers".to_string(),
                ))
            } else {
                let d = self
                    .user_defined_subscriber_list
                    .remove(&message.handle)
                    .expect("Subscriber is guaranteed to exist");
                d.stop().await;
                Ok(())
            }
        } else {
            Err(DdsError::PreconditionNotMet(
                "Subscriber can only be deleted from its parent participant".to_string(),
            ))
        }
    }
}

pub struct CreateUserDefinedTopic {
    pub topic_name: String,
    pub type_name: String,
    pub qos: QosKind<TopicQos>,
    pub a_listener: Option<Box<dyn TopicListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
    pub type_support: Arc<dyn DynamicTypeInterface + Send + Sync>,
    pub runtime_handle: tokio::runtime::Handle,
}
impl Mail for CreateUserDefinedTopic {
    type Result = DdsResult<(ActorAddress<TopicActor>, ActorAddress<StatusConditionActor>)>;
}
impl MailHandler<CreateUserDefinedTopic> for DomainParticipantActor {
    async fn handle(
        &mut self,
        message: CreateUserDefinedTopic,
    ) -> <CreateUserDefinedTopic as Mail>::Result {
        self.create_user_defined_topic(
            message.topic_name,
            message.type_name,
            message.qos,
            message.a_listener,
            message.mask,
            message.type_support,
            message.runtime_handle,
        )
        .await
    }
}

pub struct DeleteUserDefinedTopic {
    pub topic_name: String,
}
impl Mail for DeleteUserDefinedTopic {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteUserDefinedTopic> for DomainParticipantActor {
    async fn handle(
        &mut self,
        message: DeleteUserDefinedTopic,
    ) -> <DeleteUserDefinedTopic as Mail>::Result {
        if self.topic_list.contains_key(&message.topic_name) {
            if !BUILT_IN_TOPIC_NAME_LIST.contains(&message.topic_name.as_ref()) {
                for publisher in self.user_defined_publisher_list.values() {
                    if publisher
                        .send_actor_mail(publisher_actor::LookupDatawriter {
                            topic_name: message.topic_name.clone(),
                        })
                        .await
                        .receive_reply()
                        .await?
                        .is_some()
                    {
                        return Err(DdsError::PreconditionNotMet(
                            "Topic still attached to some data writer".to_string(),
                        ));
                    }
                }

                for subscriber in self.user_defined_subscriber_list.values() {
                    if subscriber
                        .send_actor_mail(subscriber_actor::LookupDatareader {
                            topic_name: message.topic_name.clone(),
                        })
                        .await
                        .receive_reply()
                        .await
                        .is_some()
                    {
                        return Err(DdsError::PreconditionNotMet(
                            "Topic still attached to some data reader".to_string(),
                        ));
                    }
                }

                let d = self
                    .topic_list
                    .remove(&message.topic_name)
                    .expect("Topic is guaranteed to exist");
                d.stop().await;
            }
            Ok(())
        } else {
            Err(DdsError::PreconditionNotMet(
                "Topic can only be deleted from its parent participant".to_string(),
            ))
        }
    }
}

pub struct FindTopic {
    pub topic_name: String,
    pub type_support: Arc<dyn DynamicTypeInterface + Send + Sync>,
    pub runtime_handle: tokio::runtime::Handle,
}
impl Mail for FindTopic {
    type Result = DdsResult<
        Option<(
            ActorAddress<TopicActor>,
            ActorAddress<StatusConditionActor>,
            String,
        )>,
    >;
}
impl MailHandler<FindTopic> for DomainParticipantActor {
    async fn handle(&mut self, message: FindTopic) -> <FindTopic as Mail>::Result {
        if let Some(r) = self
            .lookup_topicdescription(message.topic_name.clone())
            .await?
        {
            Ok(Some(r))
        } else {
            self.lookup_discovered_topic(
                message.topic_name.clone(),
                message.type_support.clone(),
                message.runtime_handle.clone(),
            )
            .await
        }
    }
}

pub struct LookupTopicdescription {
    pub topic_name: String,
}
impl Mail for LookupTopicdescription {
    type Result = DdsResult<
        Option<(
            ActorAddress<TopicActor>,
            ActorAddress<StatusConditionActor>,
            String,
        )>,
    >;
}
impl MailHandler<LookupTopicdescription> for DomainParticipantActor {
    async fn handle(
        &mut self,
        message: LookupTopicdescription,
    ) -> <LookupTopicdescription as Mail>::Result {
        self.lookup_topicdescription(message.topic_name).await
    }
}

pub struct GetInstanceHandle;
impl Mail for GetInstanceHandle {
    type Result = InstanceHandle;
}
impl MailHandler<GetInstanceHandle> for DomainParticipantActor {
    async fn handle(&mut self, _: GetInstanceHandle) -> <GetInstanceHandle as Mail>::Result {
        InstanceHandle::new(self.rtps_participant.guid().into())
    }
}

pub struct Enable {
    pub runtime_handle: tokio::runtime::Handle,
}
impl Mail for Enable {
    type Result = DdsResult<()>;
}
impl MailHandler<Enable> for DomainParticipantActor {
    async fn handle(&mut self, message: Enable) -> <Enable as Mail>::Result {
        if !self.enabled {
            self.builtin_publisher
                .send_actor_mail(publisher_actor::Enable)
                .await
                .receive_reply()
                .await;
            self.builtin_subscriber
                .send_actor_mail(subscriber_actor::Enable)
                .await
                .receive_reply()
                .await;

            for builtin_reader in self
                .builtin_subscriber
                .send_actor_mail(subscriber_actor::GetDataReaderList)
                .await
                .receive_reply()
                .await
            {
                builtin_reader
                    .send_actor_mail(data_reader_actor::Enable)
                    .await?
                    .receive_reply()
                    .await;
            }
            for builtin_writer in self
                .builtin_publisher
                .send_actor_mail(publisher_actor::GetDataWriterList)
                .await
                .receive_reply()
                .await
            {
                builtin_writer
                    .send_actor_mail(data_writer_actor::Enable {
                        data_writer_address: builtin_writer.clone(),
                        message_sender_actor: self.message_sender_actor.address(),
                        runtime_handle: message.runtime_handle.clone(),
                    })
                    .await?
                    .receive_reply()
                    .await;
            }
            self.enabled = true;
        }
        Ok(())
    }
}

pub struct IsEnabled;
impl Mail for IsEnabled {
    type Result = bool;
}
impl MailHandler<IsEnabled> for DomainParticipantActor {
    async fn handle(&mut self, _: IsEnabled) -> <IsEnabled as Mail>::Result {
        self.enabled
    }
}

pub struct IgnoreParticipant {
    pub handle: InstanceHandle,
}
impl Mail for IgnoreParticipant {
    type Result = DdsResult<()>;
}
impl MailHandler<IgnoreParticipant> for DomainParticipantActor {
    async fn handle(&mut self, message: IgnoreParticipant) -> <IgnoreParticipant as Mail>::Result {
        if self.enabled {
            self.ignored_participants.insert(message.handle);
            Ok(())
        } else {
            Err(DdsError::NotEnabled)
        }
    }
}

pub struct IgnoreSubscription {
    pub handle: InstanceHandle,
}
impl Mail for IgnoreSubscription {
    type Result = DdsResult<()>;
}
impl MailHandler<IgnoreSubscription> for DomainParticipantActor {
    async fn handle(
        &mut self,
        message: IgnoreSubscription,
    ) -> <IgnoreSubscription as Mail>::Result {
        if self.enabled {
            self.ignored_subcriptions.insert(message.handle);
            Ok(())
        } else {
            Err(DdsError::NotEnabled)
        }
    }
}

pub struct IgnorePublication {
    pub handle: InstanceHandle,
}
impl Mail for IgnorePublication {
    type Result = DdsResult<()>;
}
impl MailHandler<IgnorePublication> for DomainParticipantActor {
    async fn handle(&mut self, message: IgnorePublication) -> <IgnorePublication as Mail>::Result {
        if self.enabled {
            self.ignored_publications.insert(message.handle);
            Ok(())
        } else {
            Err(DdsError::NotEnabled)
        }
    }
}

pub struct IsEmpty;
impl Mail for IsEmpty {
    type Result = bool;
}
impl MailHandler<IsEmpty> for DomainParticipantActor {
    async fn handle(&mut self, _: IsEmpty) -> <IsEmpty as Mail>::Result {
        let no_user_defined_topics = self
            .topic_list
            .keys()
            .filter(|t| !BUILT_IN_TOPIC_NAME_LIST.contains(&t.as_ref()))
            .count()
            == 0;

        self.user_defined_publisher_list.is_empty()
            && self.user_defined_subscriber_list.is_empty()
            && no_user_defined_topics
    }
}

pub struct GetQos;
impl Mail for GetQos {
    type Result = DomainParticipantQos;
}
impl MailHandler<GetQos> for DomainParticipantActor {
    async fn handle(&mut self, _: GetQos) -> <GetQos as Mail>::Result {
        self.qos.clone()
    }
}

pub struct GetDefaultUnicastLocatorList;
impl Mail for GetDefaultUnicastLocatorList {
    type Result = Vec<Locator>;
}
impl MailHandler<GetDefaultUnicastLocatorList> for DomainParticipantActor {
    async fn handle(
        &mut self,
        _: GetDefaultUnicastLocatorList,
    ) -> <GetDefaultUnicastLocatorList as Mail>::Result {
        self.rtps_participant
            .default_unicast_locator_list()
            .to_vec()
    }
}

pub struct GetDefaultMulticastLocatorList;
impl Mail for GetDefaultMulticastLocatorList {
    type Result = Vec<Locator>;
}
impl MailHandler<GetDefaultMulticastLocatorList> for DomainParticipantActor {
    async fn handle(
        &mut self,
        _: GetDefaultMulticastLocatorList,
    ) -> <GetDefaultMulticastLocatorList as Mail>::Result {
        self.rtps_participant
            .default_multicast_locator_list()
            .to_vec()
    }
}

pub struct GetMetatrafficUnicastLocatorList;
impl Mail for GetMetatrafficUnicastLocatorList {
    type Result = Vec<Locator>;
}
impl MailHandler<GetMetatrafficUnicastLocatorList> for DomainParticipantActor {
    async fn handle(
        &mut self,
        _: GetMetatrafficUnicastLocatorList,
    ) -> <GetMetatrafficUnicastLocatorList as Mail>::Result {
        self.rtps_participant
            .metatraffic_unicast_locator_list()
            .to_vec()
    }
}

pub struct GetMetatrafficMulticastLocatorList;
impl Mail for GetMetatrafficMulticastLocatorList {
    type Result = Vec<Locator>;
}
impl MailHandler<GetMetatrafficMulticastLocatorList> for DomainParticipantActor {
    async fn handle(
        &mut self,
        _: GetMetatrafficMulticastLocatorList,
    ) -> <GetMetatrafficMulticastLocatorList as Mail>::Result {
        self.rtps_participant
            .metatraffic_multicast_locator_list()
            .to_vec()
    }
}

pub struct GetDataMaxSizeSerialized;
impl Mail for GetDataMaxSizeSerialized {
    type Result = usize;
}
impl MailHandler<GetDataMaxSizeSerialized> for DomainParticipantActor {
    async fn handle(
        &mut self,
        _: GetDataMaxSizeSerialized,
    ) -> <GetDataMaxSizeSerialized as Mail>::Result {
        self.data_max_size_serialized
    }
}

pub struct DrainSubscriberList;
impl Mail for DrainSubscriberList {
    type Result = Vec<Actor<SubscriberActor>>;
}
impl MailHandler<DrainSubscriberList> for DomainParticipantActor {
    async fn handle(&mut self, _: DrainSubscriberList) -> <DrainSubscriberList as Mail>::Result {
        self.user_defined_subscriber_list
            .drain()
            .map(|(_, a)| a)
            .collect()
    }
}

pub struct DrainPublisherList;
impl Mail for DrainPublisherList {
    type Result = Vec<Actor<PublisherActor>>;
}
impl MailHandler<DrainPublisherList> for DomainParticipantActor {
    async fn handle(&mut self, _: DrainPublisherList) -> <DrainPublisherList as Mail>::Result {
        self.user_defined_publisher_list
            .drain()
            .map(|(_, a)| a)
            .collect()
    }
}

pub struct DrainTopicList;
impl Mail for DrainTopicList {
    type Result = Vec<Actor<TopicActor>>;
}
impl MailHandler<DrainTopicList> for DomainParticipantActor {
    async fn handle(&mut self, _: DrainTopicList) -> <DrainTopicList as Mail>::Result {
        let mut drained_topic_list = Vec::new();
        let user_defined_topic_name_list: Vec<String> = self
            .topic_list
            .keys()
            .filter(|&k| !BUILT_IN_TOPIC_NAME_LIST.contains(&k.as_ref()))
            .cloned()
            .collect();
        for t in user_defined_topic_name_list {
            if let Some(removed_topic) = self.topic_list.remove(&t) {
                drained_topic_list.push(removed_topic);
            }
        }
        drained_topic_list
    }
}

pub struct SetDefaultPublisherQos {
    pub qos: QosKind<PublisherQos>,
}
impl Mail for SetDefaultPublisherQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDefaultPublisherQos> for DomainParticipantActor {
    async fn handle(
        &mut self,
        message: SetDefaultPublisherQos,
    ) -> <SetDefaultPublisherQos as Mail>::Result {
        let qos = match message.qos {
            QosKind::Default => PublisherQos::default(),
            QosKind::Specific(q) => q,
        };

        self.default_publisher_qos = qos;

        Ok(())
    }
}

pub struct SetDefaultSubscriberQos {
    pub qos: QosKind<SubscriberQos>,
}
impl Mail for SetDefaultSubscriberQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDefaultSubscriberQos> for DomainParticipantActor {
    async fn handle(
        &mut self,
        message: SetDefaultSubscriberQos,
    ) -> <SetDefaultSubscriberQos as Mail>::Result {
        let qos = match message.qos {
            QosKind::Default => SubscriberQos::default(),
            QosKind::Specific(q) => q,
        };

        self.default_subscriber_qos = qos;

        Ok(())
    }
}

pub struct SetDefaultTopicQos {
    pub qos: QosKind<TopicQos>,
}
impl Mail for SetDefaultTopicQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDefaultTopicQos> for DomainParticipantActor {
    async fn handle(
        &mut self,
        message: SetDefaultTopicQos,
    ) -> <SetDefaultTopicQos as Mail>::Result {
        let qos = match message.qos {
            QosKind::Default => TopicQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        self.default_topic_qos = qos;

        Ok(())
    }
}

pub struct GetDefaultPublisherQos;
impl Mail for GetDefaultPublisherQos {
    type Result = PublisherQos;
}
impl MailHandler<GetDefaultPublisherQos> for DomainParticipantActor {
    async fn handle(
        &mut self,
        _: GetDefaultPublisherQos,
    ) -> <GetDefaultPublisherQos as Mail>::Result {
        self.default_publisher_qos.clone()
    }
}

pub struct GetDefaultSubscriberQos;
impl Mail for GetDefaultSubscriberQos {
    type Result = SubscriberQos;
}
impl MailHandler<GetDefaultSubscriberQos> for DomainParticipantActor {
    async fn handle(
        &mut self,
        _: GetDefaultSubscriberQos,
    ) -> <GetDefaultSubscriberQos as Mail>::Result {
        self.default_subscriber_qos.clone()
    }
}

pub struct GetDefaultTopicQos;
impl Mail for GetDefaultTopicQos {
    type Result = TopicQos;
}
impl MailHandler<GetDefaultTopicQos> for DomainParticipantActor {
    async fn handle(&mut self, _: GetDefaultTopicQos) -> <GetDefaultTopicQos as Mail>::Result {
        self.default_topic_qos.clone()
    }
}

pub struct GetDiscoveredParticipants;
impl Mail for GetDiscoveredParticipants {
    type Result = Vec<InstanceHandle>;
}
impl MailHandler<GetDiscoveredParticipants> for DomainParticipantActor {
    async fn handle(
        &mut self,
        _: GetDiscoveredParticipants,
    ) -> <GetDiscoveredParticipants as Mail>::Result {
        self.discovered_participant_list.keys().cloned().collect()
    }
}

pub struct GetDiscoveredParticipantData {
    pub participant_handle: InstanceHandle,
}
impl Mail for GetDiscoveredParticipantData {
    type Result = DdsResult<ParticipantBuiltinTopicData>;
}
impl MailHandler<GetDiscoveredParticipantData> for DomainParticipantActor {
    async fn handle(
        &mut self,
        message: GetDiscoveredParticipantData,
    ) -> <GetDiscoveredParticipantData as Mail>::Result {
        Ok(self
            .discovered_participant_list
            .get(&message.participant_handle)
            .ok_or(DdsError::PreconditionNotMet(
                "Participant with this instance handle not discovered".to_owned(),
            ))?
            .dds_participant_data()
            .clone())
    }
}

pub struct GetDiscoveredTopics;
impl Mail for GetDiscoveredTopics {
    type Result = Vec<InstanceHandle>;
}
impl MailHandler<GetDiscoveredTopics> for DomainParticipantActor {
    async fn handle(&mut self, _: GetDiscoveredTopics) -> <GetDiscoveredTopics as Mail>::Result {
        self.discovered_topic_list.keys().cloned().collect()
    }
}

pub struct GetDiscoveredTopicData {
    pub topic_handle: InstanceHandle,
}
impl Mail for GetDiscoveredTopicData {
    type Result = DdsResult<TopicBuiltinTopicData>;
}
impl MailHandler<GetDiscoveredTopicData> for DomainParticipantActor {
    async fn handle(
        &mut self,
        message: GetDiscoveredTopicData,
    ) -> <GetDiscoveredTopicData as Mail>::Result {
        self.discovered_topic_list
            .get(&message.topic_handle)
            .cloned()
            .ok_or(DdsError::PreconditionNotMet(
                "Topic with this handle not discovered".to_owned(),
            ))
    }
}

pub struct SetQos {
    pub qos: DomainParticipantQos,
}
impl Mail for SetQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetQos> for DomainParticipantActor {
    async fn handle(&mut self, message: SetQos) -> <SetQos as Mail>::Result {
        self.qos = message.qos;
        Ok(())
    }
}

pub struct GetDomainId;
impl Mail for GetDomainId {
    type Result = DomainId;
}
impl MailHandler<GetDomainId> for DomainParticipantActor {
    async fn handle(&mut self, _: GetDomainId) -> <GetDomainId as Mail>::Result {
        self.domain_id
    }
}

pub struct GetBuiltInSubscriber;
impl Mail for GetBuiltInSubscriber {
    type Result = ActorAddress<SubscriberActor>;
}
impl MailHandler<GetBuiltInSubscriber> for DomainParticipantActor {
    async fn handle(&mut self, _: GetBuiltInSubscriber) -> <GetBuiltInSubscriber as Mail>::Result {
        self.builtin_subscriber.address()
    }
}

pub struct AsSpdpDiscoveredParticipantData;
impl Mail for AsSpdpDiscoveredParticipantData {
    type Result = SpdpDiscoveredParticipantData;
}
impl MailHandler<AsSpdpDiscoveredParticipantData> for DomainParticipantActor {
    async fn handle(
        &mut self,
        _: AsSpdpDiscoveredParticipantData,
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
            self.discovered_participant_list.keys().cloned().collect(),
        )
    }
}

pub struct GetStatusKind;
impl Mail for GetStatusKind {
    type Result = Vec<StatusKind>;
}
impl MailHandler<GetStatusKind> for DomainParticipantActor {
    async fn handle(&mut self, _: GetStatusKind) -> <GetStatusKind as Mail>::Result {
        self.status_kind.clone()
    }
}

pub struct GetCurrentTime;
impl Mail for GetCurrentTime {
    type Result = Time;
}
impl MailHandler<GetCurrentTime> for DomainParticipantActor {
    async fn handle(&mut self, _: GetCurrentTime) -> <GetCurrentTime as Mail>::Result {
        self.get_current_time()
    }
}

pub struct GetBuiltinPublisher;
impl Mail for GetBuiltinPublisher {
    type Result = ActorAddress<PublisherActor>;
}
impl MailHandler<GetBuiltinPublisher> for DomainParticipantActor {
    async fn handle(&mut self, _: GetBuiltinPublisher) -> <GetBuiltinPublisher as Mail>::Result {
        self.builtin_publisher.address()
    }
}

pub struct ProcessMetatrafficRtpsMessage {
    pub rtps_message: RtpsMessageRead,
    pub participant: DomainParticipantAsync,
}
impl Mail for ProcessMetatrafficRtpsMessage {
    type Result = DdsResult<()>;
}
impl MailHandler<ProcessMetatrafficRtpsMessage> for DomainParticipantActor {
    async fn handle(
        &mut self,
        message: ProcessMetatrafficRtpsMessage,
    ) -> <ProcessMetatrafficRtpsMessage as Mail>::Result {
        tracing::trace!(
            rtps_message = ?message.rtps_message,
            "Received metatraffic RTPS message"
        );
        let reception_timestamp = self.get_current_time().into();
        let participant_mask_listener = (self.listener.address(), self.status_kind.clone());
        self.builtin_subscriber
            .send_actor_mail(subscriber_actor::ProcessRtpsMessage {
                rtps_message: message.rtps_message.clone(),
                reception_timestamp,
                subscriber_address: self.builtin_subscriber.address(),
                participant: message.participant.clone(),
                participant_mask_listener,
            })
            .await;

        self.builtin_publisher
            .send_actor_mail(publisher_actor::ProcessRtpsMessage {
                rtps_message: message.rtps_message,
                message_sender_actor: self.message_sender_actor.address(),
            })
            .await;

        self.process_builtin_discovery(message.participant).await?;

        Ok(())
    }
}

pub struct ProcessUserDefinedRtpsMessage {
    pub rtps_message: RtpsMessageRead,
    pub participant: DomainParticipantAsync,
}
impl Mail for ProcessUserDefinedRtpsMessage {
    type Result = ();
}
impl MailHandler<ProcessUserDefinedRtpsMessage> for DomainParticipantActor {
    async fn handle(
        &mut self,
        message: ProcessUserDefinedRtpsMessage,
    ) -> <ProcessUserDefinedRtpsMessage as Mail>::Result {
        let participant_mask_listener = (self.listener.address(), self.status_kind.clone());
        for user_defined_subscriber_address in self.user_defined_subscriber_list.values() {
            user_defined_subscriber_address
                .send_actor_mail(subscriber_actor::ProcessRtpsMessage {
                    rtps_message: message.rtps_message.clone(),
                    reception_timestamp: self.get_current_time().into(),
                    subscriber_address: user_defined_subscriber_address.address(),
                    participant: message.participant.clone(),
                    participant_mask_listener: participant_mask_listener.clone(),
                })
                .await;
        }

        for user_defined_publisher_address in self.user_defined_publisher_list.values() {
            user_defined_publisher_address
                .send_actor_mail(publisher_actor::ProcessRtpsMessage {
                    rtps_message: message.rtps_message.clone(),
                    message_sender_actor: self.message_sender_actor.address(),
                })
                .await;
        }
    }
}

#[actor_interface]
impl DomainParticipantActor {
    #[allow(clippy::unused_unit)]
    fn set_listener(
        &mut self,
        listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
        runtime_handle: tokio::runtime::Handle,
    ) {
        self.listener = Actor::spawn(
            DomainParticipantListenerActor::new(listener),
            &runtime_handle,
            DEFAULT_ACTOR_BUFFER_SIZE,
        );
        self.status_kind = status_kind;
    }

    pub fn get_statuscondition(&self) -> ActorAddress<StatusConditionActor> {
        self.status_condition.address()
    }

    fn get_message_sender(&self) -> ActorAddress<MessageSenderActor> {
        self.message_sender_actor.address()
    }

    pub fn set_default_unicast_locator_list(&mut self, list: Vec<Locator>) {
        self.rtps_participant.set_default_unicast_locator_list(list)
    }

    pub fn set_default_multicast_locator_list(&mut self, list: Vec<Locator>) {
        self.rtps_participant
            .set_default_multicast_locator_list(list)
    }

    pub fn set_metatraffic_unicast_locator_list(&mut self, list: Vec<Locator>) {
        self.rtps_participant
            .set_metatraffic_unicast_locator_list(list)
    }

    pub fn set_metatraffic_multicast_locator_list(&mut self, list: Vec<Locator>) {
        self.rtps_participant
            .set_metatraffic_multicast_locator_list(list)
    }

    async fn add_discovered_participant(
        &mut self,
        discovered_participant_data: SpdpDiscoveredParticipantData,
        participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
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
        let is_participant_discovered = self
            .discovered_participant_list
            .contains_key(&discovered_participant_handle);
        if is_domain_id_matching
            && is_domain_tag_matching
            && !is_participant_ignored
            && !is_participant_discovered
        {
            self.add_matched_publications_detector(
                &discovered_participant_data,
                participant.clone(),
            )
            .await?;
            self.add_matched_publications_announcer(
                &discovered_participant_data,
                participant.clone(),
            )
            .await?;
            self.add_matched_subscriptions_detector(
                &discovered_participant_data,
                participant.clone(),
            )
            .await?;
            self.add_matched_subscriptions_announcer(
                &discovered_participant_data,
                participant.clone(),
            )
            .await?;
            self.add_matched_topics_detector(&discovered_participant_data, participant.clone())
                .await?;
            self.add_matched_topics_announcer(&discovered_participant_data, participant.clone())
                .await?;

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
        Ok(())
    }

    async fn remove_discovered_participant(&mut self, handle: InstanceHandle) {
        self.discovered_participant_list.remove(&handle);
    }
}

impl DomainParticipantActor {
    async fn process_builtin_discovery(
        &mut self,
        participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
        self.process_sedp_publications_discovery(participant.clone())
            .await?;
        self.process_sedp_subscriptions_discovery(participant.clone())
            .await?;
        self.process_sedp_topics_discovery().await?;
        Ok(())
    }

    async fn add_matched_publications_detector(
        &self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
        participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
        if discovered_participant_data
            .participant_proxy()
            .available_builtin_endpoints()
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR)
        {
            let participant_mask_listener = (self.listener.address(), self.status_kind.clone());
            let remote_reader_guid = Guid::new(
                discovered_participant_data
                    .participant_proxy()
                    .guid_prefix(),
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let reader_proxy = ReaderProxy::new(
                remote_reader_guid,
                remote_group_entity_id,
                discovered_participant_data
                    .participant_proxy()
                    .metatraffic_unicast_locator_list()
                    .to_vec(),
                discovered_participant_data
                    .participant_proxy()
                    .metatraffic_multicast_locator_list()
                    .to_vec(),
                expects_inline_qos,
            );
            let subscription_builtin_topic_data = SubscriptionBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: remote_reader_guid.into(),
                },
                BuiltInTopicKey::default(),
                DCPS_PUBLICATION.to_owned(),
                "DiscoveredWriterData".to_owned(),
                sedp_data_reader_qos(),
                SubscriberQos::default(),
                TopicDataQosPolicy::default(),
                String::new(),
            );
            let discovered_reader_data =
                DiscoveredReaderData::new(reader_proxy, subscription_builtin_topic_data);
            self.builtin_publisher
                .send_actor_mail(publisher_actor::AddMatchedReader {
                    discovered_reader_data,
                    default_unicast_locator_list: self
                        .rtps_participant
                        .default_unicast_locator_list()
                        .to_vec(),
                    default_multicast_locator_list: self
                        .rtps_participant
                        .default_multicast_locator_list()
                        .to_vec(),
                    publisher_address: self.builtin_publisher.address(),
                    participant,
                    participant_mask_listener,
                })
                .await
                .receive_reply()
                .await?;
        }
        Ok(())
    }

    async fn add_matched_publications_announcer(
        &self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
        participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
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

            let dds_publication_data = PublicationBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: remote_writer_guid.into(),
                },
                BuiltInTopicKey::default(),
                DCPS_PUBLICATION.to_owned(),
                "DiscoveredWriterData".to_owned(),
                sedp_data_writer_qos(),
                PublisherQos::default(),
                TopicDataQosPolicy::default(),
                String::new(),
            );
            let writer_proxy = WriterProxy::new(
                remote_writer_guid,
                remote_group_entity_id,
                discovered_participant_data
                    .participant_proxy()
                    .metatraffic_unicast_locator_list()
                    .to_vec(),
                discovered_participant_data
                    .participant_proxy()
                    .metatraffic_multicast_locator_list()
                    .to_vec(),
                data_max_size_serialized,
            );
            let discovered_writer_data =
                DiscoveredWriterData::new(dds_publication_data, writer_proxy);
            self.builtin_subscriber
                .send_actor_mail(subscriber_actor::AddMatchedWriter {
                    discovered_writer_data,
                    default_unicast_locator_list: vec![],
                    default_multicast_locator_list: vec![],
                    subscriber_address: self.builtin_subscriber.address(),
                    participant,
                    participant_mask_listener: (self.listener.address(), self.status_kind.clone()),
                })
                .await
                .receive_reply()
                .await?;
        }
        Ok(())
    }

    async fn add_matched_subscriptions_detector(
        &self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
        participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
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
            let reader_proxy = ReaderProxy::new(
                remote_reader_guid,
                remote_group_entity_id,
                discovered_participant_data
                    .participant_proxy()
                    .metatraffic_unicast_locator_list()
                    .to_vec(),
                discovered_participant_data
                    .participant_proxy()
                    .metatraffic_multicast_locator_list()
                    .to_vec(),
                expects_inline_qos,
            );
            let subscription_builtin_topic_data = SubscriptionBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: remote_reader_guid.into(),
                },
                BuiltInTopicKey::default(),
                DCPS_SUBSCRIPTION.to_owned(),
                "DiscoveredReaderData".to_owned(),
                sedp_data_reader_qos(),
                SubscriberQos::default(),
                TopicDataQosPolicy::default(),
                String::new(),
            );
            let discovered_reader_data =
                DiscoveredReaderData::new(reader_proxy, subscription_builtin_topic_data);
            self.builtin_publisher
                .send_actor_mail(publisher_actor::AddMatchedReader {
                    discovered_reader_data,
                    default_unicast_locator_list: vec![],
                    default_multicast_locator_list: vec![],
                    publisher_address: self.builtin_publisher.address(),
                    participant,
                    participant_mask_listener: (self.listener.address(), self.status_kind.clone()),
                })
                .await
                .receive_reply()
                .await?;
        }
        Ok(())
    }

    async fn add_matched_subscriptions_announcer(
        &self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
        participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
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

            let writer_proxy = WriterProxy::new(
                remote_writer_guid,
                remote_group_entity_id,
                discovered_participant_data
                    .participant_proxy()
                    .metatraffic_unicast_locator_list()
                    .to_vec(),
                discovered_participant_data
                    .participant_proxy()
                    .metatraffic_multicast_locator_list()
                    .to_vec(),
                data_max_size_serialized,
            );
            let dds_publication_data = PublicationBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: remote_writer_guid.into(),
                },
                BuiltInTopicKey::default(),
                DCPS_SUBSCRIPTION.to_owned(),
                "DiscoveredReaderData".to_owned(),
                sedp_data_writer_qos(),
                PublisherQos::default(),
                TopicDataQosPolicy::default(),
                String::new(),
            );
            let discovered_writer_data =
                DiscoveredWriterData::new(dds_publication_data, writer_proxy);
            self.builtin_subscriber
                .send_actor_mail(subscriber_actor::AddMatchedWriter {
                    discovered_writer_data,
                    default_unicast_locator_list: vec![],
                    default_multicast_locator_list: vec![],
                    subscriber_address: self.builtin_subscriber.address(),
                    participant,
                    participant_mask_listener: (self.listener.address(), self.status_kind.clone()),
                })
                .await
                .receive_reply()
                .await?;
        }

        Ok(())
    }

    async fn add_matched_topics_detector(
        &self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
        participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
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
            let reader_proxy = ReaderProxy::new(
                remote_reader_guid,
                remote_group_entity_id,
                discovered_participant_data
                    .participant_proxy()
                    .metatraffic_unicast_locator_list()
                    .to_vec(),
                discovered_participant_data
                    .participant_proxy()
                    .metatraffic_multicast_locator_list()
                    .to_vec(),
                expects_inline_qos,
            );
            let subscription_builtin_topic_data = SubscriptionBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: remote_reader_guid.into(),
                },
                BuiltInTopicKey::default(),
                DCPS_TOPIC.to_owned(),
                "DiscoveredTopicData".to_owned(),
                sedp_data_reader_qos(),
                SubscriberQos::default(),
                TopicDataQosPolicy::default(),
                String::new(),
            );
            let discovered_reader_data =
                DiscoveredReaderData::new(reader_proxy, subscription_builtin_topic_data);
            self.builtin_publisher
                .send_actor_mail(publisher_actor::AddMatchedReader {
                    discovered_reader_data,
                    default_unicast_locator_list: vec![],
                    default_multicast_locator_list: vec![],
                    publisher_address: self.builtin_publisher.address(),
                    participant,
                    participant_mask_listener: (self.listener.address(), self.status_kind.clone()),
                })
                .await
                .receive_reply()
                .await?;
        }
        Ok(())
    }

    async fn add_matched_topics_announcer(
        &self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
        participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
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

            let writer_proxy = WriterProxy::new(
                remote_writer_guid,
                remote_group_entity_id,
                discovered_participant_data
                    .participant_proxy()
                    .metatraffic_unicast_locator_list()
                    .to_vec(),
                discovered_participant_data
                    .participant_proxy()
                    .metatraffic_multicast_locator_list()
                    .to_vec(),
                data_max_size_serialized,
            );
            let dds_publication_data = PublicationBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: remote_writer_guid.into(),
                },
                BuiltInTopicKey::default(),
                DCPS_TOPIC.to_owned(),
                "DiscoveredTopicData".to_owned(),
                sedp_data_writer_qos(),
                PublisherQos::default(),
                TopicDataQosPolicy::default(),
                String::new(),
            );
            let discovered_writer_data =
                DiscoveredWriterData::new(dds_publication_data, writer_proxy);
            self.builtin_subscriber
                .send_actor_mail(subscriber_actor::AddMatchedWriter {
                    discovered_writer_data,
                    default_unicast_locator_list: vec![],
                    default_multicast_locator_list: vec![],
                    subscriber_address: self.builtin_subscriber.address(),
                    participant,
                    participant_mask_listener: (self.listener.address(), self.status_kind.clone()),
                })
                .await
                .receive_reply()
                .await?;
        }
        Ok(())
    }

    async fn process_sedp_publications_discovery(
        &mut self,
        participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
        if let Some(sedp_publications_detector) = self
            .builtin_subscriber
            .send_actor_mail(subscriber_actor::LookupDatareader {
                topic_name: DCPS_PUBLICATION.to_string(),
            })
            .await
            .receive_reply()
            .await
        {
            if let Ok(mut discovered_writer_sample_list) = sedp_publications_detector
                .send_actor_mail(data_reader_actor::Read {
                    max_samples: i32::MAX,
                    sample_states: ANY_SAMPLE_STATE.to_vec(),
                    view_states: ANY_VIEW_STATE.to_vec(),
                    instance_states: ANY_INSTANCE_STATE.to_vec(),
                    specific_instance_handle: None,
                })
                .await?
                .receive_reply()
                .await
            {
                for (discovered_writer_data, discovered_writer_sample_info) in
                    discovered_writer_sample_list.drain(..)
                {
                    match discovered_writer_sample_info.instance_state {
                        InstanceStateKind::Alive => {
                            match DiscoveredWriterData::deserialize_data(
                                discovered_writer_data
                                    .expect("Should contain data")
                                    .as_ref(),
                            ) {
                                Ok(discovered_writer_data) => {
                                    self.add_matched_writer(
                                        discovered_writer_data,
                                        participant.clone(),
                                    )
                                    .await?;
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
                                participant.clone(),
                            )
                            .await?;
                        }
                        InstanceStateKind::NotAliveNoWriters => {
                            todo!()
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn add_matched_writer(
        &mut self,
        discovered_writer_data: DiscoveredWriterData,
        participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
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
                    let participant_mask_listener =
                        (self.listener.address(), self.status_kind.clone());
                    subscriber
                        .send_actor_mail(subscriber_actor::AddMatchedWriter {
                            discovered_writer_data: discovered_writer_data.clone(),
                            default_unicast_locator_list: default_unicast_locator_list.clone(),
                            default_multicast_locator_list: default_multicast_locator_list.clone(),
                            subscriber_address,
                            participant: participant.clone(),
                            participant_mask_listener,
                        })
                        .await
                        .receive_reply()
                        .await?;
                }

                // Add writer topic to discovered topic list using the writer instance handle
                let topic_instance_handle =
                    InstanceHandle::new(discovered_writer_data.dds_publication_data().key().value);
                let writer_topic = TopicBuiltinTopicData::new(
                    BuiltInTopicKey::default(),
                    discovered_writer_data
                        .dds_publication_data()
                        .topic_name()
                        .to_owned(),
                    discovered_writer_data
                        .dds_publication_data()
                        .get_type_name()
                        .to_owned(),
                    TopicQos {
                        topic_data: discovered_writer_data
                            .dds_publication_data()
                            .topic_data()
                            .clone(),
                        durability: discovered_writer_data
                            .dds_publication_data()
                            .durability()
                            .clone(),
                        deadline: discovered_writer_data
                            .dds_publication_data()
                            .deadline()
                            .clone(),
                        latency_budget: discovered_writer_data
                            .dds_publication_data()
                            .latency_budget()
                            .clone(),
                        liveliness: discovered_writer_data
                            .dds_publication_data()
                            .liveliness()
                            .clone(),
                        reliability: discovered_writer_data
                            .dds_publication_data()
                            .reliability()
                            .clone(),
                        destination_order: discovered_writer_data
                            .dds_publication_data()
                            .destination_order()
                            .clone(),
                        history: HistoryQosPolicy::default(),
                        resource_limits: ResourceLimitsQosPolicy::default(),
                        transport_priority: TransportPriorityQosPolicy::default(),
                        lifespan: discovered_writer_data
                            .dds_publication_data()
                            .lifespan()
                            .clone(),
                        ownership: discovered_writer_data
                            .dds_publication_data()
                            .ownership()
                            .clone(),
                    },
                );
                self.discovered_topic_list
                    .insert(topic_instance_handle, writer_topic);
            }
        }
        Ok(())
    }

    async fn remove_matched_writer(
        &self,
        discovered_writer_handle: InstanceHandle,
        participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
        for subscriber in self.user_defined_subscriber_list.values() {
            let subscriber_address = subscriber.address();
            let participant_mask_listener = (self.listener.address(), self.status_kind.clone());
            subscriber
                .send_actor_mail(subscriber_actor::RemoveMatchedWriter {
                    discovered_writer_handle,
                    subscriber_address,
                    participant: participant.clone(),
                    participant_mask_listener,
                })
                .await
                .receive_reply()
                .await?;
        }
        Ok(())
    }

    async fn process_sedp_subscriptions_discovery(
        &mut self,
        participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
        if let Some(sedp_subscriptions_detector) = self
            .builtin_subscriber
            .send_actor_mail(subscriber_actor::LookupDatareader {
                topic_name: DCPS_SUBSCRIPTION.to_string(),
            })
            .await
            .receive_reply()
            .await
        {
            if let Ok(mut discovered_reader_sample_list) = sedp_subscriptions_detector
                .send_actor_mail(data_reader_actor::Read {
                    max_samples: i32::MAX,
                    sample_states: ANY_SAMPLE_STATE.to_vec(),
                    view_states: ANY_VIEW_STATE.to_vec(),
                    instance_states: ANY_INSTANCE_STATE.to_vec(),
                    specific_instance_handle: None,
                })
                .await?
                .receive_reply()
                .await
            {
                for (discovered_reader_data, discovered_reader_sample_info) in
                    discovered_reader_sample_list.drain(..)
                {
                    match discovered_reader_sample_info.instance_state {
                        InstanceStateKind::Alive => {
                            match DiscoveredReaderData::deserialize_data(
                                discovered_reader_data
                                    .expect("Should contain data")
                                    .as_ref(),
                            ) {
                                Ok(discovered_reader_data) => {
                                    self.add_matched_reader(
                                        discovered_reader_data,
                                        participant.clone(),
                                    )
                                    .await?;
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
                                participant.clone(),
                            )
                            .await?;
                        }
                        InstanceStateKind::NotAliveNoWriters => {
                            todo!()
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn add_matched_reader(
        &mut self,
        discovered_reader_data: DiscoveredReaderData,
        participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
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
                    let participant_mask_listener =
                        (self.listener.address(), self.status_kind.clone());

                    publisher
                        .send_actor_mail(publisher_actor::AddMatchedReader {
                            discovered_reader_data: discovered_reader_data.clone(),
                            default_unicast_locator_list: default_unicast_locator_list.clone(),
                            default_multicast_locator_list: default_multicast_locator_list.clone(),
                            publisher_address,
                            participant: participant.clone(),
                            participant_mask_listener,
                        })
                        .await
                        .receive_reply()
                        .await?;
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
                    TopicQos {
                        topic_data: discovered_reader_data
                            .subscription_builtin_topic_data()
                            .topic_data()
                            .clone(),
                        durability: discovered_reader_data
                            .subscription_builtin_topic_data()
                            .durability()
                            .clone(),
                        deadline: discovered_reader_data
                            .subscription_builtin_topic_data()
                            .deadline()
                            .clone(),
                        latency_budget: discovered_reader_data
                            .subscription_builtin_topic_data()
                            .latency_budget()
                            .clone(),
                        liveliness: discovered_reader_data
                            .subscription_builtin_topic_data()
                            .liveliness()
                            .clone(),
                        reliability: discovered_reader_data
                            .subscription_builtin_topic_data()
                            .reliability()
                            .clone(),
                        destination_order: discovered_reader_data
                            .subscription_builtin_topic_data()
                            .destination_order()
                            .clone(),
                        history: HistoryQosPolicy::default(),
                        resource_limits: ResourceLimitsQosPolicy::default(),
                        transport_priority: TransportPriorityQosPolicy::default(),
                        lifespan: LifespanQosPolicy::default(),
                        ownership: discovered_reader_data
                            .subscription_builtin_topic_data()
                            .ownership()
                            .clone(),
                    },
                );
                self.discovered_topic_list
                    .insert(topic_instance_handle, reader_topic);
            }
        }
        Ok(())
    }

    async fn remove_matched_reader(
        &self,
        discovered_reader_handle: InstanceHandle,
        participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
        for publisher in self.user_defined_publisher_list.values() {
            let publisher_address = publisher.address();
            let participant_mask_listener = (self.listener.address(), self.status_kind.clone());
            publisher
                .send_actor_mail(publisher_actor::RemoveMatchedReader {
                    discovered_reader_handle,
                    publisher_address,
                    participant: participant.clone(),
                    participant_mask_listener,
                })
                .await
                .receive_reply()
                .await?;
        }
        Ok(())
    }

    async fn process_sedp_topics_discovery(&mut self) -> DdsResult<()> {
        if let Some(sedp_topics_detector) = self
            .builtin_subscriber
            .send_actor_mail(subscriber_actor::LookupDatareader {
                topic_name: DCPS_TOPIC.to_string(),
            })
            .await
            .receive_reply()
            .await
        {
            if let Ok(mut discovered_topic_sample_list) = sedp_topics_detector
                .send_actor_mail(data_reader_actor::Read {
                    max_samples: i32::MAX,
                    sample_states: ANY_SAMPLE_STATE.to_vec(),
                    view_states: ANY_VIEW_STATE.to_vec(),
                    instance_states: ANY_INSTANCE_STATE.to_vec(),
                    specific_instance_handle: None,
                })
                .await?
                .receive_reply()
                .await
            {
                for (discovered_topic_data, discovered_topic_sample_info) in
                    discovered_topic_sample_list.drain(..)
                {
                    match discovered_topic_sample_info.instance_state {
                        InstanceStateKind::Alive => {
                            match DiscoveredTopicData::deserialize_data(
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
                        // Discovered topics are not deleted so it is not need to process these messages in any manner
                        InstanceStateKind::NotAliveDisposed
                        | InstanceStateKind::NotAliveNoWriters => (),
                    }
                }
            }
        }
        Ok(())
    }

    async fn add_matched_topic(&mut self, discovered_topic_data: DiscoveredTopicData) {
        let handle =
            InstanceHandle::new(discovered_topic_data.topic_builtin_topic_data().key().value);
        let is_topic_ignored = self.ignored_topic_list.contains(&handle);
        if !is_topic_ignored {
            for topic in self.topic_list.values() {
                topic
                    .send_actor_mail(topic_actor::ProcessDiscoveredTopic {
                        discovered_topic_data: discovered_topic_data.clone(),
                    })
                    .await
                    .receive_reply()
                    .await;
            }
            self.discovered_topic_list.insert(
                handle,
                discovered_topic_data.topic_builtin_topic_data().clone(),
            );
        }
    }
}
