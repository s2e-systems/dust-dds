use super::{
    any_data_reader_listener::AnyDataReaderListener,
    any_data_writer_listener::AnyDataWriterListener,
    data_writer_actor::DataWriterActor,
    domain_participant_factory_actor::{sedp_data_reader_qos, sedp_data_writer_qos},
    message_sender_actor::MessageSenderActor,
    publisher_actor::{self, PublisherActor},
    status_condition_actor::StatusConditionActor,
    subscriber_actor,
};
use crate::{
    builtin_topics::{
        BuiltInTopicKey, ParticipantBuiltinTopicData, PublicationBuiltinTopicData,
        SubscriptionBuiltinTopicData, TopicBuiltinTopicData, DCPS_PARTICIPANT, DCPS_PUBLICATION,
        DCPS_SUBSCRIPTION, DCPS_TOPIC,
    },
    dds_async::{
        data_reader::DataReaderAsync, data_writer::DataWriterAsync,
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
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantQos, PublisherQos, QosKind,
            SubscriberQos, TopicQos,
        },
        qos_policy::{
            HistoryQosPolicy, LifespanQosPolicy, ResourceLimitsQosPolicy,
            TransportPriorityQosPolicy,
        },
        status::{
            InconsistentTopicStatus, LivelinessChangedStatus, LivelinessLostStatus,
            OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
            RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
            SampleRejectedStatus, StatusKind, SubscriptionMatchedStatus,
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
            ENTITYID_PARTICIPANT, ENTITYID_UNKNOWN, USER_DEFINED_TOPIC, USER_DEFINED_WRITER_GROUP,
        },
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
        // let (sender, receiver) = mpsc_channel::<ParticipantListenerMessage>();
        // let thread = std::thread::Builder::new()
        //     .name("Domain participant listener".to_string())
        //     .spawn(move || {
        //         block_on(async {
        //             while let Some(m) = receiver.recv().await {
        //                 match m.listener_operation {
        //                     ParticipantListenerOperation::_DataAvailable => {
        //                         let data_reader = match m.listener_kind {
        //                             ListenerKind::Reader {
        //                                 reader_address,
        //                                 status_condition_address,
        //                                 subscriber,
        //                                 topic,
        //                             } => DataReaderAsync::new(
        //                                 reader_address,
        //                                 status_condition_address,
        //                                 subscriber,
        //                                 topic,
        //                             ),
        //                             ListenerKind::Writer { .. } => {
        //                                 panic!("Expected Reader on this listener")
        //                             }
        //                         };
        //                         listener.on_data_available(data_reader).await
        //                     }
        //                     ParticipantListenerOperation::SampleRejected(status) => {
        //                         let data_reader = match m.listener_kind {
        //                             ListenerKind::Reader {
        //                                 reader_address,
        //                                 status_condition_address,
        //                                 subscriber,
        //                                 topic,
        //                             } => DataReaderAsync::new(
        //                                 reader_address,
        //                                 status_condition_address,
        //                                 subscriber,
        //                                 topic,
        //                             ),
        //                             ListenerKind::Writer { .. } => {
        //                                 panic!("Expected Reader on this listener")
        //                             }
        //                         };
        //                         listener.on_sample_rejected(data_reader, status).await
        //                     }
        //                     ParticipantListenerOperation::_LivenessChanged(status) => {
        //                         let data_reader = match m.listener_kind {
        //                             ListenerKind::Reader {
        //                                 reader_address,
        //                                 status_condition_address,
        //                                 subscriber,
        //                                 topic,
        //                             } => DataReaderAsync::new(
        //                                 reader_address,
        //                                 status_condition_address,
        //                                 subscriber,
        //                                 topic,
        //                             ),
        //                             ListenerKind::Writer { .. } => {
        //                                 panic!("Expected Reader on this listener")
        //                             }
        //                         };
        //                         listener.on_liveliness_changed(data_reader, status).await
        //                     }
        //                     ParticipantListenerOperation::RequestedDeadlineMissed(status) => {
        //                         let data_reader = match m.listener_kind {
        //                             ListenerKind::Reader {
        //                                 reader_address,
        //                                 status_condition_address,
        //                                 subscriber,
        //                                 topic,
        //                             } => DataReaderAsync::new(
        //                                 reader_address,
        //                                 status_condition_address,
        //                                 subscriber,
        //                                 topic,
        //                             ),
        //                             ListenerKind::Writer { .. } => {
        //                                 panic!("Expected Reader on this listener")
        //                             }
        //                         };
        //                         listener
        //                             .on_requested_deadline_missed(data_reader, status)
        //                             .await
        //                     }
        //                     ParticipantListenerOperation::RequestedIncompatibleQos(status) => {
        //                         let data_reader = match m.listener_kind {
        //                             ListenerKind::Reader {
        //                                 reader_address,
        //                                 status_condition_address,
        //                                 subscriber,
        //                                 topic,
        //                             } => DataReaderAsync::new(
        //                                 reader_address,
        //                                 status_condition_address,
        //                                 subscriber,
        //                                 topic,
        //                             ),
        //                             ListenerKind::Writer { .. } => {
        //                                 panic!("Expected Reader on this listener")
        //                             }
        //                         };
        //                         listener
        //                             .on_requested_incompatible_qos(data_reader, status)
        //                             .await
        //                     }
        //                     ParticipantListenerOperation::SubscriptionMatched(status) => {
        //                         let data_reader = match m.listener_kind {
        //                             ListenerKind::Reader {
        //                                 reader_address,
        //                                 status_condition_address,
        //                                 subscriber,
        //                                 topic,
        //                             } => DataReaderAsync::new(
        //                                 reader_address,
        //                                 status_condition_address,
        //                                 subscriber,
        //                                 topic,
        //                             ),
        //                             ListenerKind::Writer { .. } => {
        //                                 panic!("Expected Reader on this listener")
        //                             }
        //                         };
        //                         listener.on_subscription_matched(data_reader, status).await
        //                     }
        //                     ParticipantListenerOperation::SampleLost(status) => {
        //                         let data_reader = match m.listener_kind {
        //                             ListenerKind::Reader {
        //                                 reader_address,
        //                                 status_condition_address,
        //                                 subscriber,
        //                                 topic,
        //                             } => DataReaderAsync::new(
        //                                 reader_address,
        //                                 status_condition_address,
        //                                 subscriber,
        //                                 topic,
        //                             ),
        //                             ListenerKind::Writer { .. } => {
        //                                 panic!("Expected Reader on this listener")
        //                             }
        //                         };
        //                         listener.on_sample_lost(data_reader, status).await
        //                     }
        //                     ParticipantListenerOperation::_LivelinessLost(status) => {
        //                         let data_writer = match m.listener_kind {
        //                             ListenerKind::Reader { .. } => {
        //                                 panic!("Expected Writer on this listener")
        //                             }
        //                             ListenerKind::Writer {
        //                                 writer_address,
        //                                 status_condition_address,
        //                                 publisher,
        //                                 topic,
        //                             } => DataWriterAsync::new(
        //                                 writer_address,
        //                                 status_condition_address,
        //                                 publisher,
        //                                 topic,
        //                             ),
        //                         };
        //                         listener.on_liveliness_lost(data_writer, status).await
        //                     }
        //                     ParticipantListenerOperation::_OfferedDeadlineMissed(status) => {
        //                         let data_writer = match m.listener_kind {
        //                             ListenerKind::Reader { .. } => {
        //                                 panic!("Expected Writer on this listener")
        //                             }
        //                             ListenerKind::Writer {
        //                                 writer_address,
        //                                 status_condition_address,
        //                                 publisher,
        //                                 topic,
        //                             } => DataWriterAsync::new(
        //                                 writer_address,
        //                                 status_condition_address,
        //                                 publisher,
        //                                 topic,
        //                             ),
        //                         };
        //                         listener
        //                             .on_offered_deadline_missed(data_writer, status)
        //                             .await
        //                     }
        //                     ParticipantListenerOperation::OfferedIncompatibleQos(status) => {
        //                         let data_writer = match m.listener_kind {
        //                             ListenerKind::Reader { .. } => {
        //                                 panic!("Expected Writer on this listener")
        //                             }
        //                             ListenerKind::Writer {
        //                                 writer_address,
        //                                 status_condition_address,
        //                                 publisher,
        //                                 topic,
        //                             } => DataWriterAsync::new(
        //                                 writer_address,
        //                                 status_condition_address,
        //                                 publisher,
        //                                 topic,
        //                             ),
        //                         };
        //                         listener
        //                             .on_offered_incompatible_qos(data_writer, status)
        //                             .await
        //                     }
        //                     ParticipantListenerOperation::PublicationMatched(status) => {
        //                         let data_writer = match m.listener_kind {
        //                             ListenerKind::Reader { .. } => {
        //                                 panic!("Expected Writer on this listener")
        //                             }
        //                             ListenerKind::Writer {
        //                                 writer_address,
        //                                 status_condition_address,
        //                                 publisher,
        //                                 topic,
        //                             } => DataWriterAsync::new(
        //                                 writer_address,
        //                                 status_condition_address,
        //                                 publisher,
        //                                 topic,
        //                             ),
        //                         };
        //                         listener.on_publication_matched(data_writer, status).await
        //                     }
        //                 }
        //             }
        //         });
        //     })
        //     .expect("failed to spawn thread");
        // Self { thread, sender }
        todo!()
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
    builtin_publisher: PublisherActor,
    user_defined_subscriber_list: HashMap<InstanceHandle, Actor<SubscriberActor>>,
    user_defined_subscriber_counter: u8,
    default_subscriber_qos: SubscriberQos,
    user_defined_publisher_list: HashMap<InstanceHandle, PublisherActor>,
    user_defined_publisher_counter: u8,
    default_publisher_qos: PublisherQos,
    topic_list: HashMap<String, TopicActor>,
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
                rtps_participant.clone(),
                None,
                vec![],
                vec![],
                vec![],
                &executor_handle,
            ),
            &executor_handle,
        );

        let builtin_publisher = PublisherActor::new(
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
    ) -> DdsResult<Option<()>> {
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
                self.create_user_defined_topic(
                    topic_name,
                    type_name.clone(),
                    QosKind::Specific(qos),
                    None,
                    vec![],
                    type_support,
                    executor_handle,
                )?;
                return Ok(Some(()));
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
    ) -> DdsResult<()> {
        if let Entry::Vacant(e) = self.topic_list.entry(topic_name.clone()) {
            let qos = match qos {
                QosKind::Default => self.default_topic_qos.clone(),
                QosKind::Specific(q) => q,
            };
            let topic_counter = self.user_defined_topic_counter;
            self.user_defined_topic_counter += 1;
            let entity_id = EntityId::new([topic_counter, 0, 0], USER_DEFINED_TOPIC);
            let guid = Guid::new(self.guid.prefix(), entity_id);

            let (topic, _) = TopicActor::new(
                guid,
                qos,
                type_name,
                &topic_name,
                a_listener,
                type_support,
                &executor_handle,
            );

            e.insert(topic);

            Ok(())
        } else {
            Err(DdsError::PreconditionNotMet(format!("Topic with name {} already exists. To access this topic call the lookup_topicdescription method.",topic_name)))
        }
    }

    fn lookup_topicdescription(&self, topic_name: String) -> DdsResult<Option<()>> {
        if let Some(_) = self.topic_list.get(&topic_name) {
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }
}

// ############################  Domain participant messages
pub struct CreateUserDefinedPublisher {
    pub qos: QosKind<PublisherQos>,
    pub a_listener: Option<Box<dyn PublisherListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
}
impl Mail for CreateUserDefinedPublisher {
    type Result = DdsResult<Guid>;
}
impl MailHandler<CreateUserDefinedPublisher> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: CreateUserDefinedPublisher,
    ) -> <CreateUserDefinedPublisher as Mail>::Result {
        todo!()
        // let publisher_qos = match message.qos {
        //     QosKind::Default => self.default_publisher_qos.clone(),
        //     QosKind::Specific(q) => q,
        // };
        // let publisher_counter = self.user_defined_publisher_counter;
        // self.user_defined_publisher_counter += 1;
        // let entity_id = EntityId::new([publisher_counter, 0, 0], USER_DEFINED_WRITER_GROUP);
        // let guid = Guid::new(self.guid.prefix(), entity_id);
        // let rtps_group = RtpsGroup::new(guid);
        // let status_kind = message.mask.to_vec();
        // let publisher = PublisherActor::new(
        //     publisher_qos,
        //     rtps_group,
        //     self.rtps_participant.clone(),
        //     message.a_listener,
        //     status_kind,
        //     vec![],
        //     &message.executor_handle,
        // );

        // let publisher_status_condition = publisher.get_statuscondition();

        // self.user_defined_publisher_list
        //     .insert(InstanceHandle::new(guid.into()), publisher);

        // if self
        //     .participant_address
        //     .send_actor_mail(domain_participant_actor::IsEnabled)?
        //     .receive_reply()
        //     .await
        //     && self
        //         .participant_address
        //         .send_actor_mail(domain_participant_actor::GetQos)?
        //         .receive_reply()
        //         .await
        //         .entity_factory
        //         .autoenable_created_entities
        // {
        //     publisher.enable().await?;
        // }

        // Ok(guid)
    }
}

pub struct DeleteUserDefinedPublisher {
    pub guid: Guid,
}
impl Mail for DeleteUserDefinedPublisher {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteUserDefinedPublisher> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteUserDefinedPublisher,
    ) -> <DeleteUserDefinedPublisher as Mail>::Result {
        // self.user_defined_publisher_list.remove(&message.handle)
        //     if a_publisher
        //     .publisher_address()
        //     .send_actor_mail(publisher_actor::IsEmpty)?
        //     .receive_reply()
        //     .await
        // {
        //     if let Some(deleted_publisher) = self
        //         .participant_address
        //         .send_actor_mail(domain_participant_actor::DeleteUserDefinedPublisher {
        //             handle: a_publisher.get_instance_handle().await?,
        //         })?
        //         .receive_reply()
        //         .await
        //     {
        //         deleted_publisher.stop().await;
        //         Ok(())
        //     } else {
        //         Err(DdsError::PreconditionNotMet(
        //             "Publisher can only be deleted from its parent participant".to_string(),
        //         ))
        //     }
        // } else {
        //     Err(DdsError::PreconditionNotMet(
        //         "Publisher still contains data writers".to_string(),
        //     ))
        // }
        todo!()
    }
}

pub struct CreateUserDefinedSubscriber {
    pub qos: QosKind<SubscriberQos>,
    pub a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
}
impl Mail for CreateUserDefinedSubscriber {
    type Result = DdsResult<Guid>;
}
impl MailHandler<CreateUserDefinedSubscriber> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: CreateUserDefinedSubscriber,
    ) -> <CreateUserDefinedSubscriber as Mail>::Result {
        todo!()

        //     let (subscriber_address, subscriber_status_condition) = self
        //     .participant_address
        //     .send_actor_mail(domain_participant_actor::CreateUserDefinedSubscriber {
        //         qos,
        //         a_listener,
        //         mask: mask.to_vec(),
        //         executor_handle: self.executor_handle.clone(),
        //     })?
        //     .receive_reply()
        //     .await;

        // let subscriber = SubscriberAsync::new(
        //     subscriber_address,
        //     subscriber_status_condition,
        //     self.clone(),
        // );

        // if self
        //     .participant_address
        //     .send_actor_mail(domain_participant_actor::IsEnabled)?
        //     .receive_reply()
        //     .await
        //     && self
        //         .participant_address
        //         .send_actor_mail(domain_participant_actor::GetQos)?
        //         .receive_reply()
        //         .await
        //         .entity_factory
        //         .autoenable_created_entities
        // {
        //     subscriber.enable().await?;
        // }

        // Ok(subscriber)
        // let subscriber_qos = match message.qos {
        //     QosKind::Default => self.default_subscriber_qos.clone(),
        //     QosKind::Specific(q) => q,
        // };
        // let subcriber_counter = self.user_defined_subscriber_counter;
        // self.user_defined_subscriber_counter += 1;
        // let entity_id = EntityId::new([subcriber_counter, 0, 0], USER_DEFINED_READER_GROUP);
        // let guid = Guid::new(self.guid.prefix(), entity_id);
        // let rtps_group = RtpsGroup::new(guid);
        // let subscriber_status_kind = message.mask.to_vec();
        // let domain_participant_status_kind = self.status_kind.clone();

        // let subscriber = SubscriberActor::new(
        //     subscriber_qos,
        //     rtps_group,
        //     self.rtps_participant.clone(),
        //     message.a_listener,
        //     subscriber_status_kind,
        //     domain_participant_status_kind,
        //     vec![],
        //     &message.executor_handle,
        // );
        // let subscriber_status_condition = subscriber.get_status_condition_address();

        // let subscriber_actor = Actor::spawn(subscriber, &message.executor_handle);
        // let subscriber_address = subscriber_actor.address();

        // self.user_defined_subscriber_list
        //     .insert(InstanceHandle::new(guid.into()), subscriber_actor);

        // (subscriber_address, subscriber_status_condition)
    }
}

pub struct DeleteUserDefinedSubscriber {
    pub guid: Guid,
}
impl Mail for DeleteUserDefinedSubscriber {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteUserDefinedSubscriber> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteUserDefinedSubscriber,
    ) -> <DeleteUserDefinedSubscriber as Mail>::Result {
        // self.user_defined_subscriber_list.remove(&message.handle)
        // if a_subscriber
        //     .subscriber_address()
        //     .send_actor_mail(subscriber_actor::IsEmpty)?
        //     .receive_reply()
        //     .await
        // {
        //     if let Some(deleted_subscriber) = self
        //         .participant_address
        //         .send_actor_mail(domain_participant_actor::DeleteUserDefinedSubscriber {
        //             handle: a_subscriber.get_instance_handle().await?,
        //         })?
        //         .receive_reply()
        //         .await
        //     {
        //         deleted_subscriber.stop().await;
        //         Ok(())
        //     } else {
        //         Err(DdsError::PreconditionNotMet(
        //             "Subscriber can only be deleted from its parent participant".to_string(),
        //         ))
        //     }
        // } else {
        //     Err(DdsError::PreconditionNotMet(
        //         "Subscriber still contains data readers".to_string(),
        //     ))
        // }
        todo!()
    }
}

pub struct CreateUserDefinedTopic {
    pub topic_name: String,
    pub type_name: String,
    pub qos: QosKind<TopicQos>,
    pub a_listener: Option<Box<dyn TopicListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
    pub type_support: Arc<dyn DynamicType + Send + Sync>,
}
impl Mail for CreateUserDefinedTopic {
    type Result = DdsResult<()>;
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

        // let topic = TopicAsync::new(type_name.to_string(), topic_name.to_string(), self.clone());
        // if self
        //     .participant_address
        //     .send_actor_mail(domain_participant_actor::IsEnabled)?
        //     .receive_reply()
        //     .await
        //     && self
        //         .participant_address
        //         .send_actor_mail(domain_participant_actor::GetQos)?
        //         .receive_reply()
        //         .await
        //         .entity_factory
        //         .autoenable_created_entities
        // {
        //     topic.enable().await?;
        // }

        // Ok(topic)
    }
}

pub struct DeleteUserDefinedTopic {
    pub topic_name: String,
}
impl Mail for DeleteUserDefinedTopic {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteUserDefinedTopic> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteUserDefinedTopic,
    ) -> <DeleteUserDefinedTopic as Mail>::Result {
        // let topic_name = a_topic.get_name();
        // if BUILT_IN_TOPIC_NAME_LIST.contains(&topic_name.as_str()) {
        //     return Ok(());
        // }
        // let publisher_list = self
        //     .participant_address
        //     .send_actor_mail(domain_participant_actor::GetPublisherList)?
        //     .receive_reply()
        //     .await;
        // for publisher in publisher_list {
        //     let data_writer_list = publisher
        //         .send_actor_mail(publisher_actor::GetDataWriterList)?
        //         .receive_reply()
        //         .await;
        //     for dw in data_writer_list {
        //         if dw
        //             .send_actor_mail(data_writer_actor::GetTopicName)?
        //             .receive_reply()
        //             .await?
        //             == topic_name
        //         {
        //             return Err(DdsError::PreconditionNotMet(
        //                 "Topic still attached to some data writer".to_string(),
        //             ));
        //         }
        //     }
        // }

        // let subscriber_list = self
        //     .participant_address
        //     .send_actor_mail(domain_participant_actor::GetSubscriberList)?
        //     .receive_reply()
        //     .await;
        // for subscriber in subscriber_list {
        //     let data_reader_list = subscriber
        //         .send_actor_mail(subscriber_actor::GetDataReaderList)?
        //         .receive_reply()
        //         .await;
        //     for dr in data_reader_list {
        //         if dr
        //             .send_actor_mail(data_reader_actor::GetTopicName)?
        //             .receive_reply()
        //             .await?
        //             == topic_name
        //         {
        //             return Err(DdsError::PreconditionNotMet(
        //                 "Topic still attached to some data reader".to_string(),
        //             ));
        //         }
        //     }
        // }
        // if let Some(deleted_topic) = self
        //     .participant_address
        //     .send_actor_mail(domain_participant_actor::DeleteUserDefinedTopic {
        //         topic_name: a_topic.get_name(),
        //     })?
        //     .receive_reply()
        //     .await
        // {
        //     Ok(())
        // } else {
        //     Err(DdsError::PreconditionNotMet(
        //         "Topic can only be deleted from its parent participant".to_string(),
        //     ))
        // }

        // self.topic_list.remove(&message.topic_name)

        todo!()
    }
}

pub struct FindTopic {
    pub topic_name: String,
    pub type_support: Arc<dyn DynamicType + Send + Sync>,
}
impl Mail for FindTopic {
    type Result = DdsResult<Option<()>>;
}
impl MailHandler<FindTopic> for DomainParticipantActor {
    fn handle(&mut self, message: FindTopic) -> <FindTopic as Mail>::Result {
        todo!()
        // if let Some(r) = self.lookup_topicdescription(message.topic_name.clone())? {
        //     Ok(Some(()))
        // } else {
        //     self.lookup_discovered_topic(
        //         message.topic_name.clone(),
        //         message.type_support.clone(),
        //         message.executor_handle.clone(),
        //     )
        // }
    }
}

pub struct LookupTopicdescription {
    pub topic_name: String,
}
impl Mail for LookupTopicdescription {
    type Result = DdsResult<Option<String>>;
}
impl MailHandler<LookupTopicdescription> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: LookupTopicdescription,
    ) -> <LookupTopicdescription as Mail>::Result {
        todo!()
        // self.lookup_topicdescription(message.topic_name)
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

pub struct DeleteParticipantContainedEntities;
impl Mail for DeleteParticipantContainedEntities {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteParticipantContainedEntities> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteParticipantContainedEntities,
    ) -> <DeleteParticipantContainedEntities as Mail>::Result {
        // for deleted_publisher in self
        //             .participant_address
        //             .send_actor_mail(domain_participant_actor::DrainPublisherList)?
        //             .receive_reply()
        //             .await
        //         {
        //             PublisherAsync::new(
        //                 deleted_publisher
        //                     .send_actor_mail(publisher_actor::GetGuid)
        //                     .receive_reply()
        //                     .await,
        //                 deleted_publisher.address(),
        //                 deleted_publisher
        //                     .send_actor_mail(publisher_actor::GetStatuscondition)
        //                     .receive_reply()
        //                     .await,
        //                 self.clone(),
        //             )
        //             .delete_contained_entities()
        //             .await?;
        //         }

        //         for deleted_subscriber in self
        //             .participant_address
        //             .send_actor_mail(domain_participant_actor::DrainSubscriberList)?
        //             .receive_reply()
        //             .await
        //         {
        //             SubscriberAsync::new(
        //                 deleted_subscriber.address(),
        //                 deleted_subscriber
        //                     .send_actor_mail(subscriber_actor::GetStatuscondition)
        //                     .receive_reply()
        //                     .await,
        //                 self.clone(),
        //             )
        //             .delete_contained_entities()
        //             .await?;
        //         }

        //         for deleted_topic in self
        //             .participant_address
        //             .send_actor_mail(domain_participant_actor::DrainTopicList)?
        //             .receive_reply()
        //             .await
        //         {
        //             self.announce_deleted_topic(deleted_topic).await?;
        //         }

        //         Ok(())
        todo!()
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

pub struct GetDefaultPublisherQos;
impl Mail for GetDefaultPublisherQos {
    type Result = PublisherQos;
}
impl MailHandler<GetDefaultPublisherQos> for DomainParticipantActor {
    fn handle(&mut self, _: GetDefaultPublisherQos) -> <GetDefaultPublisherQos as Mail>::Result {
        self.default_publisher_qos.clone()
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

pub struct GetDefaultSubscriberQos;
impl Mail for GetDefaultSubscriberQos {
    type Result = SubscriberQos;
}
impl MailHandler<GetDefaultSubscriberQos> for DomainParticipantActor {
    fn handle(&mut self, _: GetDefaultSubscriberQos) -> <GetDefaultSubscriberQos as Mail>::Result {
        self.default_subscriber_qos.clone()
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

pub struct GetCurrentTime;
impl Mail for GetCurrentTime {
    type Result = Time;
}
impl MailHandler<GetCurrentTime> for DomainParticipantActor {
    fn handle(&mut self, _: GetCurrentTime) -> <GetCurrentTime as Mail>::Result {
        Time::now()
    }
}

pub struct SetDomainParticipantQos {
    pub qos: QosKind<DomainParticipantQos>,
}
impl Mail for SetDomainParticipantQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDomainParticipantQos> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetDomainParticipantQos,
    ) -> <SetDomainParticipantQos as Mail>::Result {
        // let qos = match qos {
        //     QosKind::Default => DomainParticipantQos::default(),
        //     QosKind::Specific(q) => q,
        // };

        // self.participant_address
        //     .send_actor_mail(domain_participant_actor::SetQos { qos })?
        //     .receive_reply()
        //     .await?;
        // self.announce_participant().await
        todo!()
    }
}

pub struct GetDomainParticipantQos;
impl Mail for GetDomainParticipantQos {
    type Result = DdsResult<DomainParticipantQos>;
}
impl MailHandler<GetDomainParticipantQos> for DomainParticipantActor {
    fn handle(&mut self, _: GetDomainParticipantQos) -> <GetDomainParticipantQos as Mail>::Result {
        Ok(self.qos.clone())
    }
}

pub struct SetDomainParticipantListener {
    pub listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
    pub status_kind: Vec<StatusKind>,
}
impl Mail for SetDomainParticipantListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDomainParticipantListener> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetDomainParticipantListener,
    ) -> <SetDomainParticipantListener as Mail>::Result {
        if let Some(l) = self.participant_listener_thread.take() {
            l.join()?;
        }
        self.participant_listener_thread = message.listener.map(ParticipantListenerThread::new);
        self.status_kind = message.status_kind;
        Ok(())
    }
}

pub struct EnableDomainParticipant;
impl Mail for EnableDomainParticipant {
    type Result = DdsResult<()>;
}
impl MailHandler<EnableDomainParticipant> for DomainParticipantActor {
    fn handle(&mut self, _: EnableDomainParticipant) -> <EnableDomainParticipant as Mail>::Result {
        todo!()
        // if !self
        //     .participant_address
        //     .send_actor_mail(domain_participant_actor::IsEnabled)?
        //     .receive_reply()
        //     .await
        // {
        //     let builtin_publisher = self
        //         .participant_address
        //         .send_actor_mail(domain_participant_actor::GetBuiltinPublisher)?
        //         .receive_reply()
        //         .await;
        //     builtin_publisher
        //         .send_actor_mail(publisher_actor::Enable)?
        //         .receive_reply()
        //         .await;

        //     let builtin_subscriber = self
        //         .participant_address
        //         .send_actor_mail(domain_participant_actor::GetBuiltInSubscriber)?
        //         .receive_reply()
        //         .await;
        //     builtin_subscriber
        //         .send_actor_mail(subscriber_actor::Enable)?
        //         .receive_reply()
        //         .await;

        //     self.participant_address
        //         .send_actor_mail(domain_participant_actor::Enable)?
        //         .receive_reply()
        //         .await?;

        //     for builtin_reader in builtin_subscriber
        //         .send_actor_mail(subscriber_actor::GetDataReaderList)?
        //         .receive_reply()
        //         .await
        //     {
        //         builtin_reader
        //             .send_actor_mail(data_reader_actor::Enable {
        //                 data_reader_address: builtin_reader.clone(),
        //             })?
        //             .receive_reply()
        //             .await;
        //     }
        //     for builtin_writer in builtin_publisher
        //         .send_actor_mail(publisher_actor::GetDataWriterList)?
        //         .receive_reply()
        //         .await
        //     {
        //         let message_sender_actor = self
        //             .participant_address
        //             .send_actor_mail(domain_participant_actor::GetMessageSender)?
        //             .receive_reply()
        //             .await;
        //         builtin_writer
        //             .send_actor_mail(data_writer_actor::Enable {
        //                 data_writer_address: builtin_writer.clone(),
        //                 message_sender_actor,
        //                 executor_handle: self.executor_handle.clone(),
        //                 timer_handle: self.timer_handle.clone(),
        //             })?
        //             .receive_reply()
        //             .await;
        //     }

        //     self.announce_participant().await?;
        // }
        // Ok(())
    }
}

pub struct GetDomainParticipantInstanceHandle;
impl Mail for GetDomainParticipantInstanceHandle {
    type Result = DdsResult<InstanceHandle>;
}
impl MailHandler<GetDomainParticipantInstanceHandle> for DomainParticipantActor {
    fn handle(
        &mut self,
        _: GetDomainParticipantInstanceHandle,
    ) -> <GetDomainParticipantInstanceHandle as Mail>::Result {
        Ok(InstanceHandle::new(self.guid.into()))
    }
}

// ############################  Topic messages

pub struct GetInconsistentTopicStatus {
    pub topic_name: String,
}
impl Mail for GetInconsistentTopicStatus {
    type Result = DdsResult<InconsistentTopicStatus>;
}
impl MailHandler<GetInconsistentTopicStatus> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetInconsistentTopicStatus,
    ) -> <GetInconsistentTopicStatus as Mail>::Result {
        Ok(self
            .topic_list
            .get_mut(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_inconsistent_topic_status())
    }
}

pub struct SetTopicQos {
    pub topic_name: String,
    pub topic_qos: QosKind<TopicQos>,
}
impl Mail for SetTopicQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetTopicQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetTopicQos) -> <SetTopicQos as Mail>::Result {
        let qos = match message.topic_qos {
            QosKind::Default => self.default_topic_qos.clone(),
            QosKind::Specific(q) => q,
        };

        self.topic_list
            .get_mut(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?
            .set_qos(qos)
    }
}

pub struct GetTopicQos {
    pub topic_name: String,
}
impl Mail for GetTopicQos {
    type Result = DdsResult<TopicQos>;
}
impl MailHandler<GetTopicQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetTopicQos) -> <GetTopicQos as Mail>::Result {
        Ok(self
            .topic_list
            .get_mut(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_qos()
            .clone())
    }
}

pub struct EnableTopic {
    pub topic_name: String,
}
impl Mail for EnableTopic {
    type Result = DdsResult<()>;
}
impl MailHandler<EnableTopic> for DomainParticipantActor {
    fn handle(&mut self, message: EnableTopic) -> <EnableTopic as Mail>::Result {
        let topic = self
            .topic_list
            .get_mut(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?;
        if !topic.is_enabled() {
            topic.enable();
        }

        Ok(())
    }
}

pub struct GetTopicTypeSupport {
    pub topic_name: String,
}
impl Mail for GetTopicTypeSupport {
    type Result = DdsResult<Arc<dyn DynamicType + Send + Sync>>;
}
impl MailHandler<GetTopicTypeSupport> for DomainParticipantActor {
    fn handle(&mut self, message: GetTopicTypeSupport) -> <GetTopicTypeSupport as Mail>::Result {
        Ok(self
            .topic_list
            .get(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_type_support()
            .clone())
    }
}

// ############################  Publisher messages
pub struct CreateUserDefinedDataWriter {
    pub topic_name: String,
    pub qos: QosKind<DataWriterQos>,
    pub a_listener: Option<Box<dyn AnyDataWriterListener + Send>>,
    pub mask: Vec<StatusKind>,
}
impl Mail for CreateUserDefinedDataWriter {
    type Result = DdsResult<Guid>;
}
impl MailHandler<CreateUserDefinedDataWriter> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: CreateUserDefinedDataWriter,
    ) -> <CreateUserDefinedDataWriter as Mail>::Result {
        todo!()
        // let qos = match message.qos {
        //     QosKind::Default => self.default_datawriter_qos.clone(),
        //     QosKind::Specific(q) => {
        //         q.is_consistent()?;
        //         q
        //     }
        // };

        // let guid_prefix = self.rtps_group.guid().prefix();
        // let entity_kind = match message.has_key {
        //     true => USER_DEFINED_WRITER_WITH_KEY,
        //     false => USER_DEFINED_WRITER_NO_KEY,
        // };
        // let entity_key = [
        //     self.rtps_group.guid().entity_id().entity_key()[0],
        //     self.get_unique_writer_id(),
        //     0,
        // ];
        // let entity_id = EntityId::new(entity_key, entity_kind);
        // let guid = Guid::new(guid_prefix, entity_id);

        // let rtps_writer = self.transport.lock().unwrap().create_writer(guid);

        // let data_writer = DataWriterActor::new(
        //     rtps_writer,
        //     guid,
        //     Duration::new(0, 200_000_000).into(),
        //     message.topic_name,
        //     message.type_name,
        //     message.a_listener,
        //     message.mask,
        //     qos,
        //     &message.executor_handle,
        // );
        // let data_writer_actor = Actor::spawn(data_writer, &message.executor_handle);
        // let data_writer_address = data_writer_actor.address();
        // self.data_writer_list
        //     .insert(InstanceHandle::new(guid.into()), data_writer_actor);

        // if self
        //     .publisher_address
        //     .send_actor_mail(publisher_actor::IsEnabled)?
        //     .receive_reply()
        //     .await
        //     && self
        //         .publisher_address
        //         .send_actor_mail(publisher_actor::GetQos)?
        //         .receive_reply()
        //         .await
        //         .entity_factory
        //         .autoenable_created_entities
        // {
        //     data_writer.enable().await?
        // }

        // Ok(data_writer_address)
    }
}

pub struct DeleteUserDefinedDataWriter {
    pub guid: Guid,
}
impl Mail for DeleteUserDefinedDataWriter {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteUserDefinedDataWriter> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteUserDefinedDataWriter,
    ) -> <DeleteUserDefinedDataWriter as Mail>::Result {
        // if let Some(removed_writer) = self.data_writer_list.remove(&message.handle) {
        //     Ok(removed_writer)
        // } else {
        //     Err(DdsError::PreconditionNotMet(
        //         "Data writer can only be deleted from its parent publisher".to_string(),
        //     ))
        // }

        // self.announce_deleted_data_writer(&deleted_writer, topic.get_name())
        //     .await?;
        // deleted_writer.stop().await;
        // Ok(())
        todo!()
    }
}

pub struct LookupDataWriter {
    pub publisher_guid: Guid,
    pub topic_name: String,
}
impl Mail for LookupDataWriter {
    type Result = DdsResult<Option<Guid>>;
}
impl MailHandler<LookupDataWriter> for DomainParticipantActor {
    fn handle(&mut self, message: LookupDataWriter) -> <LookupDataWriter as Mail>::Result {
        todo!()
        // if let Some(_) = self
        //     .participant
        //     .participant_address()
        //     .send_actor_mail(domain_participant_actor::LookupTopicdescription {
        //         topic_name: topic_name.to_string(),
        //     })?
        //     .receive_reply()
        //     .await?
        // {
        //     let data_writer_list = self
        //         .publisher_address
        //         .send_actor_mail(publisher_actor::GetDataWriterList)?
        //         .receive_reply()
        //         .await;
        //     for dw in data_writer_list {
        //         if dw
        //             .send_actor_mail(data_writer_actor::GetTopicName)?
        //             .receive_reply()
        //             .await?
        //             == topic_name
        //         {
        //             let type_name = self
        //                 .participant_address()
        //                 .send_actor_mail(domain_participant_actor::GetTopicTypeName {
        //                     topic_name: topic_name.to_string(),
        //                 })?
        //                 .receive_reply()
        //                 .await?;
        //             let topic = TopicAsync::new(
        //                 type_name,
        //                 topic_name.to_string(),
        //                 self.participant.clone(),
        //             );
        //             let status_condition = dw
        //                 .send_actor_mail(data_writer_actor::GetStatuscondition)?
        //                 .receive_reply()
        //                 .await;
        //             return Ok(Some(DataWriterAsync::new(
        //                 dw.clone(),
        //                 status_condition,
        //                 self.clone(),
        //                 topic,
        //             )));
        //         }
        //     }
        //     Ok(None)
        // } else {
        //     Err(DdsError::BadParameter)
        // }
    }
}

pub struct DeletePublisherContainedEntities {
    pub publisher_guid: Guid,
}
impl Mail for DeletePublisherContainedEntities {
    type Result = DdsResult<()>;
}
impl MailHandler<DeletePublisherContainedEntities> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeletePublisherContainedEntities,
    ) -> <DeletePublisherContainedEntities as Mail>::Result {
        // let deleted_writer_actor_list = self
        //     .publisher_address
        //     .send_actor_mail(publisher_actor::DrainDataWriterList)?
        //     .receive_reply()
        //     .await;

        // for deleted_writer_actor in deleted_writer_actor_list {
        //     todo!();
        //     // self.announce_deleted_data_writer(&deleted_writer_actor, &topic_address)
        //     //     .await?;
        //     deleted_writer_actor.stop().await;
        // }
        // Ok(())
        todo!()
    }
}

pub struct SetDefaultDataWriterQos {
    pub publisher_guid: Guid,
    pub qos: QosKind<DataWriterQos>,
}
impl Mail for SetDefaultDataWriterQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDefaultDataWriterQos> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetDefaultDataWriterQos,
    ) -> <SetDefaultDataWriterQos as Mail>::Result {
        // let qos = match qos {
        //     QosKind::Default => {
        //         self.publisher_address
        //             .send_actor_mail(publisher_actor::GetDefaultDatawriterQos)?
        //             .receive_reply()
        //             .await
        //     }
        //     QosKind::Specific(q) => {
        //         q.is_consistent()?;
        //         q
        //     }
        // };

        // self.publisher_address
        //     .send_actor_mail(publisher_actor::SetDefaultDatawriterQos { qos })?
        //     .receive_reply()
        //     .await;

        // Ok(())
        todo!()
    }
}

pub struct GetDefaultDataWriterQos {
    pub publisher_guid: Guid,
}
impl Mail for GetDefaultDataWriterQos {
    type Result = DdsResult<DataWriterQos>;
}
impl MailHandler<GetDefaultDataWriterQos> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetDefaultDataWriterQos,
    ) -> <GetDefaultDataWriterQos as Mail>::Result {
        // let qos = match qos {
        //     QosKind::Default => {
        //         self.publisher_address
        //             .send_actor_mail(publisher_actor::GetDefaultDatawriterQos)?
        //             .receive_reply()
        //             .await
        //     }
        //     QosKind::Specific(q) => {
        //         q.is_consistent()?;
        //         q
        //     }
        // };

        // self.publisher_address
        //     .send_actor_mail(publisher_actor::SetDefaultDatawriterQos { qos })?
        //     .receive_reply()
        //     .await;

        // Ok(())
        todo!()
    }
}

pub struct SetPublisherQos {
    pub publisher_guid: Guid,
    pub qos: QosKind<PublisherQos>,
}
impl Mail for SetPublisherQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetPublisherQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetPublisherQos) -> <SetPublisherQos as Mail>::Result {
        todo!()
    }
}

pub struct GetPublisherQos {
    pub publisher_guid: Guid,
}
impl Mail for GetPublisherQos {
    type Result = DdsResult<PublisherQos>;
}
impl MailHandler<GetPublisherQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetPublisherQos) -> <GetPublisherQos as Mail>::Result {
        todo!()
    }
}

pub struct SetPublisherListener {
    pub publisher_guid: Guid,
    pub a_listener: Option<Box<dyn PublisherListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
}
impl Mail for SetPublisherListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetPublisherListener> for DomainParticipantActor {
    fn handle(&mut self, message: SetPublisherListener) -> <SetPublisherQos as Mail>::Result {
        todo!()
    }
}

pub struct EnablePublisher {
    pub publisher_guid: Guid,
}
impl Mail for EnablePublisher {
    type Result = DdsResult<()>;
}
impl MailHandler<EnablePublisher> for DomainParticipantActor {
    fn handle(&mut self, message: EnablePublisher) -> <EnablePublisher as Mail>::Result {
        todo!()
    }
}

pub struct GetPublisherInstanceHandle {
    pub publisher_guid: Guid,
}
impl Mail for GetPublisherInstanceHandle {
    type Result = DdsResult<InstanceHandle>;
}
impl MailHandler<GetPublisherInstanceHandle> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetPublisherInstanceHandle,
    ) -> <GetPublisherInstanceHandle as Mail>::Result {
        todo!()
    }
}

// ############################  Subscriber messages
pub struct CreateUserDefinedDataReader {
    pub topic_name: String,
    pub qos: QosKind<DataReaderQos>,
    pub a_listener: Option<Box<dyn AnyDataReaderListener + Send>>,
    pub mask: Vec<StatusKind>,
}
impl Mail for CreateUserDefinedDataReader {
    type Result = DdsResult<Guid>;
}
impl MailHandler<CreateUserDefinedDataReader> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: CreateUserDefinedDataReader,
    ) -> <CreateUserDefinedDataReader as Mail>::Result {
        todo!()
        // let listener = a_listener.map(|b| DataReaderActorListener {
        //     data_reader_listener: Box::new(b),
        //     subscriber_async: self.clone(),
        // });

        // let default_unicast_locator_list = self
        //     .participant_address()
        //     .send_actor_mail(domain_participant_actor::GetDefaultUnicastLocatorList)?
        //     .receive_reply()
        //     .await;
        // let default_multicast_locator_list = self
        //     .participant_address()
        //     .send_actor_mail(domain_participant_actor::GetDefaultMulticastLocatorList)?
        //     .receive_reply()
        //     .await;

        // let topic_name = a_topic.get_name();
        // let type_name = a_topic.get_type_name();
        // let type_support = self
        //     .participant_address()
        //     .send_actor_mail(domain_participant_actor::GetTopicTypeSupport {
        //         topic_name: topic_name.clone(),
        //     })?
        //     .receive_reply()
        //     .await?;
        // let has_key = {
        //     let mut has_key = false;
        //     for index in 0..type_support.get_member_count() {
        //         if type_support
        //             .get_member_by_index(index)?
        //             .get_descriptor()?
        //             .is_key
        //         {
        //             has_key = true;
        //             break;
        //         }
        //     }
        //     has_key
        // };

        // let reader_address = self
        //     .subscriber_address
        //     .send_actor_mail(subscriber_actor::CreateDatareader {
        //         topic_name,
        //         type_name,
        //         type_support,
        //         has_key,
        //         qos,
        //         a_listener: listener,
        //         mask: mask.to_vec(),
        //         default_unicast_locator_list,
        //         default_multicast_locator_list,
        //         subscriber_address: self.subscriber_address.clone(),
        //         executor_handle: self.participant.executor_handle().clone(),
        //         timer_handle: self.participant.timer_handle().clone(),
        //     })?
        //     .receive_reply()
        //     .await?;

        // let status_condition = reader_address
        //     .send_actor_mail(data_reader_actor::GetStatuscondition)?
        //     .receive_reply()
        //     .await;
        // let data_reader = DataReaderAsync::new(
        //     reader_address,
        //     status_condition,
        //     self.clone(),
        //     a_topic.clone(),
        // );

        // if self
        //     .subscriber_address
        //     .send_actor_mail(subscriber_actor::IsEnabled)?
        //     .receive_reply()
        //     .await
        //     && self
        //         .subscriber_address
        //         .send_actor_mail(subscriber_actor::GetQos)?
        //         .receive_reply()
        //         .await
        //         .entity_factory
        //         .autoenable_created_entities
        // {
        //     data_reader.enable().await?;
        // }

        // Ok(data_reader)
    }
}

pub struct DeleteUserDefinedDataReader {
    pub guid: Guid,
}
impl Mail for DeleteUserDefinedDataReader {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteUserDefinedDataReader> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteUserDefinedDataReader,
    ) -> <DeleteUserDefinedDataReader as Mail>::Result {
        // let reader_handle = a_datareader.get_instance_handle().await?;

        // let deleted_reader = self
        //     .subscriber_address
        //     .send_actor_mail(subscriber_actor::DeleteDatareader {
        //         handle: reader_handle,
        //     })?
        //     .receive_reply()
        //     .await?;

        // self.announce_deleted_data_reader(
        //     &deleted_reader,
        //     a_datareader.get_topicdescription().get_name(),
        // )
        // .await?;
        // deleted_reader.stop().await;
        // Ok(())
        todo!()
    }
}

pub struct LookupDataReader {
    pub subscriber_guid: Guid,
    pub topic_name: String,
}
impl Mail for LookupDataReader {
    type Result = DdsResult<Option<Guid>>;
}
impl MailHandler<LookupDataReader> for DomainParticipantActor {
    fn handle(&mut self, message: LookupDataReader) -> <LookupDataReader as Mail>::Result {
        todo!()
        //     if let Some(()) = self
        //     .participant_address()
        //     .send_actor_mail(domain_participant_actor::LookupTopicdescription {
        //         topic_name: topic_name.to_string(),
        //     })?
        //     .receive_reply()
        //     .await?
        // {
        //     let data_reader_list = self
        //         .subscriber_address
        //         .send_actor_mail(subscriber_actor::GetDataReaderList)?
        //         .receive_reply()
        //         .await;
        //     for dr in data_reader_list {
        //         if dr
        //             .send_actor_mail(data_reader_actor::GetTopicName)?
        //             .receive_reply()
        //             .await?
        //             == topic_name
        //         {
        //             let type_name = self
        //                 .participant_address()
        //                 .send_actor_mail(domain_participant_actor::GetTopicTypeName {
        //                     topic_name: topic_name.to_string(),
        //                 })?
        //                 .receive_reply()
        //                 .await?;
        //             let topic = TopicAsync::new(
        //                 type_name,
        //                 topic_name.to_string(),
        //                 self.participant.clone(),
        //             );
        //             let status_condition = dr
        //                 .send_actor_mail(data_reader_actor::GetStatuscondition)?
        //                 .receive_reply()
        //                 .await;
        //             return Ok(Some(DataReaderAsync::new(
        //                 dr,
        //                 status_condition,
        //                 self.clone(),
        //                 topic,
        //             )));
        //         }
        //     }
        //     Ok(None)
        // } else {
        //     Err(DdsError::BadParameter)
        // }
    }
}

pub struct DeleteSubscriberContainedEntities {
    pub subscriber_guid: Guid,
}
impl Mail for DeleteSubscriberContainedEntities {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteSubscriberContainedEntities> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteSubscriberContainedEntities,
    ) -> <DeleteSubscriberContainedEntities as Mail>::Result {
        //         let deleted_reader_actor_list = self
        //         .subscriber_address
        //         .send_actor_mail(subscriber_actor::DrainDataReaderList)?
        //         .receive_reply()
        //         .await;

        //     for deleted_reader_actor in deleted_reader_actor_list {
        //         todo!();
        //         // self.announce_deleted_data_reader(&deleted_reader_actor, &topic)
        //         //     .await?;
        //         deleted_reader_actor.stop().await;
        //     }
        //     Ok(())
        // }
        todo!()
    }
}

pub struct SetDefaultDataReaderQos {
    pub subscriber_guid: Guid,
    pub qos: QosKind<DataReaderQos>,
}
impl Mail for SetDefaultDataReaderQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDefaultDataReaderQos> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetDefaultDataReaderQos,
    ) -> <SetDefaultDataReaderQos as Mail>::Result {
        // let qos = match qos {
        //     QosKind::Default => {
        //         self.publisher_address
        //             .send_actor_mail(publisher_actor::GetDefaultDatawriterQos)?
        //             .receive_reply()
        //             .await
        //     }
        //     QosKind::Specific(q) => {
        //         q.is_consistent()?;
        //         q
        //     }
        // };

        // self.publisher_address
        //     .send_actor_mail(publisher_actor::SetDefaultDatawriterQos { qos })?
        //     .receive_reply()
        //     .await;

        // Ok(())
        todo!()
    }
}

pub struct GetDefaultDataReaderQos {
    pub subscriber_guid: Guid,
}
impl Mail for GetDefaultDataReaderQos {
    type Result = DdsResult<DataReaderQos>;
}
impl MailHandler<GetDefaultDataReaderQos> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetDefaultDataReaderQos,
    ) -> <GetDefaultDataReaderQos as Mail>::Result {
        // let qos = match qos {
        //     QosKind::Default => {
        //         self.publisher_address
        //             .send_actor_mail(publisher_actor::GetDefaultDatawriterQos)?
        //             .receive_reply()
        //             .await
        //     }
        //     QosKind::Specific(q) => {
        //         q.is_consistent()?;
        //         q
        //     }
        // };

        // self.publisher_address
        //     .send_actor_mail(publisher_actor::SetDefaultDatawriterQos { qos })?
        //     .receive_reply()
        //     .await;

        // Ok(())
        todo!()
    }
}

pub struct SetSubscriberQos {
    pub subscriber_guid: Guid,
    pub qos: QosKind<SubscriberQos>,
}
impl Mail for SetSubscriberQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetSubscriberQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetSubscriberQos) -> <SetSubscriberQos as Mail>::Result {
        todo!()
    }
}

pub struct GetSubscriberQos {
    pub subscriber_guid: Guid,
}
impl Mail for GetSubscriberQos {
    type Result = DdsResult<SubscriberQos>;
}
impl MailHandler<GetSubscriberQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetSubscriberQos) -> <GetSubscriberQos as Mail>::Result {
        todo!()
    }
}

pub struct SetSubscriberListener {
    pub subscriber_guid: Guid,
    pub a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
}
impl Mail for SetSubscriberListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetSubscriberListener> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetSubscriberListener,
    ) -> <SetSubscriberListener as Mail>::Result {
        todo!()
    }
}

pub struct EnableSubscriber {
    pub subscriber_guid: Guid,
}
impl Mail for EnableSubscriber {
    type Result = DdsResult<()>;
}
impl MailHandler<EnableSubscriber> for DomainParticipantActor {
    fn handle(&mut self, message: EnableSubscriber) -> <EnableSubscriber as Mail>::Result {
        // if !self
        //     .subscriber_address
        //     .send_actor_mail(subscriber_actor::IsEnabled)?
        //     .receive_reply()
        //     .await
        // {
        //     self.subscriber_address
        //         .send_actor_mail(subscriber_actor::Enable)?
        //         .receive_reply()
        //         .await;

        //     if self
        //         .subscriber_address
        //         .send_actor_mail(subscriber_actor::GetQos)?
        //         .receive_reply()
        //         .await
        //         .entity_factory
        //         .autoenable_created_entities
        //     {
        //         for data_reader in self
        //             .subscriber_address
        //             .send_actor_mail(subscriber_actor::GetDataReaderList)?
        //             .receive_reply()
        //             .await
        //         {
        //             data_reader
        //                 .send_actor_mail(data_reader_actor::Enable {
        //                     data_reader_address: data_reader.clone(),
        //                 })?
        //                 .receive_reply()
        //                 .await;
        //         }
        //     }
        // }

        // Ok(())
        todo!()
    }
}

pub struct GetSubscriberInstanceHandle {
    pub subscriber_guid: Guid,
}
impl Mail for GetSubscriberInstanceHandle {
    type Result = DdsResult<InstanceHandle>;
}
impl MailHandler<GetSubscriberInstanceHandle> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetSubscriberInstanceHandle,
    ) -> <GetSubscriberInstanceHandle as Mail>::Result {
        todo!()
    }
}


pub struct SetTopicList {
    pub topic_list: HashMap<String, TopicActor>,
}
impl Mail for SetTopicList {
    type Result = ();
}
impl MailHandler<SetTopicList> for DomainParticipantActor {
    fn handle(&mut self, message: SetTopicList) -> <SetTopicList as Mail>::Result {
        self.topic_list = message.topic_list;
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
    type Result = Vec<TopicActor>;
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
            if let Some(removed_topic) = self.topic_list.remove(&t) {
                drained_topic_list.push(removed_topic);
            }
        }
        drained_topic_list
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
    // pub participant: DomainParticipantAsync,
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
                // subscriber_address,
                // participant: message.participant.clone(),
                // participant_mask_listener,
            });
        }
        Ok(())
    }
}

pub struct AddMatchedReader {
    pub discovered_reader_data: DiscoveredReaderData,
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
                // publisher_address,
                // participant: message.participant.clone(),
                // participant_mask_listener,
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
            for topic in self.topic_list.values() {
                todo!()
                // topic.send_actor_mail(topic_actor::ProcessDiscoveredTopic {
                //     discovered_topic_data: message.discovered_topic_data.clone(),
                // });
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

pub struct GetStatusConditionEnabledStatuses {
    pub guid: Guid,
}
impl Mail for GetStatusConditionEnabledStatuses {
    type Result = DdsResult<Vec<StatusKind>>;
}
impl MailHandler<GetStatusConditionEnabledStatuses> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetStatusConditionEnabledStatuses,
    ) -> <GetStatusConditionEnabledStatuses as Mail>::Result {
        todo!()
    }
}

pub struct SetStatusConditionEnabledStatuses {
    pub guid: Guid,
    pub status_mask: Vec<StatusKind>,
}
impl Mail for SetStatusConditionEnabledStatuses {
    type Result = DdsResult<()>;
}
impl MailHandler<SetStatusConditionEnabledStatuses> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetStatusConditionEnabledStatuses,
    ) -> <SetStatusConditionEnabledStatuses as Mail>::Result {
        todo!()
    }
}

pub struct GetStatusConditionTriggerValue {
    pub guid: Guid,
}
impl Mail for GetStatusConditionTriggerValue {
    type Result = DdsResult<bool>;
}
impl MailHandler<GetStatusConditionTriggerValue> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetStatusConditionTriggerValue,
    ) -> <GetStatusConditionTriggerValue as Mail>::Result {
        todo!()
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

// pub(crate) async fn announce_participant(&self) -> DdsResult<()> {
//     if self
//         .participant_address
//         .send_actor_mail(domain_participant_actor::IsEnabled)?
//         .receive_reply()
//         .await
//     {
//         let builtin_publisher = self.get_builtin_publisher().await?;

//         if let Some(spdp_participant_writer) = builtin_publisher
//             .lookup_datawriter::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT)
//             .await?
//         {
//             let data = self
//                 .participant_address
//                 .send_actor_mail(domain_participant_actor::AsSpdpDiscoveredParticipantData)?
//                 .receive_reply()
//                 .await;
//             spdp_participant_writer.write(&data, None).await?;
//         }
//     }
//     Ok(())
// }

// async fn announce_deleted_topic(&self, topic: TopicActor) -> DdsResult<()> {
//     todo!()
//     // let builtin_publisher = self.get_builtin_publisher().await?;

//     // if let Some(sedp_topics_announcer) = builtin_publisher.lookup_datawriter(DCPS_TOPIC).await?
//     // {
//     //     let data = topic
//     //         .send_actor_mail(topic_actor::AsDiscoveredTopicData)
//     //         .receive_reply()
//     //         .await;
//     //     sedp_topics_announcer.dispose(&data, None).await?;
//     // }

//     // Ok(())
// }

// async fn announce_deleted_data_writer(
//     &self,
//     writer: &Actor<DataWriterActor>,
//     topic_name: String,
// ) -> DdsResult<()> {
//     let builtin_publisher = self.participant.get_builtin_publisher().await?;
//     if let Some(sedp_publications_announcer) = builtin_publisher
//         .lookup_datawriter(DCPS_PUBLICATION)
//         .await?
//     {
//         let publisher_qos = self.get_qos().await?;
//         let default_unicast_locator_list = self
//             .participant_address()
//             .send_actor_mail(domain_participant_actor::GetDefaultUnicastLocatorList)?
//             .receive_reply()
//             .await;
//         let default_multicast_locator_list = self
//             .participant_address()
//             .send_actor_mail(domain_participant_actor::GetDefaultMulticastLocatorList)?
//             .receive_reply()
//             .await;
//         let topic_data = self
//             .participant_address()
//             .send_actor_mail(domain_participant_actor::GetTopicQos { topic_name })?
//             .receive_reply()
//             .await?
//             .topic_data;
//         let xml_type = "".to_string(); //topic
//                                        // .send_actor_mail(topic_actor::GetTypeSupport)?
//                                        // .receive_reply()
//                                        // .await
//                                        // .xml_type();
//         let data = writer
//             .send_actor_mail(data_writer_actor::AsDiscoveredWriterData {
//                 publisher_qos,
//                 default_unicast_locator_list,
//                 default_multicast_locator_list,
//                 topic_data,
//                 xml_type,
//             })
//             .receive_reply()
//             .await?;
//         sedp_publications_announcer.dispose(&data, None).await?;
//     }
//     Ok(())
// }

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

pub struct GetTopicTypeName {
    pub topic_name: String,
}
impl Mail for GetTopicTypeName {
    type Result = DdsResult<String>;
}
impl MailHandler<GetTopicTypeName> for DomainParticipantActor {
    fn handle(&mut self, message: GetTopicTypeName) -> <GetTopicTypeName as Mail>::Result {
        Ok(self
            .topic_list
            .get(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_type_name()
            .to_string())
    }
}

// async fn announce_deleted_data_reader(
//     &self,
//     reader: &Actor<DataReaderActor>,
//     topic_name: String,
// ) -> DdsResult<()> {
//     let builtin_publisher = self.participant.get_builtin_publisher().await?;
//     if let Some(sedp_subscriptions_announcer) = builtin_publisher
//         .lookup_datawriter(DCPS_SUBSCRIPTION)
//         .await?
//     {
//         let subscriber_qos = self.get_qos().await?;
//         let default_unicast_locator_list = self
//             .participant_address()
//             .send_actor_mail(domain_participant_actor::GetDefaultUnicastLocatorList)?
//             .receive_reply()
//             .await;
//         let default_multicast_locator_list = self
//             .participant_address()
//             .send_actor_mail(domain_participant_actor::GetDefaultMulticastLocatorList)?
//             .receive_reply()
//             .await;
//         let topic_data = self
//             .participant_address()
//             .send_actor_mail(domain_participant_actor::GetTopicQos { topic_name })?
//             .receive_reply()
//             .await?
//             .topic_data;
//         let xml_type = "".to_string(); //topic
//                                        // .send_actor_mail(topic_actor::GetTypeSupport)?
//                                        // .receive_reply()
//                                        // .await
//                                        // .xml_type();
//         let data = reader
//             .send_actor_mail(data_reader_actor::AsDiscoveredReaderData {
//                 subscriber_qos,
//                 default_unicast_locator_list,
//                 default_multicast_locator_list,
//                 topic_data,
//                 xml_type,
//             })
//             .receive_reply()
//             .await?;
//         sedp_subscriptions_announcer.dispose(&data, None).await?;
//     }
//     Ok(())
// }

// async fn announce_reader(&self) -> DdsResult<()> {
//     let builtin_publisher = self
//         .get_subscriber()
//         .get_participant()
//         .get_builtin_publisher()
//         .await?;
//     if let Some(sedp_subscriptions_announcer) = builtin_publisher
//         .lookup_datawriter::<DiscoveredReaderData>(DCPS_SUBSCRIPTION)
//         .await?
//     {
//         let subscriber_qos = self
//             .subscriber_address()
//             .send_actor_mail(subscriber_actor::GetQos)?
//             .receive_reply()
//             .await;
//         let default_unicast_locator_list = self
//             .participant_address()
//             .send_actor_mail(domain_participant_actor::GetDefaultUnicastLocatorList)?
//             .receive_reply()
//             .await;
//         let default_multicast_locator_list = self
//             .participant_address()
//             .send_actor_mail(domain_participant_actor::GetDefaultMulticastLocatorList)?
//             .receive_reply()
//             .await;
//         let topic_data = self
//             .participant_address()
//             .send_actor_mail(domain_participant_actor::GetTopicQos {
//                 topic_name: self.topic.get_name(),
//             })?
//             .receive_reply()
//             .await?
//             .topic_data;
//         let xml_type = "".to_string(); //self
//                                        // .topic
//                                        // .topic_address()
//                                        // .send_actor_mail(topic_actor::GetTypeSupport)?
//                                        // .receive_reply()
//                                        // .await
//                                        // .xml_type();
//         let discovered_reader_data = self
//             .reader_address
//             .send_actor_mail(data_reader_actor::AsDiscoveredReaderData {
//                 subscriber_qos,
//                 default_unicast_locator_list,
//                 default_multicast_locator_list,
//                 topic_data,
//                 xml_type,
//             })?
//             .receive_reply()
//             .await?;

//         sedp_subscriptions_announcer
//             .write(&discovered_reader_data, None)
//             .await?;
//     }
//     Ok(())
// }

// async fn process_sedp_publications_discovery(&self) -> DdsResult<()> {
//     let participant = self.subscriber.get_participant();
//     let builtin_subscriber = participant.get_builtin_subscriber();

//     if let Some(sedp_publications_detector) = builtin_subscriber
//         .lookup_datareader::<DiscoveredWriterData>(DCPS_PUBLICATION)
//         .await?
//     {
//         if let Ok(mut discovered_writer_sample_list) = sedp_publications_detector
//             .read(
//                 i32::MAX,
//                 ANY_SAMPLE_STATE,
//                 ANY_VIEW_STATE,
//                 ANY_INSTANCE_STATE,
//             )
//             .await
//         {
//             for discovered_writer_sample in discovered_writer_sample_list.drain(..) {
//                 match discovered_writer_sample.sample_info().instance_state {
//                     InstanceStateKind::Alive => match discovered_writer_sample.data() {
//                         Ok(discovered_writer_data) => {
//                             participant.participant_address().send_actor_mail(
//                                 domain_participant_actor::AddMatchedWriter {
//                                     discovered_writer_data,
//                                     // participant: participant.clone(),
//                                 },
//                             )?;
//                         }
//                         Err(e) => warn!(
//                             "Received invalid DiscoveredWriterData sample. Error {:?}",
//                             e
//                         ),
//                     },
//                     InstanceStateKind::NotAliveDisposed => {
//                         participant.participant_address().send_actor_mail(
//                             domain_participant_actor::RemoveMatchedWriter {
//                                 discovered_writer_handle: discovered_writer_sample
//                                     .sample_info()
//                                     .instance_handle,
//                                 // participant: participant.clone(),
//                             },
//                         )?;
//                     }
//                     InstanceStateKind::NotAliveNoWriters => {
//                         todo!()
//                     }
//                 }
//             }
//         }
//     }
//     Ok(())
// }

// async fn announce_writer(&self) -> DdsResult<()> {
//     let builtin_publisher = self
//         .get_publisher()
//         .get_participant()
//         .get_builtin_publisher()
//         .await?;
//     if let Some(sedp_publications_announcer) = builtin_publisher
//         .lookup_datawriter::<DiscoveredWriterData>(DCPS_PUBLICATION)
//         .await?
//     {
//         let publisher_qos = self.get_publisher().get_qos().await?;
//         let default_unicast_locator_list = self
//             .participant_address()
//             .send_actor_mail(domain_participant_actor::GetDefaultUnicastLocatorList)?
//             .receive_reply()
//             .await;
//         let default_multicast_locator_list = self
//             .participant_address()
//             .send_actor_mail(domain_participant_actor::GetDefaultMulticastLocatorList)?
//             .receive_reply()
//             .await;
//         let topic_data = self
//             .participant_address()
//             .send_actor_mail(domain_participant_actor::GetTopicQos {
//                 topic_name: self.topic.get_name(),
//             })?
//             .receive_reply()
//             .await?
//             .topic_data;
//         let xml_type = "".to_string(); //self
//                                        //     .topic
//                                        //     .topic_address()
//                                        //     .send_actor_mail(topic_actor::GetTypeSupport)?
//                                        //     .receive_reply()
//                                        //     .await
//                                        //     .xml_type();
//         let discovered_writer_data = self
//             .writer_address
//             .send_actor_mail(data_writer_actor::AsDiscoveredWriterData {
//                 publisher_qos,
//                 default_unicast_locator_list,
//                 default_multicast_locator_list,
//                 topic_data,
//                 xml_type,
//             })?
//             .receive_reply()
//             .await?;
//         sedp_publications_announcer
//             .write(&discovered_writer_data, None)
//             .await?;
//     }
//     Ok(())
// }

// async fn process_sedp_subscriptions_discovery(&self) -> DdsResult<()> {
//     let participant = self.publisher.get_participant();
//     let builtin_subscriber = participant.get_builtin_subscriber();

//     if let Some(sedp_subscriptions_detector) = builtin_subscriber
//         .lookup_datareader::<DiscoveredReaderData>(DCPS_SUBSCRIPTION)
//         .await?
//     {
//         if let Ok(mut discovered_reader_sample_list) = sedp_subscriptions_detector
//             .read(
//                 i32::MAX,
//                 ANY_SAMPLE_STATE,
//                 ANY_VIEW_STATE,
//                 ANY_INSTANCE_STATE,
//             )
//             .await
//         {
//             for discovered_reader_sample in discovered_reader_sample_list.drain(..) {
//                 match discovered_reader_sample.sample_info().instance_state {
//                     InstanceStateKind::Alive => match discovered_reader_sample.data() {
//                         Ok(discovered_reader_data) => {
//                             participant.participant_address().send_actor_mail(
//                                 domain_participant_actor::AddMatchedReader {
//                                     discovered_reader_data,
//                                     // participant: participant.clone(),
//                                 },
//                             )?;
//                         }
//                         Err(e) => warn!(
//                             "Received invalid DiscoveredReaderData sample. Error {:?}",
//                             e
//                         ),
//                     },
//                     InstanceStateKind::NotAliveDisposed => {
//                         participant.participant_address().send_actor_mail(
//                             domain_participant_actor::RemoveMatchedReader {
//                                 discovered_reader_handle: discovered_reader_sample
//                                     .sample_info()
//                                     .instance_handle,
//                                 // participant: participant.clone(),
//                             },
//                         )?;
//                     }
//                     InstanceStateKind::NotAliveNoWriters => {
//                         todo!()
//                     }
//                 }
//             }
//         }
//     }
//     Ok(())
// }
