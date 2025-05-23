use super::{
    parameter_id_values::{
        PID_DATA_REPRESENTATION, PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY,
        PID_ENDPOINT_GUID, PID_GROUP_DATA, PID_GROUP_ENTITYID, PID_LATENCY_BUDGET, PID_LIFESPAN,
        PID_LIVELINESS, PID_MULTICAST_LOCATOR, PID_OWNERSHIP, PID_OWNERSHIP_STRENGTH,
        PID_PARTICIPANT_GUID, PID_PARTITION, PID_PRESENTATION, PID_RELIABILITY, PID_TOPIC_DATA,
        PID_TOPIC_NAME, PID_TYPE_NAME, PID_UNICAST_LOCATOR, PID_USER_DATA,
    },
    payload_serializer_deserializer::{
        parameter_list_deserializer::ParameterListCdrDeserializer,
        parameter_list_serializer::ParameterListCdrSerializer,
    },
};
use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    infrastructure::{
        error::DdsResult,
        qos_policy::DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
        type_support::{DdsDeserialize, DdsSerialize, TypeSupport},
    },
    transport::types::{EntityId, Guid, Locator},
};
use alloc::{boxed::Box,string::ToString, vec, vec::Vec};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WriterProxy {
    pub remote_writer_guid: Guid,
    pub remote_group_entity_id: EntityId,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DiscoveredWriterData {
    pub(crate) dds_publication_data: PublicationBuiltinTopicData,
    pub(crate) writer_proxy: WriterProxy,
}
impl TypeSupport for DiscoveredWriterData {
    fn get_type_name() -> &'static str {
        "DiscoveredWriterData"
    }

    fn get_type() -> impl crate::xtypes::dynamic_type::DynamicType {
        dust_dds::xtypes::type_object::CompleteTypeObject::TkStructure {
            struct_type: dust_dds::xtypes::type_object::CompleteStructType {
                struct_flags: dust_dds::xtypes::type_object::StructTypeFlag {
                    is_final: false,
                    is_appendable: false,
                    is_mutable: true,
                    is_nested: false,
                    is_autoid_hash: false,
                },
                header: dust_dds::xtypes::type_object::CompleteStructHeader {
                    base_type: dust_dds::xtypes::type_object::TypeIdentifier::TkNone,
                    detail: dust_dds::xtypes::type_object::CompleteTypeDetail {
                        ann_builtin: None,
                        ann_custom: None,
                        type_name: "DiscoveredWriterData".to_string(),
                    },
                },
                member_seq: vec![dust_dds::xtypes::type_object::CompleteStructMember {
                    common: dust_dds::xtypes::type_object::CommonStructMember {
                        member_id: 0x5Au32,
                        member_flags: dust_dds::xtypes::type_object::StructMemberFlag {
                            try_construct:
                                dust_dds::xtypes::dynamic_type::TryConstructKind::Discard,
                            is_external: false,
                            is_optional: false,
                            is_must_undestand: true,
                            is_key: true,
                        },
                        member_type_id: dust_dds::xtypes::type_object::TypeIdentifier::TiPlainArraySmall {
                            array_sdefn: Box::new(
                                dust_dds::xtypes::type_object::PlainArraySElemDefn {
                                    header: dust_dds::xtypes::type_object::PlainCollectionHeader {
                                        equiv_kind: 0,
                                        element_flags: dust_dds::xtypes::type_object::CollectionElementFlag {
                                            try_construct: dust_dds::xtypes::dynamic_type::TryConstructKind::Discard,
                                            is_external: false,
                                        },
                                    },
                                    array_bound_seq: vec![16],
                                    element_identifier: dust_dds::xtypes::type_object::TypeIdentifier::TkUint8Type,
                                },
                            ),
                        },
                    },
                    detail: dust_dds::xtypes::type_object::CompleteMemberDetail {
                        name: "value".to_string(),
                        ann_builtin: None,
                        ann_custom: None,
                    },
                }],
            },
        }
    }
}
impl DdsSerialize for DiscoveredWriterData {
    fn serialize_data(&self) -> DdsResult<Vec<u8>> {
        let mut serializer = ParameterListCdrSerializer::default();
        serializer.write_header()?;

        // dds_publication_data: PublicationBuiltinTopicData:

        serializer.write(PID_ENDPOINT_GUID, &self.dds_publication_data.key)?;
        // Default value is a deviation from the standard and is used for interoperability reasons:
        serializer.write_with_default(
            PID_PARTICIPANT_GUID,
            &self.dds_publication_data.participant_key,
            &Default::default(),
        )?;
        serializer.write(PID_TOPIC_NAME, &self.dds_publication_data.topic_name)?;
        serializer.write(PID_TYPE_NAME, &self.dds_publication_data.type_name)?;
        serializer.write_with_default(
            PID_DURABILITY,
            &self.dds_publication_data.durability,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_DEADLINE,
            &self.dds_publication_data.deadline,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_LATENCY_BUDGET,
            &self.dds_publication_data.latency_budget,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_LIVELINESS,
            &self.dds_publication_data.liveliness,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_RELIABILITY,
            &self.dds_publication_data.reliability,
            &DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
        )?;
        serializer.write_with_default(
            PID_LIFESPAN,
            &self.dds_publication_data.lifespan,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_USER_DATA,
            &self.dds_publication_data.user_data,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_OWNERSHIP,
            &self.dds_publication_data.ownership,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_OWNERSHIP_STRENGTH,
            &self.dds_publication_data.ownership_strength,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_DESTINATION_ORDER,
            &self.dds_publication_data.destination_order,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_PRESENTATION,
            &self.dds_publication_data.presentation,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_PARTITION,
            &self.dds_publication_data.partition,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_TOPIC_DATA,
            &self.dds_publication_data.topic_data,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_GROUP_DATA,
            &self.dds_publication_data.group_data,
            &Default::default(),
        )?;

        serializer.write_with_default(
            PID_DATA_REPRESENTATION,
            &self.dds_publication_data.representation,
            &Default::default(),
        )?;

        // writer_proxy: WriterProxy:

        // skip serilize:
        // writer_proxy.remote_writer_guid: Guid,

        serializer.write_with_default(
            PID_GROUP_ENTITYID,
            &self.writer_proxy.remote_group_entity_id,
            &Default::default(),
        )?;
        serializer
            .write_collection(PID_UNICAST_LOCATOR, &self.writer_proxy.unicast_locator_list)?;
        serializer.write_collection(
            PID_MULTICAST_LOCATOR,
            &self.writer_proxy.multicast_locator_list,
        )?;

        serializer.write_sentinel()?;
        Ok(serializer.writer)
    }
}

impl<'de> DdsDeserialize<'de> for PublicationBuiltinTopicData {
    fn deserialize_data(serialized_data: &'de [u8]) -> DdsResult<Self> {
        let pl_deserializer = ParameterListCdrDeserializer::new(serialized_data)?;
        Ok(Self {
            key: pl_deserializer.read(PID_ENDPOINT_GUID)?,
            // Default value is a deviation from the standard and is used for interoperability reasons:
            participant_key: pl_deserializer
                .read_with_default(PID_PARTICIPANT_GUID, Default::default())?,
            topic_name: pl_deserializer.read(PID_TOPIC_NAME)?,
            type_name: pl_deserializer.read(PID_TYPE_NAME)?,
            durability: pl_deserializer.read_with_default(PID_DURABILITY, Default::default())?,
            deadline: pl_deserializer.read_with_default(PID_DEADLINE, Default::default())?,
            latency_budget: pl_deserializer
                .read_with_default(PID_LATENCY_BUDGET, Default::default())?,
            liveliness: pl_deserializer.read_with_default(PID_LIVELINESS, Default::default())?,
            reliability: pl_deserializer
                .read_with_default(PID_RELIABILITY, DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER)?,
            lifespan: pl_deserializer.read_with_default(PID_LIFESPAN, Default::default())?,
            user_data: pl_deserializer.read_with_default(PID_USER_DATA, Default::default())?,
            ownership: pl_deserializer.read_with_default(PID_OWNERSHIP, Default::default())?,
            ownership_strength: pl_deserializer
                .read_with_default(PID_OWNERSHIP_STRENGTH, Default::default())?,
            destination_order: pl_deserializer
                .read_with_default(PID_DESTINATION_ORDER, Default::default())?,
            presentation: pl_deserializer
                .read_with_default(PID_PRESENTATION, Default::default())?,
            partition: pl_deserializer.read_with_default(PID_PARTITION, Default::default())?,
            topic_data: pl_deserializer.read_with_default(PID_TOPIC_DATA, Default::default())?,
            group_data: pl_deserializer.read_with_default(PID_GROUP_DATA, Default::default())?,

            representation: pl_deserializer
                .read_with_default(PID_DATA_REPRESENTATION, Default::default())?,
        })
    }
}

impl<'de> DdsDeserialize<'de> for DiscoveredWriterData {
    fn deserialize_data(serialized_data: &'de [u8]) -> DdsResult<Self> {
        let pl_deserializer = ParameterListCdrDeserializer::new(serialized_data)?;

        Ok(Self {
            dds_publication_data: PublicationBuiltinTopicData::deserialize_data(serialized_data)?,
            writer_proxy: WriterProxy {
                remote_writer_guid: pl_deserializer.read(PID_ENDPOINT_GUID)?,
                remote_group_entity_id: pl_deserializer
                    .read_with_default(PID_GROUP_ENTITYID, Default::default())?,
                unicast_locator_list: pl_deserializer.read_collection(PID_UNICAST_LOCATOR)?,
                multicast_locator_list: pl_deserializer.read_collection(PID_MULTICAST_LOCATOR)?,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        builtin_topics::BuiltInTopicKey,
        transport::types::{
            EntityId, Guid, BUILT_IN_PARTICIPANT, BUILT_IN_READER_GROUP, BUILT_IN_WRITER_WITH_KEY,
            USER_DEFINED_UNKNOWN,
        },
    };

    #[test]
    fn serialize_all_default() {
        let data = DiscoveredWriterData {
            dds_publication_data: PublicationBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                participant_key: BuiltInTopicKey {
                    value: [6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 0, 9, 0, 0, 0],
                },
                topic_name: "ab".to_string(),
                type_name: "cd".to_string(),
                durability: Default::default(),
                deadline: Default::default(),
                latency_budget: Default::default(),
                liveliness: Default::default(),
                reliability: DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
                lifespan: Default::default(),
                user_data: Default::default(),
                ownership: Default::default(),
                ownership_strength: Default::default(),
                destination_order: Default::default(),
                presentation: Default::default(),
                partition: Default::default(),
                topic_data: Default::default(),
                group_data: Default::default(),
                representation: Default::default(),
            },
            writer_proxy: WriterProxy {
                remote_writer_guid: Guid::new(
                    [5; 12],
                    EntityId::new([11, 12, 13], BUILT_IN_WRITER_WITH_KEY),
                ),
                remote_group_entity_id: EntityId::new([21, 22, 23], BUILT_IN_READER_GROUP),
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
            },
        };

        let expected = vec![
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // ,
            2, 0, 0, 0, // ,
            3, 0, 0, 0, // ,
            4, 0, 0, 0, // ,
            0x50, 0x00, 16, 0, //PID_PARTICIPANT_GUID, length
            6, 0, 0, 0, // ,
            7, 0, 0, 0, // ,
            8, 0, 0, 0, // ,
            9, 0, 0, 0, // ,
            0x05, 0x00, 0x08, 0x00, // PID_TOPIC_NAME, Length: 8
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'a', b'b', 0, 0x00, // string + padding (1 byte)
            0x07, 0x00, 0x08, 0x00, // PID_TYPE_NAME, Length: 8
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'c', b'd', 0, 0x00, // string + padding (1 byte)
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            21, 22, 23, 0xc9, // u8[3], u8
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        let result = data.serialize_data().unwrap();
        assert_eq!(result, expected);
        // assert_eq!(serialize_v1_le(&data), expected);
    }

    #[test]
    fn deserialize_all_default() {
        let expected = DiscoveredWriterData {
            dds_publication_data: PublicationBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                participant_key: BuiltInTopicKey {
                    value: [6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 0, 9, 0, 0, 0],
                },
                topic_name: "ab".to_string(),
                type_name: "cd".to_string(),
                durability: Default::default(),
                deadline: Default::default(),
                latency_budget: Default::default(),
                liveliness: Default::default(),
                reliability: DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
                lifespan: Default::default(),
                user_data: Default::default(),
                ownership: Default::default(),
                ownership_strength: Default::default(),
                destination_order: Default::default(),
                presentation: Default::default(),
                partition: Default::default(),
                topic_data: Default::default(),
                group_data: Default::default(),
                representation: Default::default(),
            },
            writer_proxy: WriterProxy {
                // must correspond to publication_builtin_topic_data.key
                remote_writer_guid: Guid::new(
                    [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0],
                    EntityId::new([4, 0, 0], USER_DEFINED_UNKNOWN),
                ),
                remote_group_entity_id: EntityId::new([21, 22, 23], BUILT_IN_PARTICIPANT),
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
            },
        };

        let mut data = &[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            21, 22, 23, 0xc1, // u8[3], u8
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // ,
            2, 0, 0, 0, // ,
            3, 0, 0, 0, // ,
            4, 0, 0, 0, // ,
            0x50, 0x00, 16, 0, //PID_PARTICIPANT_GUID, length
            6, 0, 0, 0, // ,
            7, 0, 0, 0, // ,
            8, 0, 0, 0, // ,
            9, 0, 0, 0, // ,
            0x05, 0x00, 0x08, 0x00, // PID_TOPIC_NAME, Length: 8
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'a', b'b', 0, 0x00, // string + padding (1 byte)
            0x07, 0x00, 0x08, 0x00, // PID_TYPE_NAME, Length: 8
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'c', b'd', 0, 0x00, // string + padding (1 byte)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ][..];
        let result = DiscoveredWriterData::deserialize_data(&mut data).unwrap();
        assert_eq!(result, expected);
    }
}
