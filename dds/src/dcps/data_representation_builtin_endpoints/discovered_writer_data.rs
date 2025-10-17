use super::{
    parameter_id_values::{
        PID_DATA_REPRESENTATION, PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY,
        PID_ENDPOINT_GUID, PID_GROUP_DATA, PID_GROUP_ENTITYID, PID_LATENCY_BUDGET, PID_LIFESPAN,
        PID_LIVELINESS, PID_MULTICAST_LOCATOR, PID_OWNERSHIP, PID_OWNERSHIP_STRENGTH,
        PID_PARTICIPANT_GUID, PID_PARTITION, PID_PRESENTATION, PID_RELIABILITY, PID_TOPIC_DATA,
        PID_TOPIC_NAME, PID_TYPE_NAME, PID_UNICAST_LOCATOR, PID_USER_DATA,
    },
    payload_serializer_deserializer::parameter_list_deserializer::ParameterListCdrDeserializer,
};
use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData},
    infrastructure::{
        error::DdsResult,
        qos_policy::{
            DataRepresentationQosPolicy, DeadlineQosPolicy, DestinationOrderQosPolicy,
            DurabilityQosPolicy, GroupDataQosPolicy, LatencyBudgetQosPolicy, LifespanQosPolicy,
            LivelinessQosPolicy, OwnershipQosPolicy, OwnershipStrengthQosPolicy,
            PartitionQosPolicy, PresentationQosPolicy, TopicDataQosPolicy, UserDataQosPolicy,
            DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
        },
        type_support::{DdsDeserialize, TypeSupport},
    },
    transport::types::{EntityId, Guid, Locator, ENTITYID_UNKNOWN},
    xtypes::{
        binding::XTypesBinding, data_representation::DataKind, dynamic_type::DynamicTypeBuilder,
    },
};
use alloc::{string::String, vec::Vec};

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
    fn get_type() -> dust_dds::xtypes::dynamic_type::DynamicType {
        extern crate alloc;
        struct ConvenienceDynamicTypeBuilder {
            builder: DynamicTypeBuilder,
            index: u32,
        }
        impl ConvenienceDynamicTypeBuilder {
            fn add_member<T: XTypesBinding>(&mut self, name: &str, id: i16) {
                self.builder
                    .add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
                        name: alloc::string::String::from(name),
                        id: id as u32,
                        r#type: T::get_dynamic_type(),
                        default_value: None,
                        index: self.index,
                        try_construct_kind:
                            dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,
                        label: alloc::vec::Vec::new(),
                        is_key: false,
                        is_optional: false,
                        is_must_understand: true,
                        is_shared: false,
                        is_default_label: false,
                    })
                    .unwrap();
                self.index += 1;
            }
            fn add_key_member<T: XTypesBinding>(&mut self, name: &str, id: i16) {
                self.builder
                    .add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
                        name: alloc::string::String::from(name),
                        id: id as u32,
                        r#type: T::get_dynamic_type(),
                        default_value: None,
                        index: self.index,
                        try_construct_kind:
                            dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,
                        label: alloc::vec::Vec::new(),
                        is_key: true,
                        is_optional: true,
                        is_must_understand: true,
                        is_shared: false,
                        is_default_label: false,
                    })
                    .unwrap();
                self.index += 1;
            }
            fn add_member_with_default<T: XTypesBinding + Into<DataKind>>(
                &mut self,
                name: &str,
                id: i16,
                default: T,
            ) {
                self.builder
                    .add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
                        name: alloc::string::String::from(name),
                        id: id as u32,
                        r#type: T::get_dynamic_type(),
                        default_value: Some(default.into()),
                        index: self.index,
                        try_construct_kind:
                            dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,
                        label: alloc::vec::Vec::new(),
                        is_key: false,
                        is_optional: true,
                        is_must_understand: true,
                        is_shared: false,
                        is_default_label: false,
                    })
                    .unwrap();
                self.index += 1;
            }
        }
        let mut builder = ConvenienceDynamicTypeBuilder {
            builder: dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::create_type(
                dust_dds::xtypes::dynamic_type::TypeDescriptor {
                    kind: dust_dds::xtypes::dynamic_type::TypeKind::STRUCTURE,
                    name: alloc::string::String::from("DiscoveredReaderData"),
                    base_type: None,
                    discriminator_type: None,
                    bound: alloc::vec::Vec::new(),
                    element_type: None,
                    key_element_type: None,
                    extensibility_kind: dust_dds::xtypes::dynamic_type::ExtensibilityKind::Mutable,
                    is_nested: false,
                },
            ),
            index: 0,
        };

        builder.add_key_member::<BuiltInTopicKey>("key", PID_ENDPOINT_GUID);
        // for interoperability reasons this is omitted when default (as opposed to standard):
        builder.add_member_with_default(
            "participant_key",
            PID_PARTICIPANT_GUID,
            BuiltInTopicKey::default(),
        );

        builder.add_member::<String>("topic_name", PID_TOPIC_NAME);
        builder.add_member::<String>("type_name", PID_TYPE_NAME);

        builder.add_member_with_default(
            "durability",
            PID_DURABILITY,
            DurabilityQosPolicy::default(),
        );
        builder.add_member_with_default("deadline", PID_DEADLINE, DeadlineQosPolicy::default());
        builder.add_member_with_default(
            "latency_budget",
            PID_LATENCY_BUDGET,
            LatencyBudgetQosPolicy::default(),
        );
        builder.add_member_with_default(
            "liveliness",
            PID_LIVELINESS,
            LivelinessQosPolicy::default(),
        );
        builder.add_member_with_default(
            "reliability",
            PID_RELIABILITY,
            DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
        );
        builder.add_member_with_default("lifespan", PID_LIFESPAN, LifespanQosPolicy::default());
        builder.add_member_with_default("user_data", PID_USER_DATA, UserDataQosPolicy::default());
        builder.add_member_with_default("ownership", PID_OWNERSHIP, OwnershipQosPolicy::default());
        builder.add_member_with_default(
            "ownership_strength",
            PID_OWNERSHIP_STRENGTH,
            OwnershipStrengthQosPolicy::default(),
        );
        builder.add_member_with_default(
            "destination_order",
            PID_DESTINATION_ORDER,
            DestinationOrderQosPolicy::default(),
        );
        builder.add_member_with_default(
            "presentation",
            PID_PRESENTATION,
            PresentationQosPolicy::default(),
        );
        builder.add_member_with_default("partition", PID_PARTITION, PartitionQosPolicy::default());
        builder.add_member_with_default(
            "topic_data",
            PID_TOPIC_DATA,
            TopicDataQosPolicy::default(),
        );
        builder.add_member_with_default(
            "group_data",
            PID_GROUP_DATA,
            GroupDataQosPolicy::default(),
        );
        builder.add_member_with_default(
            "representation",
            PID_DATA_REPRESENTATION,
            DataRepresentationQosPolicy::default(),
        );
        builder.add_member_with_default(
            "remote_group_entity_id",
            PID_GROUP_ENTITYID,
            ENTITYID_UNKNOWN,
        );
        builder.add_member_with_default(
            "unicast_locator_list",
            PID_UNICAST_LOCATOR,
            Vec::<Locator>::default(),
        );
        builder.add_member_with_default(
            "multicast_locator_list",
            PID_MULTICAST_LOCATOR,
            Vec::<Locator>::default(),
        );

        builder.builder.build()
    }

    fn create_sample(_src: crate::xtypes::dynamic_type::DynamicData) -> Self {
        todo!()
    }

    fn create_dynamic_sample(self) -> dust_dds::xtypes::dynamic_type::DynamicData {
        dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data(Self::get_type())
            .set_value(PID_ENDPOINT_GUID as u32, self.dds_publication_data.key)
            .set_value(
                PID_PARTICIPANT_GUID as u32,
                self.dds_publication_data.participant_key,
            )
            .set_value(PID_TOPIC_NAME as u32, self.dds_publication_data.topic_name)
            .set_value(PID_TYPE_NAME as u32, self.dds_publication_data.type_name)
            .set_value(PID_DURABILITY as u32, self.dds_publication_data.durability)
            .set_value(PID_DEADLINE as u32, self.dds_publication_data.deadline)
            .set_value(
                PID_LATENCY_BUDGET as u32,
                self.dds_publication_data.latency_budget,
            )
            .set_value(PID_LIVELINESS as u32, self.dds_publication_data.liveliness)
            .set_value(
                PID_RELIABILITY as u32,
                self.dds_publication_data.reliability,
            )
            .set_value(PID_LIFESPAN as u32, self.dds_publication_data.lifespan)
            .set_value(PID_OWNERSHIP as u32, self.dds_publication_data.ownership)
            .set_value(
                PID_OWNERSHIP_STRENGTH as u32,
                self.dds_publication_data.ownership_strength,
            )
            .set_value(
                PID_DESTINATION_ORDER as u32,
                self.dds_publication_data.destination_order,
            )
            .set_value(PID_USER_DATA as u32, self.dds_publication_data.user_data)
            .set_value(
                PID_PRESENTATION as u32,
                self.dds_publication_data.presentation,
            )
            .set_value(PID_PARTITION as u32, self.dds_publication_data.partition)
            .set_value(PID_TOPIC_DATA as u32, self.dds_publication_data.topic_data)
            .set_value(PID_GROUP_DATA as u32, self.dds_publication_data.group_data)
            .set_value(
                PID_DATA_REPRESENTATION as u32,
                self.dds_publication_data.representation,
            )
            .set_value(
                PID_GROUP_ENTITYID as u32,
                self.writer_proxy.remote_group_entity_id,
            )
            .set_value(
                PID_UNICAST_LOCATOR as u32,
                self.writer_proxy.unicast_locator_list,
            )
            .set_value(
                PID_MULTICAST_LOCATOR as u32,
                self.writer_proxy.multicast_locator_list,
            )
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
        xtypes::{pl_cdr_serializer::PlCdrLeSerializer, serializer::XTypesSerializer},
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
            // 0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x05, 0x00, 0x08, 0x00, // PID_TOPIC_NAME, Length: 8
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'a', b'b', 0, 0x00, // string + padding (1 byte)
            0x07, 0x00, 0x08, 0x00, // PID_TYPE_NAME, Length: 8
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'c', b'd', 0, 0x00, // string + padding (1 byte)
            0x50, 0x00, 16, 0, //PID_PARTICIPANT_GUID, length
            6, 0, 0, 0, // ,
            7, 0, 0, 0, // ,
            8, 0, 0, 0, // ,
            9, 0, 0, 0, // ,
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            21, 22, 23, 0xc9, // u8[3], u8
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // ,
            2, 0, 0, 0, // ,
            3, 0, 0, 0, // ,
            4, 0, 0, 0, // ,
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        let dynamic_sample = data.create_dynamic_sample();
        let result = dynamic_sample
            .serialize(PlCdrLeSerializer::new(Vec::new()))
            .unwrap().into_inner();
        assert_eq!(result, expected);
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
