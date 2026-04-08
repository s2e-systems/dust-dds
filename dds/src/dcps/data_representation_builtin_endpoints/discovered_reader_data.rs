use super::parameter_id_values::{
    DEFAULT_EXPECTS_INLINE_QOS, PID_DATA_REPRESENTATION, PID_DEADLINE, PID_DESTINATION_ORDER,
    PID_DURABILITY, PID_ENDPOINT_GUID, PID_EXPECTS_INLINE_QOS, PID_GROUP_DATA, PID_GROUP_ENTITYID,
    PID_LATENCY_BUDGET, PID_LIVELINESS, PID_MULTICAST_LOCATOR, PID_OWNERSHIP, PID_PARTICIPANT_GUID,
    PID_PARTITION, PID_PRESENTATION, PID_RELIABILITY, PID_TIME_BASED_FILTER, PID_TOPIC_DATA,
    PID_TOPIC_NAME, PID_TYPE_NAME, PID_UNICAST_LOCATOR, PID_USER_DATA,
};
use crate::{
    builtin_topics::{BuiltInTopicKey, SubscriptionBuiltinTopicData},
    dcps::data_representation_builtin_endpoints::ConvenienceTypeBuilder,
    infrastructure::qos_policy::{
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS, DataRepresentationQosPolicy,
        DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
        LatencyBudgetQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, PartitionQosPolicy,
        PresentationQosPolicy, ReliabilityQosPolicy, TimeBasedFilterQosPolicy, TopicDataQosPolicy,
        UserDataQosPolicy,
    },
    transport::types::{ENTITYID_UNKNOWN, EntityId, Guid, Locator},
    xtypes::{
        data_storage::DataStorageMapping,
        dynamic_type::{DynamicType, StaticTypeInformation},
    },
};
use alloc::{string::String, vec::Vec};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ReaderProxy {
    pub remote_reader_guid: Guid,
    pub remote_group_entity_id: EntityId,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    pub expects_inline_qos: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DiscoveredReaderData {
    pub(crate) dds_subscription_data: SubscriptionBuiltinTopicData,
    pub(crate) reader_proxy: ReaderProxy,
}

impl dust_dds::infrastructure::type_support::TypeSupport for DiscoveredReaderData {
    const TYPE_NAME: &'static str = "DiscoveredReaderData";

    const r#TYPE: &'static dyn DynamicType = &StaticTypeInformation {
        descriptor: &ConvenienceTypeBuilder::type_descriptor(Self::TYPE_NAME),
        member_list: &[
            ConvenienceTypeBuilder::key_member::<BuiltInTopicKey>(0, "key", PID_ENDPOINT_GUID),
            // for interoperability reasons this is omitted when default (as opposed to standard):
            ConvenienceTypeBuilder::member_with_default::<BuiltInTopicKey>(
                1,
                "participant_key",
                PID_PARTICIPANT_GUID,
            ),
            ConvenienceTypeBuilder::member::<String>(2, "topic_name", PID_TOPIC_NAME),
            ConvenienceTypeBuilder::member::<String>(3, "type_name", PID_TYPE_NAME),
            ConvenienceTypeBuilder::member_with_default::<DurabilityQosPolicy>(
                4,
                "durability",
                PID_DURABILITY,
            ),
            ConvenienceTypeBuilder::member_with_default::<DeadlineQosPolicy>(
                5,
                "deadline",
                PID_DEADLINE,
            ),
            ConvenienceTypeBuilder::member_with_default::<LatencyBudgetQosPolicy>(
                6,
                "latency_budget",
                PID_LATENCY_BUDGET,
            ),
            ConvenienceTypeBuilder::member_with_default::<LivelinessQosPolicy>(
                7,
                "liveliness",
                PID_LIVELINESS,
            ),
            ConvenienceTypeBuilder::member_with_default::<ReliabilityQosPolicy>(
                8,
                "reliability",
                PID_RELIABILITY,
            ),
            ConvenienceTypeBuilder::member_with_default::<OwnershipQosPolicy>(
                9,
                "ownership",
                PID_OWNERSHIP,
            ),
            ConvenienceTypeBuilder::member_with_default::<DestinationOrderQosPolicy>(
                10,
                "destination_order",
                PID_DESTINATION_ORDER,
            ),
            ConvenienceTypeBuilder::member_with_default::<UserDataQosPolicy>(
                11,
                "user_data",
                PID_USER_DATA,
            ),
            ConvenienceTypeBuilder::member_with_default::<TimeBasedFilterQosPolicy>(
                12,
                "time_based_filter",
                PID_TIME_BASED_FILTER,
            ),
            ConvenienceTypeBuilder::member_with_default::<PresentationQosPolicy>(
                13,
                "presentation",
                PID_PRESENTATION,
            ),
            ConvenienceTypeBuilder::member_with_default::<PartitionQosPolicy>(
                14,
                "partition",
                PID_PARTITION,
            ),
            ConvenienceTypeBuilder::member_with_default::<TopicDataQosPolicy>(
                15,
                "topic_data",
                PID_TOPIC_DATA,
            ),
            ConvenienceTypeBuilder::member_with_default::<GroupDataQosPolicy>(
                16,
                "group_data",
                PID_GROUP_DATA,
            ),
            ConvenienceTypeBuilder::member_with_default::<DataRepresentationQosPolicy>(
                17,
                "representation",
                PID_DATA_REPRESENTATION,
            ),
            ConvenienceTypeBuilder::member_with_default::<EntityId>(
                18,
                "remote_group_entity_id",
                PID_GROUP_ENTITYID,
            ),
            ConvenienceTypeBuilder::member_with_default::<Vec<Locator>>(
                19,
                "unicast_locator_list",
                PID_UNICAST_LOCATOR,
            ),
            ConvenienceTypeBuilder::member_with_default::<Vec<Locator>>(
                20,
                "multicast_locator_list",
                PID_MULTICAST_LOCATOR,
            ),
            ConvenienceTypeBuilder::member_with_default::<bool>(
                21,
                "expects_inline_qos",
                PID_EXPECTS_INLINE_QOS,
            ),
        ],
    };

    fn create_sample(mut src: crate::xtypes::dynamic_type::DynamicData) -> Self {
        let key = BuiltInTopicKey::try_from_storage(
            src.remove_value(PID_ENDPOINT_GUID as u32)
                .expect("Must exist"),
        )
        .expect("Must match");
        let remote_reader_guid = Guid::new(
            key.value[0..12].try_into().expect("Must match"),
            EntityId::new(
                key.value[12..15].try_into().expect("Must match"),
                key.value[15],
            ),
        );
        Self {
            dds_subscription_data: SubscriptionBuiltinTopicData {
                key,
                participant_key: src
                    .remove_value(PID_PARTICIPANT_GUID as u32)
                    .map_or(BuiltInTopicKey::default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                topic_name: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_TOPIC_NAME as u32).expect("Must exist"),
                )
                .expect("Must match"),
                type_name: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_TYPE_NAME as u32).expect("Must exist"),
                )
                .expect("Must match"),

                durability: src
                    .remove_value(PID_DURABILITY as u32)
                    .map_or(DurabilityQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                deadline: src
                    .remove_value(PID_DEADLINE as u32)
                    .map_or(DeadlineQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                latency_budget: src
                    .remove_value(PID_LATENCY_BUDGET as u32)
                    .map_or(LatencyBudgetQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                liveliness: src
                    .remove_value(PID_LIVELINESS as u32)
                    .map_or(LivelinessQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                reliability: src
                    .remove_value(PID_RELIABILITY as u32)
                    .map_or(DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS, |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                time_based_filter: src
                    .remove_value(PID_TIME_BASED_FILTER as u32)
                    .map_or(TimeBasedFilterQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                user_data: src
                    .remove_value(PID_USER_DATA as u32)
                    .map_or(UserDataQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),

                ownership: src
                    .remove_value(PID_OWNERSHIP as u32)
                    .map_or(OwnershipQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),

                destination_order: src
                    .remove_value(PID_DESTINATION_ORDER as u32)
                    .map_or(DestinationOrderQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),

                presentation: src
                    .remove_value(PID_PRESENTATION as u32)
                    .map_or(PresentationQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                partition: src
                    .remove_value(PID_PARTITION as u32)
                    .map_or(PartitionQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                topic_data: src
                    .remove_value(PID_TOPIC_DATA as u32)
                    .map_or(TopicDataQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                group_data: src
                    .remove_value(PID_GROUP_DATA as u32)
                    .map_or(GroupDataQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                representation: src
                    .remove_value(PID_DATA_REPRESENTATION as u32)
                    .map_or(DataRepresentationQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
            },
            reader_proxy: ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id: src
                    .remove_value(PID_GROUP_ENTITYID as u32)
                    .map_or(ENTITYID_UNKNOWN, |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                unicast_locator_list: src
                    .remove_value(PID_UNICAST_LOCATOR as u32)
                    .map_or(Default::default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                multicast_locator_list: src
                    .remove_value(PID_MULTICAST_LOCATOR as u32)
                    .map_or(Default::default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                expects_inline_qos: src
                    .remove_value(PID_EXPECTS_INLINE_QOS as u32)
                    .map_or(DEFAULT_EXPECTS_INLINE_QOS, |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
            },
        }
    }

    fn create_dynamic_sample(self) -> dust_dds::xtypes::dynamic_type::DynamicData {
        let mut data = dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data();
        data.set_value(
            PID_ENDPOINT_GUID as u32,
            self.dds_subscription_data.key.into_storage(),
        );
        if self.dds_subscription_data.participant_key != BuiltInTopicKey::default() {
            data.set_value(
                PID_PARTICIPANT_GUID as u32,
                self.dds_subscription_data.participant_key.into_storage(),
            );
        }
        data.set_value(
            PID_TOPIC_NAME as u32,
            self.dds_subscription_data.topic_name.into_storage(),
        );
        data.set_value(
            PID_TYPE_NAME as u32,
            self.dds_subscription_data.type_name.into_storage(),
        );
        if self.dds_subscription_data.durability != DurabilityQosPolicy::const_default() {
            data.set_value(
                PID_DURABILITY as u32,
                self.dds_subscription_data.durability.into_storage(),
            );
        }
        if self.dds_subscription_data.deadline != DeadlineQosPolicy::const_default() {
            data.set_value(
                PID_DEADLINE as u32,
                self.dds_subscription_data.deadline.into_storage(),
            );
        }
        if self.dds_subscription_data.latency_budget != LatencyBudgetQosPolicy::const_default() {
            data.set_value(
                PID_LATENCY_BUDGET as u32,
                self.dds_subscription_data.latency_budget.into_storage(),
            );
        }
        if self.dds_subscription_data.liveliness != LivelinessQosPolicy::const_default() {
            data.set_value(
                PID_LIVELINESS as u32,
                self.dds_subscription_data.liveliness.into_storage(),
            );
        }
        if self.dds_subscription_data.reliability
            != DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS
        {
            data.set_value(
                PID_RELIABILITY as u32,
                self.dds_subscription_data.reliability.into_storage(),
            );
        }
        if self.dds_subscription_data.ownership != OwnershipQosPolicy::const_default() {
            data.set_value(
                PID_OWNERSHIP as u32,
                self.dds_subscription_data.ownership.into_storage(),
            );
        }
        if self.dds_subscription_data.destination_order
            != DestinationOrderQosPolicy::const_default()
        {
            data.set_value(
                PID_DESTINATION_ORDER as u32,
                self.dds_subscription_data.destination_order.into_storage(),
            );
        }
        if self.dds_subscription_data.user_data != UserDataQosPolicy::const_default() {
            data.set_value(
                PID_USER_DATA as u32,
                self.dds_subscription_data.user_data.into_storage(),
            );
        }
        if self.dds_subscription_data.time_based_filter != TimeBasedFilterQosPolicy::const_default()
        {
            data.set_value(
                PID_TIME_BASED_FILTER as u32,
                self.dds_subscription_data.time_based_filter.into_storage(),
            );
        }
        if self.dds_subscription_data.presentation != PresentationQosPolicy::const_default() {
            data.set_value(
                PID_PRESENTATION as u32,
                self.dds_subscription_data.presentation.into_storage(),
            );
        }
        if self.dds_subscription_data.partition != PartitionQosPolicy::const_default() {
            data.set_value(
                PID_PARTITION as u32,
                self.dds_subscription_data.partition.into_storage(),
            );
        }
        if self.dds_subscription_data.topic_data != TopicDataQosPolicy::const_default() {
            data.set_value(
                PID_TOPIC_DATA as u32,
                self.dds_subscription_data.topic_data.into_storage(),
            );
        }
        if self.dds_subscription_data.group_data != GroupDataQosPolicy::const_default() {
            data.set_value(
                PID_GROUP_DATA as u32,
                self.dds_subscription_data.group_data.into_storage(),
            );
        }
        if self.dds_subscription_data.representation != DataRepresentationQosPolicy::const_default()
        {
            data.set_value(
                PID_DATA_REPRESENTATION as u32,
                self.dds_subscription_data.representation.into_storage(),
            );
        }
        if self.reader_proxy.remote_group_entity_id != ENTITYID_UNKNOWN {
            data.set_value(
                PID_GROUP_ENTITYID as u32,
                self.reader_proxy.remote_group_entity_id.into_storage(),
            );
        }
        if !self.reader_proxy.unicast_locator_list.is_empty() {
            data.set_value(
                PID_UNICAST_LOCATOR as u32,
                self.reader_proxy.unicast_locator_list.into_storage(),
            );
        }
        if !self.reader_proxy.multicast_locator_list.is_empty() {
            data.set_value(
                PID_MULTICAST_LOCATOR as u32,
                self.reader_proxy.multicast_locator_list.into_storage(),
            );
        }
        if self.reader_proxy.expects_inline_qos != DEFAULT_EXPECTS_INLINE_QOS {
            data.set_value(
                PID_EXPECTS_INLINE_QOS as u32,
                self.reader_proxy.expects_inline_qos.into_storage(),
            );
        }
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        builtin_topics::BuiltInTopicKey,
        infrastructure::type_support::TypeSupport,
        transport::types::{
            BUILT_IN_WRITER_WITH_KEY, EntityId, Guid, USER_DEFINED_READER_WITH_KEY,
            USER_DEFINED_UNKNOWN,
        },
        xtypes::{deserializer::CdrDeserializer, serializer::RtpsPlCdrSerializer},
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
            21, 22, 23, 0xc2, //
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // ,
            2, 0, 0, 0, // ,
            3, 0, 0, 0, // ,
            4, 0, 0, 0, // ,
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        assert_eq!(
            RtpsPlCdrSerializer::serialize(DiscoveredReaderData::TYPE, &data).unwrap(),
            expected
        );
    }

    #[test]
    fn serialize_with_partition() {
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
                partition: PartitionQosPolicy {
                    name: vec![String::from("one"), String::from("two")],
                },
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
            0x29, 0x00, 20, 0, // PID_PARTITION, length
            2, 0, 0, 0, // vec length
            4, 0, 0, 0, // String length
            b'o', b'n', b'e', 0, // String
            4, 0, 0, 0, // String length
            b't', b'w', b'o', 0, // String
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
        assert_eq!(
            RtpsPlCdrSerializer::serialize(DiscoveredReaderData::TYPE, &data).unwrap(),
            expected
        );
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
        }
        .create_dynamic_sample();

        let data = [
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
        ];

        assert_eq!(
            CdrDeserializer::deserialize_builtin(DiscoveredReaderData::TYPE, &data).unwrap(),
            expected
        );
    }
}
