use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataReaderQos, DataWriterQos, PublisherQos, QosKind, SubscriberQos},
        qos_policy::{PartitionQosPolicy, UserDataQosPolicy},
        status::{StatusKind, NO_STATUS},
        time::Duration,
        wait_set::{Condition, WaitSet},
    },
    topic_definition::type_support::{DdsGetKey, DdsRepresentation, DdsHasKey},
};

mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[derive(serde::Serialize, serde::Deserialize, DdsHasKey, DdsGetKey, DdsRepresentation)]
struct UserType(i32);

#[test]
fn writer_discovers_reader_in_same_participant() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic("topic_name", "UserType", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let publisher = dp
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer = publisher
        .create_datawriter::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let subscriber = dp
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let _data_reader = subscriber
        .create_datareader::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let cond = data_writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    assert_eq!(data_writer.get_matched_subscriptions().unwrap().len(), 1);
}

#[test]
fn deleted_readers_are_disposed_from_writer() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic("topic_name", "UserType", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let publisher = dp
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer = publisher
        .create_datawriter::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let subscriber = dp
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_reader = subscriber
        .create_datareader::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let cond = data_writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();
    data_writer.get_publication_matched_status().unwrap();

    subscriber.delete_datareader(&data_reader).unwrap();

    wait_set.wait(Duration::new(5, 0)).unwrap();

    assert_eq!(data_writer.get_matched_subscriptions().unwrap().len(), 0);
}

#[test]
fn updated_readers_are_announced_to_writer() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic("topic_name", "UserType", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let publisher = dp
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer = publisher
        .create_datawriter::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let subscriber = dp
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_reader = subscriber
        .create_datareader::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let cond = data_writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();
    data_writer.get_publication_matched_status().unwrap();

    let user_data_qos_policy = UserDataQosPolicy {
        value: vec![1, 2, 3, 4],
    };
    let qos = DataReaderQos {
        user_data: user_data_qos_policy.clone(),
        ..Default::default()
    };

    data_reader.set_qos(QosKind::Specific(qos)).unwrap();

    wait_set.wait(Duration::new(5, 0)).unwrap();

    let matched_subscriptions = data_writer.get_matched_subscriptions().unwrap();
    let matched_subscription_data = data_writer
        .get_matched_subscription_data(matched_subscriptions[0])
        .unwrap();
    assert_eq!(matched_subscription_data.user_data(), &user_data_qos_policy);
}

#[test]
fn reader_discovers_writer_in_same_participant() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic("topic_name", "UserType", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let publisher = dp
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let _data_writer = publisher
        .create_datawriter::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let subscriber = dp
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_reader = subscriber
        .create_datareader::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let cond = data_reader.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    assert_eq!(data_reader.get_matched_publications().unwrap().len(), 1);
}

#[test]
fn deleted_writers_are_disposed_from_reader() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic("topic_name", "UserType", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let publisher = dp
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer = publisher
        .create_datawriter::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let subscriber = dp
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_reader = subscriber
        .create_datareader::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let cond = data_reader.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();
    data_reader.get_subscription_matched_status().unwrap();

    publisher.delete_datawriter(&data_writer).unwrap();

    wait_set.wait(Duration::new(5, 0)).unwrap();

    assert_eq!(data_reader.get_matched_publications().unwrap().len(), 0);
}

#[test]
fn updated_writers_are_announced_to_reader() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic("topic_name", "UserType", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let publisher = dp
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer = publisher
        .create_datawriter::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let subscriber = dp
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_reader = subscriber
        .create_datareader::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let cond = data_reader.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();
    data_reader.get_subscription_matched_status().unwrap();

    let user_data_qos_policy = UserDataQosPolicy {
        value: vec![1, 2, 3, 4],
    };
    let qos = DataWriterQos {
        user_data: user_data_qos_policy.clone(),
        ..Default::default()
    };
    data_writer.set_qos(QosKind::Specific(qos)).unwrap();

    wait_set.wait(Duration::new(5, 0)).unwrap();

    let matched_publications = data_reader.get_matched_publications().unwrap();
    let matched_publication_data = data_reader
        .get_matched_publication_data(matched_publications[0])
        .unwrap();

    assert_eq!(matched_publication_data.user_data(), &user_data_qos_policy);
}

#[test]
fn two_participants_should_get_subscription_matched() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();

    let dp1 = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic1 = dp1
        .create_topic("topic_name", "UserType", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let publisher = dp1
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer = publisher
        .create_datawriter::<UserType>(&topic1, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let dp2 = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic2 = dp2
        .create_topic("topic_name", "UserType", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let subscriber = dp2
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();

    let _data_reader = subscriber
        .create_datareader::<UserType>(&topic2, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let cond = data_writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    assert_eq!(data_writer.get_matched_subscriptions().unwrap().len(), 1);
}

#[test]
fn participant_records_discovered_topics() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();

    let participant1 = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let participant2 = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic_names = ["Topic 1", "Topic 2"];
    let mut topics = Vec::new();
    for name in topic_names {
        topics.push(
            participant1
                .create_topic(name, "UserType", QosKind::Default, None, NO_STATUS)
                .unwrap(),
        );
    }

    let mut found_topics = Vec::new();
    for name in topic_names {
        found_topics.push(participant2.find_topic(name, Duration::new(10, 0)).unwrap());
    }

    let discovered_topic_names: Vec<String> = participant2
        .get_discovered_topics()
        .unwrap()
        .iter()
        .map(|&handle| {
            participant2
                .get_discovered_topic_data(handle)
                .unwrap()
                .name()
                .to_string()
        })
        .collect();

    assert!(discovered_topic_names.contains(&"Topic 1".to_string()));
    assert!(discovered_topic_names.contains(&"Topic 2".to_string()));
}

#[test]
fn participant_announces_updated_qos() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();

    let participant1 = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let mut qos = participant1.get_qos().unwrap();
    qos.user_data.value = vec![1, 2, 3];

    std::thread::sleep(std::time::Duration::from_secs(1));
    participant1
        .set_qos(QosKind::Specific(qos.clone()))
        .unwrap();
    qos.user_data.value = vec![4, 5, 6];
    std::thread::sleep(std::time::Duration::from_secs(1));
    participant1
        .set_qos(QosKind::Specific(qos.clone()))
        .unwrap();
    qos.user_data.value = vec![7, 8, 9];
    std::thread::sleep(std::time::Duration::from_secs(1));
    participant1.set_qos(QosKind::Specific(qos)).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(5));
}

#[test]
fn reader_discovers_disposed_writer_same_participant() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic("topic_name", "UserType", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let publisher = dp
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer = publisher
        .create_datawriter::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let subscriber = dp
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_reader = subscriber
        .create_datareader::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let cond = data_reader.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();
    data_reader.get_subscription_matched_status().unwrap();

    publisher.delete_datawriter(&data_writer).unwrap();

    wait_set.wait(Duration::new(5, 0)).unwrap();

    assert_eq!(data_reader.get_matched_publications().unwrap().len(), 0);
}

#[test]
fn publisher_and_subscriber_different_partition_not_matched() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic("topic_name", "UserType", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let publisher_qos = PublisherQos {
        partition: PartitionQosPolicy {
            name: "A".to_string(),
        },
        ..Default::default()
    };
    let publisher = dp
        .create_publisher(QosKind::Specific(publisher_qos), None, NO_STATUS)
        .unwrap();
    let _data_writer = publisher
        .create_datawriter::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let subscriber_qos = SubscriberQos {
        partition: PartitionQosPolicy {
            name: "B".to_string(),
        },
        ..Default::default()
    };
    let subscriber = dp
        .create_subscriber(QosKind::Specific(subscriber_qos), None, NO_STATUS)
        .unwrap();
    let data_reader = subscriber
        .create_datareader::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let cond = data_reader.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();

    assert!(wait_set.wait(Duration::new(5, 0)).is_err());
}

#[test]
fn publisher_and_subscriber_regex_partition_is_matched() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic("topic_name", "UserType", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let publisher_qos = PublisherQos {
        partition: PartitionQosPolicy {
            name: "ABC".to_string(),
        },
        ..Default::default()
    };
    let publisher = dp
        .create_publisher(QosKind::Specific(publisher_qos), None, NO_STATUS)
        .unwrap();
    let data_writer = publisher
        .create_datawriter::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let cond_data_writer = data_writer.get_statuscondition().unwrap();
    cond_data_writer
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let subscriber_qos = SubscriberQos {
        partition: PartitionQosPolicy {
            name: "A[B-C]+".to_string(),
        },
        ..Default::default()
    };
    let subscriber = dp
        .create_subscriber(QosKind::Specific(subscriber_qos), None, NO_STATUS)
        .unwrap();
    let data_reader = subscriber
        .create_datareader::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let cond_data_reader = data_reader.get_statuscondition().unwrap();
    cond_data_reader
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();

    let mut wait_set_data_reader = WaitSet::new();
    wait_set_data_reader
        .attach_condition(Condition::StatusCondition(cond_data_reader))
        .unwrap();

    let mut wait_set_data_writer = WaitSet::new();
    wait_set_data_writer
        .attach_condition(Condition::StatusCondition(cond_data_writer))
        .unwrap();

    assert!(wait_set_data_reader.wait(Duration::new(10, 0)).is_ok());
    assert!(wait_set_data_writer.wait(Duration::new(10, 0)).is_ok());
}

#[test]
fn publisher_regex_and_subscriber_partition_is_matched() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic("topic_name", "UserType", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let publisher_qos = PublisherQos {
        partition: PartitionQosPolicy {
            name: "A[1-2]+".to_string(),
        },
        ..Default::default()
    };
    let publisher = dp
        .create_publisher(QosKind::Specific(publisher_qos), None, NO_STATUS)
        .unwrap();
    let data_writer = publisher
        .create_datawriter::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let cond_data_writer = data_writer.get_statuscondition().unwrap();
    cond_data_writer
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let subscriber_qos = SubscriberQos {
        partition: PartitionQosPolicy {
            name: "A12".to_string(),
        },
        ..Default::default()
    };
    let subscriber = dp
        .create_subscriber(QosKind::Specific(subscriber_qos), None, NO_STATUS)
        .unwrap();
    let data_reader = subscriber
        .create_datareader::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let cond_data_reader = data_reader.get_statuscondition().unwrap();
    cond_data_reader
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();

    let mut wait_set_data_reader = WaitSet::new();
    wait_set_data_reader
        .attach_condition(Condition::StatusCondition(cond_data_reader))
        .unwrap();

    let mut wait_set_data_writer = WaitSet::new();
    wait_set_data_writer
        .attach_condition(Condition::StatusCondition(cond_data_writer))
        .unwrap();

    assert!(wait_set_data_reader.wait(Duration::new(10, 0)).is_ok());
    assert!(wait_set_data_writer.wait(Duration::new(10, 0)).is_ok());
}

#[test]
fn publisher_regex_and_subscriber_regex_partition_is_matched() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic("topic_name", "UserType", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let publisher_qos = PublisherQos {
        partition: PartitionQosPolicy {
            name: "A[1-2]+".to_string(),
        },
        ..Default::default()
    };
    let publisher = dp
        .create_publisher(QosKind::Specific(publisher_qos), None, NO_STATUS)
        .unwrap();
    let data_writer = publisher
        .create_datawriter::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let cond_data_writer = data_writer.get_statuscondition().unwrap();
    cond_data_writer
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let subscriber_qos = SubscriberQos {
        partition: PartitionQosPolicy {
            name: "A[1-2]+".to_string(),
        },
        ..Default::default()
    };
    let subscriber = dp
        .create_subscriber(QosKind::Specific(subscriber_qos), None, NO_STATUS)
        .unwrap();
    let data_reader = subscriber
        .create_datareader::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let cond_data_reader = data_reader.get_statuscondition().unwrap();
    cond_data_reader
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();

    let mut wait_set_data_reader = WaitSet::new();
    wait_set_data_reader
        .attach_condition(Condition::StatusCondition(cond_data_reader))
        .unwrap();

    let mut wait_set_data_writer = WaitSet::new();
    wait_set_data_writer
        .attach_condition(Condition::StatusCondition(cond_data_writer))
        .unwrap();

    assert!(wait_set_data_reader.wait(Duration::new(5, 0)).is_ok());
    assert!(wait_set_data_writer.wait(Duration::new(5, 0)).is_ok());
}

#[test]
fn writer_matched_to_already_existing_reader_with_matched_writer() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic("topic_name", "UserType", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let publisher = dp
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer = publisher
        .create_datawriter::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let subscriber = dp
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let _data_reader = subscriber
        .create_datareader::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let cond = data_writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data_writer2 = publisher
        .create_datawriter::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let dw2_cond = data_writer2.get_statuscondition().unwrap();
    dw2_cond
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set2 = WaitSet::new();
    wait_set2
        .attach_condition(Condition::StatusCondition(dw2_cond))
        .unwrap();

    assert!(wait_set2.wait(Duration::new(5, 0)).is_ok());
}

#[test]
fn reader_matched_to_already_existing_writer_with_matched_reader() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic("topic_name", "UserType", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let publisher = dp
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer = publisher
        .create_datawriter::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let subscriber = dp
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let _data_reader = subscriber
        .create_datareader::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let cond = data_writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data_reader2 = subscriber
        .create_datareader::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let dr2_cond = data_reader2.get_statuscondition().unwrap();
    dr2_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut wait_set2 = WaitSet::new();
    wait_set2
        .attach_condition(Condition::StatusCondition(dr2_cond))
        .unwrap();

    assert!(wait_set2.wait(Duration::new(5, 0)).is_ok());
}

#[test]
#[ignore]
fn participant_removed_after_lease_duration() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();

    let participant1 = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let participant2 = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(5));

    domain_participant_factory
        .delete_participant(&participant2)
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(110));

    let discovered_participant = participant1.get_discovered_participants().unwrap();

    assert_eq!(discovered_participant.len(), 1);
}
