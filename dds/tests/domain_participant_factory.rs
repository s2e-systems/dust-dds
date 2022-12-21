use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        error::DdsError,
        instance::HANDLE_NIL,
        qos::{DataReaderQos, DataWriterQos, DomainParticipantFactoryQos, QosKind},
        qos_policy::{EntityFactoryQosPolicy, ReliabilityQosPolicy, ReliabilityQosPolicyKind},
        status::{StatusKind, NO_STATUS},
        time::Duration,
        wait_set::{Condition, WaitSet},
    },
    subscription::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    topic_definition::type_support::{DdsSerde, DdsType},
};

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, DdsType, DdsSerde)]
struct KeyedData {
    #[key]
    id: u8,
    value: u8,
}

#[test]
fn create_not_enabled_entities() {
    let domain_id = 0;
    let domain_participant_factory = DomainParticipantFactory::get_instance();

    let qos = DomainParticipantFactoryQos {
        entity_factory: EntityFactoryQosPolicy {
            autoenable_created_entities: false,
        },
    };

    domain_participant_factory
        .set_qos(QosKind::Specific(qos))
        .unwrap();

    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    // Call an operation that should return a NotEnabled error as a check the QoS is taken
    let result = participant.ignore_topic(HANDLE_NIL);

    // Teardown before assert: Set qos back to original to prevent it affecting other test
    domain_participant_factory
        .set_qos(QosKind::Default)
        .unwrap();

    assert_eq!(result, Err(DdsError::NotEnabled));
}

#[test]
fn all_objects_are_dropped() {
    let domain_id = 0;
    let domain_participant_factory = DomainParticipantFactory::get_instance();

    {
        let participant = domain_participant_factory
            .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
            .unwrap();

        let topic = participant
            .create_topic::<KeyedData>("MyTopic", QosKind::Default, None, NO_STATUS)
            .unwrap();

        let publisher = participant
            .create_publisher(QosKind::Default, None, NO_STATUS)
            .unwrap();
        let writer_qos = DataWriterQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::Reliable,
                max_blocking_time: Duration::new(1, 0),
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
                max_blocking_time: Duration::new(1, 0),
            },
            ..Default::default()
        };
        let reader = subscriber
            .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
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
            .wait_for_acknowledgments(Duration::new(1, 0))
            .unwrap();

        let _samples = reader
            .read(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .unwrap();
    }

    assert!(domain_participant_factory
        .lookup_participant(domain_id)
        .is_none());
}
