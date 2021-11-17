use rust_dds_api::{
    builtin_topics::TopicBuiltinTopicData,
    infrastructure::qos_policy::{
        DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
        DurabilityServiceQosPolicy, HistoryQosPolicy, LatencyBudgetQosPolicy, LifespanQosPolicy,
        LivelinessQosPolicy, OwnershipQosPolicy, ResourceLimitsQosPolicy, TopicDataQosPolicy,
        TransportPriorityQosPolicy, DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
    },
};

use crate::{
    data_serialize_deserialize::{ParameterList, ParameterSerializer},
    dds_type::{DdsDeserialize, DdsSerialize, DdsType},
};

use super::{
    parameter_id_values::{
        PID_DEADLINE, PID_DURABILITY, PID_DURABILITY_SERVICE, PID_ENDPOINT_GUID,
        PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS, PID_RELIABILITY, PID_RESOURCE_LIMITS,
        PID_TOPIC_NAME, PID_TRANSPORT_PRIORITY, PID_TYPE_NAME,
    },
    serde_remote_dds_api::{
        BuiltInTopicKeyDeserialize, BuiltInTopicKeySerialize, DeadlineQosPolicySerialize,
        DestinationOrderQosPolicySerialize, DurabilityQosPolicySerialize,
        DurabilityServiceQosPolicySerialize, HistoryQosPolicySerialize,
        LatencyBudgetQosPolicySerialize, LifespanQosPolicySerialize, LivelinessQosPolicySerialize,
        OwnershipQosPolicySerialize, ReliabilityQosPolicyDeserializeDataReaderAndTopics,
        ReliabilityQosPolicySerialize, ResourceLimitsQosPolicySerialize,
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
        let mut parameter_list_serializer = ParameterSerializer::<_, E>::new(writer);
        parameter_list_serializer.serialize_payload_header()?;

        parameter_list_serializer
            .serialize_parameter(
                PID_ENDPOINT_GUID,
                &BuiltInTopicKeySerialize(&self.topic_builtin_topic_data.key),
            )
            .unwrap();
        parameter_list_serializer
            .serialize_parameter(PID_TOPIC_NAME, &self.topic_builtin_topic_data.name)
            .unwrap();
        parameter_list_serializer
            .serialize_parameter(PID_TYPE_NAME, &self.topic_builtin_topic_data.type_name)
            .unwrap();
        if self.topic_builtin_topic_data.durability != DurabilityQosPolicy::default() {
            parameter_list_serializer.serialize_parameter(
                PID_DURABILITY,
                &DurabilityQosPolicySerialize(&self.topic_builtin_topic_data.durability),
            )?;
        }
        if self.topic_builtin_topic_data.durability_service != DurabilityServiceQosPolicy::default()
        {
            parameter_list_serializer.serialize_parameter(
                PID_DURABILITY_SERVICE,
                &DurabilityServiceQosPolicySerialize(
                    &self.topic_builtin_topic_data.durability_service,
                ),
            )?;
        }
        if self.topic_builtin_topic_data.deadline != DeadlineQosPolicy::default() {
            parameter_list_serializer.serialize_parameter(
                PID_DEADLINE,
                &DeadlineQosPolicySerialize(&self.topic_builtin_topic_data.deadline),
            )?;
        }
        if self.topic_builtin_topic_data.latency_budget != LatencyBudgetQosPolicy::default() {
            parameter_list_serializer.serialize_parameter(
                PID_LATENCY_BUDGET,
                &LatencyBudgetQosPolicySerialize(&self.topic_builtin_topic_data.latency_budget),
            )?;
        }
        if self.topic_builtin_topic_data.liveliness != LivelinessQosPolicy::default() {
            parameter_list_serializer.serialize_parameter(
                PID_LIVELINESS,
                &LivelinessQosPolicySerialize(&self.topic_builtin_topic_data.liveliness),
            )?;
        }
        if self.topic_builtin_topic_data.reliability
            != DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS
        {
            parameter_list_serializer.serialize_parameter(
                PID_RELIABILITY,
                &ReliabilityQosPolicySerialize(&self.topic_builtin_topic_data.reliability),
            )?;
        }
        if self.topic_builtin_topic_data.transport_priority != TransportPriorityQosPolicy::default()
        {
            parameter_list_serializer.serialize_parameter(
                PID_TRANSPORT_PRIORITY,
                &TransportPriorityQosPolicySerialize(
                    &self.topic_builtin_topic_data.transport_priority,
                ),
            )?;
        }
        if self.topic_builtin_topic_data.lifespan != LifespanQosPolicy::default() {
            parameter_list_serializer.serialize_parameter(
                PID_LIFESPAN,
                &LifespanQosPolicySerialize(&self.topic_builtin_topic_data.lifespan),
            )?;
        }
        if self.topic_builtin_topic_data.destination_order != DestinationOrderQosPolicy::default() {
            parameter_list_serializer.serialize_parameter(
                PID_LIFESPAN,
                &DestinationOrderQosPolicySerialize(
                    &self.topic_builtin_topic_data.destination_order,
                ),
            )?;
        }
        if self.topic_builtin_topic_data.history != HistoryQosPolicy::default() {
            parameter_list_serializer.serialize_parameter(
                PID_LIFESPAN,
                &HistoryQosPolicySerialize(&self.topic_builtin_topic_data.history),
            )?;
        }
        if self.topic_builtin_topic_data.resource_limits != ResourceLimitsQosPolicy::default() {
            parameter_list_serializer.serialize_parameter(
                PID_RESOURCE_LIMITS,
                &ResourceLimitsQosPolicySerialize(&self.topic_builtin_topic_data.resource_limits),
            )?;
        }
        if self.topic_builtin_topic_data.ownership != OwnershipQosPolicy::default() {
            parameter_list_serializer.serialize_parameter(
                PID_RESOURCE_LIMITS,
                &OwnershipQosPolicySerialize(&self.topic_builtin_topic_data.ownership),
            )?;
        }
        if self.topic_builtin_topic_data.topic_data != TopicDataQosPolicy::default() {
            parameter_list_serializer.serialize_parameter(
                PID_RESOURCE_LIMITS,
                &TopicDataQosPolicySerialize(&self.topic_builtin_topic_data.topic_data),
            )?;
        }
        parameter_list_serializer.serialize_sentinel()?;
        Ok(())
    }
}

impl DdsDeserialize<'_> for SedpDiscoveredTopicData {
    fn deserialize(buf: &mut &'_ [u8]) -> rust_dds_api::return_type::DDSResult<Self> {
        let param_list = ParameterList::read(buf).unwrap();

        let key = param_list.get::<BuiltInTopicKeyDeserialize, _>(PID_ENDPOINT_GUID)?;
        let name = param_list.get::<String, _>(PID_TOPIC_NAME)?;
        let type_name = param_list.get::<String, _>(PID_TYPE_NAME)?;
        let reliability = param_list
            .get_or_default::<ReliabilityQosPolicyDeserializeDataReaderAndTopics, _>(
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
    use rust_dds_api::{
        dcps_psm::BuiltInTopicKey,
        infrastructure::qos_policy::{
            DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
            DurabilityServiceQosPolicy, HistoryQosPolicy, LatencyBudgetQosPolicy,
            LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, ResourceLimitsQosPolicy,
            TopicDataQosPolicy, TransportPriorityQosPolicy,
        },
    };

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
