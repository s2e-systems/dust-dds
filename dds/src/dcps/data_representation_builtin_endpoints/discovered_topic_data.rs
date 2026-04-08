use crate::{
    builtin_topics::{BuiltInTopicKey, TopicBuiltinTopicData},
    dcps::data_representation_builtin_endpoints::{
        ConvenienceTypeBuilder,
        parameter_id_values::{
            PID_DATA_REPRESENTATION, PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY,
            PID_ENDPOINT_GUID, PID_HISTORY, PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS,
            PID_OWNERSHIP, PID_RELIABILITY, PID_RESOURCE_LIMITS, PID_TOPIC_DATA, PID_TOPIC_NAME,
            PID_TRANSPORT_PRIORITY, PID_TYPE_NAME,
        },
    },
    infrastructure::{
        qos_policy::{
            DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS, DataRepresentationQosPolicy,
            DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, HistoryQosPolicy,
            LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy,
            ReliabilityQosPolicy, ResourceLimitsQosPolicy, TopicDataQosPolicy,
            TransportPriorityQosPolicy,
        },
        type_support::TypeSupport,
    },
    xtypes::{
        data_storage::DataStorageMapping,
        dynamic_type::{DynamicType, StaticTypeInformation},
    },
};
use alloc::string::String;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DiscoveredTopicData {
    pub(crate) topic_builtin_topic_data: TopicBuiltinTopicData,
}

impl TypeSupport for DiscoveredTopicData {
    const TYPE_NAME: &'static str = "TopicBuiltinTopicData";

    const r#TYPE: &'static dyn DynamicType = &StaticTypeInformation {
        descriptor: &ConvenienceTypeBuilder::type_descriptor(Self::TYPE_NAME),
        member_list: &[
            ConvenienceTypeBuilder::key_member::<BuiltInTopicKey>(0, "key", PID_ENDPOINT_GUID),
            ConvenienceTypeBuilder::member::<String>(1, "name", PID_TOPIC_NAME),
            ConvenienceTypeBuilder::member::<String>(2, "type_name", PID_TYPE_NAME),
            ConvenienceTypeBuilder::member_with_default::<DurabilityQosPolicy>(
                3,
                "durability",
                PID_DURABILITY,
            ),
            ConvenienceTypeBuilder::member_with_default::<DeadlineQosPolicy>(
                4,
                "deadline",
                PID_DEADLINE,
            ),
            ConvenienceTypeBuilder::member_with_default::<LatencyBudgetQosPolicy>(
                5,
                "latency_budget",
                PID_LATENCY_BUDGET,
            ),
            ConvenienceTypeBuilder::member_with_default::<LivelinessQosPolicy>(
                6,
                "liveliness",
                PID_LIVELINESS,
            ),
            ConvenienceTypeBuilder::member_with_default::<ReliabilityQosPolicy>(
                7,
                "reliability",
                PID_RELIABILITY,
            ),
            ConvenienceTypeBuilder::member_with_default::<TransportPriorityQosPolicy>(
                8,
                "transport_priority",
                PID_TRANSPORT_PRIORITY,
            ),
            ConvenienceTypeBuilder::member_with_default::<LifespanQosPolicy>(
                9,
                "lifespan",
                PID_LIFESPAN,
            ),
            ConvenienceTypeBuilder::member_with_default::<DestinationOrderQosPolicy>(
                10,
                "destination_order",
                PID_DESTINATION_ORDER,
            ),
            ConvenienceTypeBuilder::member_with_default::<HistoryQosPolicy>(
                11,
                "history",
                PID_HISTORY,
            ),
            ConvenienceTypeBuilder::member_with_default::<ResourceLimitsQosPolicy>(
                12,
                "resource_limits",
                PID_RESOURCE_LIMITS,
            ),
            ConvenienceTypeBuilder::member_with_default::<OwnershipQosPolicy>(
                13,
                "ownership",
                PID_OWNERSHIP,
            ),
            ConvenienceTypeBuilder::member_with_default::<TopicDataQosPolicy>(
                14,
                "topic_data",
                PID_TOPIC_DATA,
            ),
            ConvenienceTypeBuilder::member_with_default::<DataRepresentationQosPolicy>(
                15,
                "representation",
                PID_DATA_REPRESENTATION,
            ),
        ],
    };

    fn create_sample(mut src: crate::xtypes::dynamic_type::DynamicData) -> Self {
        Self {
            topic_builtin_topic_data: TopicBuiltinTopicData {
                key: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_ENDPOINT_GUID as u32)
                        .expect("Must exist"),
                )
                .expect("Must match"),
                type_name: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_TYPE_NAME as u32).expect("Must exist"),
                )
                .expect("Must match"),
                name: DataStorageMapping::try_from_storage(
                    src.remove_value(PID_TOPIC_NAME as u32).expect("Must exist"),
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
                destination_order: src
                    .remove_value(PID_DESTINATION_ORDER as u32)
                    .map_or(DestinationOrderQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                topic_data: src
                    .remove_value(PID_TOPIC_DATA as u32)
                    .map_or(TopicDataQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                representation: src
                    .remove_value(PID_DATA_REPRESENTATION as u32)
                    .map_or(DataRepresentationQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                transport_priority: src
                    .remove_value(PID_TRANSPORT_PRIORITY as u32)
                    .map_or(TransportPriorityQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                lifespan: src
                    .remove_value(PID_LIFESPAN as u32)
                    .map_or(LifespanQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                history: src
                    .remove_value(PID_HISTORY as u32)
                    .map_or(HistoryQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                resource_limits: src
                    .remove_value(PID_RESOURCE_LIMITS as u32)
                    .map_or(ResourceLimitsQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
                ownership: src
                    .remove_value(PID_OWNERSHIP as u32)
                    .map_or(OwnershipQosPolicy::const_default(), |x| {
                        DataStorageMapping::try_from_storage(x).expect("Must match")
                    }),
            },
        }
    }

    fn create_dynamic_sample(self) -> dust_dds::xtypes::dynamic_type::DynamicData {
        let mut data = dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data();
        data.set_value(
            PID_ENDPOINT_GUID as u32,
            self.topic_builtin_topic_data.key.into_storage(),
        );
        data.set_value(
            PID_TOPIC_NAME as u32,
            self.topic_builtin_topic_data.name.into_storage(),
        );
        data.set_value(
            PID_TYPE_NAME as u32,
            self.topic_builtin_topic_data.type_name.into_storage(),
        );
        if self.topic_builtin_topic_data.durability != Default::default() {
            data.set_value(
                PID_DURABILITY as u32,
                self.topic_builtin_topic_data.durability.into_storage(),
            );
        }
        if self.topic_builtin_topic_data.deadline != Default::default() {
            data.set_value(
                PID_DEADLINE as u32,
                self.topic_builtin_topic_data.deadline.into_storage(),
            );
        }
        if self.topic_builtin_topic_data.latency_budget != Default::default() {
            data.set_value(
                PID_LATENCY_BUDGET as u32,
                self.topic_builtin_topic_data.latency_budget.into_storage(),
            );
        }
        if self.topic_builtin_topic_data.liveliness != Default::default() {
            data.set_value(
                PID_LIVELINESS as u32,
                self.topic_builtin_topic_data.liveliness.into_storage(),
            );
        }
        if self.topic_builtin_topic_data.reliability
            != DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS
        {
            data.set_value(
                PID_RELIABILITY as u32,
                self.topic_builtin_topic_data.reliability.into_storage(),
            );
        }
        if self.topic_builtin_topic_data.transport_priority != Default::default() {
            data.set_value(
                PID_TRANSPORT_PRIORITY as u32,
                self.topic_builtin_topic_data
                    .transport_priority
                    .into_storage(),
            );
        }
        if self.topic_builtin_topic_data.lifespan != Default::default() {
            data.set_value(
                PID_LIFESPAN as u32,
                self.topic_builtin_topic_data.lifespan.into_storage(),
            );
        }
        if self.topic_builtin_topic_data.destination_order != Default::default() {
            data.set_value(
                PID_DESTINATION_ORDER as u32,
                self.topic_builtin_topic_data
                    .destination_order
                    .into_storage(),
            );
        }
        if self.topic_builtin_topic_data.history != Default::default() {
            data.set_value(
                PID_HISTORY as u32,
                self.topic_builtin_topic_data.history.into_storage(),
            );
        }
        if self.topic_builtin_topic_data.resource_limits != Default::default() {
            data.set_value(
                PID_RESOURCE_LIMITS as u32,
                self.topic_builtin_topic_data.resource_limits.into_storage(),
            );
        }
        if self.topic_builtin_topic_data.ownership != Default::default() {
            data.set_value(
                PID_OWNERSHIP as u32,
                self.topic_builtin_topic_data.ownership.into_storage(),
            );
        }
        if self.topic_builtin_topic_data.topic_data != Default::default() {
            data.set_value(
                PID_TOPIC_DATA as u32,
                self.topic_builtin_topic_data.topic_data.into_storage(),
            );
        }
        if self.topic_builtin_topic_data.representation != Default::default() {
            data.set_value(
                PID_DATA_REPRESENTATION as u32,
                self.topic_builtin_topic_data.representation.into_storage(),
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
        infrastructure::qos::TopicQos,
        xtypes::{deserializer::CdrDeserializer, serializer::RtpsPlCdrSerializer},
    };

    #[test]
    fn serialize_all_default() {
        let topic_qos = TopicQos::default();
        let data = DiscoveredTopicData {
            topic_builtin_topic_data: TopicBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                name: "ab".to_string(),
                type_name: "cd".to_string(),
                durability: topic_qos.durability,
                deadline: topic_qos.deadline,
                latency_budget: topic_qos.latency_budget,
                liveliness: topic_qos.liveliness,
                reliability: topic_qos.reliability,
                transport_priority: topic_qos.transport_priority,
                lifespan: topic_qos.lifespan,
                destination_order: topic_qos.destination_order,
                history: topic_qos.history,
                resource_limits: topic_qos.resource_limits,
                ownership: topic_qos.ownership,
                topic_data: topic_qos.topic_data,
                representation: topic_qos.representation,
            },
        }
        .create_dynamic_sample();

        let expected = [
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x05, 0x00, 8, 0, // PID_TOPIC_NAME, length
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'a', b'b', 0, 0x00, // string + padding (1 byte)
            0x07, 0x00, 8, 0, // PID_TYPE_NAME, length
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'c', b'd', 0, 0x00, // string + padding (1 byte)
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // ,
            2, 0, 0, 0, // ,
            3, 0, 0, 0, // ,
            4, 0, 0, 0, // ,
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        assert_eq!(
            RtpsPlCdrSerializer::serialize(DiscoveredTopicData::TYPE, &data).unwrap(),
            expected
        );
    }

    #[test]
    fn deserialize_all_default() {
        let topic_qos = TopicQos::default();
        let expected = DiscoveredTopicData {
            topic_builtin_topic_data: TopicBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                name: "ab".to_string(),
                type_name: "cd".to_string(),
                durability: topic_qos.durability,
                deadline: topic_qos.deadline,
                latency_budget: topic_qos.latency_budget,
                liveliness: topic_qos.liveliness,
                reliability: topic_qos.reliability,
                transport_priority: topic_qos.transport_priority,
                lifespan: topic_qos.lifespan,
                destination_order: topic_qos.destination_order,
                history: topic_qos.history,
                resource_limits: topic_qos.resource_limits,
                ownership: topic_qos.ownership,
                topic_data: topic_qos.topic_data,
                representation: topic_qos.representation,
            },
        }
        .create_dynamic_sample();

        let data = [
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // ,
            2, 0, 0, 0, // ,
            3, 0, 0, 0, // ,
            4, 0, 0, 0, // ,
            0x05, 0x00, 8, 0, // PID_TOPIC_NAME, length
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'a', b'b', 0, 0x00, // DomainTag: string + padding (1 byte)
            0x07, 0x00, 8, 0, // PID_TYPE_NAME, length
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'c', b'd', 0, 0x00, // DomainTag: string + padding (1 byte)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        assert_eq!(
            CdrDeserializer::deserialize_builtin(DiscoveredTopicData::TYPE, &data).unwrap(),
            expected
        );
    }
}
