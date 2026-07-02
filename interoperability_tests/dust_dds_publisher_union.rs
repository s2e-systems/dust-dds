include!("target/idl/union.rs");

use crate::interoperability::test::{
    UnionType, UnionTypeWrapper, VariantEnum, VariantStructure, VariantUnion,
};
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
    xtypes::type_support::TypeSupport,
};

fn main() {
    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<UnionTypeWrapper>(
            "Union",
            UnionTypeWrapper::get_type().get_name(),
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

    let unions = [
        UnionType::None,
        UnionType::Boolean(true),
        UnionType::Byte(u8::MAX / 2),
        UnionType::Int16(i16::MAX / 2),
        UnionType::Int32(i32::MAX / 2),
        UnionType::Int64(i64::MAX / 2),
        UnionType::UInt16(u16::MAX / 2),
        UnionType::UInt32(u32::MAX / 2),
        UnionType::UInt64(u64::MAX / 2),
        UnionType::Float32(f32::MAX / 2.0),
        UnionType::Float64(f64::MAX / 2.0),
        UnionType::Int8(i8::MAX / 2),
        UnionType::UInt8(u8::MAX / 2),
        UnionType::Char8(b'M'),
        UnionType::String8(String::from("Hello World!")),
        UnionType::Enum(VariantEnum::Y),
        UnionType::Structure(VariantStructure {
            x: u8::MAX / 2,
            y: u16::MAX / 2,
            z: u32::MAX / 2,
        }),
        UnionType::Union(VariantUnion::new_random()),
        UnionType::Sequence(vec![
            String::from("Hello"),
            String::from("World"),
            String::from("!"),
        ]),
        UnionType::CaseUNION_DISCRIMINATOR_ARRAY([f64::MIN, std::f64::consts::PI, f64::MAX]),
    ];

    let data = UnionTypeWrapper {
        union_type: unions[random_value(unions.len())].clone(),
        union_array: unions.clone(),
        union_sequence: unions.to_vec(),
    };

    println!("write: {data:?}");
    writer.write(data, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(30, 0))
        .unwrap();
}

fn random_value(end: usize) -> usize {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
        time::SystemTime,
    };

    let mut hasher = DefaultHasher::new();
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos()
        .hash(&mut hasher);

    (hasher.finish() as usize) % end
}
