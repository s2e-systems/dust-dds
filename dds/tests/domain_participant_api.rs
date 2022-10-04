use std::io::Write;
use std::time::Instant;

use dust_dds::dds_type::{DdsDeserialize, DdsSerde, DdsSerialize, DdsType, Endianness};
use dust_dds::return_type::{DdsError, DdsResult};
use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory, infrastructure::entity::Entity,
};

struct TestType;

impl DdsType for TestType {
    fn type_name() -> &'static str {
        "TestType"
    }

    fn has_key() -> bool {
        false
    }
}

impl DdsSerialize for TestType {
    fn serialize<W: Write, E: Endianness>(&self, _writer: W) -> DdsResult<()> {
        todo!()
    }
}

impl<'de> DdsDeserialize<'de> for TestType {
    fn deserialize(_buf: &mut &'de [u8]) -> DdsResult<Self> {
        todo!()
    }
}

#[test]
fn create_delete_publisher() {
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();

    let publisher = participant.create_publisher(None, None, 0).unwrap();

    assert_eq!(participant.delete_publisher(&publisher), Ok(()));
    assert_eq!(publisher.get_qos(), Err(DdsError::AlreadyDeleted));
    assert_eq!(
        participant.delete_publisher(&publisher),
        Err(DdsError::AlreadyDeleted)
    );
}

#[test]
fn create_delete_subscriber() {
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();

    let subscriber = participant.create_subscriber(None, None, 0).unwrap();

    assert_eq!(participant.delete_subscriber(&subscriber), Ok(()));
    assert_eq!(subscriber.get_qos(), Err(DdsError::AlreadyDeleted));
    assert_eq!(
        participant.delete_subscriber(&subscriber),
        Err(DdsError::AlreadyDeleted)
    );
}

#[test]
fn create_delete_topic() {
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();

    let topic = participant
        .create_topic::<TestType>("abc", None, None, 0)
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
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();
    let other_participant = domain_participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();

    let publisher = participant.create_publisher(None, None, 0).unwrap();
    assert_eq!(
        other_participant.delete_publisher(&publisher),
        Err(DdsError::PreconditionNotMet(
            "Publisher can only be deleted from its parent participant".to_string()
        ))
    );
}

#[test]
fn not_allowed_to_delete_subscriber_from_different_participant() {
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();
    let other_participant = domain_participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();

    let subscriber = participant.create_subscriber(None, None, 0).unwrap();
    assert_eq!(
        other_participant.delete_subscriber(&subscriber),
        Err(DdsError::PreconditionNotMet(
            "Subscriber can only be deleted from its parent participant".to_string()
        ))
    );
}

#[test]
fn not_allowed_to_delete_topic_from_different_participant() {
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();
    let other_participant = domain_participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();

    let topic = participant
        .create_topic::<TestType>("abc", None, None, 0)
        .unwrap();
    assert_eq!(
        other_participant.delete_topic(&topic),
        Err(DdsError::PreconditionNotMet(
            "Topic can only be deleted from its parent publisher".to_string()
        ))
    );
}

#[test]
fn not_allowed_to_delete_publisher_with_writer() {
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();

    let writer_topic = participant
        .create_topic::<TestType>("Test", None, None, 0)
        .expect("Error creating topic");
    let publisher = participant.create_publisher(None, None, 0).unwrap();
    let _a_datawriter = publisher
        .create_datawriter(&writer_topic, None, None, 0)
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
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();

    let reader_topic = participant
        .create_topic::<TestType>("Test", None, None, 0)
        .expect("Error creating topic");
    let subscriber = participant.create_subscriber(None, None, 0).unwrap();
    let _a_datareader = subscriber
        .create_datareader(&reader_topic, None, None, 0)
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
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();

    let reader_topic = participant
        .create_topic::<TestType>("Test", None, None, 0)
        .expect("Error creating topic");
    let subscriber = participant.create_subscriber(None, None, 0).unwrap();
    let _a_datareader = subscriber
        .create_datareader(&reader_topic, None, None, 0)
        .unwrap();

    assert_eq!(
        participant.delete_topic(&reader_topic),
        Err(DdsError::PreconditionNotMet(
            "Topic still attached to some data reader or data writer".to_string()
        ))
    );
}

#[test]
fn not_allowed_to_delete_topic_attached_to_writer() {
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();

    let writer_topic = participant
        .create_topic::<TestType>("Test", None, None, 0)
        .expect("Error creating topic");
    let publisher = participant.create_publisher(None, None, 0).unwrap();
    let _a_datawriter = publisher
        .create_datawriter(&writer_topic, None, None, 0)
        .unwrap();

    assert_eq!(
        participant.delete_topic(&writer_topic),
        Err(DdsError::PreconditionNotMet(
            "Topic still attached to some data reader or data writer".to_string()
        ))
    );
}

#[test]
fn allowed_to_delete_publisher_with_created_and_deleted_writer() {
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();

    let writer_topic = participant
        .create_topic::<TestType>("Test", None, None, 0)
        .expect("Error creating topic");
    let publisher = participant.create_publisher(None, None, 0).unwrap();
    let a_datawriter = publisher
        .create_datawriter(&writer_topic, None, None, 0)
        .unwrap();
    publisher
        .delete_datawriter(&a_datawriter)
        .expect("Failed to delete datawriter");
    assert_eq!(participant.delete_publisher(&publisher), Ok(()));
}

#[test]
fn allowed_to_delete_subscriber_with_created_and_deleted_reader() {
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();

    let reader_topic = participant
        .create_topic::<TestType>("Test", None, None, 0)
        .expect("Error creating topic");
    let subscriber = participant.create_subscriber(None, None, 0).unwrap();
    let a_datareader = subscriber
        .create_datareader(&reader_topic, None, None, 0)
        .unwrap();
    subscriber
        .delete_datareader(&a_datareader)
        .expect("Failed to delete datareader");
    assert_eq!(participant.delete_subscriber(&subscriber), Ok(()));
}

#[test]
fn allowed_to_delete_topic_with_created_and_deleted_writer() {
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();

    let writer_topic = participant
        .create_topic::<TestType>("Test", None, None, 0)
        .expect("Error creating topic");
    let publisher = participant.create_publisher(None, None, 0).unwrap();
    let a_datawriter = publisher
        .create_datawriter(&writer_topic, None, None, 0)
        .unwrap();
    publisher
        .delete_datawriter(&a_datawriter)
        .expect("Failed to delete datawriter");
    assert_eq!(participant.delete_topic(&writer_topic), Ok(()));
}

#[test]
fn allowed_to_delete_topic_with_created_and_deleted_reader() {
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();

    let reader_topic = participant
        .create_topic::<TestType>("Test", None, None, 0)
        .expect("Error creating topic");
    let subscriber = participant.create_subscriber(None, None, 0).unwrap();
    let a_datareader = subscriber
        .create_datareader::<TestType>(&reader_topic, None, None, 0)
        .unwrap();
    subscriber
        .delete_datareader(&a_datareader)
        .expect("Failed to delete datareader");
    assert_eq!(participant.delete_topic(&reader_topic), Ok(()));
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UserData(u8);

impl DdsType for UserData {
    fn type_name() -> &'static str {
        "UserData"
    }
}

impl DdsSerde for UserData {}

#[test]
fn participant_records_discovered_topics() {
    let domain_participant_factory = DomainParticipantFactory::get_instance();

    let participant1 = domain_participant_factory
        .create_participant(8, None, None, 0)
        .unwrap();
    let participant2 = domain_participant_factory
        .create_participant(8, None, None, 0)
        .unwrap();

    let topic_names = ["Topic 1", "Topic 2", "Topic 3", "Topic 4", "Topic 5"];
    for name in topic_names {
        participant1
            .create_topic::<UserData>(name, None, None, 0)
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
