include!("target/idl/optional.rs");

use self::interoperability::test::Optional;
use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        listener::NO_LISTENER,
        qos::{DataWriterQos, QosKind},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind,
        },
        status::{NO_STATUS, StatusKind},
        time::{Duration, DurationKind},
    },
    wait_set::{Condition, WaitSet},
    xtypes::type_support::Type,
};

fn main() {
    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<Optional>(
            "Optional",
            Optional::TYPE.get_name(),
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let writer_cond = writer.get_statuscondition();
    writer_cond
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(writer_cond))
        .unwrap();

    wait_set.wait(Duration::new(60, 0)).unwrap();

    let data = Optional {
        maybe_string: Some("Hello World!".to_string()),
        maybe_uint8: None,
        maybe_double: Some(12345.6789),
        maybe_array: None,
        maybe_sequence: Some(vec![f32::MIN, 0.0, f32::MAX]),
    };
    println!("write: {data:?}");
    writer.write(data, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(30, 0))
        .unwrap();
}
