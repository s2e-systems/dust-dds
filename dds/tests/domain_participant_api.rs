use std::io::Write;

use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        error::{DdsError, DdsResult},
        qos::{PublisherQos, QosKind, SubscriberQos, TopicQos},
        qos_policy::{GroupDataQosPolicy, TopicDataQosPolicy},
        status::NO_STATUS,
    },
    topic_definition::type_support::{DdsDeserialize, DdsSerialize, DdsType, Endianness},
};

mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

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
        .create_topic::<TestType>("abc", QosKind::Default, None, NO_STATUS)
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
        .create_topic::<TestType>("abc", QosKind::Default, None, NO_STATUS)
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
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let writer_topic = participant
        .create_topic::<TestType>("Test", QosKind::Default, None, NO_STATUS)
        .expect("Error creating topic");
    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let _a_datawriter = publisher
        .create_datawriter(&writer_topic, QosKind::Default, None, NO_STATUS)
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
        .create_topic::<TestType>("Test", QosKind::Default, None, NO_STATUS)
        .expect("Error creating topic");
    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let _a_datareader = subscriber
        .create_datareader(&reader_topic, QosKind::Default, None, NO_STATUS)
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
        .create_topic::<TestType>("Test", QosKind::Default, None, NO_STATUS)
        .expect("Error creating topic");
    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let _a_datareader = subscriber
        .create_datareader(&reader_topic, QosKind::Default, None, NO_STATUS)
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
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let writer_topic = participant
        .create_topic::<TestType>("Test", QosKind::Default, None, NO_STATUS)
        .expect("Error creating topic");
    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let _a_datawriter = publisher
        .create_datawriter(&writer_topic, QosKind::Default, None, NO_STATUS)
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
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let writer_topic = participant
        .create_topic::<TestType>("Test", QosKind::Default, None, NO_STATUS)
        .expect("Error creating topic");
    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let a_datawriter = publisher
        .create_datawriter(&writer_topic, QosKind::Default, None, NO_STATUS)
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
        .create_topic::<TestType>("Test", QosKind::Default, None, NO_STATUS)
        .expect("Error creating topic");
    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let a_datareader = subscriber
        .create_datareader(&reader_topic, QosKind::Default, None, NO_STATUS)
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
        .create_topic::<TestType>("Test", QosKind::Default, None, NO_STATUS)
        .expect("Error creating topic");
    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let a_datawriter = publisher
        .create_datawriter(&writer_topic, QosKind::Default, None, NO_STATUS)
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
        .create_topic::<TestType>("Test", QosKind::Default, None, NO_STATUS)
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
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(0, QosKind::Default, None, NO_STATUS)
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
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(0, QosKind::Default, None, NO_STATUS)
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
        .create_topic::<TestType>("default_topic_qos", QosKind::Default, None, NO_STATUS)
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
