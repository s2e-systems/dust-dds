use fnmatch_regex::glob_to_regex;

use super::{
    any_data_reader_listener::AnyDataReaderListener,
    any_data_writer_listener::AnyDataWriterListener,
    data_writer_actor::DataWriterActor,
    handle::InstanceHandleCounter,
    publisher_actor::{PublisherActor, PublisherListenerThread},
    status_condition_actor::{self, StatusConditionActor},
};
use crate::{
    builtin_topics::{
        BuiltInTopicKey, ParticipantBuiltinTopicData, PublicationBuiltinTopicData,
        SubscriptionBuiltinTopicData, TopicBuiltinTopicData, DCPS_PARTICIPANT, DCPS_PUBLICATION,
        DCPS_SUBSCRIPTION, DCPS_TOPIC,
    },
    dds_async::{
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
            discovered_reader_data::DiscoveredReaderData,
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::DiscoveredWriterData,
            spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        },
        data_representation_inline_qos::{
            parameter_id_values::{PID_KEY_HASH, PID_STATUS_INFO},
            types::{
                STATUS_INFO_DISPOSED, STATUS_INFO_DISPOSED_UNREGISTERED, STATUS_INFO_UNREGISTERED,
            },
        },
        runtime::{executor::Executor, mpsc::MpscSender, timer::TimerDriver},
        xtypes_glue::key_and_instance_handle::{
            get_instance_handle_from_serialized_foo, get_instance_handle_from_serialized_key,
            get_serialized_key_from_serialized_foo,
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
            DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            Length, LifespanQosPolicy, QosPolicyId, ReliabilityQosPolicy, ReliabilityQosPolicyKind,
            ResourceLimitsQosPolicy, TransportPriorityQosPolicy, DATA_REPRESENTATION_QOS_POLICY_ID,
            DEADLINE_QOS_POLICY_ID, DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID,
            LATENCYBUDGET_QOS_POLICY_ID, LIVELINESS_QOS_POLICY_ID, OWNERSHIP_QOS_POLICY_ID,
            PRESENTATION_QOS_POLICY_ID, RELIABILITY_QOS_POLICY_ID, XCDR_DATA_REPRESENTATION,
        },
        status::{
            InconsistentTopicStatus, LivelinessChangedStatus, LivelinessLostStatus,
            OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
            QosPolicyCount, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
            SampleLostStatus, SampleRejectedStatus, StatusKind, SubscriptionMatchedStatus,
        },
        time::{Duration, DurationKind, Time},
    },
    rtps::{
        cache_change::RtpsCacheChange,
        messages::submessage_elements::{Data, Parameter, ParameterList},
        reader::{ReaderCacheChange, ReaderHistoryCache},
        transport::Transport,
        types::{ChangeKind, Guid, SequenceNumber, TopicKind, ENTITYID_PARTICIPANT},
    },
    subscription::sample_info::{
        InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind, ANY_INSTANCE_STATE,
        ANY_SAMPLE_STATE, ANY_VIEW_STATE,
    },
    topic_definition::type_support::{DdsDeserialize, DdsSerialize, TypeSupport},
    xtypes::{
        dynamic_type::DynamicType, serialize::XTypesSerialize, xcdr_serializer::Xcdr1LeSerializer,
    },
};
use core::{future::Future, i32, pin::Pin};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
    thread::JoinHandle,
};

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
    transport: Box<dyn Transport>,
    instance_handle_counter: InstanceHandleCounter,
    domain_id: DomainId,
    qos: DomainParticipantQos,
    builtin_subscriber: SubscriberActor,
    builtin_publisher: PublisherActor,
    user_defined_subscriber_list: Vec<SubscriberActor>,
    default_subscriber_qos: SubscriberQos,
    user_defined_publisher_list: Vec<PublisherActor>,
    default_publisher_qos: PublisherQos,
    topic_list: HashMap<String, TopicActor>,
    default_topic_qos: TopicQos,
    discovered_participant_list: HashMap<InstanceHandle, SpdpDiscoveredParticipantData>,
    discovered_topic_list: HashMap<InstanceHandle, TopicBuiltinTopicData>,
    enabled: bool,
    ignored_participants: HashSet<InstanceHandle>,
    ignored_publications: HashSet<InstanceHandle>,
    ignored_subcriptions: HashSet<InstanceHandle>,
    ignored_topic_list: HashSet<InstanceHandle>,
    participant_listener_thread: Option<ParticipantListenerThread>,
    status_kind: Vec<StatusKind>,
    status_condition: Actor<StatusConditionActor>,
    executor: Executor,
    timer_driver: TimerDriver,
}

impl DomainParticipantActor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        domain_id: DomainId,
        domain_participant_qos: DomainParticipantQos,
        listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
        executor: Executor,
        timer_driver: TimerDriver,
        transport: Box<dyn Transport>,
    ) -> Self {
        let mut instance_handle_counter = InstanceHandleCounter::default();

        let mut topic_list = HashMap::new();
        let spdp_topic_participant_handle = instance_handle_counter.generate_new_instance_handle();

        let spdp_topic_participant = TopicActor {
            qos: TopicQos::default(),
            type_name: "SpdpDiscoveredParticipantData".to_string(),
            topic_name: DCPS_PARTICIPANT.to_owned(),
            instance_handle: spdp_topic_participant_handle,
            enabled: true,
            inconsistent_topic_status: InconsistentTopicStatus::default(),
            status_condition: Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            topic_listener_thread: None,
            status_kind: vec![],
            type_support: Arc::new(SpdpDiscoveredParticipantData::get_type()),
        };

        topic_list.insert(DCPS_PARTICIPANT.to_owned(), spdp_topic_participant);

        let sedp_topic_topics_handle = instance_handle_counter.generate_new_instance_handle();
        let sedp_topic_topics = TopicActor {
            qos: TopicQos::default(),
            type_name: "DiscoveredTopicData".to_string(),
            topic_name: DCPS_TOPIC.to_owned(),
            instance_handle: sedp_topic_topics_handle,
            enabled: true,
            inconsistent_topic_status: InconsistentTopicStatus::default(),
            status_condition: Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            topic_listener_thread: None,
            status_kind: vec![],
            type_support: Arc::new(DiscoveredTopicData::get_type()),
        };
        topic_list.insert(DCPS_TOPIC.to_owned(), sedp_topic_topics);

        let sedp_topic_publications_handle = instance_handle_counter.generate_new_instance_handle();
        let sedp_topic_publications = TopicActor {
            qos: TopicQos::default(),
            type_name: "DiscoveredWriterData".to_string(),
            topic_name: DCPS_PUBLICATION.to_owned(),
            instance_handle: sedp_topic_publications_handle,
            enabled: true,
            inconsistent_topic_status: InconsistentTopicStatus::default(),
            status_condition: Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            topic_listener_thread: None,
            status_kind: vec![],
            type_support: Arc::new(DiscoveredWriterData::get_type()),
        };
        topic_list.insert(DCPS_PUBLICATION.to_owned(), sedp_topic_publications);

        let sedp_topic_subscriptions_handle =
            instance_handle_counter.generate_new_instance_handle();
        let sedp_topic_subscriptions = TopicActor {
            qos: TopicQos::default(),
            type_name: "DiscoveredReaderData".to_string(),
            topic_name: DCPS_SUBSCRIPTION.to_owned(),
            instance_handle: sedp_topic_subscriptions_handle,
            enabled: true,
            inconsistent_topic_status: InconsistentTopicStatus::default(),
            status_condition: Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            topic_listener_thread: None,
            status_kind: vec![],
            type_support: Arc::new(DiscoveredReaderData::get_type()),
        };
        topic_list.insert(DCPS_SUBSCRIPTION.to_owned(), sedp_topic_subscriptions);

        let spdp_writer_qos = DataWriterQos {
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal,
            },
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast(1),
            },
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort,
                max_blocking_time: DurationKind::Finite(Duration::new(0, 0)),
            },
            ..Default::default()
        };

        let builtin_subscriber_handle = instance_handle_counter.generate_new_instance_handle();
        let mut builtin_subscriber = SubscriberActor::new(
            SubscriberQos::default(),
            None,
            vec![],
            builtin_subscriber_handle,
            &executor.handle(),
        );
        builtin_subscriber.enable();
        let spdp_reader_qos = DataReaderQos {
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal,
            },
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast(1),
            },
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort,
                max_blocking_time: DurationKind::Finite(Duration::new(0, 0)),
            },
            ..Default::default()
        };
        builtin_subscriber
            .create_datareader(
                &topic_list[DCPS_PARTICIPANT],
                QosKind::Specific(spdp_reader_qos),
                None,
                vec![],
                instance_handle_counter.generate_new_instance_handle(),
                transport.get_participant_discovery_reader(),
                &executor.handle(),
            )
            .unwrap();
        builtin_subscriber
            .create_datareader(
                &topic_list[DCPS_TOPIC],
                QosKind::Specific(sedp_data_reader_qos()),
                None,
                vec![],
                instance_handle_counter.generate_new_instance_handle(),
                transport.get_topics_discovery_reader(),
                &executor.handle(),
            )
            .unwrap();
        builtin_subscriber
            .create_datareader(
                &topic_list[DCPS_PUBLICATION],
                QosKind::Specific(sedp_data_reader_qos()),
                None,
                vec![],
                instance_handle_counter.generate_new_instance_handle(),
                transport.get_publications_discovery_reader(),
                &executor.handle(),
            )
            .unwrap();
        builtin_subscriber
            .create_datareader(
                &topic_list[DCPS_SUBSCRIPTION],
                QosKind::Specific(sedp_data_reader_qos()),
                None,
                vec![],
                instance_handle_counter.generate_new_instance_handle(),
                transport.get_subscriptions_discovery_reader(),
                &executor.handle(),
            )
            .unwrap();

        let builtin_publisher_handle = instance_handle_counter.generate_new_instance_handle();
        let mut builtin_publisher = PublisherActor {
            qos: PublisherQos::default(),
            instance_handle: builtin_publisher_handle,
            data_writer_list: vec![],
            enabled: true,
            default_datawriter_qos: DataWriterQos::default(),
            publisher_listener_thread: None,
            status_kind: vec![],
            status_condition: Actor::spawn(StatusConditionActor::default(), &executor.handle()),
        };

        builtin_publisher
            .create_datawriter(
                &topic_list[DCPS_PARTICIPANT],
                QosKind::Specific(spdp_writer_qos),
                None,
                vec![],
                instance_handle_counter.generate_new_instance_handle(),
                transport.get_participant_discovery_writer(),
                &executor.handle(),
            )
            .unwrap();
        builtin_publisher
            .create_datawriter(
                &topic_list[DCPS_TOPIC],
                QosKind::Specific(sedp_data_writer_qos()),
                None,
                vec![],
                instance_handle_counter.generate_new_instance_handle(),
                transport.get_topics_discovery_writer(),
                &executor.handle(),
            )
            .unwrap();
        builtin_publisher
            .create_datawriter(
                &topic_list[DCPS_PUBLICATION],
                QosKind::Specific(sedp_data_writer_qos()),
                None,
                vec![],
                instance_handle_counter.generate_new_instance_handle(),
                transport.get_publications_discovery_writer(),
                &executor.handle(),
            )
            .unwrap();
        builtin_publisher
            .create_datawriter(
                &topic_list[DCPS_SUBSCRIPTION],
                QosKind::Specific(sedp_data_writer_qos()),
                None,
                vec![],
                instance_handle_counter.generate_new_instance_handle(),
                transport.get_subscriptions_discovery_writer(),
                &executor.handle(),
            )
            .unwrap();

        let participant_listener_thread = listener.map(ParticipantListenerThread::new);
        let status_condition = Actor::spawn(StatusConditionActor::default(), &executor.handle());

        Self {
            transport,
            instance_handle_counter,
            domain_id,
            qos: domain_participant_qos,
            builtin_subscriber,
            builtin_publisher,
            user_defined_subscriber_list: Vec::new(),
            default_subscriber_qos: SubscriberQos::default(),
            user_defined_publisher_list: Vec::new(),
            default_publisher_qos: PublisherQos::default(),
            topic_list,
            default_topic_qos: TopicQos::default(),
            discovered_participant_list: HashMap::new(),
            discovered_topic_list: HashMap::new(),
            enabled: false,
            ignored_participants: HashSet::new(),
            ignored_publications: HashSet::new(),
            ignored_subcriptions: HashSet::new(),
            ignored_topic_list: HashSet::new(),
            participant_listener_thread,
            status_kind,
            status_condition,
            executor,
            timer_driver,
        }
    }

    pub fn get_current_time(&self) -> Time {
        Time::now()
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        InstanceHandle::new(self.transport.guid().into())
    }

    pub fn get_statuscondition(&self) -> ActorAddress<StatusConditionActor> {
        self.status_condition.address()
    }

    pub fn get_builtin_subscriber(&self) -> &SubscriberActor {
        &self.builtin_subscriber
    }

    pub fn announce_participant(&mut self) -> DdsResult<()> {
        if self.enabled {
            let participant_builtin_topic_data = ParticipantBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: self.transport.guid(),
                },
                user_data: self.qos.user_data.clone(),
            };
            let timestamp = self.get_current_time();
            let dcps_participant_topic = self
                .topic_list
                .get_mut(DCPS_PARTICIPANT)
                .expect("DCPS Participant topic must exist");

            if let Some(mut dw) = self
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|dw| dw.topic_name == DCPS_PARTICIPANT)
            {
                write_w_timestamp(
                    &mut dw,
                    participant_builtin_topic_data.serialize_data()?,
                    timestamp,
                    dcps_participant_topic.type_support.as_ref(),
                )?;
            }
        }
        Ok(())
    }

    pub fn announce_deleted_participant(&mut self) -> DdsResult<()> {
        if self.enabled {
            let participant_builtin_topic_data = ParticipantBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: self.transport.guid(),
                },
                user_data: self.qos.user_data.clone(),
            };
            let timestamp = self.get_current_time();
            let dcps_participant_topic = self
                .topic_list
                .get_mut(DCPS_PARTICIPANT)
                .expect("DCPS Participant topic must exist");

            if let Some(mut dw) = self
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|dw| dw.topic_name == DCPS_PARTICIPANT)
            {
                dispose_w_timestamp(
                    &mut dw,
                    participant_builtin_topic_data.serialize_data()?,
                    timestamp,
                    dcps_participant_topic.type_support.as_ref(),
                )?
            }
        }
        Ok(())
    }

    pub fn announce_created_or_modified_datawriter(
        &mut self,
        publication_builtin_topic_data: PublicationBuiltinTopicData,
    ) -> DdsResult<()> {
        if self.enabled {
            let timestamp = self.get_current_time();
            let dcps_publication_topic = self
                .topic_list
                .get(DCPS_PUBLICATION)
                .expect("DCPS Publication topic must exist");
            if let Some(mut dw) = self
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|dw| dw.topic_name == DCPS_PUBLICATION)
            {
                write_w_timestamp(
                    &mut dw,
                    publication_builtin_topic_data.serialize_data()?,
                    timestamp,
                    dcps_publication_topic.type_support.as_ref(),
                )?;
            }
        }
        Ok(())
    }

    pub fn announce_deleted_data_writer(
        &mut self,
        publication_builtin_topic_data: PublicationBuiltinTopicData,
    ) -> DdsResult<()> {
        if self.enabled {
            let timestamp = self.get_current_time();
            let dcps_publication_topic = self
                .topic_list
                .get(DCPS_PUBLICATION)
                .expect("DCPS Publication topic must exist");
            if let Some(mut dw) = self
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|dw| dw.topic_name == DCPS_PUBLICATION)
            {
                dispose_w_timestamp(
                    &mut dw,
                    publication_builtin_topic_data.serialize_data()?,
                    timestamp,
                    dcps_publication_topic.type_support.as_ref(),
                )?
            }
        }
        Ok(())
    }

    pub fn announce_created_or_modified_datareader(
        &mut self,
        subscription_builtin_topic_data: SubscriptionBuiltinTopicData,
    ) -> DdsResult<()> {
        if self.enabled {
            let timestamp = self.get_current_time();
            let dcps_subscription_topic = self
                .topic_list
                .get(DCPS_SUBSCRIPTION)
                .expect("DCPS Subscription topic must exist");
            if let Some(mut dw) = self
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|dw| dw.topic_name == DCPS_SUBSCRIPTION)
            {
                write_w_timestamp(
                    &mut dw,
                    subscription_builtin_topic_data.serialize_data()?,
                    timestamp,
                    dcps_subscription_topic.type_support.as_ref(),
                )?;
            }
        }
        Ok(())
    }

    pub fn announce_deleted_data_reader(
        &mut self,
        subscription_builtin_topic_data: SubscriptionBuiltinTopicData,
    ) -> DdsResult<()> {
        if self.enabled {
            let timestamp = self.get_current_time();
            let dcps_subscription_topic = self
                .topic_list
                .get(DCPS_SUBSCRIPTION)
                .expect("DCPS Subscription topic must exist");
            if let Some(mut dw) = self
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|dw| dw.topic_name == DCPS_SUBSCRIPTION)
            {
                dispose_w_timestamp(
                    &mut dw,
                    subscription_builtin_topic_data.serialize_data()?,
                    timestamp,
                    dcps_subscription_topic.type_support.as_ref(),
                )?
            }
        }
        Ok(())
    }

    pub fn add_discovered_topic(&mut self, discovered_topic_data: DiscoveredTopicData) {
        let handle =
            InstanceHandle::new(discovered_topic_data.topic_builtin_topic_data.key().value);
        let is_topic_ignored = self.ignored_topic_list.contains(&handle);
        if !is_topic_ignored {
            for topic in self.topic_list.values_mut() {
                let is_discovered_topic_consistent = topic.qos.topic_data
                    == discovered_topic_data.topic_builtin_topic_data.topic_data
                    && topic.qos.durability
                        == discovered_topic_data.topic_builtin_topic_data.durability
                    && topic.qos.deadline
                        == discovered_topic_data.topic_builtin_topic_data.deadline
                    && topic.qos.latency_budget
                        == discovered_topic_data
                            .topic_builtin_topic_data
                            .latency_budget
                    && topic.qos.liveliness
                        == discovered_topic_data.topic_builtin_topic_data.liveliness
                    && topic.qos.reliability
                        == discovered_topic_data.topic_builtin_topic_data.reliability
                    && topic.qos.destination_order
                        == discovered_topic_data
                            .topic_builtin_topic_data
                            .destination_order
                    && topic.qos.history == discovered_topic_data.topic_builtin_topic_data.history
                    && topic.qos.resource_limits
                        == discovered_topic_data
                            .topic_builtin_topic_data
                            .resource_limits
                    && topic.qos.transport_priority
                        == discovered_topic_data
                            .topic_builtin_topic_data
                            .transport_priority
                    && topic.qos.lifespan
                        == discovered_topic_data.topic_builtin_topic_data.lifespan
                    && topic.qos.ownership
                        == discovered_topic_data.topic_builtin_topic_data.ownership;
                if discovered_topic_data.topic_builtin_topic_data.type_name == topic.type_name
                    && discovered_topic_data.topic_builtin_topic_data.name == topic.topic_name
                    && !is_discovered_topic_consistent
                {
                    topic.inconsistent_topic_status.total_count += 1;
                    topic.inconsistent_topic_status.total_count_change += 1;
                    topic.status_condition.send_actor_mail(
                        status_condition_actor::AddCommunicationState {
                            state: StatusKind::InconsistentTopic,
                        },
                    );
                }
            }
            self.discovered_topic_list
                .insert(handle, discovered_topic_data.topic_builtin_topic_data);
        }
    }

    pub fn add_discovered_writer(&mut self, discovered_writer_data: DiscoveredWriterData) {
        let discovered_writer_participant_guid = Guid::new(
            discovered_writer_data
                .writer_proxy
                .remote_writer_guid
                .prefix(),
            ENTITYID_PARTICIPANT,
        );
        let is_participant_ignored = self.ignored_participants.contains(&InstanceHandle::new(
            discovered_writer_participant_guid.into(),
        ));
        let is_publication_ignored = self.ignored_publications.contains(&InstanceHandle::new(
            discovered_writer_data.dds_publication_data.key().value,
        ));
        if !is_publication_ignored && !is_participant_ignored {
            if let Some(_) = self.discovered_participant_list.get(&InstanceHandle::new(
                discovered_writer_participant_guid.into(),
            )) {
                for subscriber in self.user_defined_subscriber_list.iter_mut() {
                    subscriber.add_matched_writer(&discovered_writer_data);
                }

                // Add writer topic to discovered topic list using the writer instance handle
                let topic_instance_handle =
                    InstanceHandle::new(discovered_writer_data.dds_publication_data.key().value);
                let writer_topic = TopicBuiltinTopicData {
                    key: BuiltInTopicKey::default(),
                    name: discovered_writer_data
                        .dds_publication_data
                        .topic_name()
                        .to_owned(),
                    type_name: discovered_writer_data
                        .dds_publication_data
                        .get_type_name()
                        .to_owned(),
                    durability: discovered_writer_data
                        .dds_publication_data
                        .durability()
                        .clone(),
                    deadline: discovered_writer_data
                        .dds_publication_data
                        .deadline()
                        .clone(),
                    latency_budget: discovered_writer_data
                        .dds_publication_data
                        .latency_budget()
                        .clone(),
                    liveliness: discovered_writer_data
                        .dds_publication_data
                        .liveliness()
                        .clone(),
                    reliability: discovered_writer_data
                        .dds_publication_data
                        .reliability()
                        .clone(),
                    transport_priority: TransportPriorityQosPolicy::default(),
                    lifespan: discovered_writer_data
                        .dds_publication_data
                        .lifespan()
                        .clone(),
                    destination_order: discovered_writer_data
                        .dds_publication_data
                        .destination_order()
                        .clone(),
                    history: HistoryQosPolicy::default(),
                    resource_limits: ResourceLimitsQosPolicy::default(),
                    ownership: discovered_writer_data
                        .dds_publication_data
                        .ownership()
                        .clone(),
                    topic_data: discovered_writer_data
                        .dds_publication_data
                        .topic_data()
                        .clone(),
                    representation: discovered_writer_data
                        .dds_publication_data
                        .representation()
                        .clone(),
                };

                self.discovered_topic_list
                    .insert(topic_instance_handle, writer_topic);
            }
        }
    }

    pub fn add_discovered_reader(&mut self, discovered_reader_data: DiscoveredReaderData) {
        let discovered_reader_participant_guid = Guid::new(
            discovered_reader_data
                .reader_proxy()
                .remote_reader_guid
                .prefix(),
            ENTITYID_PARTICIPANT,
        );
        let is_participant_ignored = self.ignored_participants.contains(&InstanceHandle::new(
            discovered_reader_participant_guid.into(),
        ));
        let is_subscription_ignored = self.ignored_subcriptions.contains(&InstanceHandle::new(
            discovered_reader_data
                .subscription_builtin_topic_data()
                .key()
                .value,
        ));
        if !is_subscription_ignored && !is_participant_ignored {
            if let Some(_) = self.discovered_participant_list.get(&InstanceHandle::new(
                discovered_reader_participant_guid.into(),
            )) {
                for publisher in self.user_defined_publisher_list.iter_mut() {
                    let is_any_name_matched = discovered_reader_data
                        .dds_subscription_data
                        .partition
                        .name
                        .iter()
                        .any(|n| publisher.qos.partition.name.contains(n));

                    let is_any_received_regex_matched_with_partition_qos = discovered_reader_data
                        .dds_subscription_data
                        .partition
                        .name
                        .iter()
                        .filter_map(|n| glob_to_regex(n).ok())
                        .any(|regex| {
                            publisher
                                .qos
                                .partition
                                .name
                                .iter()
                                .any(|n| regex.is_match(n))
                        });

                    let is_any_local_regex_matched_with_received_partition_qos = publisher
                        .qos
                        .partition
                        .name
                        .iter()
                        .filter_map(|n| glob_to_regex(n).ok())
                        .any(|regex| {
                            discovered_reader_data
                                .dds_subscription_data
                                .partition
                                .name
                                .iter()
                                .any(|n| regex.is_match(n))
                        });

                    let is_partition_matched =
                        discovered_reader_data.dds_subscription_data.partition
                            == publisher.qos.partition
                            || is_any_name_matched
                            || is_any_received_regex_matched_with_partition_qos
                            || is_any_local_regex_matched_with_received_partition_qos;

                    if is_partition_matched {
                        for dw in publisher.data_writer_list.iter_mut().filter(|dw| {
                            dw.topic_name
                                == discovered_reader_data
                                    .subscription_builtin_topic_data()
                                    .topic_name()
                        }) {
                            let is_matched_topic_name = discovered_reader_data
                                .subscription_builtin_topic_data()
                                .topic_name()
                                == dw.topic_name;
                            let is_matched_type_name = discovered_reader_data
                                .subscription_builtin_topic_data()
                                .get_type_name()
                                == dw.type_name;

                            if is_matched_topic_name && is_matched_type_name {
                                let incompatible_qos_policy_list =
                                    get_discovered_reader_incompatible_qos_policy_list(
                                        &dw.qos,
                                        discovered_reader_data.subscription_builtin_topic_data(),
                                        &publisher.qos,
                                    );
                                let instance_handle = InstanceHandle::new(
                                    discovered_reader_data
                                        .subscription_builtin_topic_data()
                                        .key
                                        .value,
                                );
                                if incompatible_qos_policy_list.is_empty() {
                                    if dw.matched_subscription_list.get(&instance_handle)
                                        != Some(
                                            discovered_reader_data
                                                .subscription_builtin_topic_data(),
                                        )
                                    {
                                        dw.matched_subscription_list.insert(
                                            instance_handle,
                                            discovered_reader_data
                                                .subscription_builtin_topic_data()
                                                .clone(),
                                        );
                                        dw.publication_matched_status.current_count =
                                            dw.matched_subscription_list.len() as i32;
                                        dw.publication_matched_status.current_count_change += 1;
                                        dw.publication_matched_status.total_count += 1;
                                        dw.publication_matched_status.total_count_change += 1;
                                        // let type_name = self.type_name.clone();
                                        // let topic_name = self.topic_name.clone();
                                        // let participant = publisher.get_participant();
                                        // let status_condition_address = self.status_condition.address();
                                        // let topic_status_condition_address = self.topic_status_condition.clone();
                                        // let topic = TopicAsync::new(
                                        //     self.topic_address.clone(),
                                        //     topic_status_condition_address,
                                        //     type_name,
                                        //     topic_name,
                                        //     participant,
                                        // );
                                        // if self.status_kind.contains(&StatusKind::PublicationMatched) {
                                        //     let status = self.matched_subscriptions.get_publication_matched_status();
                                        //     if let Some(listener) = &self.data_writer_listener_thread {
                                        //         listener.sender().send(DataWriterListenerMessage {
                                        //             listener_operation: DataWriterListenerOperation::PublicationMatched(status),
                                        //             writer_address: data_writer_address,
                                        //             status_condition_address,
                                        //             publisher,
                                        //             topic,
                                        //         })?;
                                        //     }
                                        // } else if publisher_listener_mask.contains(&StatusKind::PublicationMatched) {
                                        //     let status = self.matched_subscriptions.get_publication_matched_status();
                                        //     if let Some(listener) = publisher_listener {
                                        //         listener.send(PublisherListenerMessage {
                                        //             listener_operation: PublisherListenerOperation::PublicationMatched(status),
                                        //             writer_address: data_writer_address,
                                        //             status_condition_address,
                                        //             publisher,
                                        //             topic,
                                        //         })?;
                                        //     }
                                        // } else if participant_listener_mask.contains(&StatusKind::PublicationMatched) {
                                        //     let status = self.matched_subscriptions.get_publication_matched_status();
                                        //     if let Some(listener) = participant_listener {
                                        //         listener.send(ParticipantListenerMessage {
                                        //             listener_operation: ParticipantListenerOperation::PublicationMatched(status),
                                        //             listener_kind: ListenerKind::Writer {
                                        //                 writer_address: data_writer_address,
                                        //                 status_condition_address,
                                        //                 publisher,
                                        //                 topic,
                                        //             },
                                        //         })?;
                                        //     }
                                        // }
                                        dw.status_condition.send_actor_mail(
                                            status_condition_actor::AddCommunicationState {
                                                state: StatusKind::PublicationMatched,
                                            },
                                        );
                                    }
                                } else if !dw
                                    .incompatible_subscription_list
                                    .contains(&instance_handle)
                                {
                                    dw.offered_incompatible_qos_status.total_count += 1;
                                    dw.offered_incompatible_qos_status.last_policy_id =
                                        incompatible_qos_policy_list[0];

                                    dw.incompatible_subscription_list.insert(instance_handle);
                                    for incompatible_qos_policy in
                                        incompatible_qos_policy_list.into_iter()
                                    {
                                        if let Some(policy_count) = dw
                                            .offered_incompatible_qos_status
                                            .policies
                                            .iter_mut()
                                            .find(|x| x.policy_id == incompatible_qos_policy)
                                        {
                                            policy_count.count += 1;
                                        } else {
                                            dw.offered_incompatible_qos_status.policies.push(
                                                QosPolicyCount {
                                                    policy_id: incompatible_qos_policy,
                                                    count: 1,
                                                },
                                            )
                                        }
                                    }
                                    // let type_name = self.type_name.clone();
                                    // let topic_name = self.topic_name.clone();
                                    // let participant = publisher.get_participant();
                                    // let status_condition_address = self.status_condition.address();
                                    // let topic_status_condition_address = self.topic_status_condition.clone();
                                    // let topic = TopicAsync::new(
                                    //     self.topic_address.clone(),
                                    //     topic_status_condition_address,
                                    //     type_name,
                                    //     topic_name,
                                    //     participant,
                                    // );

                                    // if self
                                    //     .status_kind
                                    //     .contains(&StatusKind::OfferedIncompatibleQos)
                                    // {
                                    //     let status = self
                                    //         .incompatible_subscriptions
                                    //         .get_offered_incompatible_qos_status();
                                    //     if let Some(listener) = &self.data_writer_listener_thread {
                                    //         listener.sender().send(DataWriterListenerMessage {
                                    //             listener_operation: DataWriterListenerOperation::OfferedIncompatibleQos(status),
                                    //             writer_address: data_writer_address,
                                    //             status_condition_address,
                                    //             publisher,
                                    //             topic,
                                    //         })?;
                                    //     }
                                    // } else if publisher_listener_mask.contains(&StatusKind::OfferedIncompatibleQos) {
                                    //     let status = self
                                    //         .incompatible_subscriptions
                                    //         .get_offered_incompatible_qos_status();

                                    //     if let Some(listener) = publisher_listener {
                                    //         listener.send(PublisherListenerMessage {
                                    //             listener_operation: PublisherListenerOperation::OfferedIncompatibleQos(status),
                                    //             writer_address: data_writer_address,
                                    //             status_condition_address,
                                    //             publisher,
                                    //             topic,
                                    //         })?;
                                    //     }
                                    // } else if participant_listener_mask.contains(&StatusKind::OfferedIncompatibleQos) {
                                    //     let status = self
                                    //         .incompatible_subscriptions
                                    //         .get_offered_incompatible_qos_status();
                                    //     if let Some(listener) = participant_listener {
                                    //         listener.send(ParticipantListenerMessage {
                                    //             listener_operation: ParticipantListenerOperation::OfferedIncompatibleQos(
                                    //                 status,
                                    //             ),
                                    //             listener_kind: ListenerKind::Writer {
                                    //                 writer_address: data_writer_address,
                                    //                 status_condition_address,
                                    //                 publisher,
                                    //                 topic,
                                    //             },
                                    //         })?;
                                    //     }
                                    // }
                                    dw.status_condition.send_actor_mail(
                                        status_condition_actor::AddCommunicationState {
                                            state: StatusKind::OfferedIncompatibleQos,
                                        },
                                    );
                                }
                            }
                        }
                    }
                }

                // Add reader topic to discovered topic list using the reader instance handle
                let topic_instance_handle = InstanceHandle::new(
                    discovered_reader_data
                        .subscription_builtin_topic_data()
                        .key()
                        .value,
                );
                let reader_topic = TopicBuiltinTopicData {
                    key: BuiltInTopicKey::default(),
                    name: discovered_reader_data
                        .subscription_builtin_topic_data()
                        .topic_name()
                        .to_string(),
                    type_name: discovered_reader_data
                        .subscription_builtin_topic_data()
                        .get_type_name()
                        .to_string(),

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
                    representation: discovered_reader_data
                        .subscription_builtin_topic_data()
                        .representation()
                        .clone(),
                };
                self.discovered_topic_list
                    .insert(topic_instance_handle, reader_topic);
            }
        }
    }

    pub fn remove_discovered_writer(&mut self, discovered_writer_handle: InstanceHandle) {
        for subscriber in self.user_defined_subscriber_list.iter_mut() {
            subscriber.remove_matched_writer(discovered_writer_handle);
        }
    }

    pub fn add_discovered_participant(
        &mut self,
        discovered_participant_data: SpdpDiscoveredParticipantData,
    ) {
        self.discovered_participant_list.insert(
            InstanceHandle::new(discovered_participant_data.dds_participant_data.key().value),
            discovered_participant_data,
        );
    }

    pub fn remove_discovered_participant(&mut self, discovered_participant_handle: InstanceHandle) {
        self.discovered_participant_list
            .remove(&discovered_participant_handle);
    }

    pub fn announce_topic(
        &mut self,
        topic_builtin_topic_data: TopicBuiltinTopicData,
    ) -> DdsResult<()> {
        if self.enabled {
            let timestamp = self.get_current_time();
            let dcps_topic_topic = self
                .topic_list
                .get(DCPS_TOPIC)
                .expect("DCPS Topic topic must exist");

            if let Some(mut dw) = self
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|dw| dw.topic_name == DCPS_TOPIC)
            {
                write_w_timestamp(
                    &mut dw,
                    topic_builtin_topic_data.serialize_data()?,
                    timestamp,
                    dcps_topic_topic.type_support.as_ref(),
                )?;
            }
        }
        Ok(())
    }
}
// ############################  Domain participant messages
pub struct CreateUserDefinedPublisher {
    pub qos: QosKind<PublisherQos>,
    pub a_listener: Option<Box<dyn PublisherListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
}
impl Mail for CreateUserDefinedPublisher {
    type Result = DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>;
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

        let publisher_handle = self.instance_handle_counter.generate_new_instance_handle();
        let mut publisher = PublisherActor {
            qos: publisher_qos,
            instance_handle: publisher_handle,
            data_writer_list: vec![],
            enabled: false,
            default_datawriter_qos: DataWriterQos::default(),
            publisher_listener_thread: message.a_listener.map(PublisherListenerThread::new),
            status_kind: message.mask,
            status_condition: Actor::spawn(
                StatusConditionActor::default(),
                &self.executor.handle(),
            ),
        };
        let publisher_status_condition_address = publisher.status_condition.address();

        if self.enabled && self.qos.entity_factory.autoenable_created_entities {
            publisher.enabled = true;
        }

        self.user_defined_publisher_list.push(publisher);

        Ok((publisher_handle.into(), publisher_status_condition_address))
    }
}

pub struct DeleteUserDefinedPublisher {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
}
impl Mail for DeleteUserDefinedPublisher {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteUserDefinedPublisher> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteUserDefinedPublisher,
    ) -> <DeleteUserDefinedPublisher as Mail>::Result {
        if message.participant_handle != self.get_instance_handle() {
            return Err(DdsError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant".to_string(),
            ));
        }

        if let Some(i) = self
            .user_defined_publisher_list
            .iter()
            .position(|p| p.instance_handle == message.publisher_handle)
        {
            if self.user_defined_publisher_list[i]
                .data_writer_list
                .is_empty()
            {
                self.user_defined_publisher_list.remove(i);
                Ok(())
            } else {
                Err(DdsError::PreconditionNotMet(
                    "Publisher still contains data writers".to_string(),
                ))
            }
        } else {
            Err(DdsError::AlreadyDeleted)
        }
    }
}

pub struct CreateUserDefinedSubscriber {
    pub qos: QosKind<SubscriberQos>,
    pub a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
}
impl Mail for CreateUserDefinedSubscriber {
    type Result = DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>;
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
        let subscriber_handle = self.instance_handle_counter.generate_new_instance_handle();
        let subscriber_status_kind = message.mask.to_vec();

        let mut subscriber = SubscriberActor::new(
            subscriber_qos,
            message.a_listener,
            subscriber_status_kind,
            subscriber_handle,
            &self.executor.handle(),
        );

        let subscriber_status_condition_address = subscriber.get_statuscondition();

        if self.enabled && self.qos.entity_factory.autoenable_created_entities {
            subscriber.enable();
        }

        self.user_defined_subscriber_list.push(subscriber);

        Ok((
            subscriber_handle.into(),
            subscriber_status_condition_address,
        ))
    }
}

pub struct DeleteUserDefinedSubscriber {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
}
impl Mail for DeleteUserDefinedSubscriber {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteUserDefinedSubscriber> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteUserDefinedSubscriber,
    ) -> <DeleteUserDefinedSubscriber as Mail>::Result {
        if self.get_instance_handle() != message.participant_handle {
            return Err(DdsError::PreconditionNotMet(
                "Subscriber can only be deleted from its parent participant".to_string(),
            ));
        }

        if let Some(i) = self
            .user_defined_subscriber_list
            .iter()
            .position(|s| s.get_instance_handle() == message.subscriber_handle)
        {
            if self.user_defined_subscriber_list[i].is_empty() {
                self.user_defined_subscriber_list.remove(i);
                Ok(())
            } else {
                Err(DdsError::PreconditionNotMet(
                    "Subscriber still contains data readers".to_string(),
                ))
            }
        } else {
            Err(DdsError::AlreadyDeleted)
        }
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
    type Result = DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>;
}
impl MailHandler<CreateUserDefinedTopic> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: CreateUserDefinedTopic,
    ) -> <CreateUserDefinedTopic as Mail>::Result {
        if self.topic_list.contains_key(&message.topic_name) {
            return Err(DdsError::PreconditionNotMet(format!("Topic with name {} already exists. To access this topic call the lookup_topicdescription method.",message.topic_name)));
        }

        let qos = match message.qos {
            QosKind::Default => self.default_topic_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let topic_handle = self.instance_handle_counter.generate_new_instance_handle();
        let mut topic = TopicActor {
            qos,
            type_name: message.type_name,
            topic_name: message.topic_name.clone(),
            instance_handle: topic_handle,
            enabled: false,
            inconsistent_topic_status: InconsistentTopicStatus::default(),
            status_condition: Actor::spawn(
                StatusConditionActor::default(),
                &self.executor.handle(),
            ),
            topic_listener_thread: None,
            status_kind: vec![],
            type_support: message.type_support,
        };
        let topic_status_condition_address = topic.status_condition.address();

        if self.enabled && self.qos.entity_factory.autoenable_created_entities {
            topic.enabled = true;
            let topic_qos = topic.qos.clone();
            let topic_builtin_topic_data = TopicBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: topic.instance_handle.into(),
                },
                name: topic.topic_name.clone(),
                type_name: topic.type_name.clone(),
                durability: topic_qos.durability,
                deadline: topic_qos.deadline,
                latency_budget: topic_qos.latency_budget,
                liveliness: topic_qos.liveliness,
                reliability: topic_qos.reliability,
                transport_priority: topic_qos.transport_priority,
                lifespan: topic_qos.lifespan,
                destination_order: topic_qos.destination_order,
                history: topic_qos.history,
                resource_limits: topic_qos.resource_limits,
                ownership: topic_qos.ownership,
                topic_data: topic_qos.topic_data,
                representation: topic_qos.representation,
            };
            self.announce_topic(topic_builtin_topic_data)?;
        }

        self.topic_list.insert(message.topic_name, topic);

        Ok((topic_handle.into(), topic_status_condition_address))
    }
}

pub struct DeleteUserDefinedTopic {
    pub participant_handle: InstanceHandle,
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
        if self.get_instance_handle() != message.participant_handle {
            return Err(DdsError::PreconditionNotMet(
                "Topic can only be deleted from its parent participant".to_string(),
            ));
        }

        if BUILT_IN_TOPIC_NAME_LIST.contains(&message.topic_name.as_str()) {
            return Ok(());
        }

        for publisher in self.user_defined_publisher_list.iter_mut() {
            if publisher
                .data_writer_list
                .iter_mut()
                .find(|dw| dw.topic_name == message.topic_name)
                .is_some()
            {
                return Err(DdsError::PreconditionNotMet(
                    "Topic still attached to some data writer".to_string(),
                ));
            }
        }

        for subscriber in self.user_defined_subscriber_list.iter_mut() {
            if subscriber
                .lookup_datareader_by_topic_name(&message.topic_name)
                .is_some()
            {
                return Err(DdsError::PreconditionNotMet(
                    "Topic still attached to some data reader".to_string(),
                ));
            }
        }

        self.topic_list
            .remove(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?;

        Ok(())
    }
}

pub struct FindTopic {
    pub topic_name: String,
    pub type_support: Arc<dyn DynamicType + Send + Sync>,
}
impl Mail for FindTopic {
    type Result = DdsResult<Option<(InstanceHandle, ActorAddress<StatusConditionActor>, String)>>;
}
impl MailHandler<FindTopic> for DomainParticipantActor {
    fn handle(&mut self, message: FindTopic) -> <FindTopic as Mail>::Result {
        if let Some(topic) = self.topic_list.get(&message.topic_name) {
            Ok(Some((
                topic.instance_handle.into(),
                topic.status_condition.address(),
                topic.type_name.to_owned(),
            )))
        } else {
            for discovered_topic_data in self.discovered_topic_list.values() {
                if discovered_topic_data.name() == message.topic_name {
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
                    let type_name = discovered_topic_data.type_name.clone();
                    let topic_handle = self.instance_handle_counter.generate_new_instance_handle();
                    let topic = TopicActor {
                        qos,
                        type_name: type_name.clone(),
                        topic_name: message.topic_name.clone(),
                        instance_handle: topic_handle,
                        enabled: true,
                        inconsistent_topic_status: InconsistentTopicStatus::default(),
                        status_condition: Actor::spawn(
                            StatusConditionActor::default(),
                            &self.executor.handle(),
                        ),
                        topic_listener_thread: None,
                        status_kind: vec![],
                        type_support: message.type_support,
                    };
                    let topic_status_condition_address = topic.status_condition.address();

                    self.topic_list.insert(message.topic_name, topic);
                    return Ok(Some((
                        topic_handle.into(),
                        topic_status_condition_address,
                        type_name,
                    )));
                }
            }
            Ok(None)
        }
    }
}

pub struct LookupTopicdescription {
    pub topic_name: String,
}
impl Mail for LookupTopicdescription {
    type Result = DdsResult<Option<(String, InstanceHandle, ActorAddress<StatusConditionActor>)>>;
}
impl MailHandler<LookupTopicdescription> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: LookupTopicdescription,
    ) -> <LookupTopicdescription as Mail>::Result {
        if let Some(topic) = self.topic_list.get(&message.topic_name) {
            Ok(Some((
                topic.type_name.clone(),
                topic.instance_handle.into(),
                topic.status_condition.address(),
            )))
        } else {
            Ok(None)
        }
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
        _: DeleteParticipantContainedEntities,
    ) -> <DeleteParticipantContainedEntities as Mail>::Result {
        let deleted_publisher_list: Vec<PublisherActor> =
            self.user_defined_publisher_list.drain(..).collect();
        for mut publisher in deleted_publisher_list {
            for data_writer in publisher.data_writer_list.drain(..) {
                let publication_builtin_topic_data = PublicationBuiltinTopicData {
                    key: BuiltInTopicKey {
                        value: data_writer.transport_writer.guid(),
                    },
                    participant_key: BuiltInTopicKey { value: [0; 16] },
                    topic_name: data_writer.topic_name.clone(),
                    type_name: data_writer.type_name.clone(),
                    durability: data_writer.qos.durability.clone(),
                    deadline: data_writer.qos.deadline.clone(),
                    latency_budget: data_writer.qos.latency_budget.clone(),
                    liveliness: data_writer.qos.liveliness.clone(),
                    reliability: data_writer.qos.reliability.clone(),
                    lifespan: data_writer.qos.lifespan.clone(),
                    user_data: data_writer.qos.user_data.clone(),
                    ownership: data_writer.qos.ownership.clone(),
                    ownership_strength: data_writer.qos.ownership_strength.clone(),
                    destination_order: data_writer.qos.destination_order.clone(),
                    presentation: publisher.qos.presentation.clone(),
                    partition: publisher.qos.partition.clone(),
                    topic_data: self.topic_list[&data_writer.topic_name]
                        .qos
                        .topic_data
                        .clone(),
                    group_data: publisher.qos.group_data.clone(),
                    representation: data_writer.qos.representation.clone(),
                };
                self.announce_deleted_data_writer(publication_builtin_topic_data)?;
            }
        }

        let deleted_subscriber_list: Vec<SubscriberActor> =
            self.user_defined_subscriber_list.drain(..).collect();
        for mut subscriber in deleted_subscriber_list {
            for data_reader in subscriber.delete_contained_entities() {
                let topic_qos = self.topic_list[data_reader.get_topic_name()].qos.clone();
                let subscription_builtin_topic_data = data_reader
                    .as_subscription_builtin_topic_data(subscriber.get_qos(), &topic_qos);
                self.announce_deleted_data_reader(subscription_builtin_topic_data)?;
            }
        }

        self.topic_list
            .retain(|_, x| BUILT_IN_TOPIC_NAME_LIST.contains(&x.topic_name.as_ref()));

        Ok(())
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
    type Result = DdsResult<PublisherQos>;
}
impl MailHandler<GetDefaultPublisherQos> for DomainParticipantActor {
    fn handle(&mut self, _: GetDefaultPublisherQos) -> <GetDefaultPublisherQos as Mail>::Result {
        Ok(self.default_publisher_qos.clone())
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
    type Result = DdsResult<SubscriberQos>;
}
impl MailHandler<GetDefaultSubscriberQos> for DomainParticipantActor {
    fn handle(&mut self, _: GetDefaultSubscriberQos) -> <GetDefaultSubscriberQos as Mail>::Result {
        Ok(self.default_subscriber_qos.clone())
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
    type Result = DdsResult<TopicQos>;
}
impl MailHandler<GetDefaultTopicQos> for DomainParticipantActor {
    fn handle(&mut self, _: GetDefaultTopicQos) -> <GetDefaultTopicQos as Mail>::Result {
        Ok(self.default_topic_qos.clone())
    }
}

pub struct GetDiscoveredParticipants;
impl Mail for GetDiscoveredParticipants {
    type Result = DdsResult<Vec<InstanceHandle>>;
}
impl MailHandler<GetDiscoveredParticipants> for DomainParticipantActor {
    fn handle(
        &mut self,
        _: GetDiscoveredParticipants,
    ) -> <GetDiscoveredParticipants as Mail>::Result {
        Ok(self.discovered_participant_list.keys().cloned().collect())
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
            .ok_or(DdsError::BadParameter)?
            .dds_participant_data
            .clone())
    }
}

pub struct GetDiscoveredTopics;
impl Mail for GetDiscoveredTopics {
    type Result = DdsResult<Vec<InstanceHandle>>;
}
impl MailHandler<GetDiscoveredTopics> for DomainParticipantActor {
    fn handle(&mut self, _: GetDiscoveredTopics) -> <GetDiscoveredTopics as Mail>::Result {
        Ok(self.discovered_topic_list.keys().cloned().collect())
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
        self.get_current_time()
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
        let qos = match message.qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.qos = qos;
        self.announce_participant()
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
        if !self.enabled {
            self.enabled = true;
            self.announce_participant()?;
        }
        Ok(())
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
        let topic = self
            .topic_list
            .get_mut(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?;

        let status = topic.inconsistent_topic_status.clone();
        topic.inconsistent_topic_status.total_count_change = 0;
        topic
            .status_condition
            .send_actor_mail(status_condition_actor::RemoveCommunicationState {
                state: StatusKind::InconsistentTopic,
            });
        Ok(status)
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
        let topic = self
            .topic_list
            .get_mut(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?;

        let qos = match message.topic_qos {
            QosKind::Default => self.default_topic_qos.clone(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        if topic.enabled {
            if topic.qos.durability != qos.durability
                || qos.liveliness != qos.liveliness
                || qos.reliability != qos.reliability
                || qos.destination_order != qos.destination_order
                || qos.history != qos.history
                || qos.resource_limits != qos.resource_limits
                || qos.ownership != qos.ownership
            {
                return Err(DdsError::ImmutablePolicy);
            }
        }

        topic.qos = qos;

        Ok(())
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
            .get(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?
            .qos
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
        self.topic_list
            .get_mut(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?
            .enabled = true;
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
            .get_mut(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?
            .type_support
            .clone())
    }
}

// ############################  Publisher messages
pub struct CreateUserDefinedDataWriter {
    pub publisher_handle: InstanceHandle,
    pub topic_name: String,
    pub qos: QosKind<DataWriterQos>,
    pub a_listener: Option<Box<dyn AnyDataWriterListener + Send>>,
    pub mask: Vec<StatusKind>,
}
impl Mail for CreateUserDefinedDataWriter {
    type Result = DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>;
}
impl MailHandler<CreateUserDefinedDataWriter> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: CreateUserDefinedDataWriter,
    ) -> <CreateUserDefinedDataWriter as Mail>::Result {
        let publisher = self
            .user_defined_publisher_list
            .iter_mut()
            .find(|p| p.instance_handle == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        let topic = self
            .topic_list
            .get(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?;

        let topic_kind = {
            let mut topic_kind = TopicKind::NoKey;
            for index in 0..topic.type_support.get_member_count() {
                if topic
                    .type_support
                    .get_member_by_index(index)?
                    .get_descriptor()?
                    .is_key
                {
                    topic_kind = TopicKind::WithKey;
                    break;
                }
            }
            topic_kind
        };

        let transport_writer = self
            .transport
            .create_user_defined_writer(&message.topic_name, topic_kind);
        let writer_handle = self.instance_handle_counter.generate_new_instance_handle();
        let (datawriter_handle, writer_status_condition_address) = publisher.create_datawriter(
            topic,
            message.qos,
            message.a_listener,
            message.mask,
            writer_handle,
            transport_writer,
            &self.executor.handle(),
        )?;
        if publisher.enabled && publisher.qos.entity_factory.autoenable_created_entities {
            publisher
                .data_writer_list
                .iter_mut()
                .find(|x| x.instance_handle == datawriter_handle)
                .ok_or(DdsError::AlreadyDeleted)?
                .enabled = true;

            if let Some(dcps_subscription_reader) = self
                .builtin_subscriber
                .lookup_datareader_by_topic_name(DCPS_SUBSCRIPTION)
            {
                if let Ok(sample_list) = dcps_subscription_reader.read(
                    i32::MAX,
                    ANY_SAMPLE_STATE,
                    ANY_VIEW_STATE,
                    &[InstanceStateKind::Alive],
                    None,
                ) {
                    for (sample_data, _) in sample_list {
                        if let Ok(discovered_reader_data) = DiscoveredReaderData::deserialize_data(
                            sample_data
                                .expect("Alive samples should always contain data")
                                .as_ref(),
                        ) {
                            let is_any_name_matched = discovered_reader_data
                                .dds_subscription_data
                                .partition
                                .name
                                .iter()
                                .any(|n| publisher.qos.partition.name.contains(n));

                            let is_any_received_regex_matched_with_partition_qos =
                                discovered_reader_data
                                    .dds_subscription_data
                                    .partition
                                    .name
                                    .iter()
                                    .filter_map(|n| glob_to_regex(n).ok())
                                    .any(|regex| {
                                        publisher
                                            .qos
                                            .partition
                                            .name
                                            .iter()
                                            .any(|n| regex.is_match(n))
                                    });

                            let is_any_local_regex_matched_with_received_partition_qos = publisher
                                .qos
                                .partition
                                .name
                                .iter()
                                .filter_map(|n| glob_to_regex(n).ok())
                                .any(|regex| {
                                    discovered_reader_data
                                        .dds_subscription_data
                                        .partition
                                        .name
                                        .iter()
                                        .any(|n| regex.is_match(n))
                                });

                            let is_partition_matched =
                                discovered_reader_data.dds_subscription_data.partition
                                    == publisher.qos.partition
                                    || is_any_name_matched
                                    || is_any_received_regex_matched_with_partition_qos
                                    || is_any_local_regex_matched_with_received_partition_qos;

                            if is_partition_matched {
                                for dw in publisher.data_writer_list.iter_mut().filter(|dw| {
                                    dw.topic_name
                                        == discovered_reader_data
                                            .subscription_builtin_topic_data()
                                            .topic_name()
                                }) {
                                    todo!()
                                    // dw.add_matched_reader(&discovered_reader_data, &publisher.qos);
                                }
                            }
                        }
                    }
                }
            }

            let data_writer = publisher
                .data_writer_list
                .iter()
                .find(|x| x.instance_handle == datawriter_handle)
                .ok_or(DdsError::AlreadyDeleted)?;
            let publication_builtin_topic_data = PublicationBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: data_writer.transport_writer.guid(),
                },
                participant_key: BuiltInTopicKey { value: [0; 16] },
                topic_name: data_writer.topic_name.clone(),
                type_name: data_writer.type_name.clone(),
                durability: data_writer.qos.durability.clone(),
                deadline: data_writer.qos.deadline.clone(),
                latency_budget: data_writer.qos.latency_budget.clone(),
                liveliness: data_writer.qos.liveliness.clone(),
                reliability: data_writer.qos.reliability.clone(),
                lifespan: data_writer.qos.lifespan.clone(),
                user_data: data_writer.qos.user_data.clone(),
                ownership: data_writer.qos.ownership.clone(),
                ownership_strength: data_writer.qos.ownership_strength.clone(),
                destination_order: data_writer.qos.destination_order.clone(),
                presentation: publisher.qos.presentation.clone(),
                partition: publisher.qos.partition.clone(),
                topic_data: self.topic_list[&data_writer.topic_name]
                    .qos
                    .topic_data
                    .clone(),
                group_data: publisher.qos.group_data.clone(),
                representation: data_writer.qos.representation.clone(),
            };

            self.announce_created_or_modified_datawriter(publication_builtin_topic_data)?;
        }

        Ok((datawriter_handle.into(), writer_status_condition_address))
    }
}

pub struct DeleteUserDefinedDataWriter {
    pub publisher_handle: InstanceHandle,
    pub datawriter_handle: InstanceHandle,
}
impl Mail for DeleteUserDefinedDataWriter {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteUserDefinedDataWriter> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteUserDefinedDataWriter,
    ) -> <DeleteUserDefinedDataWriter as Mail>::Result {
        let publisher = self
            .user_defined_publisher_list
            .iter_mut()
            .find(|p| p.instance_handle == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_writer_index = publisher
            .data_writer_list
            .iter()
            .position(|x| x.instance_handle == message.datawriter_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        let data_writer = publisher.data_writer_list.remove(data_writer_index);
        let publication_builtin_topic_data = PublicationBuiltinTopicData {
            key: BuiltInTopicKey {
                value: data_writer.transport_writer.guid(),
            },
            participant_key: BuiltInTopicKey { value: [0; 16] },
            topic_name: data_writer.topic_name.clone(),
            type_name: data_writer.type_name.clone(),
            durability: data_writer.qos.durability.clone(),
            deadline: data_writer.qos.deadline.clone(),
            latency_budget: data_writer.qos.latency_budget.clone(),
            liveliness: data_writer.qos.liveliness.clone(),
            reliability: data_writer.qos.reliability.clone(),
            lifespan: data_writer.qos.lifespan.clone(),
            user_data: data_writer.qos.user_data.clone(),
            ownership: data_writer.qos.ownership.clone(),
            ownership_strength: data_writer.qos.ownership_strength.clone(),
            destination_order: data_writer.qos.destination_order.clone(),
            presentation: publisher.qos.presentation.clone(),
            partition: publisher.qos.partition.clone(),
            topic_data: self.topic_list[&data_writer.topic_name]
                .qos
                .topic_data
                .clone(),
            group_data: publisher.qos.group_data.clone(),
            representation: data_writer.qos.representation.clone(),
        };
        self.announce_deleted_data_writer(publication_builtin_topic_data)?;
        Ok(())
    }
}

pub struct LookupDataWriter {
    pub publisher_handle: InstanceHandle,
    pub topic_name: String,
}
impl Mail for LookupDataWriter {
    type Result = DdsResult<Option<InstanceHandle>>;
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
    pub publisher_handle: InstanceHandle,
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
    pub publisher_handle: InstanceHandle,
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
        let publisher = self
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let qos = match message.qos {
            QosKind::Default => DataWriterQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };
        publisher.default_datawriter_qos = qos;
        Ok(())
    }
}

pub struct GetDefaultDataWriterQos {
    pub publisher_handle: InstanceHandle,
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
    pub publisher_handle: InstanceHandle,
    pub qos: QosKind<PublisherQos>,
}
impl Mail for SetPublisherQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetPublisherQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetPublisherQos) -> <SetPublisherQos as Mail>::Result {
        let publisher = self
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        let qos = match message.qos {
            QosKind::Default => self.default_publisher_qos.clone(),
            QosKind::Specific(q) => q,
        };

        if publisher.enabled {
            if publisher.qos.presentation != qos.presentation {
                return Err(DdsError::ImmutablePolicy);
            }
        }

        publisher.qos = qos;

        Ok(())
    }
}

pub struct GetPublisherQos {
    pub publisher_handle: InstanceHandle,
}
impl Mail for GetPublisherQos {
    type Result = DdsResult<PublisherQos>;
}
impl MailHandler<GetPublisherQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetPublisherQos) -> <GetPublisherQos as Mail>::Result {
        Ok(self
            .user_defined_publisher_list
            .iter()
            .find(|p| p.instance_handle == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .qos
            .clone())
    }
}

pub struct SetPublisherListener {
    pub publisher_handle: InstanceHandle,
    pub a_listener: Option<Box<dyn PublisherListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
}
impl Mail for SetPublisherListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetPublisherListener> for DomainParticipantActor {
    fn handle(&mut self, message: SetPublisherListener) -> <SetPublisherQos as Mail>::Result {
        let publisher = self
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        publisher.publisher_listener_thread = message.a_listener.map(PublisherListenerThread::new);
        publisher.status_kind = message.mask;
        Ok(())
    }
}

pub struct EnablePublisher {
    pub publisher_handle: InstanceHandle,
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
    pub publisher_handle: InstanceHandle,
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
    pub subscriber_handle: InstanceHandle,
    pub topic_name: String,
    pub qos: QosKind<DataReaderQos>,
    pub a_listener: Option<Box<dyn AnyDataReaderListener + Send>>,
    pub mask: Vec<StatusKind>,
    pub domain_participant_address: ActorAddress<DomainParticipantActor>,
}
impl Mail for CreateUserDefinedDataReader {
    type Result = DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>;
}
impl MailHandler<CreateUserDefinedDataReader> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: CreateUserDefinedDataReader,
    ) -> <CreateUserDefinedDataReader as Mail>::Result {
        struct UserDefinedReaderHistoryCache {
            pub domain_participant_address: ActorAddress<DomainParticipantActor>,
            pub subscriber_handle: InstanceHandle,
            pub data_reader_handle: InstanceHandle,
        }

        impl ReaderHistoryCache for UserDefinedReaderHistoryCache {
            fn add_change(&mut self, cache_change: ReaderCacheChange) {
                self.domain_participant_address
                    .send_actor_mail(AddCacheChange {
                        domain_participant_address: self.domain_participant_address.clone(),
                        cache_change,
                        subscriber_handle: self.subscriber_handle,
                        data_reader_handle: self.data_reader_handle,
                    })
                    .ok();
            }
        }

        let subscriber = self
            .user_defined_subscriber_list
            .iter_mut()
            .find(|s| s.get_instance_handle() == message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let topic = self
            .topic_list
            .get(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?;

        let topic_kind = {
            let mut topic_kind = TopicKind::NoKey;
            for index in 0..topic.type_support.get_member_count() {
                if topic
                    .type_support
                    .get_member_by_index(index)?
                    .get_descriptor()?
                    .is_key
                {
                    topic_kind = TopicKind::WithKey;
                    break;
                }
            }
            topic_kind
        };
        let reader_handle = self.instance_handle_counter.generate_new_instance_handle();
        let transport_reader = self.transport.create_user_defined_reader(
            &message.topic_name,
            topic_kind,
            Box::new(UserDefinedReaderHistoryCache {
                domain_participant_address: message.domain_participant_address,
                subscriber_handle: subscriber.get_instance_handle().into(),
                data_reader_handle: reader_handle,
            }),
        );
        let (datareader_guid, reader_status_condition_address) = subscriber.create_datareader(
            topic,
            message.qos,
            message.a_listener,
            message.mask,
            reader_handle,
            transport_reader,
            &self.executor.handle(),
        )?;

        if subscriber.is_enabled()
            && subscriber
                .get_qos()
                .entity_factory
                .autoenable_created_entities
        {
            subscriber
                .get_mut_datareader(datareader_guid)
                .ok_or(DdsError::AlreadyDeleted)?
                .enable();

            if let Some(dcps_publication_reader) = self
                .builtin_subscriber
                .lookup_datareader_by_topic_name(DCPS_PUBLICATION)
            {
                if let Ok(sample_list) = dcps_publication_reader.read(
                    i32::MAX,
                    ANY_SAMPLE_STATE,
                    ANY_VIEW_STATE,
                    &[InstanceStateKind::Alive],
                    None,
                ) {
                    for (sample_data, _) in sample_list {
                        if let Ok(discovered_writer_data) = DiscoveredWriterData::deserialize_data(
                            sample_data
                                .expect("Alive samples should always contain data")
                                .as_ref(),
                        ) {
                            subscriber.add_matched_writer(&discovered_writer_data);
                        }
                    }
                }
            }

            let subscription_builtin_topic_data = subscriber
                .get_datareader(datareader_guid)
                .ok_or(DdsError::AlreadyDeleted)?
                .as_subscription_builtin_topic_data(subscriber.get_qos(), &topic.qos);

            self.announce_created_or_modified_datareader(subscription_builtin_topic_data)?;
        }

        Ok((datareader_guid, reader_status_condition_address))
    }
}

pub struct DeleteUserDefinedDataReader {
    pub subscriber_handle: InstanceHandle,
    pub datareader_handle: InstanceHandle,
}
impl Mail for DeleteUserDefinedDataReader {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteUserDefinedDataReader> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteUserDefinedDataReader,
    ) -> <DeleteUserDefinedDataReader as Mail>::Result {
        let subscriber = self
            .user_defined_subscriber_list
            .iter_mut()
            .find(|s| s.get_instance_handle() == message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let dr = subscriber
            .delete_datareader(&message.datareader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let topic = &self.topic_list[dr.get_topic_name()];
        let subscription_builtin_topic_data =
            dr.as_subscription_builtin_topic_data(subscriber.get_qos(), &topic.qos);
        self.announce_deleted_data_reader(subscription_builtin_topic_data)?;
        Ok(())
    }
}

pub struct LookupDataReader {
    pub subscriber_handle: InstanceHandle,
    pub topic_name: String,
}
impl Mail for LookupDataReader {
    type Result = DdsResult<Option<(InstanceHandle, ActorAddress<StatusConditionActor>)>>;
}
impl MailHandler<LookupDataReader> for DomainParticipantActor {
    fn handle(&mut self, message: LookupDataReader) -> <LookupDataReader as Mail>::Result {
        if !self.topic_list.contains_key(&message.topic_name) {
            return Err(DdsError::BadParameter);
        }

        // Built-in subscriber is identified by the handle of the participant itself
        if self.get_instance_handle() == message.subscriber_handle {
            Ok(self
                .builtin_subscriber
                .lookup_datareader_by_topic_name(&message.topic_name)
                .map(|x: &mut DataReaderActor| (x.get_instance_handle(), x.get_statuscondition())))
        } else {
            let s = self
                .user_defined_subscriber_list
                .iter_mut()
                .find(|x| x.get_instance_handle() == message.subscriber_handle)
                .ok_or(DdsError::AlreadyDeleted)?;
            Ok(s.lookup_datareader_by_topic_name(&message.topic_name)
                .map(|x| (x.get_instance_handle(), x.get_statuscondition())))
        }
    }
}

pub struct DeleteSubscriberContainedEntities {
    pub subscriber_handle: InstanceHandle,
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
    pub subscriber_handle: InstanceHandle,
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
    pub subscriber_handle: InstanceHandle,
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
    pub subscriber_handle: InstanceHandle,
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
    pub subscriber_handle: InstanceHandle,
}
impl Mail for GetSubscriberQos {
    type Result = DdsResult<SubscriberQos>;
}
impl MailHandler<GetSubscriberQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetSubscriberQos) -> <GetSubscriberQos as Mail>::Result {
        Ok(self
            .user_defined_subscriber_list
            .iter()
            .find(|s| s.get_instance_handle() == message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_qos()
            .clone())
    }
}

pub struct SetSubscriberListener {
    pub subscriber_handle: InstanceHandle,
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
    pub subscriber_handle: InstanceHandle,
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
    pub subscriber_handle: InstanceHandle,
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

// ############################  Data writer messages
pub struct RegisterInstance {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub serialized_data: Vec<u8>,
    pub timestamp: Time,
}
impl Mail for RegisterInstance {
    type Result = DdsResult<Option<InstanceHandle>>;
}
impl MailHandler<RegisterInstance> for DomainParticipantActor {
    fn handle(&mut self, message: RegisterInstance) -> <RegisterInstance as Mail>::Result {
        todo!()
        // if !self
        //     .writer_address
        //     .send_actor_mail(data_writer_actor::IsEnabled)?
        //     .receive_reply()
        //     .await
        // {
        //     return Err(DdsError::NotEnabled);
        // }

        // let type_support = self
        //     .participant_address()
        //     .send_actor_mail(domain_participant_actor::GetTopicTypeSupport {
        //         topic_name: self.topic.get_name(),
        //     })?
        //     .receive_reply()
        //     .await?;

        // let serialized_data = instance.serialize_data()?;
        // let instance_handle =
        //     get_instance_handle_from_serialized_foo(&serialized_data, type_support.as_ref())?;

        // self.writer_address
        //     .send_actor_mail(data_writer_actor::RegisterInstanceWTimestamp { instance_handle })?
        //     .receive_reply()
        //     .await
    }
}

pub struct UnregisterInstance {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub serialized_data: Vec<u8>,
    pub timestamp: Time,
}
impl Mail for UnregisterInstance {
    type Result = DdsResult<()>;
}
impl MailHandler<UnregisterInstance> for DomainParticipantActor {
    fn handle(&mut self, message: UnregisterInstance) -> <UnregisterInstance as Mail>::Result {
        let publisher = self
            .user_defined_publisher_list
            .iter_mut()
            .find(|p| p.instance_handle == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_writer = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        if !data_writer.enabled {
            return Err(DdsError::NotEnabled);
        }

        let type_support = self.topic_list[&data_writer.topic_name]
            .type_support
            .clone();

        let has_key = {
            let mut has_key = false;
            for index in 0..type_support.get_member_count() {
                if type_support
                    .get_member_by_index(index)?
                    .get_descriptor()?
                    .is_key
                {
                    has_key = true;
                    break;
                }
            }
            has_key
        };
        if !has_key {
            return Err(DdsError::IllegalOperation);
        }

        let serialized_key = get_serialized_key_from_serialized_foo(
            &message.serialized_data,
            type_support.as_ref(),
        )?;

        let instance_handle =
            get_instance_handle_from_serialized_key(&serialized_key, type_support.as_ref())?;
        if !data_writer
            .registered_instance_list
            .contains(&instance_handle)
        {
            return Err(DdsError::BadParameter);
        }

        if let Some(t) = data_writer
            .instance_deadline_missed_task
            .remove(&instance_handle)
        {
            t.abort();
        }

        data_writer.last_change_sequence_number += 1;

        let mut serialized_status_info = Vec::new();
        let mut serializer = Xcdr1LeSerializer::new(&mut serialized_status_info);
        match data_writer
            .qos
            .writer_data_lifecycle
            .autodispose_unregistered_instances
        {
            true => {
                XTypesSerialize::serialize(&STATUS_INFO_DISPOSED_UNREGISTERED, &mut serializer)?
            }
            false => XTypesSerialize::serialize(&STATUS_INFO_UNREGISTERED, &mut serializer)?,
        }
        let pid_status_info = Parameter::new(PID_STATUS_INFO, Arc::from(serialized_status_info));

        let pid_key_hash = Parameter::new(PID_KEY_HASH, Arc::from(*instance_handle.as_ref()));
        let parameter_list = ParameterList::new(vec![pid_status_info, pid_key_hash]);

        let cache_change = RtpsCacheChange {
            kind: ChangeKind::NotAliveDisposed,
            sequence_number: data_writer.last_change_sequence_number,
            source_timestamp: Some(message.timestamp.into()),
            data_value: serialized_key.into(),
            inline_qos: parameter_list,
        };
        data_writer.transport_writer.add_change(cache_change);

        Ok(())
    }
}

pub struct LookupInstance {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub serialized_data: Vec<u8>,
}
impl Mail for LookupInstance {
    type Result = DdsResult<Option<InstanceHandle>>;
}
impl MailHandler<LookupInstance> for DomainParticipantActor {
    fn handle(&mut self, message: LookupInstance) -> <LookupInstance as Mail>::Result {
        let data_writer = self
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        let instance_handle = get_instance_handle_from_serialized_foo(
            &message.serialized_data,
            self.topic_list[&data_writer.topic_name]
                .type_support
                .as_ref(),
        )?;
        if !data_writer.enabled {
            return Err(DdsError::NotEnabled);
        }

        Ok(
            if data_writer
                .registered_instance_list
                .contains(&instance_handle)
            {
                Some(instance_handle)
            } else {
                None
            },
        )
    }
}

pub struct WriteWTimestamp {
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub serialized_data: Vec<u8>,
    pub timestamp: Time,
}
impl Mail for WriteWTimestamp {
    type Result = DdsResult<()>;
}
impl MailHandler<WriteWTimestamp> for DomainParticipantActor {
    fn handle(&mut self, message: WriteWTimestamp) -> <WriteWTimestamp as Mail>::Result {
        let now = self.get_current_time();
        let publisher = self
            .user_defined_publisher_list
            .iter_mut()
            .find(|p| p.instance_handle == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_writer = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let type_support = self.topic_list[&data_writer.topic_name]
            .type_support
            .clone();

        match data_writer.qos.lifespan.duration {
            DurationKind::Finite(lifespan) => {
                let change_lifespan =
                    crate::infrastructure::time::Time::from(message.timestamp) - now + lifespan;
                if change_lifespan > Duration::new(0, 0) {
                    todo!();
                    let timer_handle = self.timer_driver.handle();
                    self.executor.handle().spawn(async move {
                        timer_handle.sleep(change_lifespan.into()).await;
                        message
                            .participant_address
                            .send_actor_mail(RemoveWriterChange {
                                publisher_handle: message.publisher_handle,
                                data_writer_handle: message.data_writer_handle,
                                sequence_number: todo!(),
                            })
                            .ok();
                    });
                }
            }
            DurationKind::Infinite => {}
        }

        if !data_writer.enabled {
            return Err(DdsError::NotEnabled);
        }

        data_writer.last_change_sequence_number += 1;

        let instance_handle = get_instance_handle_from_serialized_foo(
            &message.serialized_data,
            type_support.as_ref(),
        )?;

        if !data_writer
            .registered_instance_list
            .contains(&instance_handle)
        {
            if data_writer.registered_instance_list.len()
                < data_writer.qos.resource_limits.max_instances
            {
                data_writer.registered_instance_list.insert(instance_handle);
            } else {
                return Err(DdsError::OutOfResources);
            }
        }

        if let Length::Limited(max_instances) = data_writer.qos.resource_limits.max_instances {
            if !data_writer.instance_samples.contains_key(&instance_handle)
                && data_writer.instance_samples.len() == max_instances as usize
            {
                return Err(DdsError::OutOfResources);
            }
        }

        if let Length::Limited(max_samples_per_instance) =
            data_writer.qos.resource_limits.max_samples_per_instance
        {
            // If the history Qos guarantess that the number of samples
            // is below the limit there is no need to check
            match data_writer.qos.history.kind {
                HistoryQosPolicyKind::KeepLast(depth) if depth <= max_samples_per_instance => {}
                _ => {
                    if let Some(s) = data_writer.instance_samples.get(&instance_handle) {
                        // Only Alive changes count towards the resource limits
                        if s.len() >= max_samples_per_instance as usize {
                            return Err(DdsError::OutOfResources);
                        }
                    }
                }
            }
        }

        if let Length::Limited(max_samples) = data_writer.qos.resource_limits.max_samples {
            let total_samples = data_writer
                .instance_samples
                .iter()
                .fold(0, |acc, (_, x)| acc + x.len());

            if total_samples >= max_samples as usize {
                return Err(DdsError::OutOfResources);
            }
        }

        let pid_key_hash = Parameter::new(PID_KEY_HASH, Arc::from(*instance_handle.as_ref()));
        let parameter_list = ParameterList::new(vec![pid_key_hash]);

        let change = RtpsCacheChange {
            kind: ChangeKind::Alive,
            sequence_number: data_writer.last_change_sequence_number,
            source_timestamp: Some(message.timestamp.into()),
            data_value: message.serialized_data.into(),
            inline_qos: parameter_list,
        };
        if let HistoryQosPolicyKind::KeepLast(depth) = data_writer.qos.history.kind {
            if let Some(s) = data_writer.instance_samples.get_mut(&instance_handle) {
                if s.len() == depth as usize {
                    if let Some(smallest_seq_num_instance) = s.pop_front() {
                        data_writer
                            .transport_writer
                            .remove_change(smallest_seq_num_instance);
                    }
                }
            }
        }

        let change_timestamp = change.source_timestamp();
        let seq_num = change.sequence_number();

        if seq_num > data_writer.max_seq_num.unwrap_or(0) {
            data_writer.max_seq_num = Some(seq_num)
        }

        if let Some(t) = data_writer
            .instance_deadline_missed_task
            .remove(&instance_handle.into())
        {
            t.abort();
        }

        if let DurationKind::Finite(deadline_missed_period) = data_writer.qos.deadline.period {
            let deadline_missed_interval = std::time::Duration::new(
                deadline_missed_period.sec() as u64,
                deadline_missed_period.nanosec(),
            );
            // let writer_status_condition = self.status_condition.address();
            // let writer_address = message.writer_address.clone();
            // let timer_handle = message.timer_handle.clone();
            // let writer_listener_mask = self.status_kind.clone();
            // let data_writer_listener_sender = self
            //     .data_writer_listener_thread
            //     .as_ref()
            //     .map(|l| l.sender().clone());
            // let publisher_listener = message.publisher_mask_listener.0.clone();
            // let publisher_listener_mask = message.publisher_mask_listener.1.clone();
            // let participant_listener = message.participant_mask_listener.0.clone();
            // let participant_listener_mask = message.participant_mask_listener.1.clone();
            // let status_condition_address = self.status_condition.address();
            // // let topic_address = self.topic_address.clone();
            // // let topic_status_condition_address = self.topic_status_condition.clone();
            // let type_name = self.type_name.clone();
            // let topic_name = self.topic_name.clone();
            // let publisher = message.publisher.clone();

            // let deadline_missed_task = message.executor_handle.spawn(async move {
            //     loop {
            //         timer_handle.sleep(deadline_missed_interval).await;
            //         let publisher_listener = publisher_listener.clone();
            //         let participant_listener = participant_listener.clone();

            //         let r: DdsResult<()> = async {
            //             writer_address.send_actor_mail(
            //                 IncrementOfferedDeadlineMissedStatus {
            //                     instance_handle: change_instance_handle.into(),
            //                 },
            //             )?;

            //             let writer_address = writer_address.clone();
            //             let status_condition_address = status_condition_address.clone();
            //             let publisher = publisher.clone();
            //             let topic = TopicAsync::new(
            //                 topic_address.clone(),
            //                 topic_status_condition_address.clone(),
            //                 type_name.clone(),
            //                 topic_name.clone(),
            //                 publisher.get_participant(),
            //             );
            //             if writer_listener_mask.contains(&StatusKind::OfferedDeadlineMissed) {
            //                 let status = writer_address
            //                     .send_actor_mail(GetOfferedDeadlineMissedStatus)?
            //                     .receive_reply()
            //                     .await;
            //                 if let Some(listener) = &data_writer_listener_sender {
            //                     listener
            //                         .send(DataWriterListenerMessage {
            //                             listener_operation:
            //                                 DataWriterListenerOperation::OfferedDeadlineMissed(
            //                                     status,
            //                                 ),
            //                             writer_address,
            //                             status_condition_address,
            //                             publisher,
            //                             topic,
            //                         })
            //                         .ok();
            //                 }
            //             } else if publisher_listener_mask
            //                 .contains(&StatusKind::OfferedDeadlineMissed)
            //             {
            //                 let status = writer_address
            //                     .send_actor_mail(GetOfferedDeadlineMissedStatus)?
            //                     .receive_reply()
            //                     .await;
            //                 if let Some(listener) = publisher_listener {
            //                     listener
            //                         .send(PublisherListenerMessage {
            //                             listener_operation:
            //                                 PublisherListenerOperation::OfferedDeadlineMissed(
            //                                     status,
            //                                 ),
            //                             writer_address,
            //                             status_condition_address,
            //                             publisher,
            //                             topic,
            //                         })
            //                         .ok();
            //                 }
            //             } else if participant_listener_mask
            //                 .contains(&StatusKind::OfferedDeadlineMissed)
            //             {
            //                 let status = writer_address
            //                     .send_actor_mail(GetOfferedDeadlineMissedStatus)?
            //                     .receive_reply()
            //                     .await;
            //                 if let Some(listener) = participant_listener {
            //                     listener
            //                     .send(ParticipantListenerMessage {
            //                         listener_operation:
            //                             ParticipantListenerOperation::_OfferedDeadlineMissed(
            //                                 status,
            //                             ),
            //                         listener_kind: ListenerKind::Writer {
            //                             writer_address,
            //                             status_condition_address,
            //                             publisher,
            //                             topic,
            //                         },
            //                     })
            //                     .ok();
            //                 }
            //             }
            //             writer_status_condition
            //                 .send_actor_mail(AddCommunicationState {
            //                     state: StatusKind::OfferedDeadlineMissed,
            //                 })?
            //                 .receive_reply()
            //                 .await;
            //             Ok(())
            //         }
            //         .await;
            //         if r.is_err() {
            //             break;
            //         }
            //     }
            // });
            // self.instance_deadline_missed_task
            //     .insert(change_instance_handle.into(), deadline_missed_task);
        }

        data_writer
            .instance_samples
            .entry(instance_handle)
            .or_insert(VecDeque::new())
            .push_back(change.sequence_number);
        data_writer.transport_writer.add_change(change);

        //     if let Some(timestamp) = change_timestamp {

        //             //             rtps_writer.get_history_cache().add_change(message.change);
        //             //             message.executor_handle.spawn(async move {
        //             //                 message.timer_handle.sleep(change_lifespan.into()).await;

        //             //                 message
        //             //                     .writer_address
        //             //                     .send_actor_mail(RemoveChange { seq_num })
        //             //                     .ok();
        // });
        // }
        // }

        Ok(())

        //     }
        // } else {
        // if writer_qos.reliability.kind == ReliabilityQosPolicyKind::Reliable {
        //     let start = std::time::Instant::now();
        //     let timer_handle = self.publisher.get_participant().timer_handle().clone();
        //     loop {
        //         if !self
        //             .writer_address
        //             .send_actor_mail(data_writer_actor::IsDataLostAfterAddingChange {
        //                 instance_handle: change.instance_handle().into(),
        //             })?
        //             .receive_reply()
        //             .await
        //         {
        //             break;
        //         }
        //         timer_handle
        //             .sleep(std::time::Duration::from_millis(20))
        //             .await;
        //         if let DurationKind::Finite(timeout) = writer_qos.reliability.max_blocking_time {
        //             if std::time::Instant::now().duration_since(start) > timeout.into() {
        //                 return Err(DdsError::Timeout);
        //             }
        //         }
        //     }
        // }
    }
}

pub struct DisposeWTimestamp {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub serialized_data: Vec<u8>,
    pub timestamp: Time,
}
impl Mail for DisposeWTimestamp {
    type Result = DdsResult<()>;
}
impl MailHandler<DisposeWTimestamp> for DomainParticipantActor {
    fn handle(&mut self, message: DisposeWTimestamp) -> <DisposeWTimestamp as Mail>::Result {
        let publisher = self
            .user_defined_publisher_list
            .iter_mut()
            .find(|p| p.instance_handle == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_writer = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        if !data_writer.enabled {
            return Err(DdsError::NotEnabled);
        }
        let type_support = self.topic_list[&data_writer.topic_name]
            .type_support
            .clone();

        let has_key = {
            let mut has_key = false;
            for index in 0..type_support.get_member_count() {
                if type_support
                    .get_member_by_index(index)?
                    .get_descriptor()?
                    .is_key
                {
                    has_key = true;
                    break;
                }
            }
            has_key
        };
        if !has_key {
            return Err(DdsError::IllegalOperation);
        }

        let serialized_key = get_serialized_key_from_serialized_foo(
            &message.serialized_data,
            type_support.as_ref(),
        )?;

        let instance_handle =
            get_instance_handle_from_serialized_key(&serialized_key, type_support.as_ref())?;
        if !data_writer
            .registered_instance_list
            .contains(&instance_handle)
        {
            return Err(DdsError::BadParameter);
        }

        if let Some(t) = data_writer
            .instance_deadline_missed_task
            .remove(&instance_handle)
        {
            t.abort();
        }

        data_writer.last_change_sequence_number += 1;

        let mut serialized_status_info = Vec::new();
        let mut serializer = Xcdr1LeSerializer::new(&mut serialized_status_info);
        XTypesSerialize::serialize(&STATUS_INFO_DISPOSED, &mut serializer)?;
        let pid_status_info = Parameter::new(PID_STATUS_INFO, Arc::from(serialized_status_info));

        let pid_key_hash = Parameter::new(PID_KEY_HASH, Arc::from(*instance_handle.as_ref()));
        let parameter_list = ParameterList::new(vec![pid_status_info, pid_key_hash]);

        let cache_change = RtpsCacheChange {
            kind: ChangeKind::NotAliveDisposed,
            sequence_number: data_writer.last_change_sequence_number,
            source_timestamp: Some(message.timestamp.into()),
            data_value: serialized_key.into(),
            inline_qos: parameter_list,
        };
        data_writer.transport_writer.add_change(cache_change);

        Ok(())
    }
}

pub struct WaitForAcknowledgments {
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub timeout: Duration,
}
impl Mail for WaitForAcknowledgments {
    type Result = Pin<Box<dyn Future<Output = DdsResult<()>> + Send>>;
}
impl MailHandler<WaitForAcknowledgments> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: WaitForAcknowledgments,
    ) -> <WaitForAcknowledgments as Mail>::Result {
        let timer_handle = self.timer_driver.handle();
        Box::pin(async move {
            timer_handle
                .timeout(
                    message.timeout.into(),
                    Box::pin(async move {
                        loop {
                            let all_changes_ack = message
                                .participant_address
                                .send_actor_mail(AreAllChangesAcknowledged {
                                    publisher_handle: message.publisher_handle,
                                    data_writer_handle: message.data_writer_handle,
                                })?
                                .receive_reply()
                                .await?;
                            if all_changes_ack {
                                return Ok(());
                            }
                        }
                    }),
                )
                .await
                .map_err(|_| DdsError::Timeout)?
        })
    }
}

pub struct GetOfferedDeadlineMissedStatus {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}
impl Mail for GetOfferedDeadlineMissedStatus {
    type Result = DdsResult<OfferedDeadlineMissedStatus>;
}
impl MailHandler<GetOfferedDeadlineMissedStatus> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetOfferedDeadlineMissedStatus,
    ) -> <GetOfferedDeadlineMissedStatus as Mail>::Result {
        let publisher = self
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_writer = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        let status = data_writer.offered_deadline_missed_status.clone();
        data_writer
            .offered_deadline_missed_status
            .total_count_change = 0;
        Ok(status)
    }
}

pub struct GetPublicationMatchedStatus {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}
impl Mail for GetPublicationMatchedStatus {
    type Result = DdsResult<PublicationMatchedStatus>;
}
impl MailHandler<GetPublicationMatchedStatus> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetPublicationMatchedStatus,
    ) -> <GetPublicationMatchedStatus as Mail>::Result {
        let publisher = self
            .user_defined_publisher_list
            .iter_mut()
            .find(|p| p.instance_handle == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_writer = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        data_writer.status_condition.send_actor_mail(
            status_condition_actor::RemoveCommunicationState {
                state: StatusKind::PublicationMatched,
            },
        );

        let status = data_writer.publication_matched_status.clone();
        data_writer.publication_matched_status.current_count_change = 0;
        data_writer.publication_matched_status.total_count_change = 0;
        Ok(status)
    }
}

pub struct GetMatchedSubscriptionData {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub subscription_handle: InstanceHandle,
}
impl Mail for GetMatchedSubscriptionData {
    type Result = DdsResult<SubscriptionBuiltinTopicData>;
}
impl MailHandler<GetMatchedSubscriptionData> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetMatchedSubscriptionData,
    ) -> <GetMatchedSubscriptionData as Mail>::Result {
        self.user_defined_publisher_list
            .iter()
            .find(|p| p.instance_handle == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .data_writer_list
            .iter()
            .find(|x| x.instance_handle == message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .matched_subscription_list
            .get(&message.subscription_handle)
            .ok_or(DdsError::BadParameter)
            .cloned()
    }
}

pub struct GetMatchedSubscriptions {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}
impl Mail for GetMatchedSubscriptions {
    type Result = DdsResult<Vec<InstanceHandle>>;
}
impl MailHandler<GetMatchedSubscriptions> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetMatchedSubscriptions,
    ) -> <GetMatchedSubscriptions as Mail>::Result {
        Ok(self
            .user_defined_publisher_list
            .iter()
            .find(|p| p.instance_handle == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .data_writer_list
            .iter()
            .find(|x| x.instance_handle == message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .matched_subscription_list
            .keys()
            .cloned()
            .collect())
    }
}

pub struct SetDataWriterQos {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub qos: QosKind<DataWriterQos>,
}
impl Mail for SetDataWriterQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDataWriterQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetDataWriterQos) -> <SetDataWriterQos as Mail>::Result {
        let publisher = self
            .user_defined_publisher_list
            .iter_mut()
            .find(|p| p.instance_handle == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_writer = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let qos = match message.qos {
            QosKind::Default => publisher.default_datawriter_qos.clone(),
            QosKind::Specific(q) => q,
        };
        qos.is_consistent()?;

        if data_writer.enabled {
            qos.check_immutability(&data_writer.qos)?;
            data_writer.qos = qos;

            let publication_builtin_topic_data = PublicationBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: data_writer.transport_writer.guid(),
                },
                participant_key: BuiltInTopicKey { value: [0; 16] },
                topic_name: data_writer.topic_name.clone(),
                type_name: data_writer.type_name.clone(),
                durability: data_writer.qos.durability.clone(),
                deadline: data_writer.qos.deadline.clone(),
                latency_budget: data_writer.qos.latency_budget.clone(),
                liveliness: data_writer.qos.liveliness.clone(),
                reliability: data_writer.qos.reliability.clone(),
                lifespan: data_writer.qos.lifespan.clone(),
                user_data: data_writer.qos.user_data.clone(),
                ownership: data_writer.qos.ownership.clone(),
                ownership_strength: data_writer.qos.ownership_strength.clone(),
                destination_order: data_writer.qos.destination_order.clone(),
                presentation: publisher.qos.presentation.clone(),
                partition: publisher.qos.partition.clone(),
                topic_data: self.topic_list[&data_writer.topic_name]
                    .qos
                    .topic_data
                    .clone(),
                group_data: publisher.qos.group_data.clone(),
                representation: data_writer.qos.representation.clone(),
            };

            self.announce_created_or_modified_datawriter(publication_builtin_topic_data)?;
        } else {
            data_writer.qos = qos;
        }

        Ok(())
    }
}

pub struct GetDataWriterQos {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}
impl Mail for GetDataWriterQos {
    type Result = DdsResult<DataWriterQos>;
}
impl MailHandler<GetDataWriterQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetDataWriterQos) -> <GetDataWriterQos as Mail>::Result {
        todo!()
        // let qos = match qos {
        //     QosKind::Default => {
        //         self.publisher_address()
        //             .send_actor_mail(publisher_actor::GetDefaultDatawriterQos)?
        //             .receive_reply()
        //             .await
        //     }
        //     QosKind::Specific(q) => q,
        // };

        // self.writer_address
        //     .send_actor_mail(data_writer_actor::SetQos { qos })?
        //     .receive_reply()
        //     .await?;
        // if self
        //     .writer_address
        //     .send_actor_mail(data_writer_actor::IsEnabled)?
        //     .receive_reply()
        //     .await
        // {
        //     self.announce_writer().await?;
        // }

        // Ok(())
    }
}

pub struct EnableDataWriter {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}
impl Mail for EnableDataWriter {
    type Result = DdsResult<()>;
}
impl MailHandler<EnableDataWriter> for DomainParticipantActor {
    fn handle(&mut self, message: EnableDataWriter) -> <EnableDataWriter as Mail>::Result {
        todo!()
        // let writer = self.writer_address();
        // if !writer
        //     .send_actor_mail(data_writer_actor::IsEnabled)?
        //     .receive_reply()
        //     .await
        // {
        //     let message_sender_actor = self
        //         .participant_address()
        //         .send_actor_mail(domain_participant_actor::GetMessageSender)?
        //         .receive_reply()
        //         .await;
        //     writer
        //         .send_actor_mail(data_writer_actor::Enable {
        //             data_writer_address: writer.clone(),
        //             message_sender_actor,
        //             executor_handle: self.publisher.get_participant().executor_handle().clone(),
        //             timer_handle: self.publisher.get_participant().timer_handle().clone(),
        //         })?
        //         .receive_reply()
        //         .await;

        //     self.announce_writer().await?;

        //     self.process_sedp_subscriptions_discovery().await?;
        // }
        // Ok(())
    }
}

pub struct GetDataWriterInstanceHandle {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}
impl Mail for GetDataWriterInstanceHandle {
    type Result = DdsResult<InstanceHandle>;
}
impl MailHandler<GetDataWriterInstanceHandle> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetDataWriterInstanceHandle,
    ) -> <GetDataWriterInstanceHandle as Mail>::Result {
        todo!()
        // let writer = self.writer_address();
        // if !writer
        //     .send_actor_mail(data_writer_actor::IsEnabled)?
        //     .receive_reply()
        //     .await
        // {
        //     let message_sender_actor = self
        //         .participant_address()
        //         .send_actor_mail(domain_participant_actor::GetMessageSender)?
        //         .receive_reply()
        //         .await;
        //     writer
        //         .send_actor_mail(data_writer_actor::Enable {
        //             data_writer_address: writer.clone(),
        //             message_sender_actor,
        //             executor_handle: self.publisher.get_participant().executor_handle().clone(),
        //             timer_handle: self.publisher.get_participant().timer_handle().clone(),
        //         })?
        //         .receive_reply()
        //         .await;

        //     self.announce_writer().await?;

        //     self.process_sedp_subscriptions_discovery().await?;
        // }
        // Ok(())
    }
}

pub struct SetDataWriterListener {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub a_listener: Option<Box<dyn AnyDataWriterListener + Send>>,
    pub status_kind: Vec<StatusKind>,
}
impl Mail for SetDataWriterListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDataWriterListener> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetDataWriterListener,
    ) -> <SetDataWriterListener as Mail>::Result {
        todo!()
        // let writer = self.writer_address();
        // if !writer
        //     .send_actor_mail(data_writer_actor::IsEnabled)?
        //     .receive_reply()
        //     .await
        // {
        //     let message_sender_actor = self
        //         .participant_address()
        //         .send_actor_mail(domain_participant_actor::GetMessageSender)?
        //         .receive_reply()
        //         .await;
        //     writer
        //         .send_actor_mail(data_writer_actor::Enable {
        //             data_writer_address: writer.clone(),
        //             message_sender_actor,
        //             executor_handle: self.publisher.get_participant().executor_handle().clone(),
        //             timer_handle: self.publisher.get_participant().timer_handle().clone(),
        //         })?
        //         .receive_reply()
        //         .await;

        //     self.announce_writer().await?;

        //     self.process_sedp_subscriptions_discovery().await?;
        // }
        // Ok(())
    }
}

// ############################  Data reader messages
pub struct Read {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub max_samples: i32,
    pub sample_states: Vec<SampleStateKind>,
    pub view_states: Vec<ViewStateKind>,
    pub instance_states: Vec<InstanceStateKind>,
    pub specific_instance_handle: Option<InstanceHandle>,
}
impl Mail for Read {
    type Result = DdsResult<Vec<(Option<Data>, SampleInfo)>>;
}
impl MailHandler<Read> for DomainParticipantActor {
    fn handle(&mut self, message: Read) -> <Read as Mail>::Result {
        let subscriber = if message.subscriber_handle == self.get_instance_handle() {
            Some(&mut self.builtin_subscriber)
        } else {
            self.user_defined_subscriber_list
                .iter_mut()
                .find(|x| x.get_instance_handle() == message.subscriber_handle)
        }
        .ok_or(DdsError::AlreadyDeleted)?;

        subscriber
            .get_mut_datareader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .read(
                message.max_samples,
                &message.sample_states,
                &message.view_states,
                &message.instance_states,
                message.specific_instance_handle,
            )
    }
}

pub struct Take {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub max_samples: i32,
    pub sample_states: Vec<SampleStateKind>,
    pub view_states: Vec<ViewStateKind>,
    pub instance_states: Vec<InstanceStateKind>,
    pub specific_instance_handle: Option<InstanceHandle>,
}
impl Mail for Take {
    type Result = DdsResult<Vec<(Option<Data>, SampleInfo)>>;
}
impl MailHandler<Take> for DomainParticipantActor {
    fn handle(&mut self, message: Take) -> <Take as Mail>::Result {
        let subscriber = self
            .user_defined_subscriber_list
            .iter_mut()
            .find(|s| s.get_instance_handle() == message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_reader = subscriber
            .get_mut_datareader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        data_reader.take(
            message.max_samples,
            message.sample_states,
            message.view_states,
            message.instance_states,
            message.specific_instance_handle,
        )
    }
}

pub struct ReadNextInstance {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub max_samples: i32,
    pub previous_handle: Option<InstanceHandle>,
    pub sample_states: Vec<SampleStateKind>,
    pub view_states: Vec<ViewStateKind>,
    pub instance_states: Vec<InstanceStateKind>,
}
impl Mail for ReadNextInstance {
    type Result = DdsResult<Vec<(Option<Data>, SampleInfo)>>;
}
impl MailHandler<ReadNextInstance> for DomainParticipantActor {
    fn handle(&mut self, message: ReadNextInstance) -> <ReadNextInstance as Mail>::Result {
        let subscriber = self
            .user_defined_subscriber_list
            .iter_mut()
            .find(|s| s.get_instance_handle() == message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_reader = subscriber
            .get_mut_datareader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        data_reader.read_next_instance(
            message.max_samples,
            message.previous_handle,
            &message.sample_states,
            &message.view_states,
            &message.instance_states,
        )
    }
}

pub struct TakeNextInstance {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub max_samples: i32,
    pub previous_handle: Option<InstanceHandle>,
    pub sample_states: Vec<SampleStateKind>,
    pub view_states: Vec<ViewStateKind>,
    pub instance_states: Vec<InstanceStateKind>,
}
impl Mail for TakeNextInstance {
    type Result = DdsResult<Vec<(Option<Data>, SampleInfo)>>;
}
impl MailHandler<TakeNextInstance> for DomainParticipantActor {
    fn handle(&mut self, message: TakeNextInstance) -> <TakeNextInstance as Mail>::Result {
        let subscriber = self
            .user_defined_subscriber_list
            .iter_mut()
            .find(|s| s.get_instance_handle() == message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_reader = subscriber
            .get_mut_datareader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        data_reader.take_next_instance(
            message.max_samples,
            message.previous_handle,
            message.sample_states,
            message.view_states,
            message.instance_states,
        )
    }
}

pub struct GetSubscriptionMatchedStatus {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}
impl Mail for GetSubscriptionMatchedStatus {
    type Result = DdsResult<SubscriptionMatchedStatus>;
}
impl MailHandler<GetSubscriptionMatchedStatus> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetSubscriptionMatchedStatus,
    ) -> <GetSubscriptionMatchedStatus as Mail>::Result {
        Ok(self
            .user_defined_subscriber_list
            .iter_mut()
            .find(|s| s.get_instance_handle() == message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_mut_datareader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_subscription_matched_status())
    }
}

pub struct WaitForHistoricalData {
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub max_wait: Duration,
}
impl Mail for WaitForHistoricalData {
    type Result = Pin<Box<dyn Future<Output = DdsResult<()>> + Send>>;
}
impl MailHandler<WaitForHistoricalData> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: WaitForHistoricalData,
    ) -> <WaitForHistoricalData as Mail>::Result {
        let timer_handle = self.timer_driver.handle();

        Box::pin(async move {
            timer_handle
                .timeout(
                    message.max_wait.into(),
                    Box::pin(async move {
                        loop {
                            if message
                                .participant_address
                                .send_actor_mail(IsHistoricalDataReceived {
                                    subscriber_handle: message.subscriber_handle,
                                    data_reader_handle: message.data_reader_handle,
                                })?
                                .receive_reply()
                                .await?
                            {
                                return Ok(());
                            }
                        }
                    }),
                )
                .await
                .map_err(|_| DdsError::Timeout)?
        })
    }
}

pub struct GetMatchedPublicationData {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub publication_handle: InstanceHandle,
}
impl Mail for GetMatchedPublicationData {
    type Result = DdsResult<PublicationBuiltinTopicData>;
}
impl MailHandler<GetMatchedPublicationData> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetMatchedPublicationData,
    ) -> <GetMatchedPublicationData as Mail>::Result {
        self.user_defined_subscriber_list
            .iter()
            .find(|s| s.get_instance_handle() == message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_datareader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_matched_publication_data(message.publication_handle)
    }
}

pub struct GetMatchedPublications {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}
impl Mail for GetMatchedPublications {
    type Result = DdsResult<Vec<InstanceHandle>>;
}
impl MailHandler<GetMatchedPublications> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetMatchedPublications,
    ) -> <GetMatchedPublications as Mail>::Result {
        Ok(self
            .user_defined_subscriber_list
            .iter()
            .find(|s| s.get_instance_handle() == message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_datareader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_matched_publications())
    }
}

pub struct SetDataReaderQos {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub qos: QosKind<DataReaderQos>,
}
impl Mail for SetDataReaderQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDataReaderQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetDataReaderQos) -> <SetDataReaderQos as Mail>::Result {
        let subscriber = self
            .user_defined_subscriber_list
            .iter_mut()
            .find(|s| s.get_instance_handle() == message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let qos = match message.qos {
            QosKind::Default => subscriber.get_default_datareader_qos().clone(),
            QosKind::Specific(q) => q,
        };

        let subscriber_qos = subscriber.get_qos().clone();
        let data_reader = subscriber
            .get_mut_datareader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        data_reader.set_qos(qos)?;
        if data_reader.is_enabled() {
            let subscription_builtin_topic_data = data_reader.as_subscription_builtin_topic_data(
                &subscriber_qos,
                &self.topic_list[data_reader.get_topic_name()].qos,
            );

            self.announce_created_or_modified_datareader(subscription_builtin_topic_data)?;
        }

        Ok(())
    }
}

pub struct GetDataReaderQos {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}
impl Mail for GetDataReaderQos {
    type Result = DdsResult<DataReaderQos>;
}
impl MailHandler<GetDataReaderQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetDataReaderQos) -> <GetDataReaderQos as Mail>::Result {
        todo!()
    }
}

pub struct EnableDataReader {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}
impl Mail for EnableDataReader {
    type Result = DdsResult<()>;
}
impl MailHandler<EnableDataReader> for DomainParticipantActor {
    fn handle(&mut self, message: EnableDataReader) -> <EnableDataReader as Mail>::Result {
        todo!()
        // if !self
        //     .reader_address
        //     .send_actor_mail(data_reader_actor::IsEnabled)?
        //     .receive_reply()
        //     .await
        // {
        //     self.reader_address
        //         .send_actor_mail(data_reader_actor::Enable {
        //             data_reader_address: self.reader_address.clone(),
        //         })?
        //         .receive_reply()
        //         .await;

        //     self.announce_reader().await?;

        //     self.process_sedp_publications_discovery().await?;
        // }
        // Ok(())
    }
}

pub struct GetDataReaderInstanceHandle {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}
impl Mail for GetDataReaderInstanceHandle {
    type Result = DdsResult<InstanceHandle>;
}
impl MailHandler<GetDataReaderInstanceHandle> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetDataReaderInstanceHandle,
    ) -> <GetDataReaderInstanceHandle as Mail>::Result {
        todo!()
    }
}

pub struct SetDataReaderListener {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub a_listener: Option<Box<dyn AnyDataReaderListener + Send>>,
    pub status_kind: Vec<StatusKind>,
}
impl Mail for SetDataReaderListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDataReaderListener> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetDataReaderListener,
    ) -> <SetDataReaderListener as Mail>::Result {
        todo!()
    }
}

// ############################  Other messages
pub struct AnnounceParticipant;
impl Mail for AnnounceParticipant {
    type Result = DdsResult<()>;
}
impl MailHandler<AnnounceParticipant> for DomainParticipantActor {
    fn handle(&mut self, _: AnnounceParticipant) -> <AnnounceParticipant as Mail>::Result {
        self.announce_participant()
    }
}

pub struct AddCacheChange {
    pub domain_participant_address: ActorAddress<DomainParticipantActor>,
    pub cache_change: ReaderCacheChange,
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}
impl Mail for AddCacheChange {
    type Result = ();
}
impl MailHandler<AddCacheChange> for DomainParticipantActor {
    fn handle(&mut self, message: AddCacheChange) -> <AddCacheChange as Mail>::Result {
        if let Some(subscriber) = self
            .user_defined_subscriber_list
            .iter_mut()
            .find(|s| s.get_instance_handle() == message.subscriber_handle)
        {
            if let Some(reader) = subscriber.get_mut_datareader(message.data_reader_handle) {
                let writer_instance_handle =
                    InstanceHandle::new(message.cache_change.writer_guid.into());

                if reader
                    .get_matched_publications()
                    .contains(&writer_instance_handle)
                {
                    if let Ok(change_instance_handle) = reader.add_change(message.cache_change) {
                        if let DurationKind::Finite(deadline_missed_period) =
                            reader.get_qos().deadline.period
                        {
                            let timer_handle = self.timer_driver.handle();
                            let deadline_missed_task = self.executor.handle().spawn(async move {
                                timer_handle.sleep(deadline_missed_period.into()).await;
                                message
                                    .domain_participant_address
                                    .send_actor_mail(DeadlineMissed {
                                        subscriber_handle: message.subscriber_handle,
                                        data_reader_handle: message.data_reader_handle,
                                        change_instance_handle,
                                    })
                                    .ok();
                            });
                            reader.add_deadline_missed_task(
                                change_instance_handle,
                                deadline_missed_task,
                            );
                        }

                        subscriber
                            .get_statuscondition()
                            .send_actor_mail(status_condition_actor::AddCommunicationState {
                                state: StatusKind::DataOnReaders,
                            })
                            .ok();
                    }
                }
            }
        }
    }
}

pub struct AddBuiltinParticipantsDetectorCacheChange {
    pub cache_change: ReaderCacheChange,
}
impl Mail for AddBuiltinParticipantsDetectorCacheChange {
    type Result = ();
}
impl MailHandler<AddBuiltinParticipantsDetectorCacheChange> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: AddBuiltinParticipantsDetectorCacheChange,
    ) -> <AddBuiltinParticipantsDetectorCacheChange as Mail>::Result {
        if let Some(reader) = self
            .builtin_subscriber
            .lookup_datareader_by_topic_name(DCPS_PARTICIPANT)
        {
            reader.add_change(message.cache_change).ok();
            if let Ok(samples) = reader.read(
                i32::MAX,
                &[SampleStateKind::NotRead],
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
                None,
            ) {
                for (sample_data, sample_info) in samples {
                    match sample_info.instance_state {
                        InstanceStateKind::Alive => {
                            if let Ok(discovered_participant_data) =
                                SpdpDiscoveredParticipantData::deserialize_data(
                                    sample_data
                                        .expect("Alive samples must contain data")
                                        .as_ref(),
                                )
                            {
                                self.add_discovered_participant(discovered_participant_data);
                            }
                        }
                        InstanceStateKind::NotAliveDisposed => {
                            self.remove_discovered_participant(sample_info.instance_handle)
                        }
                        InstanceStateKind::NotAliveNoWriters => (),
                    }
                }
            }
        }
    }
}

pub struct AddBuiltinTopicsDetectorCacheChange {
    pub cache_change: ReaderCacheChange,
}
impl Mail for AddBuiltinTopicsDetectorCacheChange {
    type Result = ();
}
impl MailHandler<AddBuiltinTopicsDetectorCacheChange> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: AddBuiltinTopicsDetectorCacheChange,
    ) -> <AddBuiltinTopicsDetectorCacheChange as Mail>::Result {
        if let Some(reader) = self
            .builtin_subscriber
            .lookup_datareader_by_topic_name(DCPS_TOPIC)
        {
            reader.add_change(message.cache_change).ok();
            if let Ok(samples) = reader.read(
                i32::MAX,
                &[SampleStateKind::NotRead],
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
                None,
            ) {
                for (sample_data, sample_info) in samples {
                    match sample_info.instance_state {
                        InstanceStateKind::Alive => {
                            if let Ok(discovered_topic_data) = DiscoveredTopicData::deserialize_data(
                                sample_data
                                    .expect("Alive samples must contain data")
                                    .as_ref(),
                            ) {
                                self.add_discovered_topic(discovered_topic_data);
                            }
                        }
                        InstanceStateKind::NotAliveDisposed => (), // Discovered topics are not deleted,
                        InstanceStateKind::NotAliveNoWriters => (),
                    }
                }
            }
        }
    }
}

pub struct AddBuiltinPublicationsDetectorCacheChange {
    pub cache_change: ReaderCacheChange,
}
impl Mail for AddBuiltinPublicationsDetectorCacheChange {
    type Result = ();
}
impl MailHandler<AddBuiltinPublicationsDetectorCacheChange> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: AddBuiltinPublicationsDetectorCacheChange,
    ) -> <AddBuiltinPublicationsDetectorCacheChange as Mail>::Result {
        if let Some(reader) = self
            .builtin_subscriber
            .lookup_datareader_by_topic_name(DCPS_PUBLICATION)
        {
            reader.add_change(message.cache_change).ok();
            if let Ok(samples) = reader.read(
                i32::MAX,
                &[SampleStateKind::NotRead],
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
                None,
            ) {
                for (sample_data, sample_info) in samples {
                    match sample_info.instance_state {
                        InstanceStateKind::Alive => {
                            if let Ok(discovered_writer_data) =
                                DiscoveredWriterData::deserialize_data(
                                    sample_data
                                        .expect("Alive samples must contain data")
                                        .as_ref(),
                                )
                            {
                                self.add_discovered_writer(discovered_writer_data);
                            }
                        }
                        InstanceStateKind::NotAliveDisposed => {
                            self.remove_discovered_writer(sample_info.instance_handle)
                        }
                        InstanceStateKind::NotAliveNoWriters => (),
                    }
                }
            }
        }
    }
}

pub struct AddBuiltinSubscriptionsDetectorCacheChange {
    pub cache_change: ReaderCacheChange,
}
impl Mail for AddBuiltinSubscriptionsDetectorCacheChange {
    type Result = ();
}
impl MailHandler<AddBuiltinSubscriptionsDetectorCacheChange> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: AddBuiltinSubscriptionsDetectorCacheChange,
    ) -> <AddBuiltinSubscriptionsDetectorCacheChange as Mail>::Result {
        if let Some(reader) = self
            .builtin_subscriber
            .lookup_datareader_by_topic_name(DCPS_SUBSCRIPTION)
        {
            reader.add_change(message.cache_change).ok();

            self.status_condition
                .send_actor_mail(status_condition_actor::AddCommunicationState {
                    state: StatusKind::DataOnReaders,
                });

            if let Ok(samples) = reader.read(
                i32::MAX,
                &[SampleStateKind::NotRead],
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
                None,
            ) {
                for (sample_data, sample_info) in samples {
                    match sample_info.instance_state {
                        InstanceStateKind::Alive => {
                            if let Ok(discovered_reader_data) =
                                DiscoveredReaderData::deserialize_data(
                                    sample_data
                                        .expect("Alive samples must contain data")
                                        .as_ref(),
                                )
                            {
                                self.add_discovered_reader(discovered_reader_data);
                            }
                        }
                        InstanceStateKind::NotAliveDisposed => {
                            for publisher in self.user_defined_publisher_list.iter_mut() {
                                for data_writer in publisher.data_writer_list.iter_mut() {
                                    if let Some(r) = data_writer
                                        .matched_subscription_list
                                        .remove(&sample_info.instance_handle)
                                    {
                                        // let type_name = self.type_name.clone();
                                        // let topic_name = self.topic_name.clone();
                                        // let participant = publisher.get_participant();
                                        // let status_condition_address = self.status_condition.address();
                                        // let topic_status_condition_address = self.topic_status_condition.clone();
                                        // let topic = TopicAsync::new(
                                        //     self.topic_address.clone(),
                                        //     topic_status_condition_address,
                                        //     type_name,
                                        //     topic_name,
                                        //     participant,
                                        // );
                                        // if self.status_kind.contains(&StatusKind::PublicationMatched) {
                                        //     let status = self.matched_subscriptions.get_publication_matched_status();
                                        //     if let Some(listener) = &self.data_writer_listener_thread {
                                        //         listener.sender().send(DataWriterListenerMessage {
                                        //             listener_operation: DataWriterListenerOperation::PublicationMatched(status),
                                        //             writer_address: data_writer_address,
                                        //             status_condition_address,
                                        //             publisher,
                                        //             topic,
                                        //         })?;
                                        //     }
                                        // } else if publisher_listener_mask.contains(&StatusKind::PublicationMatched) {
                                        //     let status = self.matched_subscriptions.get_publication_matched_status();
                                        //     if let Some(listener) = publisher_listener {
                                        //         listener.send(PublisherListenerMessage {
                                        //             listener_operation: PublisherListenerOperation::PublicationMatched(status),
                                        //             writer_address: data_writer_address,
                                        //             status_condition_address,
                                        //             publisher,
                                        //             topic,
                                        //         })?;
                                        //     }
                                        // } else if participant_listener_mask.contains(&StatusKind::PublicationMatched) {
                                        //     let status = self.matched_subscriptions.get_publication_matched_status();
                                        //     if let Some(listener) = participant_listener {
                                        //         listener.send(ParticipantListenerMessage {
                                        //             listener_operation: ParticipantListenerOperation::PublicationMatched(status),
                                        //             listener_kind: ListenerKind::Writer {
                                        //                 writer_address: data_writer_address,
                                        //                 status_condition_address,
                                        //                 publisher,
                                        //                 topic,
                                        //             },
                                        //         })?;
                                        //     }
                                        // }
                                        data_writer.status_condition.send_actor_mail(
                                            status_condition_actor::AddCommunicationState {
                                                state: StatusKind::PublicationMatched,
                                            },
                                        );
                                    }
                                }
                            }
                        }
                        InstanceStateKind::NotAliveNoWriters => (),
                    }
                }
            }
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

pub struct AnnounceDeletedParticipant;
impl Mail for AnnounceDeletedParticipant {
    type Result = DdsResult<()>;
}
impl MailHandler<AnnounceDeletedParticipant> for DomainParticipantActor {
    fn handle(
        &mut self,
        _: AnnounceDeletedParticipant,
    ) -> <AnnounceDeletedParticipant as Mail>::Result {
        self.announce_deleted_participant()
    }
}

pub struct AreAllChangesAcknowledged {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}
impl Mail for AreAllChangesAcknowledged {
    type Result = DdsResult<bool>;
}
impl MailHandler<AreAllChangesAcknowledged> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: AreAllChangesAcknowledged,
    ) -> <AreAllChangesAcknowledged as Mail>::Result {
        Ok(self
            .user_defined_publisher_list
            .iter()
            .find(|x| x.instance_handle == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .data_writer_list
            .iter()
            .find(|x| x.instance_handle == message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .transport_writer
            .are_all_changes_acknowledged())
    }
}

pub struct IsHistoricalDataReceived {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}
impl Mail for IsHistoricalDataReceived {
    type Result = DdsResult<bool>;
}
impl MailHandler<IsHistoricalDataReceived> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: IsHistoricalDataReceived,
    ) -> <IsHistoricalDataReceived as Mail>::Result {
        self.user_defined_subscriber_list
            .iter()
            .find(|x| x.get_instance_handle() == message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_datareader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .is_historical_data_received()
    }
}

pub struct RemoveWriterChange {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub sequence_number: SequenceNumber,
}
impl Mail for RemoveWriterChange {
    type Result = ();
}
impl MailHandler<RemoveWriterChange> for DomainParticipantActor {
    fn handle(&mut self, message: RemoveWriterChange) -> <RemoveWriterChange as Mail>::Result {
        if let Some(p) = self
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == message.publisher_handle)
        {
            if let Some(dw) = p
                .data_writer_list
                .iter_mut()
                .find(|x| x.instance_handle == message.data_writer_handle)
            {
                dw.transport_writer.remove_change(message.sequence_number);
            }
        }
    }
}

pub struct DeadlineMissed {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub change_instance_handle: InstanceHandle,
}
impl Mail for DeadlineMissed {
    type Result = DdsResult<()>;
}
impl MailHandler<DeadlineMissed> for DomainParticipantActor {
    fn handle(&mut self, message: DeadlineMissed) -> <DeadlineMissed as Mail>::Result {
        self.user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.get_instance_handle() == message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_mut_datareader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .on_deadline_missed(message.change_instance_handle);

        Ok(())
    }
}

fn sedp_data_reader_qos() -> DataReaderQos {
    DataReaderQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepLast(1),
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(0, 0)),
        },
        ..Default::default()
    }
}

fn sedp_data_writer_qos() -> DataWriterQos {
    DataWriterQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepLast(1),
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(0, 0)),
        },
        ..Default::default()
    }
}

fn write_w_timestamp(
    data_writer: &mut DataWriterActor,
    serialized_data: Vec<u8>,
    timestamp: Time,
    type_support: &dyn DynamicType,
) -> DdsResult<SequenceNumber> {
    if !data_writer.enabled {
        return Err(DdsError::NotEnabled);
    }

    data_writer.last_change_sequence_number += 1;

    let instance_handle = get_instance_handle_from_serialized_foo(&serialized_data, type_support)?;

    if !data_writer
        .registered_instance_list
        .contains(&instance_handle)
    {
        if data_writer.registered_instance_list.len()
            < data_writer.qos.resource_limits.max_instances
        {
            data_writer.registered_instance_list.insert(instance_handle);
        } else {
            return Err(DdsError::OutOfResources);
        }
    }

    if let Length::Limited(max_instances) = data_writer.qos.resource_limits.max_instances {
        if !data_writer.instance_samples.contains_key(&instance_handle)
            && data_writer.instance_samples.len() == max_instances as usize
        {
            return Err(DdsError::OutOfResources);
        }
    }

    if let Length::Limited(max_samples_per_instance) =
        data_writer.qos.resource_limits.max_samples_per_instance
    {
        // If the history Qos guarantess that the number of samples
        // is below the limit there is no need to check
        match data_writer.qos.history.kind {
            HistoryQosPolicyKind::KeepLast(depth) if depth <= max_samples_per_instance => {}
            _ => {
                if let Some(s) = data_writer.instance_samples.get(&instance_handle) {
                    // Only Alive changes count towards the resource limits
                    if s.len() >= max_samples_per_instance as usize {
                        return Err(DdsError::OutOfResources);
                    }
                }
            }
        }
    }

    if let Length::Limited(max_samples) = data_writer.qos.resource_limits.max_samples {
        let total_samples = data_writer
            .instance_samples
            .iter()
            .fold(0, |acc, (_, x)| acc + x.len());

        if total_samples >= max_samples as usize {
            return Err(DdsError::OutOfResources);
        }
    }

    let pid_key_hash = Parameter::new(PID_KEY_HASH, Arc::from(*instance_handle.as_ref()));
    let parameter_list = ParameterList::new(vec![pid_key_hash]);

    let change = RtpsCacheChange {
        kind: ChangeKind::Alive,
        sequence_number: data_writer.last_change_sequence_number,
        source_timestamp: Some(timestamp.into()),
        data_value: serialized_data.into(),
        inline_qos: parameter_list,
    };
    if let HistoryQosPolicyKind::KeepLast(depth) = data_writer.qos.history.kind {
        if let Some(s) = data_writer.instance_samples.get_mut(&instance_handle) {
            if s.len() == depth as usize {
                if let Some(smallest_seq_num_instance) = s.pop_front() {
                    data_writer
                        .transport_writer
                        .remove_change(smallest_seq_num_instance);
                }
            }
        }
    }

    let change_timestamp = change.source_timestamp();
    let seq_num = change.sequence_number();

    if seq_num > data_writer.max_seq_num.unwrap_or(0) {
        data_writer.max_seq_num = Some(seq_num)
    }

    if let Some(t) = data_writer
        .instance_deadline_missed_task
        .remove(&instance_handle.into())
    {
        t.abort();
    }

    if let DurationKind::Finite(deadline_missed_period) = data_writer.qos.deadline.period {
        let deadline_missed_interval = std::time::Duration::new(
            deadline_missed_period.sec() as u64,
            deadline_missed_period.nanosec(),
        );
        // let writer_status_condition = self.status_condition.address();
        // let writer_address = message.writer_address.clone();
        // let timer_handle = message.timer_handle.clone();
        // let writer_listener_mask = self.status_kind.clone();
        // let data_writer_listener_sender = self
        //     .data_writer_listener_thread
        //     .as_ref()
        //     .map(|l| l.sender().clone());
        // let publisher_listener = message.publisher_mask_listener.0.clone();
        // let publisher_listener_mask = message.publisher_mask_listener.1.clone();
        // let participant_listener = message.participant_mask_listener.0.clone();
        // let participant_listener_mask = message.participant_mask_listener.1.clone();
        // let status_condition_address = self.status_condition.address();
        // // let topic_address = self.topic_address.clone();
        // // let topic_status_condition_address = self.topic_status_condition.clone();
        // let type_name = self.type_name.clone();
        // let topic_name = self.topic_name.clone();
        // let publisher = message.publisher.clone();

        // let deadline_missed_task = message.executor_handle.spawn(async move {
        //     loop {
        //         timer_handle.sleep(deadline_missed_interval).await;
        //         let publisher_listener = publisher_listener.clone();
        //         let participant_listener = participant_listener.clone();

        //         let r: DdsResult<()> = async {
        //             writer_address.send_actor_mail(
        //                 IncrementOfferedDeadlineMissedStatus {
        //                     instance_handle: change_instance_handle.into(),
        //                 },
        //             )?;

        //             let writer_address = writer_address.clone();
        //             let status_condition_address = status_condition_address.clone();
        //             let publisher = publisher.clone();
        //             let topic = TopicAsync::new(
        //                 topic_address.clone(),
        //                 topic_status_condition_address.clone(),
        //                 type_name.clone(),
        //                 topic_name.clone(),
        //                 publisher.get_participant(),
        //             );
        //             if writer_listener_mask.contains(&StatusKind::OfferedDeadlineMissed) {
        //                 let status = writer_address
        //                     .send_actor_mail(GetOfferedDeadlineMissedStatus)?
        //                     .receive_reply()
        //                     .await;
        //                 if let Some(listener) = &data_writer_listener_sender {
        //                     listener
        //                         .send(DataWriterListenerMessage {
        //                             listener_operation:
        //                                 DataWriterListenerOperation::OfferedDeadlineMissed(
        //                                     status,
        //                                 ),
        //                             writer_address,
        //                             status_condition_address,
        //                             publisher,
        //                             topic,
        //                         })
        //                         .ok();
        //                 }
        //             } else if publisher_listener_mask
        //                 .contains(&StatusKind::OfferedDeadlineMissed)
        //             {
        //                 let status = writer_address
        //                     .send_actor_mail(GetOfferedDeadlineMissedStatus)?
        //                     .receive_reply()
        //                     .await;
        //                 if let Some(listener) = publisher_listener {
        //                     listener
        //                         .send(PublisherListenerMessage {
        //                             listener_operation:
        //                                 PublisherListenerOperation::OfferedDeadlineMissed(
        //                                     status,
        //                                 ),
        //                             writer_address,
        //                             status_condition_address,
        //                             publisher,
        //                             topic,
        //                         })
        //                         .ok();
        //                 }
        //             } else if participant_listener_mask
        //                 .contains(&StatusKind::OfferedDeadlineMissed)
        //             {
        //                 let status = writer_address
        //                     .send_actor_mail(GetOfferedDeadlineMissedStatus)?
        //                     .receive_reply()
        //                     .await;
        //                 if let Some(listener) = participant_listener {
        //                     listener
        //                     .send(ParticipantListenerMessage {
        //                         listener_operation:
        //                             ParticipantListenerOperation::_OfferedDeadlineMissed(
        //                                 status,
        //                             ),
        //                         listener_kind: ListenerKind::Writer {
        //                             writer_address,
        //                             status_condition_address,
        //                             publisher,
        //                             topic,
        //                         },
        //                     })
        //                     .ok();
        //                 }
        //             }
        //             writer_status_condition
        //                 .send_actor_mail(AddCommunicationState {
        //                     state: StatusKind::OfferedDeadlineMissed,
        //                 })?
        //                 .receive_reply()
        //                 .await;
        //             Ok(())
        //         }
        //         .await;
        //         if r.is_err() {
        //             break;
        //         }
        //     }
        // });
        // self.instance_deadline_missed_task
        //     .insert(change_instance_handle.into(), deadline_missed_task);
    }

    data_writer
        .instance_samples
        .entry(instance_handle)
        .or_insert(VecDeque::new())
        .push_back(change.sequence_number);
    data_writer.transport_writer.add_change(change);
    Ok(data_writer.last_change_sequence_number)
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
    if &writer_qos.deadline > discovered_reader_data.deadline() {
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
    if writer_qos.ownership.kind != discovered_reader_data.ownership().kind {
        incompatible_qos_policy_list.push(OWNERSHIP_QOS_POLICY_ID);
    }

    let writer_offered_representation = writer_qos
        .representation
        .value
        .first()
        .unwrap_or(&XCDR_DATA_REPRESENTATION);
    if !(discovered_reader_data
        .representation()
        .value
        .contains(writer_offered_representation)
        || (writer_offered_representation == &XCDR_DATA_REPRESENTATION
            && discovered_reader_data.representation().value.is_empty()))
    {
        incompatible_qos_policy_list.push(DATA_REPRESENTATION_QOS_POLICY_ID);
    }

    incompatible_qos_policy_list
}

fn dispose_w_timestamp(
    data_writer: &mut DataWriterActor,
    serialized_data: Vec<u8>,
    timestamp: Time,
    type_support: &dyn DynamicType,
) -> DdsResult<()> {
    if !data_writer.enabled {
        return Err(DdsError::NotEnabled);
    }

    let has_key = {
        let mut has_key = false;
        for index in 0..type_support.get_member_count() {
            if type_support
                .get_member_by_index(index)?
                .get_descriptor()?
                .is_key
            {
                has_key = true;
                break;
            }
        }
        has_key
    };
    if !has_key {
        return Err(DdsError::IllegalOperation);
    }

    let serialized_key = get_serialized_key_from_serialized_foo(&serialized_data, type_support)?;

    let instance_handle = get_instance_handle_from_serialized_key(&serialized_key, type_support)?;
    if !data_writer
        .registered_instance_list
        .contains(&instance_handle)
    {
        return Err(DdsError::BadParameter);
    }

    if let Some(t) = data_writer
        .instance_deadline_missed_task
        .remove(&instance_handle)
    {
        t.abort();
    }

    data_writer.last_change_sequence_number += 1;

    let mut serialized_status_info = Vec::new();
    let mut serializer = Xcdr1LeSerializer::new(&mut serialized_status_info);
    XTypesSerialize::serialize(&STATUS_INFO_DISPOSED, &mut serializer)?;
    let pid_status_info = Parameter::new(PID_STATUS_INFO, Arc::from(serialized_status_info));

    let pid_key_hash = Parameter::new(PID_KEY_HASH, Arc::from(*instance_handle.as_ref()));
    let parameter_list = ParameterList::new(vec![pid_status_info, pid_key_hash]);

    let cache_change = RtpsCacheChange {
        kind: ChangeKind::NotAliveDisposed,
        sequence_number: data_writer.last_change_sequence_number,
        source_timestamp: Some(timestamp.into()),
        data_value: serialized_key.into(),
        inline_qos: parameter_list,
    };
    data_writer.transport_writer.add_change(cache_change);

    Ok(())
}
