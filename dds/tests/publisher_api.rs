use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        listeners::NoOpListener,
        qos::{DataWriterQos, QosKind},
        qos_policy::UserDataQosPolicy,
        status::NO_STATUS,
    },
    topic_definition::type_support::DdsType,
};

mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[derive(DdsType)]
struct UserType(i32);

#[test]
fn get_publisher_parent_participant() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();

    let publisher_parent_participant = publisher.get_participant().unwrap();

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
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<UserType>(
            "default_data_writer_qos",
            "UserType",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datawriter::<UserType>(&topic, QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<UserType>(
            "default_data_writer_qos",
            "UserType",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher1 = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();

    let publisher2 = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();

    let writer1_1 = publisher1
        .create_datawriter::<UserType>(&topic, QosKind::Default, NoOpListener::new(), &[])
        .unwrap();
    let writer1_2 = publisher1
        .create_datawriter::<UserType>(&topic, QosKind::Default, NoOpListener::new(), &[])
        .unwrap();
    let writer2_1 = publisher2
        .create_datawriter::<UserType>(&topic, QosKind::Default, NoOpListener::new(), &[])
        .unwrap();
    let writer2_2 = publisher2
        .create_datawriter::<UserType>(&topic, QosKind::Default, NoOpListener::new(), &[])
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
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<UserType>(
            "default_data_writer_qos",
            "UserType",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();

    let writer = publisher
        .create_datawriter::<UserType>(&topic, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();

    assert!(
        writer.get_topic().unwrap().get_instance_handle().unwrap()
            == topic.get_instance_handle().unwrap()
    );
}
