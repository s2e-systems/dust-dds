use super::{
    data_writer_actor::DataWriterActor,
    domain_participant_factory_actor::{sedp_data_reader_qos, sedp_data_writer_qos},
    message_sender_actor::MessageSenderActor,
    publisher_actor::{self, PublisherActor},
    status_condition_actor::StatusConditionActor,
    subscriber_actor, topic_actor,
};
use crate::{
    builtin_topics::{
        BuiltInTopicKey, ParticipantBuiltinTopicData, PublicationBuiltinTopicData,
        SubscriptionBuiltinTopicData, TopicBuiltinTopicData, DCPS_PARTICIPANT, DCPS_PUBLICATION,
        DCPS_SUBSCRIPTION, DCPS_TOPIC,
    },
    dds_async::{
        data_reader::DataReaderAsync, data_writer::DataWriterAsync,
        domain_participant::DomainParticipantAsync,
        domain_participant_listener::DomainParticipantListenerAsync, publisher::PublisherAsync,
        publisher_listener::PublisherListenerAsync, subscriber::SubscriberAsync,
        subscriber_listener::SubscriberListenerAsync, topic::TopicAsync,
        topic_listener::TopicListenerAsync,
    },
    domain::domain_participant_factory::DomainId,
    implementation::{
        actor::{Actor, ActorAddress, Mail, MailHandler},
        actors::{
            data_reader_actor::DataReaderActor, subscriber_actor::SubscriberActor,
            topic_actor::TopicActor,
        },
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, ReaderProxy},
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::{DiscoveredWriterData, WriterProxy},
            spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
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
            HistoryQosPolicy, LifespanQosPolicy, ResourceLimitsQosPolicy,
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
            BuiltinEndpointSet, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
        },
        group::RtpsGroup,
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
    xtypes::dynamic_type::DynamicType,
};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::{Arc, Mutex},
    thread::JoinHandle,
};
use tracing::warn;

pub const BUILT_IN_TOPIC_NAME_LIST: [&str; 4] = [
    DCPS_PARTICIPANT,
    DCPS_TOPIC,
    DCPS_PUBLICATION,
    DCPS_SUBSCRIPTION,
];

pub enum ListenerKind {
    Reader {
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
    },
    Writer {
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
    },
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
    pub listener_kind: ListenerKind,
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
                            let data_reader = match m.listener_kind {
                                ListenerKind::Reader {
                                    reader_address,
                                    status_condition_address,
                                    subscriber,
                                    topic,
                                } => DataReaderAsync::new(
                                    reader_address,
                                    status_condition_address,
                                    subscriber,
                                    topic,
                                ),
                                ListenerKind::Writer { .. } => {
                                    panic!("Expected Reader on this listener")
                                }
                            };
                            listener.on_data_available(data_reader).await
                        }
                        ParticipantListenerOperation::SampleRejected(status) => {
                            let data_reader = match m.listener_kind {
                                ListenerKind::Reader {
                                    reader_address,
                                    status_condition_address,
                                    subscriber,
                                    topic,
                                } => DataReaderAsync::new(
                                    reader_address,
                                    status_condition_address,
                                    subscriber,
                                    topic,
                                ),
                                ListenerKind::Writer { .. } => {
                                    panic!("Expected Reader on this listener")
                                }
                            };
                            listener.on_sample_rejected(data_reader, status).await
                        }
                        ParticipantListenerOperation::_LivenessChanged(status) => {
                            let data_reader = match m.listener_kind {
                                ListenerKind::Reader {
                                    reader_address,
                                    status_condition_address,
                                    subscriber,
                                    topic,
                                } => DataReaderAsync::new(
                                    reader_address,
                                    status_condition_address,
                                    subscriber,
                                    topic,
                                ),
                                ListenerKind::Writer { .. } => {
                                    panic!("Expected Reader on this listener")
                                }
                            };
                            listener.on_liveliness_changed(data_reader, status).await
                        }
                        ParticipantListenerOperation::RequestedDeadlineMissed(status) => {
                            let data_reader = match m.listener_kind {
                                ListenerKind::Reader {
                                    reader_address,
                                    status_condition_address,
                                    subscriber,
                                    topic,
                                } => DataReaderAsync::new(
                                    reader_address,
                                    status_condition_address,
                                    subscriber,
                                    topic,
                                ),
                                ListenerKind::Writer { .. } => {
                                    panic!("Expected Reader on this listener")
                                }
                            };
                            listener
                                .on_requested_deadline_missed(data_reader, status)
                                .await
                        }
                        ParticipantListenerOperation::RequestedIncompatibleQos(status) => {
                            let data_reader = match m.listener_kind {
                                ListenerKind::Reader {
                                    reader_address,
                                    status_condition_address,
                                    subscriber,
                                    topic,
                                } => DataReaderAsync::new(
                                    reader_address,
                                    status_condition_address,
                                    subscriber,
                                    topic,
                                ),
                                ListenerKind::Writer { .. } => {
                                    panic!("Expected Reader on this listener")
                                }
                            };
                            listener
                                .on_requested_incompatible_qos(data_reader, status)
                                .await
                        }
                        ParticipantListenerOperation::SubscriptionMatched(status) => {
                            let data_reader = match m.listener_kind {
                                ListenerKind::Reader {
                                    reader_address,
                                    status_condition_address,
                                    subscriber,
                                    topic,
                                } => DataReaderAsync::new(
                                    reader_address,
                                    status_condition_address,
                                    subscriber,
                                    topic,
                                ),
                                ListenerKind::Writer { .. } => {
                                    panic!("Expected Reader on this listener")
                                }
                            };
                            listener.on_subscription_matched(data_reader, status).await
                        }
                        ParticipantListenerOperation::SampleLost(status) => {
                            let data_reader = match m.listener_kind {
                                ListenerKind::Reader {
                                    reader_address,
                                    status_condition_address,
                                    subscriber,
                                    topic,
                                } => DataReaderAsync::new(
                                    reader_address,
                                    status_condition_address,
                                    subscriber,
                                    topic,
                                ),
                                ListenerKind::Writer { .. } => {
                                    panic!("Expected Reader on this listener")
                                }
                            };
                            listener.on_sample_lost(data_reader, status).await
                        }
                        ParticipantListenerOperation::_LivelinessLost(status) => {
                            let data_writer = match m.listener_kind {
                                ListenerKind::Reader { .. } => {
                                    panic!("Expected Writer on this listener")
                                }
                                ListenerKind::Writer {
                                    writer_address,
                                    status_condition_address,
                                    publisher,
                                    topic,
                                } => DataWriterAsync::new(
                                    writer_address,
                                    status_condition_address,
                                    publisher,
                                    topic,
                                ),
                            };
                            listener.on_liveliness_lost(data_writer, status).await
                        }
                        ParticipantListenerOperation::_OfferedDeadlineMissed(status) => {
                            let data_writer = match m.listener_kind {
                                ListenerKind::Reader { .. } => {
                                    panic!("Expected Writer on this listener")
                                }
                                ListenerKind::Writer {
                                    writer_address,
                                    status_condition_address,
                                    publisher,
                                    topic,
                                } => DataWriterAsync::new(
                                    writer_address,
                                    status_condition_address,
                                    publisher,
                                    topic,
                                ),
                            };
                            listener
                                .on_offered_deadline_missed(data_writer, status)
                                .await
                        }
                        ParticipantListenerOperation::OfferedIncompatibleQos(status) => {
                            let data_writer = match m.listener_kind {
                                ListenerKind::Reader { .. } => {
                                    panic!("Expected Writer on this listener")
                                }
                                ListenerKind::Writer {
                                    writer_address,
                                    status_condition_address,
                                    publisher,
                                    topic,
                                } => DataWriterAsync::new(
                                    writer_address,
                                    status_condition_address,
                                    publisher,
                                    topic,
                                ),
                            };
                            listener
                                .on_offered_incompatible_qos(data_writer, status)
                                .await
                        }
                        ParticipantListenerOperation::PublicationMatched(status) => {
                            let data_writer = match m.listener_kind {
                                ListenerKind::Reader { .. } => {
                                    panic!("Expected Writer on this listener")
                                }
                                ListenerKind::Writer {
                                    writer_address,
                                    status_condition_address,
                                    publisher,
                                    topic,
                                } => DataWriterAsync::new(
                                    writer_address,
                                    status_condition_address,
                                    publisher,
                                    topic,
                                ),
                            };
                            listener.on_publication_matched(data_writer, status).await
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
    rtps_participant: Arc<Mutex<RtpsParticipant>>,
    guid: Guid,
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
        rtps_participant: Arc<Mutex<RtpsParticipant>>,
        guid: Guid,
        domain_id: DomainId,
        domain_tag: String,
        domain_participant_qos: DomainParticipantQos,
        data_max_size_serialized: usize,
        listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
        builtin_data_writer_list: Vec<DataWriterActor>,
        message_sender_actor: MessageSenderActor,
        executor: Executor,
        timer_driver: TimerDriver,
    ) -> (Self, ActorAddress<StatusConditionActor>) {
        let lease_duration = Duration::new(100, 0);
        let guid_prefix = guid.prefix();
        let executor_handle = executor.handle();

        let builtin_subscriber = Actor::spawn(
            SubscriberActor::new(
                SubscriberQos::default(),
                RtpsGroup::new(Guid::new(
                    guid_prefix,
                    EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP),
                )),
                None,
                vec![],
                vec![],
                vec![],
                &executor_handle,
            ),
            &executor_handle,
        );

        let builtin_publisher = Actor::spawn(
            PublisherActor::new(
                PublisherQos::default(),
                RtpsGroup::new(Guid::new(
                    guid_prefix,
                    EntityId::new([0, 0, 0], BUILT_IN_WRITER_GROUP),
                )),
                rtps_participant.clone(),
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
                guid,
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
        )
    }

    fn lookup_discovered_topic(
        &mut self,
        topic_name: String,
        type_support: Arc<dyn DynamicType + Send + Sync>,
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
                    representation: discovered_topic_data.representation().clone(),
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
        type_support: Arc<dyn DynamicType + Send + Sync>,
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
            let guid = Guid::new(self.guid.prefix(), entity_id);

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
        let guid = Guid::new(self.guid.prefix(), entity_id);
        let rtps_group = RtpsGroup::new(guid);
        let status_kind = message.mask.to_vec();
        let publisher = PublisherActor::new(
            publisher_qos,
            rtps_group,
            self.rtps_participant.clone(),
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
        let guid = Guid::new(self.guid.prefix(), entity_id);
        let rtps_group = RtpsGroup::new(guid);
        let subscriber_status_kind = message.mask.to_vec();
        let domain_participant_status_kind = self.status_kind.clone();

        let subscriber = SubscriberActor::new(
            subscriber_qos,
            rtps_group,
            message.a_listener,
            subscriber_status_kind,
            domain_participant_status_kind,
            vec![],
            &message.executor_handle,
        );
        let subscriber_status_condition = subscriber.get_status_condition_address();

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
    pub type_support: Arc<dyn DynamicType + Send + Sync>,
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
    pub type_support: Arc<dyn DynamicType + Send + Sync>,
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

pub struct SetTopicList {
    pub topic_list: HashMap<String, (Actor<TopicActor>, ActorAddress<StatusConditionActor>)>,
}
impl Mail for SetTopicList {
    type Result = ();
}
impl MailHandler<SetTopicList> for DomainParticipantActor {
    fn handle(&mut self, message: SetTopicList) -> <SetTopicList as Mail>::Result {
        self.topic_list = message.topic_list;
    }
}

pub struct GetInstanceHandle;
impl Mail for GetInstanceHandle {
    type Result = InstanceHandle;
}
impl MailHandler<GetInstanceHandle> for DomainParticipantActor {
    fn handle(&mut self, _: GetInstanceHandle) -> <GetInstanceHandle as Mail>::Result {
        InstanceHandle::new(self.guid.into())
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
            .lock()
            .unwrap()
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
            .lock()
            .unwrap()
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
            .lock()
            .unwrap()
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
            .lock()
            .unwrap()
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
            .dds_participant_data
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

pub struct SetBuiltInSubscriber {
    pub builtin_subscriber: Actor<SubscriberActor>,
}
impl Mail for SetBuiltInSubscriber {
    type Result = ();
}
impl MailHandler<SetBuiltInSubscriber> for DomainParticipantActor {
    fn handle(&mut self, message: SetBuiltInSubscriber) -> <SetBuiltInSubscriber as Mail>::Result {
        self.builtin_subscriber = message.builtin_subscriber;
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
        SpdpDiscoveredParticipantData {
            dds_participant_data: ParticipantBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: self.guid.into(),
                },
                user_data: self.qos.user_data.clone(),
            },
            participant_proxy: self.rtps_participant.lock().unwrap().participant_proxy(),
            lease_duration: self.lease_duration,
            discovered_participant_list: self.discovered_participant_list.keys().cloned().collect(),
        }
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
        Time::now()
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

pub struct AddDiscoveredParticipant {
    pub discovered_participant_data: SpdpDiscoveredParticipantData,
    // pub participant: DomainParticipantAsync,
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
            .participant_proxy
            .domain_id
            .unwrap_or(self.domain_id)
            == self.domain_id;
        let is_domain_tag_matching = message
            .discovered_participant_data
            .participant_proxy
            .domain_tag
            == self.domain_tag;
        let discovered_participant_handle = InstanceHandle::new(
            message
                .discovered_participant_data
                .dds_participant_data
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
                // message.participant.clone(),
            )?;
            self.add_matched_publications_announcer(
                &message.discovered_participant_data,
                // message.participant.clone(),
            )?;
            self.add_matched_subscriptions_detector(
                &message.discovered_participant_data,
                // message.participant.clone(),
            )?;
            self.add_matched_subscriptions_announcer(
                &message.discovered_participant_data,
                // message.participant.clone(),
            )?;
            self.add_matched_topics_detector(
                &message.discovered_participant_data,
                // message.participant.clone(),
            )?;
            self.add_matched_topics_announcer(
                &message.discovered_participant_data,
                // message.participant.clone(),
            )?;

            self.discovered_participant_list.insert(
                InstanceHandle::new(
                    message
                        .discovered_participant_data
                        .dds_participant_data
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
                    .writer_proxy
                    .remote_writer_guid
                    .prefix(),
                ENTITYID_PARTICIPANT,
            )
            .into(),
        ));
        let is_publication_ignored = self.ignored_publications.contains(&InstanceHandle::new(
            message
                .discovered_writer_data
                .dds_publication_data
                .key()
                .value,
        ));
        if !is_publication_ignored && !is_participant_ignored {
            if let Some(discovered_participant_data) =
                self.discovered_participant_list.get(&InstanceHandle::new(
                    Guid::new(
                        message
                            .discovered_writer_data
                            .writer_proxy
                            .remote_writer_guid
                            .prefix(),
                        ENTITYID_PARTICIPANT,
                    )
                    .into(),
                ))
            {
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
                        subscriber_address,
                        // participant: message.participant.clone(),
                        participant_mask_listener,
                    });
                }

                // Add writer topic to discovered topic list using the writer instance handle
                let topic_instance_handle = InstanceHandle::new(
                    message
                        .discovered_writer_data
                        .dds_publication_data
                        .key()
                        .value,
                );
                let writer_topic = TopicBuiltinTopicData {
                    key: BuiltInTopicKey::default(),
                    name: message
                        .discovered_writer_data
                        .dds_publication_data
                        .topic_name()
                        .to_owned(),
                    type_name: message
                        .discovered_writer_data
                        .dds_publication_data
                        .get_type_name()
                        .to_owned(),
                    durability: message
                        .discovered_writer_data
                        .dds_publication_data
                        .durability()
                        .clone(),
                    deadline: message
                        .discovered_writer_data
                        .dds_publication_data
                        .deadline()
                        .clone(),
                    latency_budget: message
                        .discovered_writer_data
                        .dds_publication_data
                        .latency_budget()
                        .clone(),
                    liveliness: message
                        .discovered_writer_data
                        .dds_publication_data
                        .liveliness()
                        .clone(),
                    reliability: message
                        .discovered_writer_data
                        .dds_publication_data
                        .reliability()
                        .clone(),
                    transport_priority: TransportPriorityQosPolicy::default(),
                    lifespan: message
                        .discovered_writer_data
                        .dds_publication_data
                        .lifespan()
                        .clone(),
                    destination_order: message
                        .discovered_writer_data
                        .dds_publication_data
                        .destination_order()
                        .clone(),
                    history: HistoryQosPolicy::default(),
                    resource_limits: ResourceLimitsQosPolicy::default(),
                    ownership: message
                        .discovered_writer_data
                        .dds_publication_data
                        .ownership()
                        .clone(),
                    topic_data: message
                        .discovered_writer_data
                        .dds_publication_data
                        .topic_data()
                        .clone(),
                    representation: message
                        .discovered_writer_data
                        .dds_publication_data
                        .representation()
                        .clone(),
                };

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
                    .remote_reader_guid
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
                            .remote_reader_guid
                            .prefix(),
                        ENTITYID_PARTICIPANT,
                    )
                    .into(),
                ))
            {
                let default_unicast_locator_list = discovered_participant_data
                    .participant_proxy
                    .default_unicast_locator_list
                    .to_vec();
                let default_multicast_locator_list = discovered_participant_data
                    .participant_proxy
                    .default_multicast_locator_list
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
                        // participant: message.participant.clone(),
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
                let reader_topic = TopicBuiltinTopicData {
                    key: BuiltInTopicKey::default(),
                    name: message
                        .discovered_reader_data
                        .subscription_builtin_topic_data()
                        .topic_name()
                        .to_string(),
                    type_name: message
                        .discovered_reader_data
                        .subscription_builtin_topic_data()
                        .get_type_name()
                        .to_string(),

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
                    representation: message
                        .discovered_reader_data
                        .subscription_builtin_topic_data()
                        .representation()
                        .clone(),
                };
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
                .topic_builtin_topic_data
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
                    .topic_builtin_topic_data
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

pub struct GetListener;
impl Mail for GetListener {
    type Result = (
        Option<MpscSender<ParticipantListenerMessage>>,
        Vec<StatusKind>,
    );
}
impl MailHandler<GetListener> for DomainParticipantActor {
    fn handle(&mut self, _: GetListener) -> <GetListener as Mail>::Result {
        (
            self.participant_listener_thread
                .as_ref()
                .map(|l| l.sender().clone()),
            self.status_kind.clone(),
        )
    }
}

impl DomainParticipantActor {
    fn add_matched_publications_detector(
        &self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
        // participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR)
        {
            let participant_mask_listener = (
                self.participant_listener_thread
                    .as_ref()
                    .map(|l| l.sender().clone()),
                self.status_kind.clone(),
            );
            let remote_reader_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let reader_proxy = ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                expects_inline_qos,
            };
            let subscription_builtin_topic_data = SubscriptionBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: remote_reader_guid.into(),
                },
                participant_key: BuiltInTopicKey::default(),
                topic_name: DCPS_PUBLICATION.to_owned(),
                type_name: "DiscoveredWriterData".to_owned(),
                durability: sedp_data_reader_qos().durability,
                deadline: sedp_data_reader_qos().deadline,
                latency_budget: sedp_data_reader_qos().latency_budget,
                liveliness: sedp_data_reader_qos().liveliness,
                reliability: sedp_data_reader_qos().reliability,
                ownership: sedp_data_reader_qos().ownership,
                destination_order: sedp_data_reader_qos().destination_order,
                user_data: sedp_data_reader_qos().user_data,
                time_based_filter: sedp_data_reader_qos().time_based_filter,
                presentation: Default::default(),
                partition: Default::default(),
                topic_data: Default::default(),
                group_data: Default::default(),
                xml_type: Default::default(),
                representation: sedp_data_reader_qos().representation,
            };
            let discovered_reader_data =
                DiscoveredReaderData::new(reader_proxy, subscription_builtin_topic_data);
            let default_unicast_locator_list = self
                .rtps_participant
                .lock()
                .unwrap()
                .default_unicast_locator_list()
                .to_vec();
            let default_multicast_locator_list = self
                .rtps_participant
                .lock()
                .unwrap()
                .default_multicast_locator_list()
                .to_vec();
            self.builtin_publisher
                .send_actor_mail(publisher_actor::AddMatchedReader {
                    discovered_reader_data,
                    default_unicast_locator_list,
                    default_multicast_locator_list,
                    publisher_address: self.builtin_publisher.address(),
                    // participant,
                    participant_mask_listener,
                    message_sender_actor: self.message_sender_actor.address(),
                });
        }
        Ok(())
    }

    fn add_matched_publications_announcer(
        &self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
        // participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let data_max_size_serialized = Default::default();

            let dds_publication_data = PublicationBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: remote_writer_guid.into(),
                },
                participant_key: BuiltInTopicKey::default(),
                topic_name: DCPS_PUBLICATION.to_owned(),
                type_name: "DiscoveredWriterData".to_owned(),
                durability: sedp_data_writer_qos().durability,
                deadline: sedp_data_writer_qos().deadline,
                latency_budget: sedp_data_writer_qos().latency_budget,
                liveliness: sedp_data_writer_qos().liveliness,
                reliability: sedp_data_writer_qos().reliability,
                lifespan: sedp_data_writer_qos().lifespan,
                user_data: sedp_data_writer_qos().user_data,
                ownership: sedp_data_writer_qos().ownership,
                ownership_strength: sedp_data_writer_qos().ownership_strength,
                destination_order: sedp_data_writer_qos().destination_order,
                presentation: Default::default(),
                partition: Default::default(),
                topic_data: Default::default(),
                group_data: Default::default(),
                xml_type: Default::default(),
                representation: sedp_data_writer_qos().representation,
            };
            let writer_proxy = WriterProxy {
                remote_writer_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                data_max_size_serialized,
            };
            let discovered_writer_data = DiscoveredWriterData {
                dds_publication_data,
                writer_proxy,
            };
            self.builtin_subscriber
                .send_actor_mail(subscriber_actor::AddMatchedWriter {
                    discovered_writer_data,
                    subscriber_address: self.builtin_subscriber.address(),
                    // participant,
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
        // participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR)
        {
            let remote_reader_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let reader_proxy = ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                expects_inline_qos,
            };
            let subscription_builtin_topic_data = SubscriptionBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: remote_reader_guid.into(),
                },
                participant_key: BuiltInTopicKey::default(),
                topic_name: DCPS_SUBSCRIPTION.to_owned(),
                type_name: "DiscoveredReaderData".to_owned(),
                durability: sedp_data_reader_qos().durability,
                deadline: sedp_data_reader_qos().deadline,
                latency_budget: sedp_data_reader_qos().latency_budget,
                liveliness: sedp_data_reader_qos().liveliness,
                reliability: sedp_data_reader_qos().reliability,
                ownership: sedp_data_reader_qos().ownership,
                destination_order: sedp_data_reader_qos().destination_order,
                user_data: sedp_data_reader_qos().user_data,
                time_based_filter: sedp_data_reader_qos().time_based_filter,
                presentation: Default::default(),
                partition: Default::default(),
                topic_data: Default::default(),
                group_data: Default::default(),
                xml_type: Default::default(),
                representation: sedp_data_reader_qos().representation,
            };
            let discovered_reader_data =
                DiscoveredReaderData::new(reader_proxy, subscription_builtin_topic_data);
            self.builtin_publisher
                .send_actor_mail(publisher_actor::AddMatchedReader {
                    discovered_reader_data,
                    default_unicast_locator_list: vec![],
                    default_multicast_locator_list: vec![],
                    publisher_address: self.builtin_publisher.address(),
                    // participant,
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
        // participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let data_max_size_serialized = Default::default();

            let writer_proxy = WriterProxy {
                remote_writer_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                data_max_size_serialized,
            };
            let dds_publication_data = PublicationBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: remote_writer_guid.into(),
                },
                participant_key: BuiltInTopicKey::default(),
                topic_name: DCPS_SUBSCRIPTION.to_owned(),
                type_name: "DiscoveredReaderData".to_owned(),
                durability: sedp_data_writer_qos().durability,
                deadline: sedp_data_writer_qos().deadline,
                latency_budget: sedp_data_writer_qos().latency_budget,
                liveliness: sedp_data_writer_qos().liveliness,
                reliability: sedp_data_writer_qos().reliability,
                lifespan: sedp_data_writer_qos().lifespan,
                user_data: sedp_data_writer_qos().user_data,
                ownership: sedp_data_writer_qos().ownership,
                ownership_strength: sedp_data_writer_qos().ownership_strength,
                destination_order: sedp_data_writer_qos().destination_order,
                presentation: Default::default(),
                partition: Default::default(),
                topic_data: Default::default(),
                group_data: Default::default(),
                xml_type: Default::default(),
                representation: sedp_data_writer_qos().representation,
            };
            let discovered_writer_data = DiscoveredWriterData {
                dds_publication_data,
                writer_proxy,
            };
            self.builtin_subscriber
                .send_actor_mail(subscriber_actor::AddMatchedWriter {
                    discovered_writer_data,
                    subscriber_address: self.builtin_subscriber.address(),
                    // participant,
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
        // participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR)
        {
            let remote_reader_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let reader_proxy = ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                expects_inline_qos,
            };
            let subscription_builtin_topic_data = SubscriptionBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: remote_reader_guid.into(),
                },
                participant_key: BuiltInTopicKey::default(),
                topic_name: DCPS_TOPIC.to_owned(),
                type_name: "DiscoveredTopicData".to_owned(),
                durability: sedp_data_reader_qos().durability,
                deadline: sedp_data_reader_qos().deadline,
                latency_budget: sedp_data_reader_qos().latency_budget,
                liveliness: sedp_data_reader_qos().liveliness,
                reliability: sedp_data_reader_qos().reliability,
                ownership: sedp_data_reader_qos().ownership,
                destination_order: sedp_data_reader_qos().destination_order,
                user_data: sedp_data_reader_qos().user_data,
                time_based_filter: sedp_data_reader_qos().time_based_filter,
                presentation: Default::default(),
                partition: Default::default(),
                topic_data: Default::default(),
                group_data: Default::default(),
                xml_type: Default::default(),
                representation: sedp_data_reader_qos().representation,
            };
            let discovered_reader_data =
                DiscoveredReaderData::new(reader_proxy, subscription_builtin_topic_data);
            self.builtin_publisher
                .send_actor_mail(publisher_actor::AddMatchedReader {
                    discovered_reader_data,
                    default_unicast_locator_list: vec![],
                    default_multicast_locator_list: vec![],
                    publisher_address: self.builtin_publisher.address(),
                    // participant,
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
        // participant: DomainParticipantAsync,
    ) -> DdsResult<()> {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let data_max_size_serialized = Default::default();

            let writer_proxy = WriterProxy {
                remote_writer_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                data_max_size_serialized,
            };
            let dds_publication_data = PublicationBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: remote_writer_guid.into(),
                },
                participant_key: BuiltInTopicKey::default(),
                topic_name: DCPS_TOPIC.to_owned(),
                type_name: "DiscoveredTopicData".to_owned(),
                durability: sedp_data_writer_qos().durability,
                deadline: sedp_data_writer_qos().deadline,
                latency_budget: sedp_data_writer_qos().latency_budget,
                liveliness: sedp_data_writer_qos().liveliness,
                reliability: sedp_data_writer_qos().reliability,
                lifespan: sedp_data_writer_qos().lifespan,
                user_data: sedp_data_writer_qos().user_data,
                ownership: sedp_data_writer_qos().ownership,
                ownership_strength: sedp_data_writer_qos().ownership_strength,
                destination_order: sedp_data_writer_qos().destination_order,
                presentation: Default::default(),
                partition: Default::default(),
                topic_data: Default::default(),
                group_data: Default::default(),
                xml_type: Default::default(),
                representation: sedp_data_writer_qos().representation,
            };
            let discovered_writer_data = DiscoveredWriterData {
                dds_publication_data,
                writer_proxy,
            };
            self.builtin_subscriber
                .send_actor_mail(subscriber_actor::AddMatchedWriter {
                    discovered_writer_data,
                    subscriber_address: self.builtin_subscriber.address(),
                    // participant,
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
                                    // participant: participant.clone(),
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
