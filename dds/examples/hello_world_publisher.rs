use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        error::DdsResult,
        qos::{DataWriterQos, QosKind},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind,
        },
        status::{StatusKind, NO_STATUS},
        time::{Duration, DurationKind},
        wait_set::{Condition, WaitSet},
    },
    topic_definition::type_support::{
        DdsKeyDeserialize, DdsSerializedKey, RepresentationType, CDR_LE,
    },
    DdsType,
};

use serde::ser::SerializeStruct;

#[derive(serde::Serialize, serde::Deserialize, DdsType)]
pub struct NestedType {
    #[key]
    value: u32,
}

#[derive(serde::Serialize)]
pub struct NestedTypeKeyHolder_Serialize<'a> {
    value: &'a u32,
}

#[derive(serde::Deserialize)]
pub struct NestedTypeKeyHolder_Deserialize {
    value: u32,
}

impl<'a> From<&'a NestedType> for NestedTypeKeyHolder_Serialize<'a> {
    fn from(value: &'a NestedType) -> Self {
        Self {
            value: &value.value,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, DdsType, Debug)]
struct HelloWorldType {
    #[key]
    id: u8,
    msg: String,
}

#[derive(serde::Serialize)]
struct HelloWorldTypeKeyHolder_Serialize<'a> {
    id: &'a u8,
}

impl<'a> From<&'a HelloWorldType> for HelloWorldTypeKeyHolder_Serialize<'a> {
    fn from(value: &'a HelloWorldType) -> Self {
        Self { id: &value.id }
    }
}

#[derive(serde::Deserialize)]
struct HelloWorldTypeKeyHolder_Deserialize {
    id: u8,
}

impl DdsKeyDeserialize for HelloWorldType {
    type OwningKeyHolder = HelloWorldTypeKeyHolder_Deserialize;
}

pub trait DdsDeserializeSketch {
    fn set_key_fields_from_serialized_key(&mut self, key: DdsSerializedKey) -> DdsResult<()>;
}

// impl DdsDeserializeSketch for HelloWorldType {
//     fn set_key_fields_from_serialized_key(&mut self, key: DdsSerializedKey) -> DdsResult<()> {
//         let mut key = dds_deserialize_key::<HelloWorldType>(key.as_ref())?;
//         std::mem::swap(&mut self.id, &mut key.id);

//         Ok(())
//     }
// }

// fn get_serialized_key_from_data<T>(bytes: &[u8]) -> DdsResult<Vec<u8>>
// where
//     for<'a> T: serde::Deserialize<'a> + DdsKeySerialize<'a> + 'a,
//     for<'a> <T as DdsKeySerialize<'a>>::BorrowKeyHolder: From<&'a T> + serde::Serialize,
// {
//     dds_serialize_key(&dds_deserialize::<T>(bytes)?)
// }

pub trait DdsSerializeSketch {
    const REPRESENTATION_IDENTIFIER: RepresentationType;

    fn serialize_key<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer;
}

impl DdsSerializeSketch for HelloWorldType {
    const REPRESENTATION_IDENTIFIER: RepresentationType = CDR_LE;

    fn serialize_key<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("HelloWorldType", 2)?;
        s.serialize_field("id", &self.id)?;

        s.end()
    }
}

fn main() {
    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic(
            "HelloWorld",
            "HelloWorldType",
            QosKind::Default,
            None,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
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
        .create_datawriter(&topic, QosKind::Specific(writer_qos), None, NO_STATUS)
        .unwrap();
    let writer_cond = writer.get_statuscondition().unwrap();
    writer_cond
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(writer_cond))
        .unwrap();

    wait_set.wait(Duration::new(60, 0)).unwrap();

    let hello_world = HelloWorldType {
        id: 8,
        msg: "Hello world!".to_string(),
    };

    writer.write(&hello_world, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(30, 0))
        .unwrap();
}
