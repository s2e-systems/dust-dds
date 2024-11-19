use fnmatch_regex::glob_to_regex;

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
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::DiscoveredWriterData,
            spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        },
        listeners::{
            publisher_listener::PublisherListenerThread,
            subscriber_listener::SubscriberListenerThread,
        },
        runtime::{executor::Executor, mpsc::MpscSender, timer::TimerDriver},
        status_condition::status_condition_actor::{self, StatusConditionActor},
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
            LifespanQosPolicy, QosPolicyId, ReliabilityQosPolicy, ReliabilityQosPolicyKind,
            ResourceLimitsQosPolicy, TransportPriorityQosPolicy, DATA_REPRESENTATION_QOS_POLICY_ID,
            DEADLINE_QOS_POLICY_ID, DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID,
            LATENCYBUDGET_QOS_POLICY_ID, LIVELINESS_QOS_POLICY_ID, OWNERSHIP_QOS_POLICY_ID,
            PRESENTATION_QOS_POLICY_ID, RELIABILITY_QOS_POLICY_ID, XCDR_DATA_REPRESENTATION,
        },
        status::{
            LivelinessChangedStatus, LivelinessLostStatus, OfferedDeadlineMissedStatus,
            OfferedIncompatibleQosStatus, PublicationMatchedStatus, QosPolicyCount,
            RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
            SampleRejectedStatus, StatusKind, SubscriptionMatchedStatus,
        },
        time::{Duration, DurationKind, Time},
    },
    rtps::{
        messages::submessage_elements::Data,
        reader::ReaderCacheChange,
        transport::Transport,
        types::{ChangeKind, Guid, SequenceNumber, TopicKind, ENTITYID_PARTICIPANT},
    },
    subscription::sample_info::{
        InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind, ANY_INSTANCE_STATE,
        ANY_VIEW_STATE,
    },
    topic_definition::type_support::{DdsDeserialize, DdsSerialize, TypeSupport},
    xtypes::dynamic_type::DynamicType,
};
use core::{future::Future, i32, pin::Pin};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    thread::JoinHandle,
};

use super::{
    entities::{
        data_reader::DataReaderActor, data_writer::DataWriterActor, publisher::PublisherActor,
        subscriber::SubscriberActor, topic::TopicActor,
    },
    handle::InstanceHandleCounter,
};

pub struct DomainParticipantActor {
    pub transport: Box<dyn Transport>,
    pub instance_handle_counter: InstanceHandleCounter,
    pub domain_id: DomainId,
    pub qos: DomainParticipantQos,
    pub builtin_subscriber: SubscriberActor,
    pub builtin_publisher: PublisherActor,
    pub user_defined_subscriber_list: Vec<SubscriberActor>,
    pub default_subscriber_qos: SubscriberQos,
    pub user_defined_publisher_list: Vec<PublisherActor>,
    pub default_publisher_qos: PublisherQos,
    pub topic_list: HashMap<String, TopicActor>,
    pub default_topic_qos: TopicQos,
    pub discovered_participant_list: HashMap<InstanceHandle, SpdpDiscoveredParticipantData>,
    pub discovered_topic_list: HashMap<InstanceHandle, TopicBuiltinTopicData>,
    pub enabled: bool,
    pub ignored_participants: HashSet<InstanceHandle>,
    pub ignored_publications: HashSet<InstanceHandle>,
    pub ignored_subcriptions: HashSet<InstanceHandle>,
    pub ignored_topic_list: HashSet<InstanceHandle>,
    pub participant_listener_thread: Option<ParticipantListenerThread>,
    pub status_kind: Vec<StatusKind>,
    pub status_condition: Actor<StatusConditionActor>,
    pub executor: Executor,
    pub timer_driver: TimerDriver,
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

        let mut instance_handle_counter = InstanceHandleCounter::default();

        let mut topic_list = HashMap::new();
        let spdp_topic_participant_handle = instance_handle_counter.generate_new_instance_handle();

        let mut spdp_topic_participant = TopicActor::new(
            TopicQos::default(),
            "SpdpDiscoveredParticipantData".to_string(),
            DCPS_PARTICIPANT.to_owned(),
            spdp_topic_participant_handle,
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            vec![],
            Arc::new(SpdpDiscoveredParticipantData::get_type()),
        );
        spdp_topic_participant.enable();

        topic_list.insert(DCPS_PARTICIPANT.to_owned(), spdp_topic_participant);

        let sedp_topic_topics_handle = instance_handle_counter.generate_new_instance_handle();
        let mut sedp_topic_topics = TopicActor::new(
            TopicQos::default(),
            "DiscoveredTopicData".to_string(),
            DCPS_TOPIC.to_owned(),
            sedp_topic_topics_handle,
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            vec![],
            Arc::new(DiscoveredTopicData::get_type()),
        );
        sedp_topic_topics.enable();

        topic_list.insert(DCPS_TOPIC.to_owned(), sedp_topic_topics);

        let sedp_topic_publications_handle = instance_handle_counter.generate_new_instance_handle();
        let mut sedp_topic_publications = TopicActor::new(
            TopicQos::default(),
            "DiscoveredWriterData".to_string(),
            DCPS_PUBLICATION.to_owned(),
            sedp_topic_publications_handle,
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            vec![],
            Arc::new(DiscoveredWriterData::get_type()),
        );
        sedp_topic_publications.enable();
        topic_list.insert(DCPS_PUBLICATION.to_owned(), sedp_topic_publications);

        let sedp_topic_subscriptions_handle =
            instance_handle_counter.generate_new_instance_handle();
        let mut sedp_topic_subscriptions = TopicActor::new(
            TopicQos::default(),
            "DiscoveredReaderData".to_string(),
            DCPS_SUBSCRIPTION.to_owned(),
            sedp_topic_subscriptions_handle,
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            vec![],
            Arc::new(DiscoveredReaderData::get_type()),
        );
        sedp_topic_subscriptions.enable();
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

        let dcps_participant_reader = DataReaderActor::new(
            instance_handle_counter.generate_new_instance_handle(),
            spdp_reader_qos,
            topic_list[DCPS_PARTICIPANT].topic_name().to_owned(),
            topic_list[DCPS_PARTICIPANT].type_name().to_owned(),
            topic_list[DCPS_PARTICIPANT].type_support().clone(),
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            Vec::new(),
            transport.get_participant_discovery_reader(),
        );
        let dcps_topic_reader = DataReaderActor::new(
            instance_handle_counter.generate_new_instance_handle(),
            sedp_data_reader_qos(),
            topic_list[DCPS_TOPIC].topic_name().to_owned(),
            topic_list[DCPS_TOPIC].type_name().to_owned(),
            topic_list[DCPS_TOPIC].type_support().clone(),
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            Vec::new(),
            transport.get_topics_discovery_reader(),
        );
        let dcps_publication_reader = DataReaderActor::new(
            instance_handle_counter.generate_new_instance_handle(),
            sedp_data_reader_qos(),
            topic_list[DCPS_PUBLICATION].topic_name().to_owned(),
            topic_list[DCPS_PUBLICATION].type_name().to_owned(),
            topic_list[DCPS_PUBLICATION].type_support().clone(),
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            Vec::new(),
            transport.get_topics_discovery_reader(),
        );
        let dcps_subscription_reader = DataReaderActor::new(
            instance_handle_counter.generate_new_instance_handle(),
            sedp_data_reader_qos(),
            topic_list[DCPS_SUBSCRIPTION].topic_name().to_owned(),
            topic_list[DCPS_SUBSCRIPTION].type_name().to_owned(),
            topic_list[DCPS_SUBSCRIPTION].type_support().clone(),
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            Vec::new(),
            transport.get_topics_discovery_reader(),
        );

        let mut builtin_subscriber = SubscriberActor::new(
            instance_handle_counter.generate_new_instance_handle(),
            SubscriberQos::default(),
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            vec![],
        );
        builtin_subscriber.enable();
        builtin_subscriber.insert_data_reader(dcps_participant_reader);
        builtin_subscriber.insert_data_reader(dcps_topic_reader);
        builtin_subscriber.insert_data_reader(dcps_publication_reader);
        builtin_subscriber.insert_data_reader(dcps_subscription_reader);

        let mut dcps_participant_writer = DataWriterActor::new(
            instance_handle_counter.generate_new_instance_handle(),
            transport.get_participant_discovery_writer(),
            topic_list[DCPS_PARTICIPANT].topic_name().to_owned(),
            topic_list[DCPS_PARTICIPANT].type_name().to_owned(),
            topic_list[DCPS_PARTICIPANT].type_support().clone(),
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            vec![],
            spdp_writer_qos,
        );
        dcps_participant_writer.enable();

        let mut dcps_topics_writer = DataWriterActor::new(
            instance_handle_counter.generate_new_instance_handle(),
            transport.get_topics_discovery_writer(),
            topic_list[DCPS_TOPIC].topic_name().to_owned(),
            topic_list[DCPS_TOPIC].type_name().to_owned(),
            topic_list[DCPS_TOPIC].type_support().clone(),
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            vec![],
            sedp_data_writer_qos(),
        );
        dcps_topics_writer.enable();
        let mut dcps_publications_writer = DataWriterActor::new(
            instance_handle_counter.generate_new_instance_handle(),
            transport.get_publications_discovery_writer(),
            topic_list[DCPS_PUBLICATION].topic_name().to_owned(),
            topic_list[DCPS_PUBLICATION].type_name().to_owned(),
            topic_list[DCPS_PUBLICATION].type_support().clone(),
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            vec![],
            sedp_data_writer_qos(),
        );
        dcps_publications_writer.enable();

        let mut dcps_subscriptions_writer = DataWriterActor::new(
            instance_handle_counter.generate_new_instance_handle(),
            transport.get_subscriptions_discovery_writer(),
            topic_list[DCPS_SUBSCRIPTION].topic_name().to_owned(),
            topic_list[DCPS_SUBSCRIPTION].type_name().to_owned(),
            topic_list[DCPS_SUBSCRIPTION].type_support().clone(),
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            vec![],
            sedp_data_writer_qos(),
        );
        dcps_subscriptions_writer.enable();
        let mut builtin_publisher = PublisherActor::new(
            PublisherQos::default(),
            instance_handle_counter.generate_new_instance_handle(),
            None,
            vec![],
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
        );
        builtin_publisher.enable();
        builtin_publisher.insert_data_writer(dcps_participant_writer);
        builtin_publisher.insert_data_writer(dcps_topics_writer);
        builtin_publisher.insert_data_writer(dcps_publications_writer);
        builtin_publisher.insert_data_writer(dcps_subscriptions_writer);

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
                .data_writer_list_mut()
                .find(|dw| dw.topic_name() == DCPS_PARTICIPANT)
            {
                dw.write_w_timestamp(participant_builtin_topic_data.serialize_data()?, timestamp)?;
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
                .data_writer_list_mut()
                .find(|dw| dw.topic_name() == DCPS_PARTICIPANT)
            {
                dw.dispose_w_timestamp(participant_builtin_topic_data.serialize_data()?, timestamp)?
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
                .data_writer_list_mut()
                .find(|dw| dw.topic_name() == DCPS_PUBLICATION)
            {
                dw.write_w_timestamp(publication_builtin_topic_data.serialize_data()?, timestamp)?;
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
                .data_writer_list_mut()
                .find(|dw| dw.topic_name() == DCPS_PUBLICATION)
            {
                dw.dispose_w_timestamp(publication_builtin_topic_data.serialize_data()?, timestamp)?
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
                .data_writer_list_mut()
                .find(|dw| dw.topic_name() == DCPS_SUBSCRIPTION)
            {
                dw.write_w_timestamp(subscription_builtin_topic_data.serialize_data()?, timestamp)?;
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
                .data_writer_list_mut()
                .find(|dw| dw.topic_name() == DCPS_SUBSCRIPTION)
            {
                dw.dispose_w_timestamp(
                    subscription_builtin_topic_data.serialize_data()?,
                    timestamp,
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
                let topic_qos = topic.qos();
                let is_discovered_topic_consistent = topic_qos.topic_data
                    == discovered_topic_data.topic_builtin_topic_data.topic_data
                    && topic_qos.durability
                        == discovered_topic_data.topic_builtin_topic_data.durability
                    && topic_qos.deadline
                        == discovered_topic_data.topic_builtin_topic_data.deadline
                    && topic_qos.latency_budget
                        == discovered_topic_data
                            .topic_builtin_topic_data
                            .latency_budget
                    && topic_qos.liveliness
                        == discovered_topic_data.topic_builtin_topic_data.liveliness
                    && topic_qos.reliability
                        == discovered_topic_data.topic_builtin_topic_data.reliability
                    && topic_qos.destination_order
                        == discovered_topic_data
                            .topic_builtin_topic_data
                            .destination_order
                    && topic_qos.history == discovered_topic_data.topic_builtin_topic_data.history
                    && topic_qos.resource_limits
                        == discovered_topic_data
                            .topic_builtin_topic_data
                            .resource_limits
                    && topic_qos.transport_priority
                        == discovered_topic_data
                            .topic_builtin_topic_data
                            .transport_priority
                    && topic_qos.lifespan
                        == discovered_topic_data.topic_builtin_topic_data.lifespan
                    && topic_qos.ownership
                        == discovered_topic_data.topic_builtin_topic_data.ownership;
                if discovered_topic_data.topic_builtin_topic_data.type_name == topic.type_name()
                    && discovered_topic_data.topic_builtin_topic_data.name == topic.topic_name()
                    && !is_discovered_topic_consistent
                {
                    topic.add_inconsistent_topic_status();
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
        let discovered_writer_handle =
            InstanceHandle::new(discovered_writer_data.dds_publication_data.key().value);
        let is_publication_ignored = self
            .ignored_publications
            .contains(&discovered_writer_handle);
        if !is_publication_ignored && !is_participant_ignored {
            if let Some(_) = self.discovered_participant_list.get(&InstanceHandle::new(
                discovered_writer_participant_guid.into(),
            )) {
                for subscriber in self.user_defined_subscriber_list.iter_mut() {
                    let is_any_name_matched = discovered_writer_data
                        .dds_publication_data
                        .partition
                        .name
                        .iter()
                        .any(|n| subscriber.qos().partition.name.contains(n));

                    let is_any_received_regex_matched_with_partition_qos = discovered_writer_data
                        .dds_publication_data
                        .partition
                        .name
                        .iter()
                        .filter_map(|n| glob_to_regex(n).ok())
                        .any(|regex| {
                            subscriber
                                .qos()
                                .partition
                                .name
                                .iter()
                                .any(|n| regex.is_match(n))
                        });

                    let is_any_local_regex_matched_with_received_partition_qos = subscriber
                        .qos()
                        .partition
                        .name
                        .iter()
                        .filter_map(|n| glob_to_regex(n).ok())
                        .any(|regex| {
                            discovered_writer_data
                                .dds_publication_data
                                .partition
                                .name
                                .iter()
                                .any(|n| regex.is_match(n))
                        });

                    let is_partition_matched =
                        discovered_writer_data.dds_publication_data.partition
                            == subscriber.qos().partition
                            || is_any_name_matched
                            || is_any_received_regex_matched_with_partition_qos
                            || is_any_local_regex_matched_with_received_partition_qos;
                    if is_partition_matched {
                        let subscriber_qos = subscriber.qos().clone();
                        for data_reader in subscriber.data_reader_list_mut().filter(|dr| {
                            dr.topic_name()
                                == discovered_writer_data.dds_publication_data.topic_name
                        }) {
                            let publication_builtin_topic_data =
                                &discovered_writer_data.dds_publication_data;
                            if publication_builtin_topic_data.topic_name == data_reader.topic_name()
                                && publication_builtin_topic_data.type_name
                                    == data_reader.type_name()
                            {
                                let instance_handle = InstanceHandle::new(
                                    discovered_writer_data.dds_publication_data.key.value,
                                );
                                let incompatible_qos_policy_list =
                                    get_discovered_writer_incompatible_qos_policy_list(
                                        data_reader,
                                        &discovered_writer_data,
                                        &subscriber_qos,
                                    );
                                if incompatible_qos_policy_list.is_empty() {
                                    todo!()
                                    // let insert_matched_publication_result =
                                    //     data_reader.matched_publication_list.insert(
                                    //         instance_handle,
                                    //         publication_builtin_topic_data.clone(),
                                    //     );
                                    // match insert_matched_publication_result {
                                    //     Some(value) if &value != publication_builtin_topic_data => {
                                    //         data_reader.subscription_matched_status.total_count +=
                                    //             1;
                                    //         data_reader
                                    //             .subscription_matched_status
                                    //             .total_count_change += 1;
                                    //         data_reader
                                    //             .subscription_matched_status
                                    //             .last_publication_handle = discovered_writer_handle;
                                    //         data_reader
                                    //             .subscription_matched_status
                                    //             .current_count += 1;
                                    //         data_reader
                                    //             .subscription_matched_status
                                    //             .current_count_change += 1;

                                    //         // const SUBSCRIPTION_MATCHED_STATUS_KIND: &StatusKind = &StatusKind::SubscriptionMatched;
                                    //         // let type_name = self.type_name.clone();
                                    //         // let topic_name = self.topic_name.clone();
                                    //         // let reader_address = data_reader_address.clone();
                                    //         // let status_condition_address = self.status_condition.address();
                                    //         // let subscriber = subscriber.clone();

                                    //         // let topic_status_condition_address = self.topic_status_condition.clone();
                                    //         // let topic = TopicAsync::new(
                                    //         //     self.topic_address.clone(),
                                    //         //     topic_status_condition_address.clone(),
                                    //         //     type_name.clone(),
                                    //         //     topic_name.clone(),
                                    //         //     subscriber.get_participant(),
                                    //         // );
                                    //         // if self
                                    //         //     .data_reader_status_kind
                                    //         //     .contains(SUBSCRIPTION_MATCHED_STATUS_KIND)
                                    //         // {
                                    //         //     let status = self
                                    //         //         .subscription_matched_status
                                    //         //         .read_and_reset(self.matched_publication_list.len() as i32);
                                    //         //     if let Some(listener) = &self.data_reader_listener_thread {
                                    //         //         listener.sender().send(DataReaderListenerMessage {
                                    //         //             listener_operation: DataReaderListenerOperation::SubscriptionMatched(status),
                                    //         //             reader_address,
                                    //         //             status_condition_address,
                                    //         //             subscriber,
                                    //         //             topic,
                                    //         //         })?;
                                    //         //     }
                                    //         // } else if subscriber_listener_mask.contains(SUBSCRIPTION_MATCHED_STATUS_KIND) {
                                    //         //     let status = self
                                    //         //         .subscription_matched_status
                                    //         //         .read_and_reset(self.matched_publication_list.len() as i32);
                                    //         //     if let Some(listener) = subscriber_listener {
                                    //         //         listener.send(SubscriberListenerMessage {
                                    //         //             listener_operation: SubscriberListenerOperation::SubscriptionMatched(status),
                                    //         //             reader_address,
                                    //         //             status_condition_address,
                                    //         //             subscriber,
                                    //         //             topic,
                                    //         //         })?;
                                    //         //     }
                                    //         // } else if participant_listener_mask.contains(SUBSCRIPTION_MATCHED_STATUS_KIND) {
                                    //         //     let status = self
                                    //         //         .subscription_matched_status
                                    //         //         .read_and_reset(self.matched_publication_list.len() as i32);
                                    //         //     if let Some(listener) = participant_listener {
                                    //         //         listener.send(ParticipantListenerMessage {
                                    //         //             listener_operation: ParticipantListenerOperation::SubscriptionMatched(status),
                                    //         //             listener_kind: ListenerKind::Reader {
                                    //         //                 reader_address,
                                    //         //                 status_condition_address,
                                    //         //                 subscriber,
                                    //         //                 topic,
                                    //         //             },
                                    //         //         })?;
                                    //         //     }
                                    //         // }
                                    //         data_reader.status_condition.send_actor_mail(
                                    //             status_condition_actor::AddCommunicationState {
                                    //                 state: StatusKind::SubscriptionMatched,
                                    //             },
                                    //         );
                                    //     }
                                    //     None => {
                                    //         data_reader.subscription_matched_status.total_count +=
                                    //             1;
                                    //         data_reader
                                    //             .subscription_matched_status
                                    //             .total_count_change += 1;
                                    //         data_reader
                                    //             .subscription_matched_status
                                    //             .last_publication_handle = discovered_writer_handle;
                                    //         data_reader
                                    //             .subscription_matched_status
                                    //             .current_count += 1;
                                    //         data_reader
                                    //             .subscription_matched_status
                                    //             .current_count_change += 1;

                                    //         // const SUBSCRIPTION_MATCHED_STATUS_KIND: &StatusKind = &StatusKind::SubscriptionMatched;
                                    //         // let type_name = self.type_name.clone();
                                    //         // let topic_name = self.topic_name.clone();
                                    //         // let reader_address = data_reader_address.clone();
                                    //         // let status_condition_address = self.status_condition.address();
                                    //         // let subscriber = subscriber.clone();

                                    //         // let topic_status_condition_address = self.topic_status_condition.clone();
                                    //         // let topic = TopicAsync::new(
                                    //         //     self.topic_address.clone(),
                                    //         //     topic_status_condition_address.clone(),
                                    //         //     type_name.clone(),
                                    //         //     topic_name.clone(),
                                    //         //     subscriber.get_participant(),
                                    //         // );
                                    //         // if self
                                    //         //     .data_reader_status_kind
                                    //         //     .contains(SUBSCRIPTION_MATCHED_STATUS_KIND)
                                    //         // {
                                    //         //     let status = self
                                    //         //         .subscription_matched_status
                                    //         //         .read_and_reset(self.matched_publication_list.len() as i32);
                                    //         //     if let Some(listener) = &self.data_reader_listener_thread {
                                    //         //         listener.sender().send(DataReaderListenerMessage {
                                    //         //             listener_operation: DataReaderListenerOperation::SubscriptionMatched(status),
                                    //         //             reader_address,
                                    //         //             status_condition_address,
                                    //         //             subscriber,
                                    //         //             topic,
                                    //         //         })?;
                                    //         //     }
                                    //         // } else if subscriber_listener_mask.contains(SUBSCRIPTION_MATCHED_STATUS_KIND) {
                                    //         //     let status = self
                                    //         //         .subscription_matched_status
                                    //         //         .read_and_reset(self.matched_publication_list.len() as i32);
                                    //         //     if let Some(listener) = subscriber_listener {
                                    //         //         listener.send(SubscriberListenerMessage {
                                    //         //             listener_operation: SubscriberListenerOperation::SubscriptionMatched(status),
                                    //         //             reader_address,
                                    //         //             status_condition_address,
                                    //         //             subscriber,
                                    //         //             topic,
                                    //         //         })?;
                                    //         //     }
                                    //         // } else if participant_listener_mask.contains(SUBSCRIPTION_MATCHED_STATUS_KIND) {
                                    //         //     let status = self
                                    //         //         .subscription_matched_status
                                    //         //         .read_and_reset(self.matched_publication_list.len() as i32);
                                    //         //     if let Some(listener) = participant_listener {
                                    //         //         listener.send(ParticipantListenerMessage {
                                    //         //             listener_operation: ParticipantListenerOperation::SubscriptionMatched(status),
                                    //         //             listener_kind: ListenerKind::Reader {
                                    //         //                 reader_address,
                                    //         //                 status_condition_address,
                                    //         //                 subscriber,
                                    //         //                 topic,
                                    //         //             },
                                    //         //         })?;
                                    //         //     }
                                    //         // }
                                    //         data_reader.status_condition.send_actor_mail(
                                    //             status_condition_actor::AddCommunicationState {
                                    //                 state: StatusKind::SubscriptionMatched,
                                    //             },
                                    //         );
                                    //     }
                                    //     _ => (),

                                    // }
                                } else {
                                    data_reader.add_requested_incompatible_qos(
                                        instance_handle,
                                        incompatible_qos_policy_list,
                                    );

                                    // let type_name = self.type_name.clone();
                                    // let topic_name = self.topic_name.clone();
                                    // let topic_status_condition_address = self.topic_status_condition.clone();
                                    // let reader_address = data_reader_address.clone();
                                    // let status_condition_address = self.status_condition.address();
                                    // let subscriber = subscriber.clone();
                                    // let topic = TopicAsync::new(
                                    //     self.topic_address.clone(),
                                    //     topic_status_condition_address.clone(),
                                    //     type_name.clone(),
                                    //     topic_name.clone(),
                                    //     subscriber.get_participant(),
                                    // );
                                    // if self
                                    //     .data_reader_status_kind
                                    //     .contains(&StatusKind::RequestedIncompatibleQos)
                                    // {
                                    //     let status = self.requested_incompatible_qos_status.read_and_reset();
                                    //     if let Some(listener) = &self.data_reader_listener_thread {
                                    //         listener.sender().send(DataReaderListenerMessage {
                                    //             listener_operation: DataReaderListenerOperation::RequestedIncompatibleQos(
                                    //                 status,
                                    //             ),
                                    //             reader_address,
                                    //             status_condition_address,
                                    //             subscriber,
                                    //             topic,
                                    //         })?;
                                    //     }
                                    // } else if subscriber_listener_mask.contains(&StatusKind::RequestedIncompatibleQos) {
                                    //     let status = self.requested_incompatible_qos_status.read_and_reset();
                                    //     if let Some(listener) = subscriber_listener {
                                    //         listener.send(SubscriberListenerMessage {
                                    //             listener_operation: SubscriberListenerOperation::RequestedIncompatibleQos(
                                    //                 status,
                                    //             ),
                                    //             reader_address,
                                    //             status_condition_address,
                                    //             subscriber,
                                    //             topic,
                                    //         })?;
                                    //     }
                                    // } else if participant_listener_mask.contains(&StatusKind::RequestedIncompatibleQos) {
                                    //     let status = self.requested_incompatible_qos_status.read_and_reset();
                                    //     if let Some(listener) = participant_listener {
                                    //         listener.send(ParticipantListenerMessage {
                                    //             listener_operation: ParticipantListenerOperation::RequestedIncompatibleQos(
                                    //                 status,
                                    //             ),
                                    //             listener_kind: ListenerKind::Reader {
                                    //                 reader_address,
                                    //                 status_condition_address,
                                    //                 subscriber,
                                    //                 topic,
                                    //             },
                                    //         })?;
                                    //     }
                                    // }
                                }
                            }
                        }
                    }
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
                        .any(|n| publisher.qos().partition.name.contains(n));

                    let is_any_received_regex_matched_with_partition_qos = discovered_reader_data
                        .dds_subscription_data
                        .partition
                        .name
                        .iter()
                        .filter_map(|n| glob_to_regex(n).ok())
                        .any(|regex| {
                            publisher
                                .qos()
                                .partition
                                .name
                                .iter()
                                .any(|n| regex.is_match(n))
                        });

                    let is_any_local_regex_matched_with_received_partition_qos = publisher
                        .qos()
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
                            == publisher.qos().partition
                            || is_any_name_matched
                            || is_any_received_regex_matched_with_partition_qos
                            || is_any_local_regex_matched_with_received_partition_qos;

                    if is_partition_matched {
                        let publisher_qos = publisher.qos().clone();
                        for dw in publisher.data_writer_list_mut().filter(|dw| {
                            dw.topic_name()
                                == discovered_reader_data
                                    .subscription_builtin_topic_data()
                                    .topic_name()
                        }) {
                            let is_matched_topic_name = discovered_reader_data
                                .subscription_builtin_topic_data()
                                .topic_name()
                                == dw.topic_name();
                            let is_matched_type_name = discovered_reader_data
                                .subscription_builtin_topic_data()
                                .get_type_name()
                                == dw.type_name();

                            if is_matched_topic_name && is_matched_type_name {
                                let incompatible_qos_policy_list =
                                    get_discovered_reader_incompatible_qos_policy_list(
                                        &dw.qos(),
                                        discovered_reader_data.subscription_builtin_topic_data(),
                                        &publisher_qos,
                                    );
                                let instance_handle = InstanceHandle::new(
                                    discovered_reader_data
                                        .subscription_builtin_topic_data()
                                        .key
                                        .value,
                                );
                                if incompatible_qos_policy_list.is_empty() {
                                    if dw.get_matched_subscription_data(&instance_handle)
                                        != Some(
                                            discovered_reader_data
                                                .subscription_builtin_topic_data(),
                                        )
                                    {
                                        dw.add_matched_subscription(
                                            discovered_reader_data
                                                .subscription_builtin_topic_data()
                                                .clone(),
                                        );
                                    }
                                } else {
                                    dw.add_incompatible_subscription(
                                        instance_handle,
                                        incompatible_qos_policy_list,
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
            for data_reader in subscriber.data_reader_list_mut() {
                todo!()
                // let matched_publication = data_reader
                //     .matched_publication_list
                //     .remove(&discovered_writer_handle);
                // if let Some(w) = matched_publication {
                //     data_reader.subscription_matched_status.total_count += 1;
                //     data_reader.subscription_matched_status.total_count_change += 1;
                //     data_reader
                //         .subscription_matched_status
                //         .last_publication_handle = discovered_writer_handle;
                //     data_reader.subscription_matched_status.current_count += 1;
                //     data_reader.subscription_matched_status.current_count_change += 1;

                //     // const SUBSCRIPTION_MATCHED_STATUS_KIND: &StatusKind = &StatusKind::SubscriptionMatched;
                //     // let type_name = self.type_name.clone();
                //     // let topic_name = self.topic_name.clone();
                //     // let reader_address = data_reader_address.clone();
                //     // let status_condition_address = self.status_condition.address();
                //     // let subscriber = subscriber.clone();

                //     // let topic_status_condition_address = self.topic_status_condition.clone();
                //     // let topic = TopicAsync::new(
                //     //     self.topic_address.clone(),
                //     //     topic_status_condition_address.clone(),
                //     //     type_name.clone(),
                //     //     topic_name.clone(),
                //     //     subscriber.get_participant(),
                //     // );
                //     // if self
                //     //     .data_reader_status_kind
                //     //     .contains(SUBSCRIPTION_MATCHED_STATUS_KIND)
                //     // {
                //     //     let status = self
                //     //         .subscription_matched_status
                //     //         .read_and_reset(self.matched_publication_list.len() as i32);
                //     //     if let Some(listener) = &self.data_reader_listener_thread {
                //     //         listener.sender().send(DataReaderListenerMessage {
                //     //             listener_operation: DataReaderListenerOperation::SubscriptionMatched(status),
                //     //             reader_address,
                //     //             status_condition_address,
                //     //             subscriber,
                //     //             topic,
                //     //         })?;
                //     //     }
                //     // } else if subscriber_listener_mask.contains(SUBSCRIPTION_MATCHED_STATUS_KIND) {
                //     //     let status = self
                //     //         .subscription_matched_status
                //     //         .read_and_reset(self.matched_publication_list.len() as i32);
                //     //     if let Some(listener) = subscriber_listener {
                //     //         listener.send(SubscriberListenerMessage {
                //     //             listener_operation: SubscriberListenerOperation::SubscriptionMatched(status),
                //     //             reader_address,
                //     //             status_condition_address,
                //     //             subscriber,
                //     //             topic,
                //     //         })?;
                //     //     }
                //     // } else if participant_listener_mask.contains(SUBSCRIPTION_MATCHED_STATUS_KIND) {
                //     //     let status = self
                //     //         .subscription_matched_status
                //     //         .read_and_reset(self.matched_publication_list.len() as i32);
                //     //     if let Some(listener) = participant_listener {
                //     //         listener.send(ParticipantListenerMessage {
                //     //             listener_operation: ParticipantListenerOperation::SubscriptionMatched(status),
                //     //             listener_kind: ListenerKind::Reader {
                //     //                 reader_address,
                //     //                 status_condition_address,
                //     //                 subscriber,
                //     //                 topic,
                //     //             },
                //     //         })?;
                //     //     }
                //     // }
                //     data_reader.status_condition.send_actor_mail(
                //         status_condition_actor::AddCommunicationState {
                //             state: StatusKind::SubscriptionMatched,
                //         },
                //     );
                // }
            }
        }
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
                .data_writer_list_mut()
                .find(|dw| dw.topic_name() == DCPS_TOPIC)
            {
                dw.write_w_timestamp(topic_builtin_topic_data.serialize_data()?, timestamp)?;
            }
        }
        Ok(())
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
    type Result = DdsResult<()>;
}
impl MailHandler<AddCacheChange> for DomainParticipantActor {
    fn handle(&mut self, message: AddCacheChange) -> <AddCacheChange as Mail>::Result {
        let subscriber = self
            .user_defined_subscriber_list
            .iter_mut()
            .find(|s| s.instance_handle() == message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        let data_reader = subscriber
            .get_mut_data_reader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let writer_instance_handle = InstanceHandle::new(message.cache_change.writer_guid.into());

        if data_reader
            .get_matched_publication_data(&writer_instance_handle)
            .is_some()
        {
            if let Ok(change_instance_handle) = data_reader.add_reader_change(message.cache_change)
            {
                subscriber.status_condition().send_actor_mail(
                    status_condition_actor::AddCommunicationState {
                        state: StatusKind::DataOnReaders,
                    },
                );
            }
        }
        Ok(())
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
        match message.cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(discovered_participant_data) =
                    SpdpDiscoveredParticipantData::deserialize_data(
                        message.cache_change.data_value.as_ref(),
                    )
                {
                    self.discovered_participant_list.insert(
                        InstanceHandle::new(
                            discovered_participant_data.dds_participant_data.key().value,
                        ),
                        discovered_participant_data,
                    );
                }
            }
            ChangeKind::NotAliveDisposed => {
                if let Ok(discovered_participant_handle) =
                    InstanceHandle::deserialize_data(message.cache_change.data_value.as_ref())
                {
                    self.discovered_participant_list
                        .remove(&discovered_participant_handle);
                }
            }
            ChangeKind::AliveFiltered
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => (), // Do nothing,
        }
        if let Some(mut reader) = self
            .builtin_subscriber
            .data_reader_list_mut()
            .find(|dr| dr.topic_name() == DCPS_PARTICIPANT)
        {
            reader.add_reader_change(message.cache_change).ok();
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
        match message.cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(discovered_topic_data) =
                    DiscoveredTopicData::deserialize_data(message.cache_change.data_value.as_ref())
                {
                }
            }
            ChangeKind::NotAliveDisposed => todo!(),
            ChangeKind::AliveFiltered
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => (),
        }
        if let Some(reader) = self
            .builtin_subscriber
            .data_reader_list_mut()
            .find(|dr| dr.topic_name() == DCPS_TOPIC)
        {
            reader.add_reader_change(message.cache_change).ok();
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
                                todo!()
                                // self.add_discovered_topic(discovered_topic_data);
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
            .data_reader_list_mut()
            .find(|dr| dr.topic_name() == DCPS_PUBLICATION)
        {
            reader.add_reader_change(message.cache_change).ok();
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
                                todo!()
                                // self.add_discovered_writer(discovered_writer_data);
                            }
                        }
                        InstanceStateKind::NotAliveDisposed => {
                            todo!()
                            // self.remove_discovered_writer(sample_info.instance_handle)
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
            .data_reader_list_mut()
            .find(|dr| dr.topic_name() == DCPS_SUBSCRIPTION)
        {
            reader.add_reader_change(message.cache_change).ok();

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
                                todo!()
                                // self.add_discovered_reader(discovered_reader_data);
                            }
                        }
                        InstanceStateKind::NotAliveDisposed => {
                            for publisher in self.user_defined_publisher_list.iter_mut() {
                                for data_writer in publisher.data_writer_list_mut() {
                                    todo!()
                                    // if let Some(r) = data_writer
                                    //     .matched_subscription_list
                                    //     .remove(&sample_info.instance_handle)
                                    // {
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
                                    //     data_writer.status_condition.send_actor_mail(
                                    //         status_condition_actor::AddCommunicationState {
                                    //             state: StatusKind::PublicationMatched,
                                    //         },
                                    //     );
                                    // }
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
            .find(|x| x.instance_handle() == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .transport_writer()
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
        let subscriber = self
            .user_defined_subscriber_list
            .iter()
            .find(|x| x.instance_handle() == message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_reader = subscriber
            .get_data_reader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        if !data_reader.enabled() {
            return Err(DdsError::NotEnabled);
        };

        match data_reader.qos().durability.kind {
            DurabilityQosPolicyKind::Volatile => Err(DdsError::IllegalOperation),
            DurabilityQosPolicyKind::TransientLocal
            | DurabilityQosPolicyKind::Transient
            | DurabilityQosPolicyKind::Persistent => Ok(()),
        }?;

        Ok(data_reader.transport_reader().is_historical_data_received())
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
            .find(|x| x.instance_handle() == message.publisher_handle)
        {
            if let Some(dw) = p.get_mut_data_writer(message.data_writer_handle) {
                dw.remove_change(message.sequence_number);
            }
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

fn get_discovered_writer_incompatible_qos_policy_list(
    data_reader: &DataReaderActor,
    discovered_writer_data: &DiscoveredWriterData,
    subscriber_qos: &SubscriberQos,
) -> Vec<QosPolicyId> {
    let writer_info = &discovered_writer_data.dds_publication_data;

    let mut incompatible_qos_policy_list = Vec::new();

    if subscriber_qos.presentation.access_scope > writer_info.presentation().access_scope
        || subscriber_qos.presentation.coherent_access != writer_info.presentation().coherent_access
        || subscriber_qos.presentation.ordered_access != writer_info.presentation().ordered_access
    {
        incompatible_qos_policy_list.push(PRESENTATION_QOS_POLICY_ID);
    }
    if &data_reader.qos().durability > writer_info.durability() {
        incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
    }
    if &data_reader.qos().deadline < writer_info.deadline() {
        incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
    }
    if &data_reader.qos().latency_budget > writer_info.latency_budget() {
        incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
    }
    if &data_reader.qos().liveliness > writer_info.liveliness() {
        incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
    }
    if data_reader.qos().reliability.kind > writer_info.reliability().kind {
        incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
    }
    if &data_reader.qos().destination_order > writer_info.destination_order() {
        incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
    }
    if data_reader.qos().ownership.kind != writer_info.ownership().kind {
        incompatible_qos_policy_list.push(OWNERSHIP_QOS_POLICY_ID);
    }

    let writer_offered_representation = writer_info
        .representation()
        .value
        .first()
        .unwrap_or(&XCDR_DATA_REPRESENTATION);
    if !data_reader
        .qos()
        .representation
        .value
        .contains(writer_offered_representation)
    {
        // Empty list is interpreted as containing XCDR_DATA_REPRESENTATION
        if !(writer_offered_representation == &XCDR_DATA_REPRESENTATION
            && data_reader.qos().representation.value.is_empty())
        {
            incompatible_qos_policy_list.push(DATA_REPRESENTATION_QOS_POLICY_ID)
        }
    }

    incompatible_qos_policy_list
}

fn get_topic_kind(type_support: &dyn DynamicType) -> TopicKind {
    for index in 0..type_support.get_member_count() {
        if let Ok(m) = type_support.get_member_by_index(index) {
            if let Ok(d) = m.get_descriptor() {
                if d.is_key {
                    return TopicKind::WithKey;
                }
            }
        }
    }
    TopicKind::NoKey
}
