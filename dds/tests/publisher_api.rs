use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataWriterQos, QosKind},
        qos_policy::UserDataQosPolicy,
        status::NO_STATUS,
    },
    topic_definition::type_support::{DdsSerde, DdsType},
};

mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[derive(serde::Serialize, serde::Deserialize, DdsType, DdsSerde)]
struct UserType(i32);

#[test]
fn get_publisher_parent_participant() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher_parent_participant = publisher.get_participant().unwrap();

    assert_eq!(participant, publisher_parent_participant);
}

#[test]
fn default_data_writer_qos() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<UserType>("default_data_writer_qos", QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
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
        .create_datawriter(&topic, QosKind::Default, None, NO_STATUS)
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
