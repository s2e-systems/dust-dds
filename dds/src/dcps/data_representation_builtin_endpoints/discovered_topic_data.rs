use crate::{
    builtin_topics::TopicBuiltinTopicData,
    dcps::data_representation_builtin_endpoints::{
        parameter_id_values::{
            PID_DATA_REPRESENTATION, PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY,
            PID_ENDPOINT_GUID, PID_HISTORY, PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS,
            PID_OWNERSHIP, PID_RELIABILITY, PID_RESOURCE_LIMITS, PID_TOPIC_DATA, PID_TOPIC_NAME,
            PID_TRANSPORT_PRIORITY, PID_TYPE_INFORMATION, PID_TYPE_NAME,
        },
        rtps_data_representation::CdrResult,
        rtps_data_representation_serialization::ParameterListSerializer,
    },
    infrastructure::qos_policy::{
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS, DataRepresentationQosPolicy,
        DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, HistoryQosPolicy,
        LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy,
        ResourceLimitsQosPolicy, TopicDataQosPolicy, TransportPriorityQosPolicy,
    },
    xtypes::{
        deserializer::deserialize_top_level_type,
        type_support::{Type, TypeSupport},
    },
};

#[derive(Debug, PartialEq, Clone)]
pub struct DiscoveredTopicData {
    pub(crate) topic_builtin_topic_data: TopicBuiltinTopicData,
}

impl DiscoveredTopicData {
    pub fn to_bytes(self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let mut pl = ParameterListSerializer::new(&mut buffer);
        pl.write_header();
        pl.write_xcdr1_parameter(PID_ENDPOINT_GUID, self.topic_builtin_topic_data.key);
        pl.write_cdr_parameter(PID_TOPIC_NAME, self.topic_builtin_topic_data.name);
        pl.write_cdr_parameter(PID_TYPE_NAME, self.topic_builtin_topic_data.type_name);
        if let Some(type_information) = self.topic_builtin_topic_data.type_information {
            pl.write_xcdr2_parameter(PID_TYPE_INFORMATION, type_information);
        }
        if self.topic_builtin_topic_data.durability != DurabilityQosPolicy::default() {
            pl.write_xcdr1_parameter(PID_DURABILITY, self.topic_builtin_topic_data.durability);
        }
        if self.topic_builtin_topic_data.deadline != DeadlineQosPolicy::default() {
            pl.write_xcdr1_parameter(PID_DEADLINE, self.topic_builtin_topic_data.deadline);
        }
        if self.topic_builtin_topic_data.latency_budget != LatencyBudgetQosPolicy::default() {
            pl.write_xcdr1_parameter(
                PID_LATENCY_BUDGET,
                self.topic_builtin_topic_data.latency_budget,
            );
        }
        if self.topic_builtin_topic_data.liveliness != LivelinessQosPolicy::default() {
            pl.write_xcdr1_parameter(PID_LIVELINESS, self.topic_builtin_topic_data.liveliness);
        }
        if self.topic_builtin_topic_data.reliability
            != DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS
        {
            pl.write_xcdr1_parameter(PID_RELIABILITY, self.topic_builtin_topic_data.reliability);
        }
        if self.topic_builtin_topic_data.transport_priority != TransportPriorityQosPolicy::default()
        {
            pl.write_xcdr1_parameter(
                PID_TRANSPORT_PRIORITY,
                self.topic_builtin_topic_data.transport_priority,
            );
        }
        if self.topic_builtin_topic_data.lifespan != LifespanQosPolicy::default() {
            pl.write_xcdr1_parameter(PID_LIFESPAN, self.topic_builtin_topic_data.lifespan);
        }
        if self.topic_builtin_topic_data.destination_order != DestinationOrderQosPolicy::default() {
            pl.write_xcdr1_parameter(
                PID_DESTINATION_ORDER,
                self.topic_builtin_topic_data.destination_order,
            );
        }
        if self.topic_builtin_topic_data.history != HistoryQosPolicy::default() {
            pl.write_xcdr1_parameter(PID_HISTORY, self.topic_builtin_topic_data.history);
        }
        if self.topic_builtin_topic_data.resource_limits != ResourceLimitsQosPolicy::default() {
            pl.write_xcdr1_parameter(
                PID_RESOURCE_LIMITS,
                self.topic_builtin_topic_data.resource_limits,
            );
        }
        if self.topic_builtin_topic_data.ownership != OwnershipQosPolicy::default() {
            pl.write_xcdr1_parameter(PID_OWNERSHIP, self.topic_builtin_topic_data.ownership);
        }
        if self.topic_builtin_topic_data.topic_data != TopicDataQosPolicy::default() {
            pl.write_xcdr1_parameter(PID_TOPIC_DATA, self.topic_builtin_topic_data.topic_data);
        }
        if self.topic_builtin_topic_data.representation != DataRepresentationQosPolicy::default() {
            pl.write_xcdr1_parameter(
                PID_DATA_REPRESENTATION,
                self.topic_builtin_topic_data.representation,
            );
        }
        pl.write_sentinel();
        buffer
    }

    pub fn from_bytes(bytes: &[u8]) -> CdrResult<Self> {
        let topic_builtin_topic_data = TopicBuiltinTopicData::create_sample(
            &mut deserialize_top_level_type(TopicBuiltinTopicData::TYPE, bytes)?,
        );
        Ok(Self {
            topic_builtin_topic_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{builtin_topics::BuiltInTopicKey, infrastructure::qos::TopicQos};

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
                type_information: None,
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
        .to_bytes();

        let expected = [
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // ,
            2, 0, 0, 0, // ,
            3, 0, 0, 0, // ,
            4, 0, 0, 0, // ,
            0x05, 0x00, 8, 0, // PID_TOPIC_NAME, length
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'a', b'b', 0, 0x00, // string + padding (1 byte)
            0x07, 0x00, 8, 0, // PID_TYPE_NAME, length
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'c', b'd', 0, 0x00, // string + padding (1 byte)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        assert_eq!(data, expected.to_vec());
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
                type_information: None,
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
        };

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
        assert_eq!(DiscoveredTopicData::from_bytes(&data).unwrap(), expected);
    }

    #[test]
    fn deserialize_all_default_from_bytes() {
        let topic_qos = TopicQos::default();
        let expected = DiscoveredTopicData {
            topic_builtin_topic_data: TopicBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                name: "ab".to_string(),
                type_name: "cd".to_string(),
                type_information: None,
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
        };

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

        assert_eq!(DiscoveredTopicData::from_bytes(&data).unwrap(), expected);
    }
}
