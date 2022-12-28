use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::UserDataQosPolicy,
        status::{StatusKind, NO_STATUS},
        time::Duration,
        wait_set::{Condition, WaitSet},
    },
    topic_definition::type_support::{DdsSerde, DdsType},
};

#[derive(serde::Serialize, serde::Deserialize, DdsType, DdsSerde)]
struct UserType(i32);

#[test]
fn writer_discovers_reader_in_same_participant() {
    let domain_id = 0;
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic::<UserType>("topic_name", QosKind::Default, None, NO_STATUS)
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
    let domain_id = 0;
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic::<UserType>("topic_name", QosKind::Default, None, NO_STATUS)
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
        .attach_condition(Condition::StatusCondition(cond.clone()))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    subscriber.delete_datareader(&data_reader).unwrap();

    wait_set.wait(Duration::new(5, 0)).unwrap();

    assert_eq!(data_writer.get_matched_subscriptions().unwrap().len(), 0);
}

#[test]
fn updated_readers_are_announced_to_writer() {
    let domain_id = 0;
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic::<UserType>("topic_name", QosKind::Default, None, NO_STATUS)
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
        .attach_condition(Condition::StatusCondition(cond.clone()))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

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
    assert_eq!(matched_subscription_data.user_data, user_data_qos_policy);
}

#[test]
fn reader_discovers_writer_in_same_participant() {
    let domain_id = 0;
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic::<UserType>("topic_name", QosKind::Default, None, NO_STATUS)
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
    wait_set.wait(Duration::new(5, 0)).unwrap();

    assert_eq!(data_reader.get_matched_publications().unwrap().len(), 1);
}

#[test]
fn deleted_writers_are_disposed_from_reader() {
    let domain_id = 0;
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic::<UserType>("topic_name", QosKind::Default, None, NO_STATUS)
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

    publisher.delete_datawriter(&data_writer).unwrap();

    wait_set.wait(Duration::new(5, 0)).unwrap();

    assert_eq!(data_reader.get_matched_publications().unwrap().len(), 0);
}

#[test]
fn updated_writers_are_announced_to_reader() {
    let domain_id = 0;
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic::<UserType>("topic_name", QosKind::Default, None, NO_STATUS)
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

    assert_eq!(matched_publication_data.user_data, user_data_qos_policy);
}

#[test]
fn participant_records_discovered_topics() {
    let domain_id = 0;

    let domain_participant_factory = DomainParticipantFactory::get_instance();

    let participant1 = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let participant2 = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic_names = ["Topic 1", "Topic 2", "Topic 3", "Topic 4", "Topic 5"];
    for name in topic_names {
        participant1
            .create_topic::<UserType>(name, QosKind::Default, None, NO_STATUS)
            .unwrap();
    }

    for name in topic_names {
        participant2
            .find_topic::<UserType>(name, Duration::new(2, 0))
            .unwrap();
    }

    let discovered_topic_names: Vec<String> = participant2
        .get_discovered_topics()
        .unwrap()
        .iter()
        .map(|&handle| participant2.get_discovered_topic_data(handle).unwrap().name)
        .collect();
    assert!(discovered_topic_names.contains(&"Topic 1".to_string()));
    assert!(discovered_topic_names.contains(&"Topic 2".to_string()));
    assert!(discovered_topic_names.contains(&"Topic 3".to_string()));
    assert!(discovered_topic_names.contains(&"Topic 4".to_string()));
    assert!(discovered_topic_names.contains(&"Topic 5".to_string()));
}

#[test]
fn participant_announces_updated_qos() {
    let domain_id = 0;

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
    participant1
        .set_qos(QosKind::Specific(qos.clone()))
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(5));
}




#[test]
fn reader_discovers_disposed_writer_same_participant() {
    let domain_id = 0;
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = dp
        .create_topic::<UserType>("topic_name", QosKind::Default, None, NO_STATUS)
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

    publisher.delete_datawriter(&data_writer).unwrap();

    wait_set.wait(Duration::new(5, 0)).unwrap();

    assert_eq!(data_reader.get_matched_publications().unwrap().len(), 0);
}