use std::{collections::HashMap, sync::Arc};

use super::{entities::domain_participant::DomainParticipantEntity, handle::InstanceHandleCounter};
use crate::{
    builtin_topics::{DCPS_PARTICIPANT, DCPS_PUBLICATION, DCPS_SUBSCRIPTION, DCPS_TOPIC},
    dds_async::domain_participant_listener::DomainParticipantListenerAsync,
    domain::domain_participant_factory::DomainId,
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::DiscoveredWriterData,
            spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        },
        domain_participant_backend::entities::{
            data_reader::DataReaderEntity, data_writer::DataWriterEntity,
            publisher::PublisherEntity, subscriber::SubscriberEntity, topic::TopicEntity,
        },
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        instance::InstanceHandle,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantQos, PublisherQos, SubscriberQos,
            TopicQos,
        },
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        status::StatusKind,
        time::{Duration, DurationKind},
    },
    rtps::transport::Transport,
    runtime::{actor::Actor, executor::Executor, timer::TimerDriver},
    topic_definition::type_support::TypeSupport,
};

pub struct DomainParticipantActor {
    pub transport: Box<dyn Transport>,
    pub instance_handle_counter: InstanceHandleCounter,
    pub domain_participant: DomainParticipantEntity,
    pub executor: Executor,
    pub timer_driver: TimerDriver,
}

impl DomainParticipantActor {
    pub fn new(
        domain_id: DomainId,
        domain_participant_qos: DomainParticipantQos,
        listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
        executor: Executor,
        timer_driver: TimerDriver,
        mut transport: Box<dyn Transport>,
    ) -> Self {
        let mut instance_handle_counter = InstanceHandleCounter::default();
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

        let mut topic_list = HashMap::new();
        let spdp_topic_participant_handle = instance_handle_counter.generate_new_instance_handle();

        let mut spdp_topic_participant = TopicEntity::new(
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
        let mut sedp_topic_topics = TopicEntity::new(
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
        let mut sedp_topic_publications = TopicEntity::new(
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
        let mut sedp_topic_subscriptions = TopicEntity::new(
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

        let dcps_participant_reader = DataReaderEntity::new(
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
        let dcps_topic_reader = DataReaderEntity::new(
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
        let dcps_publication_reader = DataReaderEntity::new(
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
        let dcps_subscription_reader = DataReaderEntity::new(
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

        let mut builtin_subscriber = SubscriberEntity::new(
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

        let mut dcps_participant_writer = DataWriterEntity::new(
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

        let mut dcps_topics_writer = DataWriterEntity::new(
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
        let mut dcps_publications_writer = DataWriterEntity::new(
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

        let mut dcps_subscriptions_writer = DataWriterEntity::new(
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
        let mut builtin_publisher = PublisherEntity::new(
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
        let instance_handle = InstanceHandle::new(transport.guid().into());
        let status_condition = Actor::spawn(StatusConditionActor::default(), &executor.handle());
        let domain_participant = DomainParticipantEntity::new(
            domain_id,
            domain_participant_qos,
            listener,
            status_kind,
            status_condition,
            instance_handle,
            builtin_publisher,
            builtin_subscriber,
            topic_list,
        );

        Self {
            transport,
            instance_handle_counter,
            domain_participant,
            executor,
            timer_driver,
        }
    }
}
