use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataReaderQos, QosKind},
        qos_policy::UserDataQosPolicy,
        status::NO_STATUS,
    },
    topic_definition::type_support::DdsType,
};

mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[derive(serde::Serialize, serde::Deserialize, DdsType)]
struct UserType(i32);

#[test]
fn get_subscriber_parent_participant() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();

    let subscriber_parent_participant = subscriber.get_participant().unwrap();

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
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic(
            "default_data_reader_qos",
            "UserType",
            QosKind::Default,
            None,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
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
        .create_datareader::<UserType>(&topic, QosKind::Default, None, NO_STATUS)
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
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic(
            "default_data_writer_qos",
            "UserType",
            QosKind::Default,
            None,
            NO_STATUS,
        )
        .unwrap();

    let subscriber1 = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();

    let subscriber2 = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();

    let reader1_1 = subscriber1
        .create_datareader::<UserType>(&topic, QosKind::Default, None, &[])
        .unwrap();
    let reader1_2 = subscriber1
        .create_datareader::<UserType>(&topic, QosKind::Default, None, &[])
        .unwrap();
    let reader2_1 = subscriber2
        .create_datareader::<UserType>(&topic, QosKind::Default, None, &[])
        .unwrap();
    let reader2_2 = subscriber2
        .create_datareader::<UserType>(&topic, QosKind::Default, None, &[])
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
fn data_reader_get_topicdescription() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic(
            "default_data_writer_qos",
            "UserType",
            QosKind::Default,
            None,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();

    let reader = subscriber
        .create_datareader::<UserType>(&topic, QosKind::Default, None, &[])
        .unwrap();

    assert!(reader.get_topicdescription().unwrap() == topic);
}
