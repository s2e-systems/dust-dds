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
        actor::{Actor, ActorAddress, Mail, MailHandler},
        actors::{
            data_reader_actor::DataReaderActor, subscriber_actor::SubscriberActor,
            topic_actor::TopicActor,
        },
        runtime::{
            executor::{block_on, Executor, ExecutorHandle},
            mpsc::{mpsc_channel, MpscSender},
            timer::{TimerDriver, TimerHandle},
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
        status::{
            LivelinessChangedStatus, LivelinessLostStatus, OfferedDeadlineMissedStatus,
            OfferedIncompatibleQosStatus, PublicationMatchedStatus, RequestedDeadlineMissedStatus,
            RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus, StatusKind,
            SubscriptionMatchedStatus,
        },
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
        message_receiver::MessageReceiver,
        messages::{
            overall_structure::{RtpsMessageRead, RtpsSubmessageReadKind},
            types::Count,
        },
        participant::RtpsParticipant,
        types::{
            EntityId, Guid, Locator, BUILT_IN_READER_GROUP, BUILT_IN_WRITER_GROUP,
            ENTITYID_PARTICIPANT, ENTITYID_UNKNOWN, USER_DEFINED_READER_GROUP, USER_DEFINED_TOPIC,
            USER_DEFINED_WRITER_GROUP,
        },
    },
    subscription::sample_info::{
        InstanceStateKind, SampleStateKind, ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE,
    },
    topic_definition::type_support::{
        deserialize_rtps_classic_cdr, serialize_rtps_classic_cdr_le, DdsHasKey, DdsKey, DdsTypeXml,
        DynamicTypeInterface,
    },
};

use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::Arc,
    thread::JoinHandle,
    time::{SystemTime, UNIX_EPOCH},
};

use super::{
    data_writer_actor::DataWriterActor,
    domain_participant_factory_actor::{sedp_data_reader_qos, sedp_data_writer_qos},
    message_sender_actor::MessageSenderActor,
    publisher_actor::{self, PublisherActor},
    status_condition_actor::StatusConditionActor,
    subscriber_actor, topic_actor,
};

pub const BUILT_IN_TOPIC_NAME_LIST: [&str; 4] = [
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
                let foo_key = Foo::get_key_from_serialized_data(serialized_foo)?;
                serialize_rtps_classic_cdr_le(&foo_key)
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

pub enum ParticipantListenerOperation {
    _DataAvailable,
    SampleRejected(SampleRejectedStatus),
    _LivenessChanged(LivelinessChangedStatus),
    RequestedDeadlineMissed(RequestedDeadlineMissedStatus),
    RequestedIncompatibleQos(RequestedIncompatibleQosStatus),
    SubscriptionMatched(SubscriptionMatchedStatus),
    SampleLost(SampleLostStatus),
    _LivelinessLost(LivelinessLostStatus),
    _OfferedDeadlineMissed(OfferedDeadlineMissedStatus),
    OfferedIncompatibleQos(OfferedIncompatibleQosStatus),
    PublicationMatched(PublicationMatchedStatus),
}

pub struct ParticipantListenerMessage {
    pub listener_operation: ParticipantListenerOperation,
}

struct ParticipantListenerThread {
    thread: JoinHandle<()>,
    sender: MpscSender<ParticipantListenerMessage>,
}

impl ParticipantListenerThread {
    fn new(mut listener: Box<dyn DomainParticipantListenerAsync + Send>) -> Self {
        let (sender, receiver) = mpsc_channel::<ParticipantListenerMessage>();
        let thread = std::thread::spawn(move || {
            block_on(async {
                while let Some(m) = receiver.recv().await {
                    match m.listener_operation {
                        ParticipantListenerOperation::_DataAvailable => {
                            listener.on_data_available(&()).await
                        }
                        ParticipantListenerOperation::SampleRejected(status) => {
                            listener.on_sample_rejected(&(), status).await
                        }
                        ParticipantListenerOperation::_LivenessChanged(status) => {
                            listener.on_liveliness_changed(&(), status).await
                        }
                        ParticipantListenerOperation::RequestedDeadlineMissed(status) => {
                            listener.on_requested_deadline_missed(&(), status).await
                        }
                        ParticipantListenerOperation::RequestedIncompatibleQos(status) => {
                            listener.on_requested_incompatible_qos(&(), status).await
                        }
                        ParticipantListenerOperation::SubscriptionMatched(status) => {
                            listener.on_subscription_matched(&(), status).await
                        }
                        ParticipantListenerOperation::SampleLost(status) => {
                            listener.on_sample_lost(&(), status).await
                        }
                        ParticipantListenerOperation::_LivelinessLost(status) => {
                            listener.on_liveliness_lost(&(), status).await
                        }
                        ParticipantListenerOperation::_OfferedDeadlineMissed(status) => {
                            listener.on_offered_deadline_missed(&(), status).await
                        }
                        ParticipantListenerOperation::OfferedIncompatibleQos(status) => {
                            listener.on_offered_incompatible_qos(&(), status).await
                        }
                        ParticipantListenerOperation::PublicationMatched(status) => {
                            listener.on_publication_matched(&(), status).await
                        }
                    }
                }
            });
        });
        Self { thread, sender }
    }

    fn sender(&self) -> &MpscSender<ParticipantListenerMessage> {
        &self.sender
    }

    fn join(self) -> DdsResult<()> {
        self.sender.close();
        self.thread.join()?;
        Ok(())
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
    topic_list: HashMap<String, (Actor<TopicActor>, ActorAddress<StatusConditionActor>)>,
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
    participant_listener_thread: Option<ParticipantListenerThread>,
    status_kind: Vec<StatusKind>,
    status_condition: Actor<StatusConditionActor>,
    message_sender_actor: Actor<MessageSenderActor>,
    executor: Executor,
    timer_driver: TimerDriver,
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
        topic_list: HashMap<String, (Actor<TopicActor>, ActorAddress<StatusConditionActor>)>,
        builtin_data_writer_list: Vec<DataWriterActor>,
        builtin_data_reader_list: Vec<DataReaderActor>,
        message_sender_actor: MessageSenderActor,
        executor: Executor,
        timer_driver: TimerDriver,
    ) -> (
        Self,
        ActorAddress<StatusConditionActor>,
        ActorAddress<SubscriberActor>,
        ActorAddress<StatusConditionActor>,
    ) {
        let lease_duration = Duration::new(100, 0);
        let guid_prefix = rtps_participant.guid().prefix();
        let executor_handle = executor.handle();

        let (builtin_subscriber, builtin_subscriber_status_condition) = SubscriberActor::new(
            SubscriberQos::default(),
            RtpsGroup::new(Guid::new(
                guid_prefix,
                EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP),
            )),
            None,
            vec![],
            builtin_data_reader_list,
            &executor_handle,
        );
        let builtin_subscriber = Actor::spawn(builtin_subscriber, &executor_handle);
        let builtin_subscriber_address = builtin_subscriber.address();

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
                &executor_handle,
            ),
            &executor_handle,
        );

        let status_condition = Actor::spawn(StatusConditionActor::default(), &executor_handle);
        let status_condition_address = status_condition.address();
        let participant_listener_thread = listener.map(ParticipantListenerThread::new);
        (
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
                participant_listener_thread,
                status_kind,
                status_condition,
                message_sender_actor: Actor::spawn(message_sender_actor, &executor_handle),
                executor,
                timer_driver,
            },
            status_condition_address,
            builtin_subscriber_address,
            builtin_subscriber_status_condition,
        )
    }

    fn lookup_discovered_topic(
        &mut self,
        topic_name: String,
        type_support: Arc<dyn DynamicTypeInterface + Send + Sync>,
        executor_handle: ExecutorHandle,
    ) -> DdsResult<Option<(ActorAddress<TopicActor>, ActorAddress<StatusConditionActor>)>> {
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
                let (topic_address, status_condition_address) = self.create_user_defined_topic(
                    topic_name,
                    type_name.clone(),
                    QosKind::Specific(qos),
                    None,
                    vec![],
                    type_support,
                    executor_handle,
                )?;
                return Ok(Some((topic_address, status_condition_address)));
            }
        }
        Ok(None)
    }

    #[allow(clippy::too_many_arguments)]
    fn create_user_defined_topic(
        &mut self,
        topic_name: String,
        type_name: String,
        qos: QosKind<TopicQos>,
        a_listener: Option<Box<dyn TopicListenerAsync + Send>>,
        _mask: Vec<StatusKind>,
        type_support: Arc<dyn DynamicTypeInterface + Send + Sync>,
        executor_handle: ExecutorHandle,
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

            let (topic, topic_status_condition) = TopicActor::new(
                guid,
                qos,
                type_name,
                &topic_name,
                a_listener,
                type_support,
                &executor_handle,
            );

            let topic_actor: crate::implementation::actor::Actor<TopicActor> =
                Actor::spawn(topic, &executor_handle);
            let topic_address = topic_actor.address();

            e.insert((topic_actor, topic_status_condition.clone()));

            Ok((topic_address, topic_status_condition))
        } else {
            Err(DdsError::PreconditionNotMet(format!("Topic with name {} already exists. To access this topic call the lookup_topicdescription method.",topic_name)))
        }
    }

    fn lookup_topicdescription(
        &self,
        topic_name: String,
    ) -> DdsResult<Option<(ActorAddress<TopicActor>, ActorAddress<StatusConditionActor>)>> {
        if let Some((topic, topic_status_condition)) = self.topic_list.get(&topic_name) {
            Ok(Some((topic.address(), topic_status_condition.clone())))
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
    pub executor_handle: ExecutorHandle,
}
impl Mail for CreateUserDefinedPublisher {
    type Result = (
        ActorAddress<PublisherActor>,
        ActorAddress<StatusConditionActor>,
    );
}
impl MailHandler<CreateUserDefinedPublisher> for DomainParticipantActor {
    fn handle(
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
            &message.executor_handle,
        );

        let publisher_status_condition = publisher.get_statuscondition();
        let publisher_actor = Actor::spawn(publisher, &message.executor_handle);
        let publisher_address = publisher_actor.address();

        self.user_defined_publisher_list
            .insert(InstanceHandle::new(guid.into()), publisher_actor);

        (publisher_address, publisher_status_condition)
    }
}

pub struct DeleteUserDefinedPublisher {
    pub handle: InstanceHandle,
}
impl Mail for DeleteUserDefinedPublisher {
    type Result = Option<Actor<PublisherActor>>;
}
impl MailHandler<DeleteUserDefinedPublisher> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteUserDefinedPublisher,
    ) -> <DeleteUserDefinedPublisher as Mail>::Result {
        self.user_defined_publisher_list.remove(&message.handle)
    }
}

pub struct GetPublisherList;
impl Mail for GetPublisherList {
    type Result = Vec<ActorAddress<PublisherActor>>;
}
impl MailHandler<GetPublisherList> for DomainParticipantActor {
    fn handle(&mut self, _: GetPublisherList) -> <GetPublisherList as Mail>::Result {
        self.user_defined_publisher_list
            .values()
            .map(|p| p.address())
            .collect()
    }
}

pub struct CreateUserDefinedSubscriber {
    pub qos: QosKind<SubscriberQos>,
    pub a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
    pub executor_handle: ExecutorHandle,
}
impl Mail for CreateUserDefinedSubscriber {
    type Result = (
        ActorAddress<SubscriberActor>,
        ActorAddress<StatusConditionActor>,
    );
}
impl MailHandler<CreateUserDefinedSubscriber> for DomainParticipantActor {
    fn handle(
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

        let (subscriber, subscriber_status_condition) = SubscriberActor::new(
            subscriber_qos,
            rtps_group,
            message.a_listener,
            status_kind,
            vec![],
            &message.executor_handle,
        );

        let subscriber_actor = Actor::spawn(subscriber, &message.executor_handle);
        let subscriber_address = subscriber_actor.address();

        self.user_defined_subscriber_list
            .insert(InstanceHandle::new(guid.into()), subscriber_actor);

        (subscriber_address, subscriber_status_condition)
    }
}

pub struct DeleteUserDefinedSubscriber {
    pub handle: InstanceHandle,
}
impl Mail for DeleteUserDefinedSubscriber {
    type Result = Option<Actor<SubscriberActor>>;
}
impl MailHandler<DeleteUserDefinedSubscriber> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteUserDefinedSubscriber,
    ) -> <DeleteUserDefinedSubscriber as Mail>::Result {
        self.user_defined_subscriber_list.remove(&message.handle)
    }
}

pub struct GetSubscriberList;
impl Mail for GetSubscriberList {
    type Result = Vec<ActorAddress<SubscriberActor>>;
}
impl MailHandler<GetSubscriberList> for DomainParticipantActor {
    fn handle(&mut self, _: GetSubscriberList) -> <GetSubscriberList as Mail>::Result {
        self.user_defined_subscriber_list
            .values()
            .map(|p| p.address())
            .collect()
    }
}

pub struct CreateUserDefinedTopic {
    pub topic_name: String,
    pub type_name: String,
    pub qos: QosKind<TopicQos>,
    pub a_listener: Option<Box<dyn TopicListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
    pub type_support: Arc<dyn DynamicTypeInterface + Send + Sync>,
    pub executor_handle: ExecutorHandle,
}
impl Mail for CreateUserDefinedTopic {
    type Result = DdsResult<(ActorAddress<TopicActor>, ActorAddress<StatusConditionActor>)>;
}
impl MailHandler<CreateUserDefinedTopic> for DomainParticipantActor {
    fn handle(
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
            message.executor_handle,
        )
    }
}

pub struct DeleteUserDefinedTopic {
    pub topic_name: String,
}
impl Mail for DeleteUserDefinedTopic {
    type Result = Option<Actor<TopicActor>>;
}
impl MailHandler<DeleteUserDefinedTopic> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteUserDefinedTopic,
    ) -> <DeleteUserDefinedTopic as Mail>::Result {
        self.topic_list.remove(&message.topic_name).map(|t| t.0)
    }
}

pub struct FindTopic {
    pub topic_name: String,
    pub type_support: Arc<dyn DynamicTypeInterface + Send + Sync>,
    pub executor_handle: ExecutorHandle,
}
impl Mail for FindTopic {
    type Result = DdsResult<Option<(ActorAddress<TopicActor>, ActorAddress<StatusConditionActor>)>>;
}
impl MailHandler<FindTopic> for DomainParticipantActor {
    fn handle(&mut self, message: FindTopic) -> <FindTopic as Mail>::Result {
        if let Some(r) = self.lookup_topicdescription(message.topic_name.clone())? {
            Ok(Some(r))
        } else {
            self.lookup_discovered_topic(
                message.topic_name.clone(),
                message.type_support.clone(),
                message.executor_handle.clone(),
            )
        }
    }
}

pub struct LookupTopicdescription {
    pub topic_name: String,
}
impl Mail for LookupTopicdescription {
    type Result = DdsResult<Option<(ActorAddress<TopicActor>, ActorAddress<StatusConditionActor>)>>;
}
impl MailHandler<LookupTopicdescription> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: LookupTopicdescription,
    ) -> <LookupTopicdescription as Mail>::Result {
        self.lookup_topicdescription(message.topic_name)
    }
}

pub struct GetInstanceHandle;
impl Mail for GetInstanceHandle {
    type Result = InstanceHandle;
}
impl MailHandler<GetInstanceHandle> for DomainParticipantActor {
    fn handle(&mut self, _: GetInstanceHandle) -> <GetInstanceHandle as Mail>::Result {
        InstanceHandle::new(self.rtps_participant.guid().into())
    }
}

pub struct Enable;
impl Mail for Enable {
    type Result = DdsResult<()>;
}
impl MailHandler<Enable> for DomainParticipantActor {
    fn handle(&mut self, _: Enable) -> <Enable as Mail>::Result {
        self.enabled = true;

        Ok(())
    }
}

pub struct IsEnabled;
impl Mail for IsEnabled {
    type Result = bool;
}
impl MailHandler<IsEnabled> for DomainParticipantActor {
    fn handle(&mut self, _: IsEnabled) -> <IsEnabled as Mail>::Result {
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
    fn handle(&mut self, message: IgnoreParticipant) -> <IgnoreParticipant as Mail>::Result {
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
    fn handle(&mut self, message: IgnoreSubscription) -> <IgnoreSubscription as Mail>::Result {
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
    fn handle(&mut self, message: IgnorePublication) -> <IgnorePublication as Mail>::Result {
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
    fn handle(&mut self, _: IsEmpty) -> <IsEmpty as Mail>::Result {
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
    fn handle(&mut self, _: GetQos) -> <GetQos as Mail>::Result {
        self.qos.clone()
    }
}

pub struct GetDefaultUnicastLocatorList;
impl Mail for GetDefaultUnicastLocatorList {
    type Result = Vec<Locator>;
}
impl MailHandler<GetDefaultUnicastLocatorList> for DomainParticipantActor {
    fn handle(
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
    fn handle(
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
    fn handle(
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
    fn handle(
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
    fn handle(
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
    fn handle(&mut self, _: DrainSubscriberList) -> <DrainSubscriberList as Mail>::Result {
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
    fn handle(&mut self, _: DrainPublisherList) -> <DrainPublisherList as Mail>::Result {
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
    fn handle(&mut self, _: DrainTopicList) -> <DrainTopicList as Mail>::Result {
        let mut drained_topic_list = Vec::new();
        let user_defined_topic_name_list: Vec<String> = self
            .topic_list
            .keys()
            .filter(|&k| !BUILT_IN_TOPIC_NAME_LIST.contains(&k.as_ref()))
            .cloned()
            .collect();
        for t in user_defined_topic_name_list {
            if let Some((removed_topic, _)) = self.topic_list.remove(&t) {
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
    fn handle(
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
    fn handle(
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
    fn handle(&mut self, message: SetDefaultTopicQos) -> <SetDefaultTopicQos as Mail>::Result {
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
    fn handle(&mut self, _: GetDefaultPublisherQos) -> <GetDefaultPublisherQos as Mail>::Result {
        self.default_publisher_qos.clone()
    }
}

pub struct GetDefaultSubscriberQos;
impl Mail for GetDefaultSubscriberQos {
    type Result = SubscriberQos;
}
impl MailHandler<GetDefaultSubscriberQos> for DomainParticipantActor {
    fn handle(&mut self, _: GetDefaultSubscriberQos) -> <GetDefaultSubscriberQos as Mail>::Result {
        self.default_subscriber_qos.clone()
    }
}

pub struct GetDefaultTopicQos;
impl Mail for GetDefaultTopicQos {
    type Result = TopicQos;
}
impl MailHandler<GetDefaultTopicQos> for DomainParticipantActor {
    fn handle(&mut self, _: GetDefaultTopicQos) -> <GetDefaultTopicQos as Mail>::Result {
        self.default_topic_qos.clone()
    }
}

pub struct GetDiscoveredParticipants;
impl Mail for GetDiscoveredParticipants {
    type Result = Vec<InstanceHandle>;
}
impl MailHandler<GetDiscoveredParticipants> for DomainParticipantActor {
    fn handle(
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
    fn handle(
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
    fn handle(&mut self, _: GetDiscoveredTopics) -> <GetDiscoveredTopics as Mail>::Result {
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
    fn handle(
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
    fn handle(&mut self, message: SetQos) -> <SetQos as Mail>::Result {
        self.qos = message.qos;
        Ok(())
    }
}

pub struct GetDomainId;
impl Mail for GetDomainId {
    type Result = DomainId;
}
impl MailHandler<GetDomainId> for DomainParticipantActor {
    fn handle(&mut self, _: GetDomainId) -> <GetDomainId as Mail>::Result {
        self.domain_id
    }
}

pub struct GetBuiltInSubscriber;
impl Mail for GetBuiltInSubscriber {
    type Result = ActorAddress<SubscriberActor>;
}
impl MailHandler<GetBuiltInSubscriber> for DomainParticipantActor {
    fn handle(&mut self, _: GetBuiltInSubscriber) -> <GetBuiltInSubscriber as Mail>::Result {
        self.builtin_subscriber.address()
    }
}

pub struct AsSpdpDiscoveredParticipantData;
impl Mail for AsSpdpDiscoveredParticipantData {
    type Result = SpdpDiscoveredParticipantData;
}
impl MailHandler<AsSpdpDiscoveredParticipantData> for DomainParticipantActor {
    fn handle(
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
    fn handle(&mut self, _: GetStatusKind) -> <GetStatusKind as Mail>::Result {
        self.status_kind.clone()
    }
}

pub struct GetCurrentTime;
impl Mail for GetCurrentTime {
    type Result = Time;
}
impl MailHandler<GetCurrentTime> for DomainParticipantActor {
    fn handle(&mut self, _: GetCurrentTime) -> <GetCurrentTime as Mail>::Result {
        self.get_current_time()
    }
}

pub struct GetBuiltinPublisher;
impl Mail for GetBuiltinPublisher {
    type Result = ActorAddress<PublisherActor>;
}
impl MailHandler<GetBuiltinPublisher> for DomainParticipantActor {
    fn handle(&mut self, _: GetBuiltinPublisher) -> <GetBuiltinPublisher as Mail>::Result {
        self.builtin_publisher.address()
    }
}

pub struct ProcessMetatrafficRtpsMessage {
    pub rtps_message: RtpsMessageRead,
    pub participant: DomainParticipantAsync,
    pub executor_handle: ExecutorHandle,
}
impl Mail for ProcessMetatrafficRtpsMessage {
    type Result = DdsResult<()>;
}
impl MailHandler<ProcessMetatrafficRtpsMessage> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: ProcessMetatrafficRtpsMessage,
    ) -> <ProcessMetatrafficRtpsMessage as Mail>::Result {
        tracing::trace!(
            rtps_message = ?message.rtps_message,
            "Received metatraffic RTPS message"
        );
        let reception_timestamp = self.get_current_time().into();
        let mut message_receiver = MessageReceiver::new(message.rtps_message);
        while let Some(submessage) = message_receiver.next() {
            match submessage {
                RtpsSubmessageReadKind::Data(data_submessage) => {
                    let participant_mask_listener = (
                        self.participant_listener_thread
                            .as_ref()
                            .map(|l| l.sender().clone()),
                        self.status_kind.clone(),
                    );
                    self.builtin_subscriber.send_actor_mail(
                        subscriber_actor::ProcessDataSubmessage {
                            data_submessage,
                            source_guid_prefix: message_receiver.source_guid_prefix(),
                            source_timestamp: message_receiver.source_timestamp(),
                            reception_timestamp,
                            subscriber_address: self.builtin_subscriber.address(),
                            participant: message.participant.clone(),
                            participant_mask_listener,
                            executor_handle: message.executor_handle.clone(),
                            timer_handle: self.timer_driver.handle(),
                        },
                    );
                }
                RtpsSubmessageReadKind::DataFrag(data_frag_submessage) => {
                    let participant_mask_listener = (
                        self.participant_listener_thread
                            .as_ref()
                            .map(|l| l.sender().clone()),
                        self.status_kind.clone(),
                    );
                    self.builtin_subscriber.send_actor_mail(
                        subscriber_actor::ProcessDataFragSubmessage {
                            data_frag_submessage,
                            source_guid_prefix: message_receiver.source_guid_prefix(),
                            source_timestamp: message_receiver.source_timestamp(),
                            reception_timestamp,
                            subscriber_address: self.builtin_subscriber.address(),
                            participant: message.participant.clone(),
                            participant_mask_listener,
                            executor_handle: message.executor_handle.clone(),
                            timer_handle: self.timer_driver.handle(),
                        },
                    );
                }
                RtpsSubmessageReadKind::Gap(gap_submessage) => {
                    self.builtin_subscriber.send_actor_mail(
                        subscriber_actor::ProcessGapSubmessage {
                            gap_submessage,
                            source_guid_prefix: message_receiver.source_guid_prefix(),
                        },
                    );
                }
                RtpsSubmessageReadKind::Heartbeat(heartbeat_submessage) => {
                    self.builtin_subscriber.send_actor_mail(
                        subscriber_actor::ProcessHeartbeatSubmessage {
                            heartbeat_submessage,
                            source_guid_prefix: message_receiver.source_guid_prefix(),
                            message_sender_actor: self.message_sender_actor.address(),
                        },
                    );
                }
                RtpsSubmessageReadKind::HeartbeatFrag(heartbeat_frag_submessage) => {
                    self.builtin_subscriber.send_actor_mail(
                        subscriber_actor::ProcessHeartbeatFragSubmessage {
                            heartbeat_frag_submessage,
                            source_guid_prefix: message_receiver.source_guid_prefix(),
                        },
                    );
                }
                RtpsSubmessageReadKind::AckNack(acknack_submessage) => {
                    self.builtin_publisher.send_actor_mail(
                        publisher_actor::ProcessAckNackSubmessage {
                            acknack_submessage,
                            source_guid_prefix: message_receiver.source_guid_prefix(),
                            message_sender_actor: self.message_sender_actor.address(),
                        },
                    );
                }
                RtpsSubmessageReadKind::NackFrag(nackfrag_submessage) => {
                    self.builtin_publisher.send_actor_mail(
                        publisher_actor::ProcessNackFragSubmessage {
                            nackfrag_submessage,
                            source_guid_prefix: message_receiver.source_guid_prefix(),
                        },
                    );
                }
                _ => (),
            }
        }

        message.executor_handle.spawn(async move {
            process_discovery_data(message.participant.clone())
                .await
                .ok();
        });

        Ok(())
    }
}

pub struct ProcessUserDefinedRtpsMessage {
    pub rtps_message: RtpsMessageRead,
    pub participant: DomainParticipantAsync,
    pub executor_handle: ExecutorHandle,
}
impl Mail for ProcessUserDefinedRtpsMessage {
    type Result = ();
}
impl MailHandler<ProcessUserDefinedRtpsMessage> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: ProcessUserDefinedRtpsMessage,
    ) -> <ProcessUserDefinedRtpsMessage as Mail>::Result {
        let reception_timestamp = self.get_current_time().into();
        let mut message_receiver = MessageReceiver::new(message.rtps_message);
        while let Some(submessage) = message_receiver.next() {
            match submessage {
                RtpsSubmessageReadKind::Data(data_submessage) => {
                    for user_defined_subscriber_actor in self.user_defined_subscriber_list.values()
                    {
                        let participant_mask_listener = (
                            self.participant_listener_thread
                                .as_ref()
                                .map(|l| l.sender().clone()),
                            self.status_kind.clone(),
                        );
                        user_defined_subscriber_actor.send_actor_mail(
                            subscriber_actor::ProcessDataSubmessage {
                                data_submessage: data_submessage.clone(),
                                source_guid_prefix: message_receiver.source_guid_prefix(),
                                source_timestamp: message_receiver.source_timestamp(),
                                reception_timestamp,
                                subscriber_address: user_defined_subscriber_actor.address(),
                                participant: message.participant.clone(),
                                participant_mask_listener,
                                executor_handle: message.executor_handle.clone(),
                                timer_handle: self.timer_driver.handle(),
                            },
                        );
                    }
                }
                RtpsSubmessageReadKind::DataFrag(data_frag_submessage) => {
                    for user_defined_subscriber_actor in self.user_defined_subscriber_list.values()
                    {
                        let participant_mask_listener = (
                            self.participant_listener_thread
                                .as_ref()
                                .map(|l| l.sender().clone()),
                            self.status_kind.clone(),
                        );
                        user_defined_subscriber_actor.send_actor_mail(
                            subscriber_actor::ProcessDataFragSubmessage {
                                data_frag_submessage: data_frag_submessage.clone(),
                                source_guid_prefix: message_receiver.source_guid_prefix(),
                                source_timestamp: message_receiver.source_timestamp(),
                                reception_timestamp,
                                subscriber_address: user_defined_subscriber_actor.address(),
                                participant: message.participant.clone(),
                                participant_mask_listener,
                                executor_handle: message.executor_handle.clone(),
                                timer_handle: self.timer_driver.handle(),
                            },
                        );
                    }
                }
                RtpsSubmessageReadKind::Gap(gap_submessage) => {
                    for user_defined_subscriber_actor in self.user_defined_subscriber_list.values()
                    {
                        user_defined_subscriber_actor.send_actor_mail(
                            subscriber_actor::ProcessGapSubmessage {
                                gap_submessage: gap_submessage.clone(),
                                source_guid_prefix: message_receiver.source_guid_prefix(),
                            },
                        );
                    }
                }
                RtpsSubmessageReadKind::Heartbeat(heartbeat_submessage) => {
                    for user_defined_subscriber_actor in self.user_defined_subscriber_list.values()
                    {
                        user_defined_subscriber_actor.send_actor_mail(
                            subscriber_actor::ProcessHeartbeatSubmessage {
                                heartbeat_submessage: heartbeat_submessage.clone(),
                                source_guid_prefix: message_receiver.source_guid_prefix(),
                                message_sender_actor: self.message_sender_actor.address(),
                            },
                        );
                    }
                }
                RtpsSubmessageReadKind::HeartbeatFrag(heartbeat_frag_submessage) => {
                    for user_defined_subscriber_actor in self.user_defined_subscriber_list.values()
                    {
                        user_defined_subscriber_actor.send_actor_mail(
                            subscriber_actor::ProcessHeartbeatFragSubmessage {
                                heartbeat_frag_submessage: heartbeat_frag_submessage.clone(),
                                source_guid_prefix: message_receiver.source_guid_prefix(),
                            },
                        );
                    }
                }
                RtpsSubmessageReadKind::AckNack(acknack_submessage) => {
                    for user_defined_publisher_actor in self.user_defined_publisher_list.values() {
                        user_defined_publisher_actor.send_actor_mail(
                            publisher_actor::ProcessAckNackSubmessage {
                                acknack_submessage: acknack_submessage.clone(),
                                source_guid_prefix: message_receiver.source_guid_prefix(),
                                message_sender_actor: self.message_sender_actor.address(),
                            },
                        );
                    }
                }
                RtpsSubmessageReadKind::NackFrag(nackfrag_submessage) => {
                    for user_defined_publisher_actor in self.user_defined_publisher_list.values() {
                        user_defined_publisher_actor.send_actor_mail(
                            publisher_actor::ProcessNackFragSubmessage {
                                nackfrag_submessage: nackfrag_submessage.clone(),
                                source_guid_prefix: message_receiver.source_guid_prefix(),
                            },
                        );
                    }
                }
                _ => (),
            }
        }
    }
}

pub struct SetListener {
    pub listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
    pub status_kind: Vec<StatusKind>,
}
impl Mail for SetListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetListener> for DomainParticipantActor {
    fn handle(&mut self, message: SetListener) -> <SetListener as Mail>::Result {
        if let Some(l) = self.participant_listener_thread.take() {
            l.join()?;
        }
        self.participant_listener_thread = message.listener.map(ParticipantListenerThread::new);
        self.status_kind = message.status_kind;
        Ok(())
    }
}

pub struct GetStatuscondition;
impl Mail for GetStatuscondition {
    type Result = ActorAddress<StatusConditionActor>;
}
impl MailHandler<GetStatuscondition> for DomainParticipantActor {
    fn handle(&mut self, _: GetStatuscondition) -> <GetStatuscondition as Mail>::Result {
        self.status_condition.address()
    }
}

pub struct GetMessageSender;
impl Mail for GetMessageSender {
    type Result = ActorAddress<MessageSenderActor>;
}
impl MailHandler<GetMessageSender> for DomainParticipantActor {
    fn handle(&mut self, _: GetMessageSender) -> <GetMessageSender as Mail>::Result {
        self.message_sender_actor.address()
    }
}

pub struct SetDefaultUnicastLocatorList {
    pub list: Vec<Locator>,
}
impl Mail for SetDefaultUnicastLocatorList {
    type Result = ();
}
impl MailHandler<SetDefaultUnicastLocatorList> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetDefaultUnicastLocatorList,
    ) -> <SetDefaultUnicastLocatorList as Mail>::Result {
        self.rtps_participant
            .set_default_unicast_locator_list(message.list)
    }
}

pub struct SetDefaultMulticastLocatorList {
    pub list: Vec<Locator>,
}
impl Mail for SetDefaultMulticastLocatorList {
    type Result = ();
}
impl MailHandler<SetDefaultMulticastLocatorList> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetDefaultMulticastLocatorList,
    ) -> <SetDefaultMulticastLocatorList as Mail>::Result {
        self.rtps_participant
            .set_default_multicast_locator_list(message.list)
    }
}

pub struct SetMetatrafficUnicastLocatorList {
    pub list: Vec<Locator>,
}
impl Mail for SetMetatrafficUnicastLocatorList {
    type Result = ();
}
impl MailHandler<SetMetatrafficUnicastLocatorList> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetMetatrafficUnicastLocatorList,
    ) -> <SetMetatrafficUnicastLocatorList as Mail>::Result {
        self.rtps_participant
            .set_metatraffic_unicast_locator_list(message.list)
    }
}

pub struct SetMetatrafficMulticastLocatorList {
    pub list: Vec<Locator>,
}
impl Mail for SetMetatrafficMulticastLocatorList {
    type Result = ();
}
impl MailHandler<SetMetatrafficMulticastLocatorList> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetMetatrafficMulticastLocatorList,
    ) -> <SetMetatrafficMulticastLocatorList as Mail>::Result {
        self.rtps_participant
            .set_metatraffic_multicast_locator_list(message.list)
    }
}

pub struct AddDiscoveredParticipant {
    pub discovered_participant_data: SpdpDiscoveredParticipantData,
    pub participant: DomainParticipantAsync,
}
impl Mail for AddDiscoveredParticipant {
    type Result = DdsResult<()>;
}
impl MailHandler<AddDiscoveredParticipant> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: AddDiscoveredParticipant,
    ) -> <AddDiscoveredParticipant as Mail>::Result {
        // Check that the domainId of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        // AND
        // Check that the domainTag of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        // IN CASE no domain id was transmitted the a local domain id is assumed
        // (as specified in Table 9.19 - ParameterId mapping and default values)
        let is_domain_id_matching = message
            .discovered_participant_data
            .participant_proxy()
            .domain_id()
            .unwrap_or(self.domain_id)
            == self.domain_id;
        let is_domain_tag_matching = message
            .discovered_participant_data
            .participant_proxy()
            .domain_tag()
            == self.domain_tag;
        let discovered_participant_handle = InstanceHandle::new(
            message
                .discovered_participant_data
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
                &message.discovered_participant_data,
                message.participant.clone(),
            )?;
            self.add_matched_publications_announcer(
                &message.discovered_participant_data,
                message.participant.clone(),
            )?;
            self.add_matched_subscriptions_detector(
                &message.discovered_participant_data,
                message.participant.clone(),
            )?;
            self.add_matched_subscriptions_announcer(
                &message.discovered_participant_data,
                message.participant.clone(),
            )?;
            self.add_matched_topics_detector(
                &message.discovered_participant_data,
                message.participant.clone(),
            )?;
            self.add_matched_topics_announcer(
                &message.discovered_participant_data,
                message.participant.clone(),
            )?;

            self.discovered_participant_list.insert(
                InstanceHandle::new(
                    message
                        .discovered_participant_data
                        .dds_participant_data()
                        .key()
                        .value,
                ),
                message.discovered_participant_data,
            );
        }
        Ok(())
    }
}

pub struct RemoveDiscoveredParticipant {
    pub handle: InstanceHandle,
}
impl Mail for RemoveDiscoveredParticipant {
    type Result = ();
}
impl MailHandler<RemoveDiscoveredParticipant> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: RemoveDiscoveredParticipant,
    ) -> <RemoveDiscoveredParticipant as Mail>::Result {
        self.discovered_participant_list.remove(&message.handle);
    }
}

pub struct AddMatchedWriter {
    pub discovered_writer_data: DiscoveredWriterData,
    pub participant: DomainParticipantAsync,
}
impl Mail for AddMatchedWriter {
    type Result = DdsResult<()>;
}
impl MailHandler<AddMatchedWriter> for DomainParticipantActor {
    fn handle(&mut self, message: AddMatchedWriter) -> <AddMatchedWriter as Mail>::Result {
        let is_participant_ignored = self.ignored_participants.contains(&InstanceHandle::new(
            Guid::new(
                message
                    .discovered_writer_data
                    .writer_proxy()
                    .remote_writer_guid()
                    .prefix(),
                ENTITYID_PARTICIPANT,
            )
            .into(),
        ));
        let is_publication_ignored = self.ignored_publications.contains(&InstanceHandle::new(
            message
                .discovered_writer_data
                .dds_publication_data()
                .key()
                .value,
        ));
        if !is_publication_ignored && !is_participant_ignored {
            if let Some(discovered_participant_data) =
                self.discovered_participant_list.get(&InstanceHandle::new(
                    Guid::new(
                        message
                            .discovered_writer_data
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
                        self.participant_listener_thread
                            .as_ref()
                            .map(|l| l.sender().clone()),
                        self.status_kind.clone(),
                    );
                    subscriber.send_actor_mail(subscriber_actor::AddMatchedWriter {
                        discovered_writer_data: message.discovered_writer_data.clone(),
                        default_unicast_locator_list: default_unicast_locator_list.clone(),
                        default_multicast_locator_list: default_multicast_locator_list.clone(),
                        subscriber_address,
                        participant: message.participant.clone(),
                        participant_mask_listener,
                    });
                }

                // Add writer topic to discovered topic list using the writer instance handle
                let topic_instance_handle = InstanceHandle::new(
                    message
                        .discovered_writer_data
                        .dds_publication_data()
                        .key()
                        .value,
                );
                let writer_topic = TopicBuiltinTopicData::new(
                    BuiltInTopicKey::default(),
                    message
                        .discovered_writer_data
                        .dds_publication_data()
                        .topic_name()
                        .to_owned(),
                    message
                        .discovered_writer_data
                        .dds_publication_data()
                        .get_type_name()
                        .to_owned(),
                    TopicQos {
                        topic_data: message
                            .discovered_writer_data
                            .dds_publication_data()
                            .topic_data()
                            .clone(),
                        durability: message
                            .discovered_writer_data
                            .dds_publication_data()
                            .durability()
                            .clone(),
                        deadline: message
                            .discovered_writer_data
                            .dds_publication_data()
                            .deadline()
                            .clone(),
                        latency_budget: message
                            .discovered_writer_data
                            .dds_publication_data()
                            .latency_budget()
                            .clone(),
                        liveliness: message
                            .discovered_writer_data
                            .dds_publication_data()
                            .liveliness()
                            .clone(),
                        reliability: message
                            .discovered_writer_data
                            .dds_publication_data()
                            .reliability()
                            .clone(),
                        destination_order: message
                            .discovered_writer_data
                            .dds_publication_data()
                            .destination_order()
                            .clone(),
                        history: HistoryQosPolicy::default(),
                        resource_limits: ResourceLimitsQosPolicy::default(),
                        transport_priority: TransportPriorityQosPolicy::default(),
                        lifespan: message
                            .discovered_writer_data
                            .dds_publication_data()
                            .lifespan()
                            .clone(),
                        ownership: message
                            .discovered_writer_data
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
}

pub struct RemoveMatchedWriter {
    pub discovered_writer_handle: InstanceHandle,
    pub participant: DomainParticipantAsync,
}
impl Mail for RemoveMatchedWriter {
    type Result = DdsResult<()>;
}
impl MailHandler<RemoveMatchedWriter> for DomainParticipantActor {
    fn handle(&mut self, message: RemoveMatchedWriter) -> <RemoveMatchedWriter as Mail>::Result {
        for subscriber in self.user_defined_subscriber_list.values() {
            let subscriber_address = subscriber.address();
            let participant_mask_listener = (
                self.participant_listener_thread
                    .as_ref()
                    .map(|l| l.sender().clone()),
                self.status_kind.clone(),
            );
            subscriber.send_actor_mail(subscriber_actor::RemoveMatchedWriter {
                discovered_writer_handle: message.discovered_writer_handle,
                subscriber_address,
                participant: message.participant.clone(),
                participant_mask_listener,
            });
        }
        Ok(())
    }
}

pub struct AddMatchedReader {
    pub discovered_reader_data: DiscoveredReaderData,
    pub participant: DomainParticipantAsync,
}
impl Mail for AddMatchedReader {
    type Result = DdsResult<()>;
}
impl MailHandler<AddMatchedReader> for DomainParticipantActor {
    fn handle(&mut self, message: AddMatchedReader) -> <AddMatchedReader as Mail>::Result {
        let is_participant_ignored = self.ignored_participants.contains(&InstanceHandle::new(
            Guid::new(
                message
                    .discovered_reader_data
                    .reader_proxy()
                    .remote_reader_guid()
                    .prefix(),
                ENTITYID_PARTICIPANT,
            )
            .into(),
        ));
        let is_subscription_ignored = self.ignored_subcriptions.contains(&InstanceHandle::new(
            message
                .discovered_reader_data
                .subscription_builtin_topic_data()
                .key()
                .value,
        ));
        if !is_subscription_ignored && !is_participant_ignored {
            if let Some(discovered_participant_data) =
                self.discovered_participant_list.get(&InstanceHandle::new(
                    Guid::new(
                        message
                            .discovered_reader_data
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
                    let participant_mask_listener = (
                        self.participant_listener_thread
                            .as_ref()
                            .map(|l| l.sender().clone()),
                        self.status_kind.clone(),
                    );

                    publisher.send_actor_mail(publisher_actor::AddMatchedReader {
                        discovered_reader_data: message.discovered_reader_data.clone(),
                        default_unicast_locator_list: default_unicast_locator_list.clone(),
                        default_multicast_locator_list: default_multicast_locator_list.clone(),
                        publisher_address,
                        participant: message.participant.clone(),
                        participant_mask_listener,
                        message_sender_actor: self.message_sender_actor.address(),
                    });
                }

                // Add reader topic to discovered topic list using the reader instance handle
                let topic_instance_handle = InstanceHandle::new(
                    message
                        .discovered_reader_data
                        .subscription_builtin_topic_data()
                        .key()
                        .value,
                );
                let reader_topic = TopicBuiltinTopicData::new(
                    BuiltInTopicKey::default(),
                    message
                        .discovered_reader_data
                        .subscription_builtin_topic_data()
                        .topic_name()
                        .to_string(),
                    message
                        .discovered_reader_data
                        .subscription_builtin_topic_data()
                        .get_type_name()
                        .to_string(),
                    TopicQos {
                        topic_data: message
                            .discovered_reader_data
                            .subscription_builtin_topic_data()
                            .topic_data()
                            .clone(),
                        durability: message
                            .discovered_reader_data
                            .subscription_builtin_topic_data()
                            .durability()
                            .clone(),
                        deadline: message
                            .discovered_reader_data
                            .subscription_builtin_topic_data()
                            .deadline()
                            .clone(),
                        latency_budget: message
                            .discovered_reader_data
                            .subscription_builtin_topic_data()
                            .latency_budget()
                            .clone(),
                        liveliness: message
                            .discovered_reader_data
                            .subscription_builtin_topic_data()
                            .liveliness()
                            .clone(),
                        reliability: message
                            .discovered_reader_data
                            .subscription_builtin_topic_data()
                            .reliability()
                            .clone(),
                        destination_order: message
                            .discovered_reader_data
                            .subscription_builtin_topic_data()
                            .destination_order()
                            .clone(),
                        history: HistoryQosPolicy::default(),
                        resource_limits: ResourceLimitsQosPolicy::default(),
                        transport_priority: TransportPriorityQosPolicy::default(),
                        lifespan: LifespanQosPolicy::default(),
                        ownership: message
                            .discovered_reader_data
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
}

pub struct RemoveMatchedReader {
    pub discovered_reader_handle: InstanceHandle,
    pub participant: DomainParticipantAsync,
}
impl Mail for RemoveMatchedReader {
    type Result = DdsResult<()>;
}
impl MailHandler<RemoveMatchedReader> for DomainParticipantActor {
    fn handle(&mut self, message: RemoveMatchedReader) -> <RemoveMatchedReader as Mail>::Result {
        for publisher in self.user_defined_publisher_list.values() {
            let publisher_address = publisher.address();
            let participant_mask_listener = (
                self.participant_listener_thread
                    .as_ref()
                    .map(|l| l.sender().clone()),
                self.status_kind.clone(),
            );
            publisher.send_actor_mail(publisher_actor::RemoveMatchedReader {
                discovered_reader_handle: message.discovered_reader_handle,
                publisher_address,
                participant: message.participant.clone(),
                participant_mask_listener,
            });
        }
        Ok(())
    }
}

pub struct AddMatchedTopic {
    pub discovered_topic_data: DiscoveredTopicData,
}
impl Mail for AddMatchedTopic {
    type Result = ();
}
impl MailHandler<AddMatchedTopic> for DomainParticipantActor {
    fn handle(&mut self, message: AddMatchedTopic) -> <AddMatchedTopic as Mail>::Result {
        let handle = InstanceHandle::new(
            message
                .discovered_topic_data
                .topic_builtin_topic_data()
                .key()
                .value,
        );
        let is_topic_ignored = self.ignored_topic_list.contains(&handle);
        if !is_topic_ignored {
            for (topic, _) in self.topic_list.values() {
                topic.send_actor_mail(topic_actor::ProcessDiscoveredTopic {
                    discovered_topic_data: message.discovered_topic_data.clone(),
                });
            }
            self.discovered_topic_list.insert(
                handle,
                message
                    .discovered_topic_data
                    .topic_builtin_topic_data()
                    .clone(),
            );
        }
    }
}

pub struct GetExecutorHandle;
impl Mail for GetExecutorHandle {
    type Result = ExecutorHandle;
}
impl MailHandler<GetExecutorHandle> for DomainParticipantActor {
    fn handle(&mut self, _: GetExecutorHandle) -> <GetExecutorHandle as Mail>::Result {
        self.executor.handle()
    }
}

pub struct GetTimerHandle;
impl Mail for GetTimerHandle {
    type Result = TimerHandle;
}
impl MailHandler<GetTimerHandle> for DomainParticipantActor {
    fn handle(&mut self, _: GetTimerHandle) -> <GetTimerHandle as Mail>::Result {
        self.timer_driver.handle()
    }
}

impl DomainParticipantActor {
    fn add_matched_publications_detector(
        &self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
        participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
        if discovered_participant_data
            .participant_proxy()
            .available_builtin_endpoints()
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR)
        {
            let participant_mask_listener = (
                self.participant_listener_thread
                    .as_ref()
                    .map(|l| l.sender().clone()),
                self.status_kind.clone(),
            );
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
                    message_sender_actor: self.message_sender_actor.address(),
                });
        }
        Ok(())
    }

    fn add_matched_publications_announcer(
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
                    participant_mask_listener: (
                        self.participant_listener_thread
                            .as_ref()
                            .map(|l| l.sender().clone()),
                        self.status_kind.clone(),
                    ),
                });
        }
        Ok(())
    }

    fn add_matched_subscriptions_detector(
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
                    participant_mask_listener: (
                        self.participant_listener_thread
                            .as_ref()
                            .map(|l| l.sender().clone()),
                        self.status_kind.clone(),
                    ),
                    message_sender_actor: self.message_sender_actor.address(),
                });
        }
        Ok(())
    }

    fn add_matched_subscriptions_announcer(
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
                    participant_mask_listener: (
                        self.participant_listener_thread
                            .as_ref()
                            .map(|l| l.sender().clone()),
                        self.status_kind.clone(),
                    ),
                });
        }

        Ok(())
    }

    fn add_matched_topics_detector(
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
                    participant_mask_listener: (
                        self.participant_listener_thread
                            .as_ref()
                            .map(|l| l.sender().clone()),
                        self.status_kind.clone(),
                    ),
                    message_sender_actor: self.message_sender_actor.address(),
                });
        }
        Ok(())
    }

    fn add_matched_topics_announcer(
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
                    participant_mask_listener: (
                        self.participant_listener_thread
                            .as_ref()
                            .map(|l| l.sender().clone()),
                        self.status_kind.clone(),
                    ),
                });
        }
        Ok(())
    }
}

async fn process_discovery_data(participant: DomainParticipantAsync) -> DdsResult<()> {
    process_spdp_participant_discovery(&participant).await?;
    process_sedp_publications_discovery(&participant).await?;
    process_sedp_subscriptions_discovery(&participant).await?;
    process_sedp_topics_discovery(&participant).await
}

async fn process_spdp_participant_discovery(participant: &DomainParticipantAsync) -> DdsResult<()> {
    let builtin_subscriber = participant.get_builtin_subscriber();

    if let Ok(Some(spdp_participant_reader)) = builtin_subscriber
        .lookup_datareader::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT)
        .await
    {
        if let Ok(spdp_discovered_participant_list) = spdp_participant_reader
            .read(
                i32::MAX,
                &[SampleStateKind::NotRead],
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
            )
            .await
        {
            for discovered_participant_sample in spdp_discovered_participant_list {
                match discovered_participant_sample.sample_info().instance_state {
                    InstanceStateKind::Alive => {
                        if let Ok(discovered_participant_data) =
                            discovered_participant_sample.data()
                        {
                            participant
                                .participant_address()
                                .send_actor_mail(AddDiscoveredParticipant {
                                    discovered_participant_data,
                                    participant: participant.clone(),
                                })?
                                .receive_reply()
                                .await?;
                        }
                    }
                    InstanceStateKind::NotAliveDisposed | InstanceStateKind::NotAliveNoWriters => {
                        participant
                            .participant_address()
                            .send_actor_mail(RemoveDiscoveredParticipant {
                                handle: discovered_participant_sample.sample_info().instance_handle,
                            })?
                            .receive_reply()
                            .await;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn process_sedp_publications_discovery(
    participant: &DomainParticipantAsync,
) -> DdsResult<()> {
    let builtin_subscriber = participant.get_builtin_subscriber();

    if let Some(sedp_publications_detector) = builtin_subscriber
        .lookup_datareader::<DiscoveredWriterData>(DCPS_PUBLICATION)
        .await?
    {
        if let Ok(mut discovered_writer_sample_list) = sedp_publications_detector
            .read(
                i32::MAX,
                ANY_SAMPLE_STATE,
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
            )
            .await
        {
            for discovered_writer_sample in discovered_writer_sample_list.drain(..) {
                match discovered_writer_sample.sample_info().instance_state {
                    InstanceStateKind::Alive => match discovered_writer_sample.data() {
                        Ok(discovered_writer_data) => {
                            participant.participant_address().send_actor_mail(
                                AddMatchedWriter {
                                    discovered_writer_data,
                                    participant: participant.clone(),
                                },
                            )?;
                        }
                        Err(e) => warn!(
                            "Received invalid DiscoveredWriterData sample. Error {:?}",
                            e
                        ),
                    },
                    InstanceStateKind::NotAliveDisposed => {
                        participant
                            .participant_address()
                            .send_actor_mail(RemoveMatchedWriter {
                                discovered_writer_handle: discovered_writer_sample
                                    .sample_info()
                                    .instance_handle,
                                participant: participant.clone(),
                            })?;
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

async fn process_sedp_subscriptions_discovery(
    participant: &DomainParticipantAsync,
) -> DdsResult<()> {
    let builtin_subscriber = participant.get_builtin_subscriber();

    if let Some(sedp_subscriptions_detector) = builtin_subscriber
        .lookup_datareader::<DiscoveredReaderData>(DCPS_SUBSCRIPTION)
        .await?
    {
        if let Ok(mut discovered_reader_sample_list) = sedp_subscriptions_detector
            .read(
                i32::MAX,
                ANY_SAMPLE_STATE,
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
            )
            .await
        {
            for discovered_reader_sample in discovered_reader_sample_list.drain(..) {
                match discovered_reader_sample.sample_info().instance_state {
                    InstanceStateKind::Alive => match discovered_reader_sample.data() {
                        Ok(discovered_reader_data) => {
                            participant.participant_address().send_actor_mail(
                                AddMatchedReader {
                                    discovered_reader_data,
                                    participant: participant.clone(),
                                },
                            )?;
                        }
                        Err(e) => warn!(
                            "Received invalid DiscoveredReaderData sample. Error {:?}",
                            e
                        ),
                    },
                    InstanceStateKind::NotAliveDisposed => {
                        participant
                            .participant_address()
                            .send_actor_mail(RemoveMatchedReader {
                                discovered_reader_handle: discovered_reader_sample
                                    .sample_info()
                                    .instance_handle,
                                participant: participant.clone(),
                            })?;
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

async fn process_sedp_topics_discovery(participant: &DomainParticipantAsync) -> DdsResult<()> {
    let builtin_subscriber = participant.get_builtin_subscriber();
    if let Some(sedp_topics_detector) = builtin_subscriber
        .lookup_datareader::<DiscoveredTopicData>(DCPS_TOPIC)
        .await?
    {
        if let Ok(mut discovered_topic_sample_list) = sedp_topics_detector
            .read(
                i32::MAX,
                ANY_SAMPLE_STATE,
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
            )
            .await
        {
            for discovered_topic_sample in discovered_topic_sample_list.drain(..) {
                match discovered_topic_sample.sample_info().instance_state {
                    InstanceStateKind::Alive => match discovered_topic_sample.data() {
                        Ok(discovered_topic_data) => {
                            participant
                                .participant_address()
                                .send_actor_mail(AddMatchedTopic {
                                    discovered_topic_data,
                                })?;
                        }
                        Err(e) => {
                            warn!("Received invalid DiscoveredTopicData sample. Error {:?}", e)
                        }
                    },
                    // Discovered topics are not deleted so it is not need to process these messages in any manner
                    InstanceStateKind::NotAliveDisposed | InstanceStateKind::NotAliveNoWriters => {}
                }
            }
        }
    }
    Ok(())
}
