include!("target/idl/inheritance.rs");

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
        .create_topic::<interoperability::test::Cat>(
            "Inheritance",
            interoperability::test::Cat::TYPE.get_name(),
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

    let data = interoperability::test::Cat {
        parent: interoperability::test::Animal {
            id: 1,
            name: "Zoe".to_string(),
            age: 1,
        },
        lives: 7,
    };
    println!("write: {data:?}");
    writer.write(data, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(30, 0))
        .unwrap();

    let data_to_dispose = interoperability::test::Cat {
        parent: interoperability::test::Animal {
            id: 1,
            name: String::new(),
            age: 0,
        },
        lives: 0,
    };
    println!("dispose: {data_to_dispose:?}");
    writer.dispose(data_to_dispose, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(30, 0))
        .unwrap();
}
