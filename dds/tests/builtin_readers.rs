use dust_dds::{
    builtin_topics::{
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataReaderQos, DataWriterQos, DomainParticipantQos, QosKind, TopicQos},
        qos_policy::{TopicDataQosPolicy, UserDataQosPolicy},
        status::NO_STATUS,
        time::Duration,
        wait_set::{Condition, WaitSet},
    },
    subscription::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    topic_definition::type_support::{DdsSerde, DdsType},
};

mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, DdsType, DdsSerde)]
struct MyData {
    #[key]
    id: u8,
    value: u8,
}

#[test]
fn builtin_reader_access() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let builtin_subscriber = participant.get_builtin_subscriber().unwrap();

    assert!(builtin_subscriber
        .lookup_datareader::<ParticipantBuiltinTopicData>("DCPSParticipant")
        .is_ok());

    assert!(builtin_subscriber
        .lookup_datareader::<TopicBuiltinTopicData>("DCPSTopic")
        .is_ok());

    assert!(builtin_subscriber
        .lookup_datareader::<PublicationBuiltinTopicData>("DCPSPublication")
        .is_ok());

    assert!(builtin_subscriber
        .lookup_datareader::<SubscriptionBuiltinTopicData>("DCPSSubscription")
        .is_ok());
}

#[test]
fn get_discovery_data_from_builtin_reader() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_user_data = vec![1, 2];
    let topic_user_data = vec![3, 4];
    let reader_user_data = vec![5, 6];
    let writer_user_data = vec![7, 8];

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let participant2 = DomainParticipantFactory::get_instance()
        .create_participant(
            domain_id,
            QosKind::Specific(DomainParticipantQos {
                user_data: UserDataQosPolicy {
                    value: participant_user_data.clone(),
                },
                ..Default::default()
            }),
            None,
            NO_STATUS,
        )
        .unwrap();

    let topic = participant2
        .create_topic::<MyData>(
            "topic_name",
            QosKind::Specific(TopicQos {
                topic_data: TopicDataQosPolicy {
                    value: topic_user_data.clone(),
                },
                ..Default::default()
            }),
            None,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant2
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();

    let _data_writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(DataWriterQos {
                user_data: UserDataQosPolicy {
                    value: writer_user_data.clone(),
                },
                ..Default::default()
            }),
            None,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant2
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();

    let _data_reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Specific(DataReaderQos {
                user_data: UserDataQosPolicy {
                    value: reader_user_data.clone(),
                },
                ..Default::default()
            }),
            None,
            NO_STATUS,
        )
        .unwrap();
    let builtin_subscriber = participant.get_builtin_subscriber().unwrap();

    let participants_reader = builtin_subscriber
        .lookup_datareader::<ParticipantBuiltinTopicData>("DCPSParticipant")
        .unwrap()
        .unwrap();

    let topics_reader = builtin_subscriber
        .lookup_datareader::<TopicBuiltinTopicData>("DCPSTopic")
        .unwrap()
        .unwrap();

    let publications_reader = builtin_subscriber
        .lookup_datareader::<PublicationBuiltinTopicData>("DCPSPublication")
        .unwrap()
        .unwrap();

    let subscriptions_reader = builtin_subscriber
        .lookup_datareader::<SubscriptionBuiltinTopicData>("DCPSSubscription")
        .unwrap()
        .unwrap();

    let subscriptions_reader_cond = subscriptions_reader.get_statuscondition().unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(subscriptions_reader_cond))
        .unwrap();
    wait_set.wait(Duration::new(4, 0)).unwrap();

    // Note: Participant also discovers itself
    let participant_samples = participants_reader
        .read(2, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    let topic_samples = topics_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    let subscription_samples = subscriptions_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    let publication_samples = publications_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert!(participant_samples
        .iter()
        .find(|x| &x.data.as_ref().unwrap().user_data.value == &participant_user_data)
        .is_some());

    assert_eq!(
        &topic_samples[0].data.as_ref().unwrap().topic_data.value,
        &topic_user_data
    );

    assert_eq!(
        &subscription_samples[0]
            .data
            .as_ref()
            .unwrap()
            .user_data
            .value,
        &reader_user_data
    );

    assert_eq!(
        &publication_samples[0]
            .data
            .as_ref()
            .unwrap()
            .user_data
            .value,
        &writer_user_data
    );
}
