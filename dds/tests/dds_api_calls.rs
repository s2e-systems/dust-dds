use dust_dds::{
    builtin_topics::{
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        error::DdsError,
        listener::NO_LISTENER,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantQos, PublisherQos, QosKind,
            SubscriberQos, TopicQos,
        },
        qos_policy::{
            GroupDataQosPolicy, HistoryQosPolicy, HistoryQosPolicyKind, Length,
            PresentationQosPolicy, PresentationQosPolicyAccessScopeKind, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind, ResourceLimitsQosPolicy, TopicDataQosPolicy,
            UserDataQosPolicy,
        },
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        status::{NO_STATUS, StatusKind},
        time::{Duration, DurationKind},
        type_support::DdsType,
    },
    topic_definition::topic_description::TopicDescription,
    wait_set::{Condition, WaitSet},
    xtypes::type_support::TypeSupport,
};

mod domain_id_generator;
use domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[derive(Clone, Debug, PartialEq, DdsType)]
struct KeyedData {
    #[dust_dds(key)]
    id: u8,
    value: u32,
}

#[derive(DdsType)]
struct UserType(i32);

#[test]
fn data_writer_write_more_than_max_instances_should_fail() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let data_writer_qos = DataWriterQos {
        resource_limits: ResourceLimitsQosPolicy {
            max_samples: Length::Unlimited,
            max_instances: Length::Limited(1),
            max_samples_per_instance: Length::Unlimited,
        },
        ..Default::default()
    };
    let data_writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(data_writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let data_instance1 = KeyedData { id: 1, value: 0 };
    let data_instance2 = KeyedData { id: 2, value: 0 };
    data_writer.write(data_instance1, None).unwrap();

    let result = data_writer.write(data_instance2, None);
    assert_eq!(result, Err(DdsError::OutOfResources));
}

#[test]
fn data_writer_write_more_than_max_samples_per_instances_should_fail() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let data_writer_qos = DataWriterQos {
        resource_limits: ResourceLimitsQosPolicy {
            max_samples: Length::Unlimited,
            max_instances: Length::Unlimited,
            max_samples_per_instance: Length::Limited(1),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let data_writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(data_writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let data1_instance1 = KeyedData { id: 1, value: 0 };
    let data2_instance1 = KeyedData { id: 1, value: 1 };
    data_writer.write(data1_instance1, None).unwrap();

    let result = data_writer.write(data2_instance1, None);
    assert_eq!(result, Err(DdsError::OutOfResources));
}

#[test]
fn data_writer_write_more_than_max_samples_should_fail() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let data_writer_qos = DataWriterQos {
        resource_limits: ResourceLimitsQosPolicy {
            max_samples: Length::Limited(2),
            max_instances: Length::Unlimited,
            max_samples_per_instance: Length::Limited(1),
        },
        ..Default::default()
    };
    let data_writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(data_writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let data_instance1 = KeyedData { id: 1, value: 0 };
    let data_instance2 = KeyedData { id: 2, value: 0 };
    let data_instance3 = KeyedData { id: 3, value: 0 };
    data_writer.write(data_instance1, None).unwrap();
    data_writer.write(data_instance2, None).unwrap();

    let result = data_writer.write(data_instance3, None);
    assert_eq!(result, Err(DdsError::OutOfResources));
}

#[test]
fn get_publisher_parent_participant() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let publisher_parent_participant = publisher.get_participant();

    assert_eq!(
        participant.get_instance_handle(),
        publisher_parent_participant.get_instance_handle()
    );
}

#[test]
fn default_data_writer_qos() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<UserType>(
            "default_data_writer_qos",
            "UserType",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let user_data = vec![1, 2, 3];
    let qos = DataWriterQos {
        user_data: UserDataQosPolicy {
            value: user_data.clone(),
        },
        ..Default::default()
    };

    publisher
        .set_default_datawriter_qos(QosKind::Specific(qos))
        .unwrap();

    let writer = publisher
        .create_datawriter::<UserType>(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    assert_eq!(
        &publisher
            .get_default_datawriter_qos()
            .unwrap()
            .user_data
            .value,
        &user_data
    );
    assert_eq!(&writer.get_qos().unwrap().user_data.value, &user_data);
}

#[test]
fn different_writers_have_different_instance_handles() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<UserType>(
            "default_data_writer_qos",
            "UserType",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher1 = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let publisher2 = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer1_1 = publisher1
        .create_datawriter::<UserType>(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer1_2 = publisher1
        .create_datawriter::<UserType>(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer2_1 = publisher2
        .create_datawriter::<UserType>(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer2_2 = publisher2
        .create_datawriter::<UserType>(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    assert_ne!(
        writer1_1.get_instance_handle(),
        writer1_2.get_instance_handle()
    );
    assert_ne!(
        writer1_2.get_instance_handle(),
        writer2_1.get_instance_handle()
    );
    assert_ne!(
        writer2_1.get_instance_handle(),
        writer2_2.get_instance_handle()
    );
}

#[test]
fn data_writer_get_topic_should_return_same_topic_as_used_for_creation() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<UserType>(
            "default_data_writer_qos",
            "UserType",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer = publisher
        .create_datawriter::<UserType>(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    assert!(writer.get_topic().get_instance_handle() == topic.get_instance_handle())
}

#[test]
fn data_writer_qos_history_keep_all() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let domain_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic_qos = TopicQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let topic = domain_participant
        .create_topic::<UserType>(
            "data_writer_qos_history_keep_all",
            UserType::get_type().get_name(),
            QosKind::Specific(topic_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = domain_participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let _writer = publisher
        .create_datawriter::<UserType>(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
}

#[test]
fn end_coherent_changes_without_begin_fails() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    assert!(matches!(
        publisher.end_coherent_changes(),
        Err(DdsError::PreconditionNotMet(_))
    ));
}

#[test]
fn coherent_changes_nested_calls() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    publisher.begin_coherent_changes().unwrap();
    publisher.begin_coherent_changes().unwrap();

    publisher.end_coherent_changes().unwrap();
    publisher.end_coherent_changes().unwrap();

    assert!(matches!(
        publisher.end_coherent_changes(),
        Err(DdsError::PreconditionNotMet(_))
    ));
}

#[test]
fn coherent_changes_with_different_presentation_qos() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    for access_scope in [
        PresentationQosPolicyAccessScopeKind::Instance,
        PresentationQosPolicyAccessScopeKind::Topic,
        PresentationQosPolicyAccessScopeKind::Group,
    ] {
        let publisher_qos = PublisherQos {
            presentation: PresentationQosPolicy {
                access_scope,
                coherent_access: true,
                ordered_access: false,
            },
            ..Default::default()
        };
        let publisher = participant
            .create_publisher(QosKind::Specific(publisher_qos), NO_LISTENER, NO_STATUS)
            .unwrap();

        publisher.begin_coherent_changes().unwrap();
        publisher.end_coherent_changes().unwrap();
    }
}

#[test]
fn coherent_changes_on_deleted_publisher_fails() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    participant.delete_publisher(&publisher).unwrap();

    assert_eq!(
        publisher.begin_coherent_changes(),
        Err(DdsError::AlreadyDeleted)
    );
    assert_eq!(
        publisher.end_coherent_changes(),
        Err(DdsError::AlreadyDeleted)
    );
}

#[test]
fn get_subscriber_parent_participant() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let subscriber_parent_participant = subscriber.get_participant();

    assert_eq!(
        participant.get_instance_handle(),
        subscriber_parent_participant.get_instance_handle()
    );
}

#[test]
fn default_data_reader_qos() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<UserType>(
            "default_data_reader_qos",
            "UserType",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let user_data = vec![1, 2, 3];
    let qos = DataReaderQos {
        user_data: UserDataQosPolicy {
            value: user_data.clone(),
        },
        ..Default::default()
    };

    subscriber
        .set_default_datareader_qos(QosKind::Specific(qos))
        .unwrap();

    let reader = subscriber
        .create_datareader::<UserType>(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    assert_eq!(
        &subscriber
            .get_default_datareader_qos()
            .unwrap()
            .user_data
            .value,
        &user_data
    );
    assert_eq!(&reader.get_qos().unwrap().user_data.value, &user_data);
}

#[test]
fn different_readers_have_different_instance_handles() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<UserType>(
            "default_data_writer_qos",
            "UserType",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber1 = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let subscriber2 = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let reader1_1 = subscriber1
        .create_datareader::<UserType>(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader1_2 = subscriber1
        .create_datareader::<UserType>(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader2_1 = subscriber2
        .create_datareader::<UserType>(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader2_2 = subscriber2
        .create_datareader::<UserType>(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    assert_ne!(
        reader1_1.get_instance_handle(),
        reader1_2.get_instance_handle()
    );
    assert_ne!(
        reader1_2.get_instance_handle(),
        reader2_1.get_instance_handle()
    );
    assert_ne!(
        reader2_1.get_instance_handle(),
        reader2_2.get_instance_handle()
    );
}

#[test]
fn data_reader_get_topicdescription_should_return_same_topic_as_used_for_creation() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<UserType>(
            "default_data_writer_qos",
            "UserType",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let reader = subscriber
        .create_datareader::<UserType>(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    assert_eq!(reader.get_topicdescription().get_name(), topic.get_name())
}

#[test]
fn create_delete_publisher() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
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
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
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
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<UserType>("abc", "UserType", QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    assert_eq!(participant.delete_topic(&topic), Ok(()));
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
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let other_participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
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
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let other_participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
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
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let other_participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<UserType>("abc", "UserType", QosKind::Default, NO_LISTENER, NO_STATUS)
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
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer_topic = participant
        .create_topic::<UserType>("Test", "UserType", QosKind::Default, NO_LISTENER, NO_STATUS)
        .expect("Error creating topic");
    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let _a_datawriter = publisher
        .create_datawriter::<UserType>(&writer_topic, QosKind::Default, NO_LISTENER, NO_STATUS)
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
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let reader_topic = participant
        .create_topic::<UserType>("Test", "UserType", QosKind::Default, NO_LISTENER, NO_STATUS)
        .expect("Error creating topic");
    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let _a_datareader = subscriber
        .create_datareader::<UserType>(&reader_topic, QosKind::Default, NO_LISTENER, NO_STATUS)
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
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_topic = participant
        .create_topic::<UserType>("Test", "UserType", QosKind::Default, NO_LISTENER, NO_STATUS)
        .expect("Error creating topic");
    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let _a_datareader = subscriber
        .create_datareader::<UserType>(&reader_topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    assert_eq!(
        participant.delete_topic(&reader_topic),
        Err(DdsError::PreconditionNotMet(
            "Topic still attached to some data writer or data reader".to_string()
        ))
    );
}

#[test]
fn not_allowed_to_delete_topic_attached_to_writer() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_topic = participant
        .create_topic::<UserType>("Test", "UserType", QosKind::Default, NO_LISTENER, NO_STATUS)
        .expect("Error creating topic");
    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let _a_datawriter = publisher
        .create_datawriter::<UserType>(&writer_topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    assert_eq!(
        participant.delete_topic(&writer_topic),
        Err(DdsError::PreconditionNotMet(
            "Topic still attached to some data writer or data reader".to_string()
        ))
    );
}

#[test]
fn not_allowed_to_create_topic_with_builtin_topic_name() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    assert!(
        participant
            .create_topic::<UserType>(
                "DCPSParticipant",
                "UserType",
                QosKind::Default,
                NO_LISTENER,
                NO_STATUS
            )
            .is_err()
    );

    assert!(
        participant
            .create_topic::<UserType>(
                "DCPSTopic",
                "UserType",
                QosKind::Default,
                NO_LISTENER,
                NO_STATUS
            )
            .is_err()
    );

    assert!(
        participant
            .create_topic::<UserType>(
                "DCPSPublication",
                "UserType",
                QosKind::Default,
                NO_LISTENER,
                NO_STATUS
            )
            .is_err()
    );

    assert!(
        participant
            .create_topic::<UserType>(
                "DCPSSubscription",
                "UserType",
                QosKind::Default,
                NO_LISTENER,
                NO_STATUS
            )
            .is_err()
    );
}

#[test]
fn allowed_to_delete_publisher_with_created_and_deleted_writer() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer_topic = participant
        .create_topic::<UserType>("Test", "UserType", QosKind::Default, NO_LISTENER, NO_STATUS)
        .expect("Error creating topic");
    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let a_datawriter = publisher
        .create_datawriter::<UserType>(&writer_topic, QosKind::Default, NO_LISTENER, NO_STATUS)
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
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_topic = participant
        .create_topic::<UserType>("Test", "UserType", QosKind::Default, NO_LISTENER, NO_STATUS)
        .expect("Error creating topic");
    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let a_datareader = subscriber
        .create_datareader::<UserType>(&reader_topic, QosKind::Default, NO_LISTENER, NO_STATUS)
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
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_topic = participant
        .create_topic::<UserType>("Test", "UserType", QosKind::Default, NO_LISTENER, NO_STATUS)
        .expect("Error creating topic");
    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let a_datawriter = publisher
        .create_datawriter::<UserType>(&writer_topic, QosKind::Default, NO_LISTENER, NO_STATUS)
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
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_topic = participant
        .create_topic::<UserType>("Test", "UserType", QosKind::Default, NO_LISTENER, NO_STATUS)
        .expect("Error creating topic");
    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let a_datareader = subscriber
        .create_datareader::<UserType>(&reader_topic, QosKind::Default, NO_LISTENER, NO_STATUS)
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
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
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
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
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
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
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
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
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
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
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
        .create_topic::<UserType>(
            "default_topic_qos",
            "UserType",
            QosKind::Default,
            NO_LISTENER,
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

    assert_eq!(&topic.get_qos().unwrap().topic_data.value, &topic_data)
}

#[test]
fn builtin_topic_access() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    assert!(
        participant
            .lookup_topicdescription("DCPSParticipant")
            .is_ok()
    );

    assert!(participant.lookup_topicdescription("DCPSTopic").is_ok());

    assert!(
        participant
            .lookup_topicdescription("DCPSPublication")
            .is_ok()
    );

    assert!(
        participant
            .lookup_topicdescription("DCPSSubscription")
            .is_ok()
    );
}

#[test]
fn builtin_topics_accessible_after_delete_contained_entities() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    participant.delete_contained_entities().unwrap();

    assert!(
        participant
            .lookup_topicdescription("DCPSParticipant")
            .is_ok()
    );

    assert!(participant.lookup_topicdescription("DCPSTopic").is_ok());

    assert!(
        participant
            .lookup_topicdescription("DCPSPublication")
            .is_ok()
    );

    assert!(
        participant
            .lookup_topicdescription("DCPSSubscription")
            .is_ok()
    );
}

#[test]
fn builtin_reader_access() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let builtin_subscriber = participant.get_builtin_subscriber();

    assert!(
        builtin_subscriber
            .lookup_datareader::<ParticipantBuiltinTopicData>("DCPSParticipant")
            .is_ok()
    );

    assert!(
        builtin_subscriber
            .lookup_datareader::<TopicBuiltinTopicData>("DCPSTopic")
            .is_ok()
    );

    assert!(
        builtin_subscriber
            .lookup_datareader::<PublicationBuiltinTopicData>("DCPSPublication")
            .is_ok()
    );

    assert!(
        builtin_subscriber
            .lookup_datareader::<SubscriptionBuiltinTopicData>("DCPSSubscription")
            .is_ok()
    );
}

#[test]
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
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let topic = participant
        .create_topic::<UserType>(
            "topic_name",
            "UserType",
            QosKind::Specific(TopicQos {
                topic_data: TopicDataQosPolicy {
                    value: topic_user_data.clone(),
                },
                ..Default::default()
            }),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let data_writer = publisher
        .create_datawriter::<KeyedData>(
            &topic,
            QosKind::Specific(DataWriterQos {
                user_data: UserDataQosPolicy {
                    value: writer_user_data.clone(),
                },
                ..Default::default()
            }),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let data_reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(DataReaderQos {
                user_data: UserDataQosPolicy {
                    value: reader_user_data.clone(),
                },
                ..Default::default()
            }),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    // Wait for reader to be matched to data writer
    let cond = data_writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    // Wait for the writer to be matched to data reader
    let cond = data_reader.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let builtin_subscriber = participant.get_builtin_subscriber();

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

    let participant_samples = participants_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    let subscription_samples = subscriptions_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    let publication_samples = publications_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    let start_time = std::time::Instant::now();
    loop {
        if let Ok(topic_samples) =
            topics_reader.read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        {
            assert_eq!(
                &topic_samples[0].data.as_ref().unwrap().topic_data().value,
                &topic_user_data
            );
            break;
        }

        if start_time.elapsed() > std::time::Duration::from_secs(10) {
            panic!("Topic discovery data not received")
        }
    }

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
fn ignore_participant() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant1 = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let participant2 = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    participant1
        .ignore_participant(participant2.get_instance_handle())
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(5));

    // Participant 1
    let participants = participant1.get_discovered_participants().unwrap();
    assert_eq!(participants.len(), 1);
    assert_eq!(participants[0], participant1.get_instance_handle());

    // Participant 2
    let participants = participant2.get_discovered_participants().unwrap();
    assert_eq!(participants.len(), 2);
    assert!(participants.contains(&participant2.get_instance_handle()));
    assert!(participants.contains(&participant1.get_instance_handle()));
}

#[test]
#[ignore = "Functionality needs to be revisited"]
fn ignore_publication() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter::<KeyedData>(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    participant
        .ignore_publication(writer.get_instance_handle())
        .unwrap();

    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    // Readers and writers from ignored participant should never match
    let cond = reader.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    assert!(wait_set.wait(Duration::new(2, 0)).is_err());
}

#[test]
#[ignore = "Functionality needs to be revisited"]
fn ignore_subscription() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    participant
        .ignore_subscription(reader.get_instance_handle())
        .unwrap();

    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter::<KeyedData>(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    // Readers and writers from ignored participant should never match
    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    assert!(wait_set.wait(Duration::new(2, 0)).is_err());
}

#[test]
fn find_topic_timeout() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant.find_topic::<UserType>("abc", Duration::new(0, 500_000));

    assert!(matches!(topic, Err(DdsError::Timeout)));
}

#[test]
#[ignore = "Not yet implemented"]
fn create_delete_content_filtered_topic() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<UserType>("abc", "UserType", QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let filter_expression = String::from(r#"value =  %0 "#);
    let expression_parameters = vec![String::from(r#"10"#)];
    let content_filtered_topic = participant
        .create_contentfilteredtopic(
            "filter_abc",
            &topic,
            filter_expression,
            expression_parameters,
        )
        .unwrap();

    assert_eq!(
        participant.delete_contentfilteredtopic(&content_filtered_topic),
        Ok(())
    );
    assert_eq!(
        participant.delete_contentfilteredtopic(&content_filtered_topic),
        Err(DdsError::AlreadyDeleted)
    );
}
