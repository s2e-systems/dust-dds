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

// TODO: remove when dust_dds_gen adds support for union
pub mod interoperability {
    pub mod test {
        use dust_dds::infrastructure::type_support::DdsType;

        #[derive(DdsType, Debug, Clone, PartialEq)]
        #[dust_dds(name = "interoperability::test::UnionTypeWrapper")]
        pub struct UnionTypeWrapper {
            pub r#union: UnionType,
            pub union_array: [UnionType; 20],
            pub union_sequence: Vec<UnionType>,
        }

        #[derive(DdsType, Debug, Clone, PartialEq)]
        #[dust_dds(name = "interoperability::test::UnionType", discriminator = UnionDescriptor)]
        pub enum UnionType {
            #[dust_dds(discriminator = UnionDescriptor::None)]
            None,
            #[dust_dds(discriminator = UnionDescriptor::Boolean)]
            Boolean(bool),
            #[dust_dds(discriminator = UnionDescriptor::Byte)]
            Byte(u8),
            #[dust_dds(discriminator = UnionDescriptor::Int16)]
            Int16(i16),
            #[dust_dds(discriminator = UnionDescriptor::Int32)]
            Int32(i32),
            #[dust_dds(discriminator = UnionDescriptor::Int64)]
            Int64(i64),
            #[dust_dds(discriminator = UnionDescriptor::UInt16)]
            UInt16(u16),
            #[dust_dds(discriminator = UnionDescriptor::UInt32)]
            UInt32(u32),
            #[dust_dds(discriminator = UnionDescriptor::UInt64)]
            UInt64(u64),
            #[dust_dds(discriminator = UnionDescriptor::Float32)]
            Float32(f32),
            #[dust_dds(discriminator = UnionDescriptor::Float64)]
            Float64(f64),
            #[dust_dds(discriminator = UnionDescriptor::Int8)]
            Int8(i8),
            #[dust_dds(discriminator = UnionDescriptor::UInt8)]
            UInt8(u8),
            #[dust_dds(discriminator = UnionDescriptor::Char8)]
            Char8(u8),
            #[dust_dds(discriminator = UnionDescriptor::String8)]
            String8(String),
            #[dust_dds(discriminator = UnionDescriptor::Enum)]
            Enum(VariantEnum),
            #[dust_dds(discriminator = UnionDescriptor::Structure)]
            Structure(VariantStructure),
            #[dust_dds(discriminator = UnionDescriptor::Union)]
            Union(VariantUnion),
            #[dust_dds(discriminator = UnionDescriptor::Sequence)]
            Sequence(Vec<String>),
            #[dust_dds(discriminator = UnionDescriptor::Array)]
            Array([f64; 3]),
        }

        #[derive(DdsType, Debug, Clone, PartialEq, Eq)]
        #[dust_dds(name = "interoperability::test::UnionDiscriminator")]
        pub enum UnionDescriptor {
            None = 0,
            Boolean = 1,
            Byte = 2,
            Int16 = 4,
            Int32 = 8,
            Int64 = 16,
            UInt16 = 32,
            UInt32 = 64,
            UInt64 = 128,
            Float32 = 256,
            Float64 = 512,
            // Float128 = 1024,
            Int8 = 2048,
            UInt8 = 4096,
            Char8 = 8192,
            // Char16 = 16_384,
            String8 = 32_768,
            // String16 = 65_536,
            // Alias = 131_072,
            Enum = 262_144,
            // Bitmask = 524_288,
            // Annotation = 1_048_576,
            Structure = 2_097_152,
            Union = 4_194_304,
            // Bitset = 8_388_608,
            Sequence = 16_777_216,
            Array = 33_554_432,
            // Map = 67_108_864
        }

        #[derive(DdsType, Debug, Clone, PartialEq, Eq)]
        #[dust_dds(name = "interoperability::test::VariantEnum")]
        pub enum VariantEnum {
            X,
            Y,
            Z,
        }

        #[derive(DdsType, Debug, Clone, PartialEq, Eq)]
        #[dust_dds(name = "interoperability::test::VariantStructure")]
        pub struct VariantStructure {
            pub x: u8,
            pub y: u16,
            pub z: u32,
        }

        #[derive(DdsType, Debug, Clone, PartialEq)]
        #[dust_dds(name = "interoperability::test::VariantUnion", discriminator = u16)]
        pub enum VariantUnion {
            #[dust_dds(discriminator = [10, 20, 30])]
            X(u8),
            #[dust_dds(discriminator = [100, 200, 300])]
            Y(u16),
            #[dust_dds(discriminator = [1000, 2000, 3000])]
            Z(u32),
        }
    }
}

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
            VariantUnion::X(x) => assert_eq!(*x, u8::MAX / 2),
            VariantUnion::Y(y) => assert_eq!(*y, u16::MAX / 2),
            VariantUnion::Z(z) => assert_eq!(*z, u32::MAX / 2),
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
