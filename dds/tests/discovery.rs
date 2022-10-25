use std::time::Instant;

use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        entity::Entity,
        status::StatusKind,
        time::Duration,
        wait_set::{Condition, WaitSet},
    },
};

#[derive(serde::Serialize, serde::Deserialize)]
struct UserType(i32);

impl dust_dds::dds_type::DdsSerde for UserType {}
impl dust_dds::dds_type::DdsType for UserType {
    fn type_name() -> &'static str {
        "UserType"
    }
}

#[test]
fn writer_discovers_reader_in_same_participant() {
    let domain_id = 0;
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, None, None, &[])
        .unwrap();

    let topic = dp
        .create_topic::<UserType>("topic_name", None, None, &[])
        .unwrap();
    let publisher = dp.create_publisher(None, None, &[]).unwrap();
    let data_writer = publisher
        .create_datawriter::<UserType>(&topic, None, None, &[])
        .unwrap();
    let subscriber = dp.create_subscriber(None, None, &[]).unwrap();
    let _data_reader = subscriber
        .create_datareader::<UserType>(&topic, None, None, &[])
        .unwrap();
    let cond = data_writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatchedStatus])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    assert_eq!(data_writer.get_matched_subscriptions().unwrap().len(), 1);
}

#[test]
fn reader_discovers_writer_in_same_participant() {
    let domain_id = 6;
    let dp = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, None, None, &[])
        .unwrap();

    let topic = dp
        .create_topic::<UserType>("topic_name", None, None, &[])
        .unwrap();
    let publisher = dp.create_publisher(None, None, &[]).unwrap();
    let _data_writer = publisher
        .create_datawriter::<UserType>(&topic, None, None, &[])
        .unwrap();
    let subscriber = dp.create_subscriber(None, None, &[]).unwrap();
    let data_reader = subscriber
        .create_datareader::<UserType>(&topic, None, None, &[])
        .unwrap();
    let cond = data_reader.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatchedStatus])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    assert_eq!(data_reader.get_matched_publications().unwrap().len(), 1);
}

#[test]
fn participant_records_discovered_topics() {
    let domain_id = 7;

    let domain_participant_factory = DomainParticipantFactory::get_instance();

    let participant1 = domain_participant_factory
        .create_participant(domain_id, None, None, &[])
        .unwrap();
    let participant2 = domain_participant_factory
        .create_participant(domain_id, None, None, &[])
        .unwrap();

    let topic_names = ["Topic 1", "Topic 2", "Topic 3", "Topic 4", "Topic 5"];
    for name in topic_names {
        participant1
            .create_topic::<UserType>(name, None, None, &[])
            .unwrap();
    }

    // Wait for topics to be discovered
    let waiting_time = Instant::now();
    while participant2.get_discovered_topics().unwrap().len() < topic_names.len() {
        std::thread::sleep(std::time::Duration::from_millis(50));

        if waiting_time.elapsed() > std::time::Duration::from_secs(5) {
            panic!("Topic discovery is taking too long")
        }
    }

    let mut discovered_topic_names: Vec<String> = participant2
        .get_discovered_topics()
        .unwrap()
        .iter()
        .map(|&handle| participant2.get_discovered_topic_data(handle).unwrap().name)
        .collect();
    discovered_topic_names.sort();

    assert_eq!(topic_names, discovered_topic_names.as_slice());
}
