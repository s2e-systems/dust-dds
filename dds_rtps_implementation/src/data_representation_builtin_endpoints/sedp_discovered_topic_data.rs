use rust_dds_api::{builtin_topics::TopicBuiltinTopicData, infrastructure::qos_policy::{DurabilityQosPolicy, DurabilityServiceQosPolicy}};

use crate::{data_serialize_deserialize::ParameterSerializer, dds_type::{DdsDeserialize, DdsSerialize, DdsType}};

use super::{dds_serialize_deserialize_impl::{BuiltInTopicKeySerialize, DurabilityQosPolicySerialize, DurabilityServiceQosPolicySerialize}, parameter_id_values::{PID_DURABILITY, PID_DURABILITY_SERVICE, PID_ENDPOINT_GUID, PID_TOPIC_NAME, PID_TYPE_NAME}};

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

        parameter_list_serializer
            .serialize_parameter(PID_ENDPOINT_GUID, &BuiltInTopicKeySerialize(&self.topic_builtin_topic_data.key))
            .unwrap();
        parameter_list_serializer
            .serialize_parameter(PID_TOPIC_NAME, &self.topic_builtin_topic_data.name)
            .unwrap();
        parameter_list_serializer
            .serialize_parameter(PID_TYPE_NAME, &self.topic_builtin_topic_data.type_name)
            .unwrap();
        if self.topic_builtin_topic_data.durability != DurabilityQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(PID_DURABILITY, &DurabilityQosPolicySerialize(&self.topic_builtin_topic_data.durability))
                .unwrap();
        }
        if self.topic_builtin_topic_data.durability_service != DurabilityServiceQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(PID_DURABILITY_SERVICE, &DurabilityServiceQosPolicySerialize(&self.topic_builtin_topic_data.durability_service))
                .unwrap();
        }
        // pub deadline: DeadlineQosPolicy,
        // pub latency_budget: LatencyBudgetQosPolicy,
        // pub liveliness: LivelinessQosPolicy,
        // pub reliability: ReliabilityQosPolicy,
        // pub transport_priority: TransportPriorityQosPolicy,
        // pub lifespan: LifespanQosPolicy,
        // pub destination_order: DestinationOrderQosPolicy,
        // pub history: HistoryQosPolicy,
        // pub resource_limits: ResourceLimitsQosPolicy,
        // pub ownership: OwnershipQosPolicy,
        // pub topic_data: TopicDataQosPolicy,

        Ok(())
    }
}

impl DdsDeserialize<'_> for SedpDiscoveredTopicData {
    fn deserialize(_buf: &mut &'_ [u8]) -> rust_dds_api::return_type::DDSResult<Self> {
        todo!()
    }
}


#[cfg(test)]
mod tests {
    use rust_dds_api::{
        dcps_psm::{BuiltInTopicKey, Duration},
        infrastructure::qos_policy::{
            DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
            DurabilityServiceQosPolicy, HistoryQosPolicy, LatencyBudgetQosPolicy,
            LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind, ResourceLimitsQosPolicy, TopicDataQosPolicy,
            TransportPriorityQosPolicy,
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
                key: BuiltInTopicKey { value: [1, 2, 3] },
                name: "ab".to_string(),
                type_name: "cd".to_string(),
                durability: DurabilityQosPolicy::default(),
                durability_service: DurabilityServiceQosPolicy::default(),
                deadline: DeadlineQosPolicy::default(),
                latency_budget: LatencyBudgetQosPolicy::default(),
                liveliness: LivelinessQosPolicy::default(),
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos,
                    max_blocking_time: Duration::new(7, 476),
                },
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
            0x5a, 0x00, 12, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // long,
            2, 0, 0, 0, // long,
            3, 0, 0, 0, // long,
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
}