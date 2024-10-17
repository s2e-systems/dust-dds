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
        messages::submessage_elements::Data,
        participant::RtpsParticipant,
        types::{
            EntityId, Guid, Locator, BUILT_IN_READER_GROUP, BUILT_IN_WRITER_GROUP,
            ENTITYID_PARTICIPANT, ENTITYID_UNKNOWN, USER_DEFINED_TOPIC, USER_DEFINED_WRITER_GROUP,
        },
    },
    subscription::sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
    topic_definition::type_support::DdsSerialize,
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
        topic_list: HashMap<String, TopicActor>,
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
                topic_list,
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

    pub fn get_topic_by_name(&mut self, topic_name: &str) -> DdsResult<&mut TopicActor> {
        self.topic_list
            .get_mut(topic_name)
            .ok_or(DdsError::AlreadyDeleted)
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

            let topic =
                TopicActor::new(guid, qos, type_name, &topic_name, a_listener, type_support);

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

    pub fn ignore_participant(&mut self, handle: InstanceHandle) -> DdsResult<()> {
        if self.enabled {
            self.ignored_participants.insert(handle);
            Ok(())
        } else {
            Err(DdsError::NotEnabled)
        }
    }

    pub fn ignore_subscription(&mut self, handle: InstanceHandle) -> DdsResult<()> {
        if self.enabled {
            self.ignored_subcriptions.insert(handle);
            Ok(())
        } else {
            Err(DdsError::NotEnabled)
        }
    }

    pub fn ignore_publication(&mut self, handle: InstanceHandle) -> DdsResult<()> {
        if self.enabled {
            self.ignored_publications.insert(handle);
            Ok(())
        } else {
            Err(DdsError::NotEnabled)
        }
    }

    pub fn set_default_publisher_qos(&mut self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => PublisherQos::default(),
            QosKind::Specific(q) => q,
        };

        self.default_publisher_qos = qos;
        Ok(())
    }

    pub fn get_default_publisher_qos(&self) -> DdsResult<PublisherQos> {
        Ok(self.default_publisher_qos.clone())
    }

    pub fn set_default_subscriber_qos(&mut self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => SubscriberQos::default(),
            QosKind::Specific(q) => q,
        };

        self.default_subscriber_qos = qos;

        Ok(())
    }

    pub fn get_default_subscriber_qos(&self) -> DdsResult<SubscriberQos> {
        Ok(self.default_subscriber_qos.clone())
    }

    pub fn set_default_topic_qos(&mut self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => TopicQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        self.default_topic_qos = qos;

        Ok(())
    }

    pub fn get_default_topic_qos(&self) -> DdsResult<TopicQos> {
        Ok(self.default_topic_qos.clone())
    }

    pub fn get_discovered_participants(&self) -> DdsResult<Vec<InstanceHandle>> {
        Ok(self.discovered_participant_list.keys().cloned().collect())
    }

    pub fn get_discovered_participant_data(
        &self,
        participant_handle: InstanceHandle,
    ) -> DdsResult<ParticipantBuiltinTopicData> {
        Ok(self
            .discovered_participant_list
            .get(&participant_handle)
            .ok_or(DdsError::PreconditionNotMet(
                "Participant with this instance handle not discovered".to_owned(),
            ))?
            .dds_participant_data
            .clone())
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
            .ok_or(DdsError::PreconditionNotMet(
                "Topic with this handle not discovered".to_owned(),
            ))
    }

    pub fn get_current_time(&self) -> Time {
        Time::now()
    }

    pub fn get_qos(&self) -> &DomainParticipantQos {
        &self.qos
    }

    pub fn set_qos(&mut self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
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

    pub fn enable(&mut self) -> DdsResult<()> {
        if !self.enabled {
            self.builtin_publisher.enable()?;

            self.enabled = true;
            self.announce_participant()?;

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
        }
        Ok(())
    }

    pub fn set_listener(
        &mut self,
        listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
    ) -> DdsResult<()> {
        if let Some(l) = self.participant_listener_thread.take() {
            l.join()?;
        }
        self.participant_listener_thread = listener.map(ParticipantListenerThread::new);
        self.status_kind = status_kind;
        Ok(())
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(InstanceHandle::new(self.guid.into()))
    }

    pub fn announce_participant(&mut self) -> DdsResult<()> {
        if self.enabled {
            let spdp_discovered_participant_data = self.as_spdp_discovered_participant_data();
            let timestamp = self.get_current_time();
            let dcps_participant_topic = self
                .topic_list
                .get_mut(DCPS_PARTICIPANT)
                .expect("DCPS Participant topic must exist");

            if let Some(dw) = self
                .builtin_publisher
                .lookup_datawriter_by_topic_name(DCPS_PARTICIPANT)
            {
                dw.write_w_timestamp(
                    spdp_discovered_participant_data.serialize_data()?,
                    timestamp,
                    dcps_participant_topic.get_type_support().as_ref(),
                )?
            }
        }
        Ok(())
    }

    fn as_spdp_discovered_participant_data(&self) -> SpdpDiscoveredParticipantData {
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
