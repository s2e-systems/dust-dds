use rust_dds_api::{
    builtin_topics::TopicBuiltinTopicData,
    infrastructure::qos_policy::{
        DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
        DurabilityServiceQosPolicy, HistoryQosPolicy, LatencyBudgetQosPolicy, LifespanQosPolicy,
        LivelinessQosPolicy, OwnershipQosPolicy, ResourceLimitsQosPolicy, TopicDataQosPolicy,
        TransportPriorityQosPolicy,
    },
};

use crate::dds_type::{DdsDeserialize, DdsSerialize, DdsType};

use super::{
    parameter_id_values::{
        PID_DEADLINE, PID_DURABILITY, PID_DURABILITY_SERVICE, PID_ENDPOINT_GUID,
        PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS, PID_RELIABILITY, PID_RESOURCE_LIMITS,
        PID_TOPIC_NAME, PID_TRANSPORT_PRIORITY, PID_TYPE_NAME,
    },
    parameter_list_deserializer::ParameterListDeserializer,
    parameter_list_serializer::ParameterListSerializer,
    serde_remote_dds_api::{
        BuiltInTopicKeyDeserialize, BuiltInTopicKeySerialize, DeadlineQosPolicySerialize,
        DestinationOrderQosPolicySerialize, DurabilityQosPolicySerialize,
        DurabilityServiceQosPolicySerialize, HistoryQosPolicySerialize,
        LatencyBudgetQosPolicySerialize, LifespanQosPolicySerialize, LivelinessQosPolicySerialize,
        OwnershipQosPolicySerialize, ReliabilityQosPolicyDataReaderAndTopics,
        ReliabilityQosPolicyDataReaderAndTopicsDeserialize,
        ReliabilityQosPolicyDataReaderAndTopicsSerialize, ResourceLimitsQosPolicySerialize,
        TopicDataQosPolicySerialize, TransportPriorityQosPolicySerialize,
    },
};

#[derive(Debug, PartialEq)]
pub struct SedpDiscoveredTopicData {
    pub topic_builtin_topic_data: TopicBuiltinTopicData,
}

impl DdsType for SedpDiscoveredTopicData {
    fn type_name() -> &'static str {
        "SedpDiscoveredTopicData"
    }

    fn has_key() -> bool {
        true
    }
}

impl DdsSerialize for SedpDiscoveredTopicData {
    fn serialize<W: std::io::Write, E: crate::dds_type::Endianness>(
        &self,
        writer: W,
    ) -> rust_dds_api::return_type::DDSResult<()> {
        let mut parameter_list_serializer = ParameterListSerializer::<_, E>::new(writer);
        parameter_list_serializer.serialize_payload_header()?;

        parameter_list_serializer.serialize_parameter::<BuiltInTopicKeySerialize, _>(
            PID_ENDPOINT_GUID,
            &self.topic_builtin_topic_data.key,
        )?;
        parameter_list_serializer.serialize_parameter::<String, _>(
            PID_TOPIC_NAME,
            &self.topic_builtin_topic_data.name,
        )?;
        parameter_list_serializer.serialize_parameter::<String, _>(
            PID_TYPE_NAME,
            &self.topic_builtin_topic_data.type_name,
        )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<DurabilityQosPolicySerialize, _>(
                PID_DURABILITY,
                &self.topic_builtin_topic_data.durability,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<DurabilityServiceQosPolicySerialize, _>(
                PID_DURABILITY_SERVICE,
                &self.topic_builtin_topic_data.durability_service,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<DeadlineQosPolicySerialize, _>(
                PID_DEADLINE,
                &self.topic_builtin_topic_data.deadline,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<LatencyBudgetQosPolicySerialize, _>(
                PID_LATENCY_BUDGET,
                &self.topic_builtin_topic_data.latency_budget,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<LivelinessQosPolicySerialize, _>(
                PID_LIVELINESS,
                &self.topic_builtin_topic_data.liveliness,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<ReliabilityQosPolicyDataReaderAndTopicsSerialize, _>(
                PID_RELIABILITY,
                &ReliabilityQosPolicyDataReaderAndTopics(&self.topic_builtin_topic_data.reliability),
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<TransportPriorityQosPolicySerialize, _>(
                PID_TRANSPORT_PRIORITY,
                &self.topic_builtin_topic_data.transport_priority,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<LifespanQosPolicySerialize, _>(
                PID_LIFESPAN,
                &self.topic_builtin_topic_data.lifespan,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<DestinationOrderQosPolicySerialize, _>(
                PID_LIFESPAN,
                &self.topic_builtin_topic_data.destination_order,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<HistoryQosPolicySerialize, _>(
                PID_LIFESPAN,
                &self.topic_builtin_topic_data.history,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<ResourceLimitsQosPolicySerialize, _>(
                PID_RESOURCE_LIMITS,
                &self.topic_builtin_topic_data.resource_limits,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<OwnershipQosPolicySerialize, _>(
                PID_RESOURCE_LIMITS,
                &self.topic_builtin_topic_data.ownership,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<TopicDataQosPolicySerialize, _>(
                PID_RESOURCE_LIMITS,
                &self.topic_builtin_topic_data.topic_data,
            )?;
        parameter_list_serializer.serialize_sentinel()
    }
}

impl DdsDeserialize<'_> for SedpDiscoveredTopicData {
    fn deserialize(buf: &mut &'_ [u8]) -> rust_dds_api::return_type::DDSResult<Self> {
        let param_list = ParameterListDeserializer::read(buf).unwrap();

        let key = param_list.get::<BuiltInTopicKeyDeserialize, _>(PID_ENDPOINT_GUID)?;
        let name = param_list.get::<String, _>(PID_TOPIC_NAME)?;
        let type_name = param_list.get::<String, _>(PID_TYPE_NAME)?;
        let reliability = param_list
            .get_or_default::<ReliabilityQosPolicyDataReaderAndTopicsDeserialize, _>(
                PID_RELIABILITY,
            )?;

        let topic_builtin_topic_data = TopicBuiltinTopicData {
            key,
            name,
            type_name,
            durability: DurabilityQosPolicy::default(),
            durability_service: DurabilityServiceQosPolicy::default(),
            deadline: DeadlineQosPolicy::default(),
            latency_budget: LatencyBudgetQosPolicy::default(),
            liveliness: LivelinessQosPolicy::default(),
            reliability, //DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
            transport_priority: TransportPriorityQosPolicy::default(),
            lifespan: LifespanQosPolicy::default(),
            ownership: OwnershipQosPolicy::default(),
            destination_order: DestinationOrderQosPolicy::default(),
            history: HistoryQosPolicy::default(),
            resource_limits: ResourceLimitsQosPolicy::default(),
            topic_data: TopicDataQosPolicy::default(),
        };
        Ok(Self {
            topic_builtin_topic_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use rust_dds_api::{dcps_psm::BuiltInTopicKey, infrastructure::qos_policy::{DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS, DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, DurabilityServiceQosPolicy, HistoryQosPolicy, LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, ResourceLimitsQosPolicy, TopicDataQosPolicy, TransportPriorityQosPolicy}};

    use super::*;

    fn to_bytes_le<S: DdsSerialize>(value: &S) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        value
            .serialize::<_, crate::dds_type::LittleEndian>(&mut writer)
            .unwrap();
        writer
    }

    #[test]
    fn serialize_all_default() {
        let data = SedpDiscoveredTopicData {
            topic_builtin_topic_data: TopicBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [1, 2, 3, 4],
                },
                name: "ab".to_string(),
                type_name: "cd".to_string(),
                durability: DurabilityQosPolicy::default(),
                durability_service: DurabilityServiceQosPolicy::default(),
                deadline: DeadlineQosPolicy::default(),
                latency_budget: LatencyBudgetQosPolicy::default(),
                liveliness: LivelinessQosPolicy::default(),
                reliability: DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
                transport_priority: TransportPriorityQosPolicy::default(),
                lifespan: LifespanQosPolicy::default(),
                destination_order: DestinationOrderQosPolicy::default(),
                history: HistoryQosPolicy::default(),
                resource_limits: ResourceLimitsQosPolicy::default(),
                ownership: OwnershipQosPolicy::default(),
                topic_data: TopicDataQosPolicy::default(),
            },
        };

        let expected = vec![
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // long,
            2, 0, 0, 0, // long,
            3, 0, 0, 0, // long,
            4, 0, 0, 0, // long,
            0x05, 0x00, 8, 0, // PID_TOPIC_NAME, length
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'a', b'b', 0, 0x00, // DomainTag: string + padding (1 byte)
            0x07, 0x00, 8, 0, // PID_TYPE_NAME, length
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'c', b'd', 0, 0x00, // DomainTag: string + padding (1 byte)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        assert_eq!(to_bytes_le(&data), expected);
    }

    #[test]
    fn deserialize_all_default() {
        let expected = SedpDiscoveredTopicData {
            topic_builtin_topic_data: TopicBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [1, 2, 3, 4],
                },
                name: "ab".to_string(),
                type_name: "cd".to_string(),
                durability: DurabilityQosPolicy::default(),
                durability_service: DurabilityServiceQosPolicy::default(),
                deadline: DeadlineQosPolicy::default(),
                latency_budget: LatencyBudgetQosPolicy::default(),
                liveliness: LivelinessQosPolicy::default(),
                reliability: DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
                transport_priority: TransportPriorityQosPolicy::default(),
                lifespan: LifespanQosPolicy::default(),
                destination_order: DestinationOrderQosPolicy::default(),
                history: HistoryQosPolicy::default(),
                resource_limits: ResourceLimitsQosPolicy::default(),
                ownership: OwnershipQosPolicy::default(),
                topic_data: TopicDataQosPolicy::default(),
            },
        };

        let mut data = &[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // long,
            2, 0, 0, 0, // long,
            3, 0, 0, 0, // long,
            4, 0, 0, 0, // long,
            0x05, 0x00, 8, 0, // PID_TOPIC_NAME, length
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'a', b'b', 0, 0x00, // DomainTag: string + padding (1 byte)
            0x07, 0x00, 8, 0, // PID_TYPE_NAME, length
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'c', b'd', 0, 0x00, // DomainTag: string + padding (1 byte)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ][..];
        let result: SedpDiscoveredTopicData = DdsDeserialize::deserialize(&mut data).unwrap();
        assert_eq!(result, expected);
    }
}
