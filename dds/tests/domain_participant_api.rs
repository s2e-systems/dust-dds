use dust_dds::{
    builtin_topics::{
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        error::DdsError,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantQos, PublisherQos, QosKind,
            SubscriberQos, TopicQos,
        },
        qos_policy::{
            GroupDataQosPolicy, ReliabilityQosPolicy, ReliabilityQosPolicyKind, TopicDataQosPolicy,
            UserDataQosPolicy,
        },
        status::{StatusKind, NO_STATUS},
        time::{Duration, DurationKind},
        wait_set::{Condition, WaitSet},
    },
    subscription::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    topic_definition::type_support::{DdsKey, DdsType},
};

mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[derive(serde::Serialize, serde::Deserialize, DdsType)]
struct TestType(u8);

impl DdsKey for TestType {
    type KeyHolder = ();
    type OwningKeyHolder = ();

    fn get_key(&self) -> Self::KeyHolder {
        ()
    }

    fn set_key_from_holder(&mut self, _key_holder: Self::OwningKeyHolder) {}
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, DdsType)]
struct MyData {
    #[key]
    id: u8,
    value: u8,
}

impl DdsKey for MyData {
    type KeyHolder = u8;
    type OwningKeyHolder = u8;

    fn get_key(&self) -> Self::KeyHolder {
        self.id
    }

    fn set_key_from_holder(&mut self, key_holder: Self::OwningKeyHolder) {
        self.id = key_holder;
    }
}

#[test]
fn create_delete_publisher() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();

    assert_eq!(participant.delete_publisher(&publisher), Ok(()));
    assert_eq!(publisher.get_qos(), Err(DdsError::AlreadyDeleted));
    assert_eq!(
        participant.delete_publisher(&publisher),
        Err(DdsError::AlreadyDeleted)
    );
}

#[test]
fn create_delete_subscriber() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();

    assert_eq!(participant.delete_subscriber(&subscriber), Ok(()));
    assert_eq!(subscriber.get_qos(), Err(DdsError::AlreadyDeleted));
    assert_eq!(
        participant.delete_subscriber(&subscriber),
        Err(DdsError::AlreadyDeleted)
    );
}

#[test]
fn create_delete_topic() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic("abc", "TestType", QosKind::Default, None, NO_STATUS)
        .unwrap();

    assert_eq!(participant.delete_topic(&topic), Ok(()));
    assert_eq!(topic.get_qos(), Err(DdsError::AlreadyDeleted));
    assert_eq!(
        participant.delete_topic(&topic),
        Err(DdsError::AlreadyDeleted)
    );
}

#[test]
fn not_allowed_to_delete_publisher_from_different_participant() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let other_participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    assert_eq!(
        other_participant.delete_publisher(&publisher),
        Err(DdsError::PreconditionNotMet(
            "Publisher can only be deleted from its parent participant".to_string()
        ))
    );
}

#[test]
fn not_allowed_to_delete_subscriber_from_different_participant() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let other_participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    assert_eq!(
        other_participant.delete_subscriber(&subscriber),
        Err(DdsError::PreconditionNotMet(
            "Subscriber can only be deleted from its parent participant".to_string()
        ))
    );
}

#[test]
fn not_allowed_to_delete_topic_from_different_participant() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let other_participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic("abc", "TestType", QosKind::Default, None, NO_STATUS)
        .unwrap();
    assert_eq!(
        other_participant.delete_topic(&topic),
        Err(DdsError::PreconditionNotMet(
            "Topic can only be deleted from its parent participant".to_string()
        ))
    );
}

#[test]
fn not_allowed_to_delete_publisher_with_writer() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let writer_topic = participant
        .create_topic("Test", "TestType", QosKind::Default, None, NO_STATUS)
        .expect("Error creating topic");
    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let _a_datawriter = publisher
        .create_datawriter::<TestType>(&writer_topic, QosKind::Default, None, NO_STATUS)
        .unwrap();

    assert_eq!(
        participant.delete_publisher(&publisher),
        Err(DdsError::PreconditionNotMet(
            "Publisher still contains data writers".to_string()
        ))
    );
}

#[test]
fn not_allowed_to_delete_subscriber_with_reader() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let reader_topic = participant
        .create_topic("Test", "TestType", QosKind::Default, None, NO_STATUS)
        .expect("Error creating topic");
    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let _a_datareader = subscriber
        .create_datareader::<TestType>(&reader_topic, QosKind::Default, None, NO_STATUS)
        .unwrap();

    assert_eq!(
        participant.delete_subscriber(&subscriber),
        Err(DdsError::PreconditionNotMet(
            "Subscriber still contains data readers".to_string()
        ))
    );
}

#[test]
fn not_allowed_to_delete_topic_attached_to_reader() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let reader_topic = participant
        .create_topic("Test", "TestType", QosKind::Default, None, NO_STATUS)
        .expect("Error creating topic");
    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let _a_datareader = subscriber
        .create_datareader::<TestType>(&reader_topic, QosKind::Default, None, NO_STATUS)
        .unwrap();

    assert_eq!(
        participant.delete_topic(&reader_topic),
        Err(DdsError::PreconditionNotMet(
            "Topic still attached to some data reader".to_string()
        ))
    );
}

#[test]
fn not_allowed_to_delete_topic_attached_to_writer() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let writer_topic = participant
        .create_topic("Test", "TestType", QosKind::Default, None, NO_STATUS)
        .expect("Error creating topic");
    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let _a_datawriter = publisher
        .create_datawriter::<TestType>(&writer_topic, QosKind::Default, None, NO_STATUS)
        .unwrap();

    assert_eq!(
        participant.delete_topic(&writer_topic),
        Err(DdsError::PreconditionNotMet(
            "Topic still attached to some data writer".to_string()
        ))
    );
}

#[test]
fn allowed_to_delete_publisher_with_created_and_deleted_writer() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let writer_topic = participant
        .create_topic("Test", "TestType", QosKind::Default, None, NO_STATUS)
        .expect("Error creating topic");
    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let a_datawriter = publisher
        .create_datawriter::<TestType>(&writer_topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    publisher
        .delete_datawriter(&a_datawriter)
        .expect("Failed to delete datawriter");
    assert_eq!(participant.delete_publisher(&publisher), Ok(()));
}

#[test]
fn allowed_to_delete_subscriber_with_created_and_deleted_reader() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let reader_topic = participant
        .create_topic("Test", "TestType", QosKind::Default, None, NO_STATUS)
        .expect("Error creating topic");
    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let a_datareader = subscriber
        .create_datareader::<TestType>(&reader_topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    subscriber
        .delete_datareader(&a_datareader)
        .expect("Failed to delete datareader");
    assert_eq!(participant.delete_subscriber(&subscriber), Ok(()));
}

#[test]
fn allowed_to_delete_topic_with_created_and_deleted_writer() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let writer_topic = participant
        .create_topic("Test", "TestType", QosKind::Default, None, NO_STATUS)
        .expect("Error creating topic");
    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let a_datawriter = publisher
        .create_datawriter::<TestType>(&writer_topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    publisher
        .delete_datawriter(&a_datawriter)
        .expect("Failed to delete datawriter");
    assert_eq!(participant.delete_topic(&writer_topic), Ok(()));
}

#[test]
fn allowed_to_delete_topic_with_created_and_deleted_reader() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let reader_topic = participant
        .create_topic("Test", "TestType", QosKind::Default, None, NO_STATUS)
        .expect("Error creating topic");
    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let a_datareader = subscriber
        .create_datareader::<TestType>(&reader_topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    subscriber
        .delete_datareader(&a_datareader)
        .expect("Failed to delete datareader");
    assert_eq!(participant.delete_topic(&reader_topic), Ok(()));
}

#[test]
fn default_publisher_qos() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let group_data = vec![1, 2, 3];
    let qos = PublisherQos {
        group_data: GroupDataQosPolicy {
            value: group_data.clone(),
        },
        ..Default::default()
    };

    participant
        .set_default_publisher_qos(QosKind::Specific(qos))
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();

    assert_eq!(
        &participant
            .get_default_publisher_qos()
            .unwrap()
            .group_data
            .value,
        &group_data
    );
    assert_eq!(&publisher.get_qos().unwrap().group_data.value, &group_data);
}

#[test]
fn default_subscriber_qos() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let group_data = vec![1, 2, 3];
    let qos = SubscriberQos {
        group_data: GroupDataQosPolicy {
            value: group_data.clone(),
        },
        ..Default::default()
    };

    participant
        .set_default_subscriber_qos(QosKind::Specific(qos))
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();

    assert_eq!(
        &participant
            .get_default_subscriber_qos()
            .unwrap()
            .group_data
            .value,
        &group_data
    );
    assert_eq!(&subscriber.get_qos().unwrap().group_data.value, &group_data);
}

#[test]
fn default_topic_qos() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic_data = vec![1, 2, 3];
    let qos = TopicQos {
        topic_data: TopicDataQosPolicy {
            value: topic_data.clone(),
        },
        ..Default::default()
    };

    participant
        .set_default_topic_qos(QosKind::Specific(qos))
        .unwrap();

    let topic = participant
        .create_topic(
            "default_topic_qos",
            "TestType",
            QosKind::Default,
            None,
            NO_STATUS,
        )
        .unwrap();

    assert_eq!(
        &participant
            .get_default_topic_qos()
            .unwrap()
            .topic_data
            .value,
        &topic_data
    );
    assert_eq!(&topic.get_qos().unwrap().topic_data.value, &topic_data);
}

#[test]
#[ignore = "Broken after refactor"]
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
#[ignore = "Broken after refactor"]
fn get_discovery_data_from_builtin_reader() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_user_data = vec![1, 2];
    let topic_user_data = vec![3, 4];
    let reader_user_data = vec![5, 6];
    let writer_user_data = vec![7, 8];

    let participant = DomainParticipantFactory::get_instance()
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

    let topic = participant
        .create_topic(
            "topic_name",
            "MyData",
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

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();

    let _data_writer = publisher
        .create_datawriter::<MyData>(
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

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();

    let _data_reader = subscriber
        .create_datareader::<MyData>(
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

    let participants_reader_cond = participants_reader.get_statuscondition().unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(participants_reader_cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let participant_samples = participants_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    let topics_reader_cond = topics_reader.get_statuscondition().unwrap();
    topics_reader_cond
        .set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(topics_reader_cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let topic_samples = topics_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    let subscriptions_reader_cond = subscriptions_reader.get_statuscondition().unwrap();
    subscriptions_reader_cond
        .set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(subscriptions_reader_cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let subscription_samples = subscriptions_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    let publications_reader_cond = publications_reader.get_statuscondition().unwrap();
    publications_reader_cond
        .set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(publications_reader_cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let publication_samples = publications_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(
        &participant_samples[0]
            .data
            .as_ref()
            .unwrap()
            .user_data()
            .value,
        &participant_user_data
    );

    assert_eq!(
        &topic_samples[0].data.as_ref().unwrap().topic_data().value,
        &topic_user_data
    );

    assert_eq!(
        &subscription_samples[0]
            .data
            .as_ref()
            .unwrap()
            .user_data()
            .value,
        &reader_user_data
    );

    assert_eq!(
        &publication_samples[0]
            .data
            .as_ref()
            .unwrap()
            .user_data()
            .value,
        &writer_user_data
    );
}

#[test]
fn ignore_publication() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic("MyTopic", "MyData", QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter::<MyData>(&topic, QosKind::Specific(writer_qos), None, NO_STATUS)
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    participant
        .ignore_publication(writer.get_instance_handle().unwrap())
        .unwrap();

    let reader = subscriber
        .create_datareader::<MyData>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    // Readers and writers from ignored participant should never match
    let cond = reader.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    assert!(wait_set.wait(Duration::new(2, 0)).is_err());
}

#[test]
fn ignore_subscription() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic("MyTopic", "MyData", QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<MyData>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    participant
        .ignore_subscription(reader.get_instance_handle().unwrap())
        .unwrap();

    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter::<MyData>(&topic, QosKind::Specific(writer_qos), None, NO_STATUS)
        .unwrap();

    // Readers and writers from ignored participant should never match
    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    assert!(wait_set.wait(Duration::new(2, 0)).is_err());
}

#[test]
fn ignore_participant() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant1 = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let participant2 = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    participant1
        .ignore_participant(participant2.get_instance_handle().unwrap())
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(5));

    // Participant should only discover itself
    assert_eq!(participant1.get_discovered_participants().unwrap().len(), 1);
}
