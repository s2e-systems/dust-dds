use super::{
    parameter_id_values::{
        DEFAULT_EXPECTS_INLINE_QOS, PID_DATA_REPRESENTATION, PID_DEADLINE, PID_DESTINATION_ORDER,
        PID_DURABILITY, PID_ENDPOINT_GUID, PID_EXPECTS_INLINE_QOS, PID_GROUP_DATA,
        PID_GROUP_ENTITYID, PID_LATENCY_BUDGET, PID_LIVELINESS, PID_MULTICAST_LOCATOR,
        PID_OWNERSHIP, PID_PARTICIPANT_GUID, PID_PARTITION, PID_PRESENTATION, PID_RELIABILITY,
        PID_TIME_BASED_FILTER, PID_TOPIC_DATA, PID_TOPIC_NAME, PID_TYPE_NAME, PID_UNICAST_LOCATOR,
        PID_USER_DATA,
    },
    payload_serializer_deserializer::parameter_list_deserializer::ParameterListCdrDeserializer,
};
use crate::{
    builtin_topics::{BuiltInTopicKey, SubscriptionBuiltinTopicData},
    infrastructure::{
        error::DdsResult,
        qos_policy::{
            DataRepresentationQosPolicy, DeadlineQosPolicy, DestinationOrderQosPolicy,
            DurabilityQosPolicy, GroupDataQosPolicy, LatencyBudgetQosPolicy, LivelinessQosPolicy,
            OwnershipQosPolicy, PartitionQosPolicy, PresentationQosPolicy,
            TimeBasedFilterQosPolicy, TopicDataQosPolicy, UserDataQosPolicy,
            DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
        },
        type_support::DdsDeserialize,
    },
    transport::types::{EntityId, Guid, Locator, ENTITYID_UNKNOWN},
    xtypes::{
        binding::{DataKind, XTypesBinding},
        dynamic_type::DynamicTypeBuilder,
    },
};
use alloc::vec::Vec;
use dust_dds::infrastructure::type_support::TypeSupport;

#[derive(Debug, PartialEq, Eq, Clone, TypeSupport)]
#[dust_dds(extensibility = "mutable")]
pub struct ReaderProxy {
    #[dust_dds(id=PID_ENDPOINT_GUID as u32, non_serialized)]
    pub remote_reader_guid: Guid,
    #[dust_dds(id=PID_GROUP_ENTITYID as u32)]
    pub remote_group_entity_id: EntityId,
    #[dust_dds(id=PID_UNICAST_LOCATOR as u32)]
    pub unicast_locator_list: Vec<Locator>,
    #[dust_dds(id=PID_MULTICAST_LOCATOR as u32)]
    pub multicast_locator_list: Vec<Locator>,
    #[dust_dds(id=PID_EXPECTS_INLINE_QOS as u32)]
    pub expects_inline_qos: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DiscoveredReaderData {
    pub(crate) dds_subscription_data: SubscriptionBuiltinTopicData,
    pub(crate) reader_proxy: ReaderProxy,
}

impl dust_dds::infrastructure::type_support::TypeSupport for DiscoveredReaderData {
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
        builder.add_member::<BuiltInTopicKey>("participant_key", PID_PARTICIPANT_GUID);

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
            DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
        );
        builder.add_member_with_default("ownership", PID_OWNERSHIP, OwnershipQosPolicy::default());
        builder.add_member_with_default(
            "destination_order",
            PID_DESTINATION_ORDER,
            DestinationOrderQosPolicy::default(),
        );
        builder.add_member_with_default("user_data", PID_USER_DATA, UserDataQosPolicy::default());
        builder.add_member_with_default(
            "time_based_filter",
            PID_TIME_BASED_FILTER,
            TimeBasedFilterQosPolicy::default(),
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
            "expects_inline_qos",
            PID_EXPECTS_INLINE_QOS,
            DEFAULT_EXPECTS_INLINE_QOS,
        );
        builder.builder.build()
    }
    fn create_dynamic_sample(self) -> dust_dds::xtypes::dynamic_type::DynamicData {
        let mut data =
            dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data(Self::get_type());
        data.set_value(
            PID_ENDPOINT_GUID as u32,
            self.dds_subscription_data.key.into(),
        )
        .unwrap();
        data.set_value(
            PID_PARTICIPANT_GUID as u32,
            self.dds_subscription_data.participant_key.into(),
        )
        .unwrap();
        data.set_value(
            PID_TOPIC_NAME as u32,
            self.dds_subscription_data.topic_name.into(),
        )
        .unwrap();
        data.set_value(
            PID_TYPE_NAME as u32,
            self.dds_subscription_data.type_name.into(),
        )
        .unwrap();
        data.set_value(
            PID_DURABILITY as u32,
            self.dds_subscription_data.durability.into(),
        )
        .unwrap();
        data.set_value(
            PID_DEADLINE as u32,
            self.dds_subscription_data.deadline.into(),
        )
        .unwrap();
        data.set_value(
            PID_LATENCY_BUDGET as u32,
            self.dds_subscription_data.latency_budget.into(),
        )
        .unwrap();
        data.set_value(
            PID_LIVELINESS as u32,
            self.dds_subscription_data.liveliness.into(),
        )
        .unwrap();
        data.set_value(
            PID_RELIABILITY as u32,
            self.dds_subscription_data.reliability.into(),
        )
        .unwrap();
        data.set_value(
            PID_OWNERSHIP as u32,
            self.dds_subscription_data.ownership.into(),
        )
        .unwrap();
        data.set_value(
            PID_DESTINATION_ORDER as u32,
            self.dds_subscription_data.destination_order.into(),
        )
        .unwrap();
        data.set_value(
            PID_USER_DATA as u32,
            self.dds_subscription_data.user_data.into(),
        )
        .unwrap();
        data.set_value(
            PID_TIME_BASED_FILTER as u32,
            self.dds_subscription_data.time_based_filter.into(),
        )
        .unwrap();
        data.set_value(
            PID_PRESENTATION as u32,
            self.dds_subscription_data.presentation.into(),
        )
        .unwrap();
        data.set_value(
            PID_PARTITION as u32,
            self.dds_subscription_data.partition.into(),
        )
        .unwrap();
        data.set_value(
            PID_TOPIC_DATA as u32,
            self.dds_subscription_data.topic_data.into(),
        )
        .unwrap();
        data.set_value(
            PID_GROUP_DATA as u32,
            self.dds_subscription_data.group_data.into(),
        )
        .unwrap();
        data.set_value(
            PID_DATA_REPRESENTATION as u32,
            self.dds_subscription_data.representation.into(),
        )
        .unwrap();
        data.set_value(
            PID_GROUP_ENTITYID as u32,
            self.reader_proxy.remote_group_entity_id.into(),
        )
        .unwrap();
        data.set_value(
            PID_UNICAST_LOCATOR as u32,
            self.reader_proxy.unicast_locator_list.into(),
        )
        .unwrap();
        data.set_value(
            PID_MULTICAST_LOCATOR as u32,
            self.reader_proxy.multicast_locator_list.into(),
        )
        .unwrap();
        data.set_value(
            PID_EXPECTS_INLINE_QOS as u32,
            self.reader_proxy.expects_inline_qos.into(),
        )
        .unwrap();
        data
    }
}

impl<'de> DdsDeserialize<'de> for SubscriptionBuiltinTopicData {
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
            reliability: pl_deserializer.read_with_default(
                PID_RELIABILITY,
                DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
            )?,
            ownership: pl_deserializer.read_with_default(PID_OWNERSHIP, Default::default())?,
            destination_order: pl_deserializer
                .read_with_default(PID_DESTINATION_ORDER, Default::default())?,
            user_data: pl_deserializer.read_with_default(PID_USER_DATA, Default::default())?,
            time_based_filter: pl_deserializer
                .read_with_default(PID_TIME_BASED_FILTER, Default::default())?,
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

impl<'de> DdsDeserialize<'de> for DiscoveredReaderData {
    fn deserialize_data(serialized_data: &'de [u8]) -> DdsResult<Self> {
        let pl_deserializer = ParameterListCdrDeserializer::new(serialized_data)?;

        Ok(Self {
            dds_subscription_data: SubscriptionBuiltinTopicData::deserialize_data(serialized_data)?,
            reader_proxy: ReaderProxy {
                remote_reader_guid: pl_deserializer.read(PID_ENDPOINT_GUID)?,
                remote_group_entity_id: pl_deserializer
                    .read_with_default(PID_GROUP_ENTITYID, Default::default())?,
                unicast_locator_list: pl_deserializer.read_collection(PID_UNICAST_LOCATOR)?,
                multicast_locator_list: pl_deserializer.read_collection(PID_MULTICAST_LOCATOR)?,
                expects_inline_qos: pl_deserializer
                    .read_with_default(PID_EXPECTS_INLINE_QOS, DEFAULT_EXPECTS_INLINE_QOS)?,
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
            EntityId, Guid, BUILT_IN_WRITER_WITH_KEY, USER_DEFINED_READER_WITH_KEY,
            USER_DEFINED_UNKNOWN,
        },
        xtypes::{pl_cdr_serializer::PlCdrLeSerializer, serialize::XTypesSerialize},
    };

    #[test]
    fn serialize_all_default() {
        let data = DiscoveredReaderData {
            dds_subscription_data: SubscriptionBuiltinTopicData {
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
                reliability: DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
                ownership: Default::default(),
                destination_order: Default::default(),
                user_data: Default::default(),
                time_based_filter: Default::default(),
                presentation: Default::default(),
                partition: Default::default(),
                topic_data: Default::default(),
                group_data: Default::default(),
                representation: Default::default(),
            },
            reader_proxy: ReaderProxy {
                remote_reader_guid: Guid::new(
                    [5; 12],
                    EntityId::new([11, 12, 13], USER_DEFINED_READER_WITH_KEY),
                ),
                remote_group_entity_id: EntityId::new([21, 22, 23], BUILT_IN_WRITER_WITH_KEY),
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                expects_inline_qos: false,
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
            21, 22, 23, 0xc2, //
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // ,
            2, 0, 0, 0, // ,
            3, 0, 0, 0, // ,
            4, 0, 0, 0, // ,
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        let dynamic_sample = data.create_dynamic_sample();

        let mut buffer = vec![];
        dynamic_sample
            .serialize(&mut PlCdrLeSerializer::new(&mut buffer))
            .unwrap();

        assert_eq!(buffer, expected);
    }

    #[test]
    fn deserialize_all_default() {
        let expected = DiscoveredReaderData {
            reader_proxy: ReaderProxy {
                remote_reader_guid: Guid::new(
                    [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0],
                    EntityId::new([4, 0, 0], USER_DEFINED_UNKNOWN),
                ),
                remote_group_entity_id: EntityId::new([21, 22, 23], BUILT_IN_WRITER_WITH_KEY),
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                expects_inline_qos: false,
            },
            dds_subscription_data: SubscriptionBuiltinTopicData {
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
                reliability: DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
                ownership: Default::default(),
                destination_order: Default::default(),
                user_data: Default::default(),
                time_based_filter: Default::default(),
                presentation: Default::default(),
                partition: Default::default(),
                topic_data: Default::default(),
                group_data: Default::default(),
                representation: Default::default(),
            },
        };

        let mut data = &[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            21, 22, 23, 0xc2, // u8[3], u8
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
        let result = DiscoveredReaderData::deserialize_data(&mut data).unwrap();
        assert_eq!(result, expected);
    }
}
