use super::parameter_id_values::{
    PID_DATA_REPRESENTATION, PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY,
    PID_ENDPOINT_GUID, PID_GROUP_DATA, PID_GROUP_ENTITYID, PID_LATENCY_BUDGET, PID_LIFESPAN,
    PID_LIVELINESS, PID_MULTICAST_LOCATOR, PID_OWNERSHIP, PID_OWNERSHIP_STRENGTH,
    PID_PARTICIPANT_GUID, PID_PARTITION, PID_PRESENTATION, PID_RELIABILITY, PID_TOPIC_DATA,
    PID_TOPIC_NAME, PID_TYPE_INFORMATION, PID_TYPE_NAME, PID_UNICAST_LOCATOR, PID_USER_DATA,
};
use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData},
    infrastructure::{
        qos_policy::{
            DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER, DataRepresentationQosPolicy,
            DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
            LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy,
            OwnershipStrengthQosPolicy, PartitionQosPolicy, PresentationQosPolicy,
            TopicDataQosPolicy, UserDataQosPolicy,
        },
        type_support::TypeSupport,
    },
    transport::types::{ENTITYID_UNKNOWN, EntityId, Guid, Locator},
    xtypes::{
        binding::XTypesBinding, data_storage::DataStorageMapping, dynamic_type::DynamicTypeBuilder,
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
    /// XTypes TypeInformation, stored as raw XCDR-encoded bytes for wire compatibility.
    /// This allows discovery to work with TypeInformation from other XTypes implementations
    /// without requiring full deserialization of the complex TypeInformation structure.
    pub(crate) type_information: Option<Vec<u8>>,
}
impl TypeSupport for DiscoveredWriterData {
    #[inline]
    fn get_type_name() -> &'static str {
        "DiscoveredWriterData"
    }

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
                        is_optional: false,
                        is_must_understand: true,
                        is_shared: false,
                        is_default_label: false,
                    })
                    .unwrap();
                self.index += 1;
            }
            fn add_member_with_default<T: XTypesBinding + DataStorageMapping>(
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
                        default_value: Some(default.into_storage()),
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
                    name: alloc::string::String::from(Self::get_type_name()),
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
        builder.add_member_with_default(
            "type_information",
            PID_TYPE_INFORMATION,
            Vec::<u8>::default(),
        );

        builder.builder.build()
    }

    fn create_sample(mut src: crate::xtypes::dynamic_type::DynamicData) -> Self {
        let key = BuiltInTopicKey::try_from_storage(
            src.remove_value(PID_ENDPOINT_GUID as u32)
                .expect("Must exist"),
        )
        .expect("Must match");
        let remote_writer_guid = Guid::new(
            key.value[0..12].try_into().expect("Must match"),
            EntityId::new(
                key.value[12..15].try_into().expect("Must match"),
                key.value[15],
            ),
        );
        Self {
            dds_publication_data: PublicationBuiltinTopicData {
                key,
                participant_key: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_PARTICIPANT_GUID as u32)
                        .expect("Must exist"),
                )
                .expect("Must match"),
                topic_name: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_TOPIC_NAME as u32).expect("Must exist"),
                )
                .expect("Must match"),
                type_name: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_TYPE_NAME as u32).expect("Must exist"),
                )
                .expect("Must match"),
                durability: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_DURABILITY as u32).expect("Must exist"),
                )
                .expect("Must match"),
                deadline: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_DEADLINE as u32).expect("Must exist"),
                )
                .expect("Must match"),
                latency_budget: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_LATENCY_BUDGET as u32)
                        .expect("Must exist"),
                )
                .expect("Must match"),
                liveliness: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_LIVELINESS as u32).expect("Must exist"),
                )
                .expect("Must match"),
                reliability: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_RELIABILITY as u32)
                        .expect("Must exist"),
                )
                .expect("Must match"),
                lifespan: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_LIFESPAN as u32).expect("Must exist"),
                )
                .expect("Must match"),
                user_data: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_USER_DATA as u32).expect("Must exist"),
                )
                .expect("Must match"),
                ownership: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_OWNERSHIP as u32).expect("Must exist"),
                )
                .expect("Must match"),
                ownership_strength: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_OWNERSHIP_STRENGTH as u32)
                        .expect("Must exist"),
                )
                .expect("Must match"),
                destination_order: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_DESTINATION_ORDER as u32)
                        .expect("Must exist"),
                )
                .expect("Must match"),
                presentation: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_PRESENTATION as u32)
                        .expect("Must exist"),
                )
                .expect("Must match"),
                partition: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_PARTITION as u32).expect("Must exist"),
                )
                .expect("Must match"),
                topic_data: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_TOPIC_DATA as u32).expect("Must exist"),
                )
                .expect("Must match"),
                group_data: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_GROUP_DATA as u32).expect("Must exist"),
                )
                .expect("Must match"),
                representation: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_DATA_REPRESENTATION as u32)
                        .expect("Must exist"),
                )
                .expect("Must match"),
            },
            writer_proxy: WriterProxy {
                remote_writer_guid,
                remote_group_entity_id: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_GROUP_ENTITYID as u32)
                        .expect("Must exist"),
                )
                .expect("Must match"),
                unicast_locator_list: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_UNICAST_LOCATOR as u32)
                        .expect("Must exist"),
                )
                .expect("Must match"),
                multicast_locator_list: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_MULTICAST_LOCATOR as u32)
                        .expect("Must exist"),
                )
                .expect("Must match"),
            },
            type_information: src
                .remove_value(PID_TYPE_INFORMATION as u32)
                .ok()
                .and_then(|v| Vec::<u8>::try_from_storage(v).ok())
                .filter(|v| !v.is_empty()),
        }
    }

    fn create_dynamic_sample(self) -> dust_dds::xtypes::dynamic_type::DynamicData {
        let mut data =
            dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data(Self::get_type());
        data.set_value(
            PID_ENDPOINT_GUID as u32,
            self.dds_publication_data.key.into_storage(),
        );
        data.set_value(
            PID_PARTICIPANT_GUID as u32,
            self.dds_publication_data.participant_key.into_storage(),
        );
        data.set_value(
            PID_TOPIC_NAME as u32,
            self.dds_publication_data.topic_name.into_storage(),
        );
        data.set_value(
            PID_TYPE_NAME as u32,
            self.dds_publication_data.type_name.into_storage(),
        );
        data.set_value(
            PID_DURABILITY as u32,
            self.dds_publication_data.durability.into_storage(),
        );
        data.set_value(
            PID_DEADLINE as u32,
            self.dds_publication_data.deadline.into_storage(),
        );
        data.set_value(
            PID_LATENCY_BUDGET as u32,
            self.dds_publication_data.latency_budget.into_storage(),
        );
        data.set_value(
            PID_LIVELINESS as u32,
            self.dds_publication_data.liveliness.into_storage(),
        );
        data.set_value(
            PID_RELIABILITY as u32,
            self.dds_publication_data.reliability.into_storage(),
        );
        data.set_value(
            PID_LIFESPAN as u32,
            self.dds_publication_data.lifespan.into_storage(),
        );
        data.set_value(
            PID_OWNERSHIP as u32,
            self.dds_publication_data.ownership.into_storage(),
        );
        data.set_value(
            PID_OWNERSHIP_STRENGTH as u32,
            self.dds_publication_data.ownership_strength.into_storage(),
        );
        data.set_value(
            PID_DESTINATION_ORDER as u32,
            self.dds_publication_data.destination_order.into_storage(),
        );
        data.set_value(
            PID_USER_DATA as u32,
            self.dds_publication_data.user_data.into_storage(),
        );
        data.set_value(
            PID_PRESENTATION as u32,
            self.dds_publication_data.presentation.into_storage(),
        );
        data.set_value(
            PID_PARTITION as u32,
            self.dds_publication_data.partition.into_storage(),
        );
        data.set_value(
            PID_TOPIC_DATA as u32,
            self.dds_publication_data.topic_data.into_storage(),
        );
        data.set_value(
            PID_GROUP_DATA as u32,
            self.dds_publication_data.group_data.into_storage(),
        );
        data.set_value(
            PID_DATA_REPRESENTATION as u32,
            self.dds_publication_data.representation.into_storage(),
        );
        data.set_value(
            PID_GROUP_ENTITYID as u32,
            self.writer_proxy.remote_group_entity_id.into_storage(),
        );
        data.set_value(
            PID_UNICAST_LOCATOR as u32,
            self.writer_proxy.unicast_locator_list.into_storage(),
        );
        data.set_value(
            PID_MULTICAST_LOCATOR as u32,
            self.writer_proxy.multicast_locator_list.into_storage(),
        );
        data.set_value(
            PID_TYPE_INFORMATION as u32,
            self.type_information.unwrap_or_default().into_storage(),
        );
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        builtin_topics::BuiltInTopicKey,
        transport::types::{
            BUILT_IN_PARTICIPANT, BUILT_IN_READER_GROUP, BUILT_IN_WRITER_WITH_KEY, EntityId, Guid,
            USER_DEFINED_UNKNOWN,
        },
        xtypes::{deserializer::CdrDeserializer, serializer::RtpsPlCdrSerializer},
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
            type_information: None,
        }
        .create_dynamic_sample();

        let expected = [
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
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

        assert_eq!(RtpsPlCdrSerializer::serialize(&data).unwrap(), expected);
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
            type_information: None,
        };

        let data = [
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
        ];
        // Deserialize to DynamicData and then create the sample to compare at struct level
        let deserialized_dynamic =
            CdrDeserializer::deserialize_builtin(DiscoveredWriterData::get_type(), &data).unwrap();
        let actual = DiscoveredWriterData::create_sample(deserialized_dynamic);
        assert_eq!(actual, expected);
    }

    #[test]
    fn serialize_deserialize_with_type_information() {
        use crate::infrastructure::qos_policy::DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER;

        let type_info = vec![0x01u8, 0x02, 0x03, 0x04];
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
                    [21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32],
                    EntityId::new([21, 22, 23], BUILT_IN_WRITER_WITH_KEY),
                ),
                remote_group_entity_id: EntityId::new([21, 22, 23], BUILT_IN_READER_GROUP),
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
            },
            type_information: Some(type_info.clone()),
        };

        // Serialize
        let dynamic_sample = data.create_dynamic_sample();
        let serialized = RtpsPlCdrSerializer::serialize(&dynamic_sample).unwrap();

        // Deserialize
        let deserialized_dynamic =
            CdrDeserializer::deserialize_builtin(DiscoveredWriterData::get_type(), &serialized)
                .unwrap();
        let deserialized = DiscoveredWriterData::create_sample(deserialized_dynamic);

        // Verify type_information is preserved
        assert_eq!(deserialized.type_information, Some(type_info));
    }
}
