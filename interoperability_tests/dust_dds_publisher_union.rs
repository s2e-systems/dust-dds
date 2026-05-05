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
        type_support::TypeSupport,
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

        impl VariantUnion {
            pub fn new_random() -> Self {
                match crate::random_value(3) {
                    0 => Self::X(u8::MAX / 2),
                    1 => Self::Y(u16::MAX / 2),
                    2 => Self::Z(u32::MAX / 2),
                    _ => unreachable!(),
                }
            }
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
        .create_topic::<UnionTypeWrapper>(
            "Union",
            UnionTypeWrapper::TYPE_NAME,
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
        UnionType::Array([f64::MIN, std::f64::consts::PI, f64::MAX]),
    ];

    let data = UnionTypeWrapper {
        union: unions[random_value(unions.len())].clone(),
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
