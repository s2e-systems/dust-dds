use std::io::Write;

use rust_dds::{
    domain::domain_participant::DomainParticipant,
    domain_participant_factory::DomainParticipantFactory, infrastructure::entity::Entity,
};
use rust_dds_rtps_implementation::dds_type::{DdsDeserialize, DdsSerialize, DdsType, Endianness};

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
    fn serialize<W: Write, E: Endianness>(&self, _writer: W) -> rust_dds::DDSResult<()> {
        todo!()
    }
}

impl<'de> DdsDeserialize<'de> for TestType {
    fn deserialize(_buf: &mut &'de [u8]) -> rust_dds::DDSResult<Self> {
        todo!()
    }
}

#[test]
fn create_delete_publisher() {
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();
    participant.enable().unwrap();
    let _my_topic = participant
        .create_topic::<TestType>("my_topic", None, None, 0)
        .unwrap();
    let _publisher = participant.create_publisher(None, None, 0).unwrap();
    // publisher
    // .create_datawriter(&my_topic, None, None, 0)
    // .unwrap();
    // std::thread::sleep(std::time::Duration::from_secs(2));

    // assert_eq!(participant.delete_publisher(&publisher), Ok(()));
    // assert_eq!(publisher.get_qos(), Err(DDSError::AlreadyDeleted));
    // assert_eq!(
    //     participant.delete_publisher(&publisher),
    //     Err(DDSError::AlreadyDeleted)
    // );
}

// #[test]
// fn create_delete_subscriber() {
//     let participant = DomainParticipantFactory::create_participant(0, None, None, 0).unwrap();
//     let subscriber = participant.create_subscriber(None, None, 0).unwrap();
//     assert_eq!(participant.delete_subscriber(&subscriber), Ok(()));
//     assert_eq!(subscriber.get_qos(), Err(DDSError::AlreadyDeleted));
//     assert_eq!(
//         participant.delete_subscriber(&subscriber),
//         Err(DDSError::AlreadyDeleted)
//     );
// }

// #[test]
// fn create_delete_topic() {
//     let participant = DomainParticipantFactory::create_participant(0, None, None, 0).unwrap();
//     let topic = participant
//         .create_topic::<TestType>("abc", None, None, 0)
//         .unwrap();

//     assert_eq!(participant.delete_topic(&topic), Ok(()));
//     assert_eq!(topic.get_qos(), Err(DDSError::AlreadyDeleted));
//     assert_eq!(
//         participant.delete_topic(&topic),
//         Err(DDSError::AlreadyDeleted)
//     );
// }

// #[test]
// fn not_allowed_to_delete_publisher_from_different_participant() {
//     let participant = DomainParticipantFactory::create_participant(0, None, None, 0).unwrap();
//     let other_participant = DomainParticipantFactory::create_participant(1, None, None, 0).unwrap();
//     let publisher = participant.create_publisher(None, None, 0).unwrap();
//     assert_eq!(
//         other_participant.delete_publisher(&publisher),
//         Err(DDSError::PreconditionNotMet(
//             "Publisher can only be deleted from its parent participant".to_string()
//         ))
//     );
// }

// #[test]
// fn not_allowed_to_delete_subscriber_from_different_participant() {
//     let participant = DomainParticipantFactory::create_participant(0, None, None, 0).unwrap();
//     let other_participant = DomainParticipantFactory::create_participant(1, None, None, 0).unwrap();
//     let subscriber = participant.create_subscriber(None, None, 0).unwrap();
//     assert_eq!(
//         other_participant.delete_subscriber(&subscriber),
//         Err(DDSError::PreconditionNotMet(
//             "Subscriber can only be deleted from its parent participant".to_string()
//         ))
//     );
// }

// #[test]
// fn not_allowed_to_delete_topic_from_different_participant() {
//     let participant = DomainParticipantFactory::create_participant(0, None, None, 0).unwrap();
//     let other_participant = DomainParticipantFactory::create_participant(1, None, None, 0).unwrap();
//     let topic = participant
//         .create_topic::<TestType>("abc", None, None, 0)
//         .unwrap();
//     assert_eq!(
//         other_participant.delete_topic(&topic),
//         Err(DDSError::PreconditionNotMet(
//             "Topic can only be deleted from its parent participant".to_string()
//         ))
//     );
// }

// #[test]
// fn not_allowed_to_delete_publisher_with_writer() {
//     let participant = DomainParticipantFactory::create_participant(0, None, None, 0).unwrap();
//     let writer_topic = participant
//         .create_topic::<TestType>("Test", None, None, 0)
//         .expect("Error creating topic");
//     let publisher = participant.create_publisher(None, None, 0).unwrap();
//     let _a_datawriter = publisher
//         .create_datawriter(&writer_topic, None, None, 0)
//         .unwrap();

//     assert_eq!(
//         participant.delete_publisher(&publisher),
//         Err(DDSError::PreconditionNotMet(
//             "Publisher still contains data writers".to_string()
//         ))
//     );
// }

// #[test]
// fn not_allowed_to_delete_subscriber_with_reader() {
//     let participant = DomainParticipantFactory::create_participant(0, None, None, 0).unwrap();
//     let reader_topic = participant
//         .create_topic::<TestType>("Test", None, None, 0)
//         .expect("Error creating topic");
//     let subscriber = participant.create_subscriber(None, None, 0).unwrap();
//     let _a_datareader = subscriber
//         .create_datareader(&reader_topic, None, None, 0)
//         .unwrap();

//     assert_eq!(
//         participant.delete_subscriber(&subscriber),
//         Err(DDSError::PreconditionNotMet(
//             "Subscriber still contains data readers".to_string()
//         ))
//     );
// }

// #[test]
// fn not_allowed_to_delete_topic_attached_to_reader() {
//     let participant = DomainParticipantFactory::create_participant(0, None, None, 0).unwrap();
//     let reader_topic = participant
//         .create_topic::<TestType>("Test", None, None, 0)
//         .expect("Error creating topic");
//     let subscriber = participant.create_subscriber(None, None, 0).unwrap();
//     let _a_datareader = subscriber
//         .create_datareader(&reader_topic, None, None, 0)
//         .unwrap();

//     assert_eq!(
//         participant.delete_topic(&reader_topic),
//         Err(DDSError::PreconditionNotMet(
//             "Topic still attached to some data reader or data writer".to_string()
//         ))
//     );
// }

// #[test]
// fn not_allowed_to_delete_topic_attached_to_writer() {
//     let participant = DomainParticipantFactory::create_participant(0, None, None, 0).unwrap();
//     let writer_topic = participant
//         .create_topic::<TestType>("Test", None, None, 0)
//         .expect("Error creating topic");
//     let publisher = participant.create_publisher(None, None, 0).unwrap();
//     let _a_datawriter = publisher
//         .create_datawriter(&writer_topic, None, None, 0)
//         .unwrap();

//     assert_eq!(
//         participant.delete_topic(&writer_topic),
//         Err(DDSError::PreconditionNotMet(
//             "Topic still attached to some data reader or data writer".to_string()
//         ))
//     );
// }

// #[test]
// fn allowed_to_delete_publisher_with_created_and_deleted_writer() {
//     let participant = DomainParticipantFactory::create_participant(0, None, None, 0).unwrap();
//     let writer_topic = participant
//         .create_topic::<TestType>("Test", None, None, 0)
//         .expect("Error creating topic");
//     let publisher = participant.create_publisher(None, None, 0).unwrap();
//     let a_datawriter = publisher
//         .create_datawriter(&writer_topic, None, None, 0)
//         .unwrap();
//     publisher
//         .delete_datawriter(&a_datawriter)
//         .expect("Failed to delete datawriter");
//     assert_eq!(participant.delete_publisher(&publisher), Ok(()));
// }

// #[test]
// fn allowed_to_delete_subscriber_with_created_and_deleted_reader() {
//     let participant = DomainParticipantFactory::create_participant(0, None, None, 0).unwrap();
//     let reader_topic = participant
//         .create_topic::<TestType>("Test", None, None, 0)
//         .expect("Error creating topic");
//     let subscriber = participant.create_subscriber(None, None, 0).unwrap();
//     let a_datareader = subscriber
//         .create_datareader(&reader_topic, None, None, 0)
//         .unwrap();
//     subscriber
//         .delete_datareader(&a_datareader)
//         .expect("Failed to delete datareader");
//     assert_eq!(participant.delete_subscriber(&subscriber), Ok(()));
// }

// #[test]
// fn allowed_to_delete_topic_with_created_and_deleted_writer() {
//     let participant = DomainParticipantFactory::create_participant(0, None, None, 0).unwrap();
//     let writer_topic = participant
//         .create_topic::<TestType>("Test", None, None, 0)
//         .expect("Error creating topic");
//     let publisher = participant.create_publisher(None, None, 0).unwrap();
//     let a_datawriter = publisher
//         .create_datawriter(&writer_topic, None, None, 0)
//         .unwrap();
//     publisher
//         .delete_datawriter(&a_datawriter)
//         .expect("Failed to delete datawriter");
//     assert_eq!(participant.delete_topic(&writer_topic), Ok(()));
// }

// #[test]
// fn allowed_to_delete_topic_with_created_and_deleted_reader() {
//     let participant = DomainParticipantFactory::create_participant(0, None, None, 0).unwrap();
//     let reader_topic = participant
//         .create_topic::<TestType>("Test", None, None, 0)
//         .expect("Error creating topic");
//     let subscriber = participant.create_subscriber(None, None, 0).unwrap();
//     let a_datareader = subscriber
//         .create_datareader::<TestType>(&reader_topic, None, None, 0)
//         .unwrap();
//     subscriber
//         .delete_datareader(&a_datareader)
//         .expect("Failed to delete datareader");
//     assert_eq!(participant.delete_topic(&reader_topic), Ok(()));
// }
