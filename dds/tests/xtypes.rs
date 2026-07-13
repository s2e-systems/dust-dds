use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        listener::NO_LISTENER,
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{
            ReliabilityQosPolicy, ReliabilityQosPolicyKind, TypeConsistencyEnforcementQosPolicy,
            TypeConsistencyKind::AllowTypeCoercion,
        },
        status::{NO_STATUS, StatusKind},
        time::{Duration, DurationKind},
        type_support::DdsType,
    },
    wait_set::{Condition, WaitSet},
};

mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;
#[derive(DdsType, Debug, PartialEq, Clone)]
#[dust_dds(extensibility = "appendable")]
struct A1 {
    x1: i32,
}

#[derive(DdsType, Debug, PartialEq)]
#[dust_dds(extensibility = "appendable")]
struct A2 {
    x1: i32,
    x2: i32,
}

#[derive(DdsType, Debug, PartialEq)]
#[dust_dds(extensibility = "appendable")]
struct A3 {
    x1: i32,
    x3: i32,
    x2: i32,
}

#[test]
#[ignore = "not yet working"]
fn ext_appendable_struct_2() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant1 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic1 = participant1
        .create_topic::<A1>("A", "A", QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let publisher = participant1
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic1,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let participant2 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic2 = participant2
        .create_topic::<A2>("A", "A", QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber = participant2
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        type_consistency: TypeConsistencyEnforcementQosPolicy {
            kind: AllowTypeCoercion,
            ignore_sequence_bounds: true,
            ignore_string_bounds: true,
            ignore_member_names: false,
            prevent_type_widening: false,
            force_type_validation: false,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<A2>(
            &topic2,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data = A1 { x1: 1 };

    writer.write(data.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    assert_eq!(
        reader.read_next_sample().unwrap().data.as_ref().unwrap().x1,
        data.x1
    );
}

#[test]
#[ignore = "not yet working"]
fn ext_appendable_struct_4() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant1 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic1 = participant1
        .create_topic::<A2>("A", "A", QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let status_cond = topic1.get_statuscondition();
    status_cond
        .set_enabled_statuses(&[StatusKind::InconsistentTopic])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(status_cond))
        .unwrap();

    let participant2 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let _topic2 = participant2
        .create_topic::<A3>("A", "A", QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    wait_set.wait(Duration::new(10, 0)).unwrap();
}
