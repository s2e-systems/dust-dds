use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataReaderQos, DataWriterQos, DomainParticipantQos, QosKind},
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind, UserDataQosPolicy},
        status::{StatusKind, NO_STATUS},
        time::{Duration, DurationKind},
        wait_set::{Condition, WaitSet},
    },
    subscription::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    topic_definition::type_support::{DdsGetKey, DdsRepresentation, DdsType},
};

mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, DdsType, DdsGetKey, DdsRepresentation,
)]
struct KeyedData {
    #[key]
    id: u8,
    value: u8,
}

#[test]
fn default_participant_qos() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();

    let user_data = vec![1, 2, 3];
    let qos = DomainParticipantQos {
        user_data: UserDataQosPolicy {
            value: user_data.clone(),
        },
        ..Default::default()
    };

    domain_participant_factory
        .set_default_participant_qos(QosKind::Specific(qos))
        .unwrap();

    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    domain_participant_factory
        .set_default_participant_qos(QosKind::Default)
        .unwrap();

    assert_eq!(participant.get_qos().unwrap().user_data.value, user_data);
}

#[test]
fn create_delete_participant() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    assert!(domain_participant_factory
        .delete_participant(&participant)
        .is_ok())
}

#[test]
fn not_allowed_to_delete_participant_with_entities() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic("Test", "KeyedData", QosKind::Default, None, NO_STATUS)
        .expect("Error creating topic");
    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let _datareader = subscriber
        .create_datareader::<KeyedData>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let _datawriter = publisher
        .create_datawriter::<KeyedData>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();

    assert!(domain_participant_factory
        .delete_participant(&participant)
        .is_err());
}

#[test]
fn allowed_to_delete_participant_after_delete_contained_entities() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic("Test", "KeyedData", QosKind::Default, None, NO_STATUS)
        .expect("Error creating topic");
    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let _datareader = subscriber
        .create_datareader::<KeyedData>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let _datawriter = publisher
        .create_datawriter::<KeyedData>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();

    participant.delete_contained_entities().unwrap();

    assert!(domain_participant_factory
        .delete_participant(&participant)
        .is_ok());
}

#[test]
#[ignore = "Not yet fixed with actors"]
fn all_objects_are_dropped() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();

    {
        let participant = domain_participant_factory
            .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
            .unwrap();

        let topic = participant
            .create_topic("MyTopic", "KeyedData", QosKind::Default, None, NO_STATUS)
            .unwrap();

        let publisher = participant
            .create_publisher(QosKind::Default, None, NO_STATUS)
            .unwrap();
        let writer_qos = DataWriterQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::Reliable,
                max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
            },
            ..Default::default()
        };
        let writer = publisher
            .create_datawriter(&topic, QosKind::Specific(writer_qos), None, NO_STATUS)
            .unwrap();

        let subscriber = participant
            .create_subscriber(QosKind::Default, None, NO_STATUS)
            .unwrap();
        let reader_qos = DataReaderQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::Reliable,
                max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
            },
            ..Default::default()
        };
        let reader = subscriber
            .create_datareader::<KeyedData>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
            .unwrap();

        let cond = writer.get_statuscondition().unwrap();
        cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
            .unwrap();

        let mut wait_set = WaitSet::new();
        wait_set
            .attach_condition(Condition::StatusCondition(cond))
            .unwrap();
        wait_set.wait(Duration::new(5, 0)).unwrap();

        let data1 = KeyedData { id: 1, value: 1 };
        let data2 = KeyedData { id: 2, value: 10 };
        let data3 = KeyedData { id: 3, value: 20 };

        writer.write(&data1, None).unwrap();
        writer.write(&data2, None).unwrap();
        writer.write(&data3, None).unwrap();

        writer
            .wait_for_acknowledgments(Duration::new(10, 0))
            .unwrap();

        let _samples = reader
            .read(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .unwrap();
    }

    assert!(domain_participant_factory
        .lookup_participant(domain_id)
        .unwrap()
        .is_none());
}

#[test]
#[ignore = "Not yet fixed with actors"]
fn objects_are_correctly_dropped() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let topic_name = "MyTopic";
    {
        let participant = domain_participant_factory
            .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
            .unwrap();
        {
            let topic = participant
                .create_topic(topic_name, "KeyedData", QosKind::Default, None, NO_STATUS)
                .unwrap();
            {
                let publisher = participant
                    .create_publisher(QosKind::Default, None, NO_STATUS)
                    .unwrap();
                {
                    let _writer = publisher
                        .create_datawriter::<KeyedData>(&topic, QosKind::Default, None, NO_STATUS)
                        .unwrap();
                    {
                        let subscriber = participant
                            .create_subscriber(QosKind::Default, None, NO_STATUS)
                            .unwrap();
                        {
                            let _reader = subscriber
                                .create_datareader::<KeyedData>(
                                    &topic,
                                    QosKind::Default,
                                    None,
                                    NO_STATUS,
                                )
                                .unwrap();

                            assert!(subscriber
                                .lookup_datareader::<KeyedData>(topic_name)
                                .unwrap()
                                .is_some(),);
                        }
                        assert!(subscriber
                            .lookup_datareader::<KeyedData>(topic_name)
                            .unwrap()
                            .is_none(),);
                    }

                    assert!(publisher
                        .lookup_datawriter::<KeyedData>(topic_name)
                        .unwrap()
                        .is_some(),);
                }
                assert!(publisher
                    .lookup_datawriter::<KeyedData>(topic_name)
                    .unwrap()
                    .is_none(),);
            }
            assert!(participant
                .lookup_topicdescription(topic_name)
                .unwrap()
                .is_some());
        }
        assert!(participant
            .lookup_topicdescription(topic_name)
            .unwrap()
            .is_none());
        assert!(domain_participant_factory
            .lookup_participant(domain_id)
            .unwrap()
            .is_some());
    }
    assert!(domain_participant_factory
        .lookup_participant(domain_id)
        .unwrap()
        .is_none());
}
