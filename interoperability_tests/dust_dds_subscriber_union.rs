include!("target/idl/union.rs");

use crate::interoperability::test::{
    UnionType, UnionTypeWrapper, VariantEnum, VariantStructure, VariantUnion,
};
use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        listener::NO_LISTENER,
        qos::{DataReaderQos, QosKind},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind,
        },
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        status::{NO_STATUS, StatusKind},
        time::{Duration, DurationKind},
    },
    wait_set::{Condition, WaitSet},
};

fn main() {
    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .find_topic::<UnionTypeWrapper>("Union", Duration::new(120, 0))
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<UnionTypeWrapper>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let reader_cond = reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(reader_cond.clone()))
        .unwrap();

    wait_set.wait(Duration::new(60, 0)).unwrap();

    reader_cond
        .set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    wait_set.wait(Duration::new(30, 0)).unwrap();

    let samples = reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    let data = samples[0].data.as_ref().unwrap();
    println!("read: {data:?}");
    assert_eq!(data.union_sequence.len(), 20);
    union_checker(&data.union);
    for r#union in &data.union_array {
        union_checker(r#union);
    }
    for r#union in &data.union_sequence {
        union_checker(r#union);
    }

    // Sleep to allow sending acknowledgements
    std::thread::sleep(std::time::Duration::from_secs(2));
}

fn union_checker(union_type: &UnionType) {
    match union_type {
        UnionType::None => (),
        UnionType::Boolean(boolean) => {
            assert!(boolean);
        }
        UnionType::Byte(byte) => {
            assert_eq!(*byte, u8::MAX / 2);
        }
        UnionType::Int16(int16) => {
            assert_eq!(*int16, i16::MAX / 2);
        }
        UnionType::Int32(int32) => {
            assert_eq!(*int32, i32::MAX / 2);
        }
        UnionType::Int64(int64) => {
            assert_eq!(*int64, i64::MAX / 2);
        }
        UnionType::UInt16(uint16) => {
            assert_eq!(*uint16, u16::MAX / 2);
        }
        UnionType::UInt32(uint32) => {
            assert_eq!(*uint32, u32::MAX / 2);
        }
        UnionType::UInt64(uint64) => {
            assert_eq!(*uint64, u64::MAX / 2);
        }
        UnionType::Float32(float32) => {
            assert_eq!(*float32, f32::MAX / 2.0);
        }
        UnionType::Float64(float64) => {
            assert_eq!(*float64, f64::MAX / 2.0);
        }
        UnionType::Int8(int8) => {
            assert_eq!(*int8, i8::MAX / 2);
        }
        UnionType::UInt8(uint8) => {
            assert_eq!(*uint8, u8::MAX / 2);
        }
        UnionType::Char8(r#char) => {
            assert_eq!(char::from(*r#char), 'M');
        }
        UnionType::String8(string) => {
            assert_eq!(string, "Hello World!");
        }
        UnionType::Enum(r#enum) => {
            assert_eq!(*r#enum, VariantEnum::Y);
        }
        UnionType::Structure(structure) => {
            assert_eq!(
                structure,
                &VariantStructure {
                    x: u8::MAX / 2,
                    y: u16::MAX / 2,
                    z: u32::MAX / 2
                }
            );
        }
        UnionType::Union(r#union) => match r#union {
            VariantUnion::X10(x) | VariantUnion::X20(x) | VariantUnion::X30(x) => {
                assert_eq!(*x, u8::MAX / 2)
            }
            VariantUnion::Y100(y) | VariantUnion::Y200(y) | VariantUnion::Y300(y) => {
                assert_eq!(*y, u16::MAX / 2)
            }
            VariantUnion::Z1000(z) | VariantUnion::Z2000(z) | VariantUnion::Z3000(z) => {
                assert_eq!(*z, u32::MAX / 2)
            }
        },
        UnionType::Sequence(sequence) => {
            assert_eq!(sequence.len(), 3);
            assert_eq!(sequence[0], "Hello");
            assert_eq!(sequence[1], "World");
            assert_eq!(sequence[2], "!");
        }
        UnionType::Array(array) => {
            assert_eq!(array[0], f64::MIN);
            assert_eq!(array[1], std::f64::consts::PI);
            assert_eq!(array[2], f64::MAX);
        }
    }
}
