use crate::builtin_topics::{BuiltInTopicKey, TopicBuiltinTopicData};
use crate::infrastructure::error::DdsResult;
use crate::infrastructure::qos_policy::{
    DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, HistoryQosPolicy,
    LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy,
    ReliabilityQosPolicy, ResourceLimitsQosPolicy, TopicDataQosPolicy, TransportPriorityQosPolicy,
    DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
};
use crate::{
    implementation::parameter_list_serde::{
        parameter_list_deserializer::ParameterListDeserializer,
        parameter_list_serializer::ParameterListSerializer,
    },
    topic_definition::type_support::{DdsDeserialize, DdsSerialize, DdsType, Endianness},
};

use super::parameter_id_values::{
    PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY, PID_ENDPOINT_GUID, PID_HISTORY,
    PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS, PID_OWNERSHIP, PID_RELIABILITY,
    PID_RESOURCE_LIMITS, PID_TOPIC_DATA, PID_TOPIC_NAME, PID_TRANSPORT_PRIORITY, PID_TYPE_NAME,
};

#[derive(Debug, PartialEq, Eq, serde::Serialize, derive_more::From, derive_more::Into)]
pub struct ReliabilityQosPolicyDataReaderAndTopics<'a>(pub &'a ReliabilityQosPolicy);
impl<'a> Default for ReliabilityQosPolicyDataReaderAndTopics<'a> {
    fn default() -> Self {
        Self(&DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS)
    }
}
#[derive(Debug, PartialEq, Eq, serde::Serialize, derive_more::From)]
pub struct ReliabilityQosPolicyDataReaderAndTopicsSerialize<'a>(
    pub &'a ReliabilityQosPolicyDataReaderAndTopics<'a>,
);

#[derive(Debug, PartialEq, Eq, serde::Deserialize, derive_more::Into)]
pub struct ReliabilityQosPolicyDataReaderAndTopicsDeserialize(pub ReliabilityQosPolicy);
impl Default for ReliabilityQosPolicyDataReaderAndTopicsDeserialize {
    fn default() -> Self {
        Self(DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS)
    }
}

pub const DCPS_TOPIC: &str = "DCPSTopic";

#[derive(Debug, PartialEq, Eq)]
pub struct DiscoveredTopicData {
    pub topic_builtin_topic_data: TopicBuiltinTopicData,
}

impl DdsType for DiscoveredTopicData {
    fn type_name() -> &'static str {
        "DiscoveredTopicData"
    }

    fn has_key() -> bool {
        true
    }

    fn get_serialized_key<E: Endianness>(&self) -> Vec<u8> {
        self.topic_builtin_topic_data.key.value.to_vec()
    }

    fn set_key_fields_from_serialized_key(&mut self, _key: &[u8]) -> DdsResult<()> {
        if Self::has_key() {
            unimplemented!("DdsType with key must provide an implementation for set_key_fields_from_serialized_key")
        }
        Ok(())
    }
}

impl DdsSerialize for DiscoveredTopicData {
    fn serialize<W: std::io::Write, E: Endianness>(&self, writer: W) -> DdsResult<()> {
        let mut parameter_list_serializer = ParameterListSerializer::<_, E>::new(writer);
        parameter_list_serializer.serialize_payload_header()?;

        parameter_list_serializer.serialize_parameter::<&BuiltInTopicKey, _>(
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
        parameter_list_serializer.serialize_parameter_if_not_default::<&DurabilityQosPolicy, _>(
            PID_DURABILITY,
            &self.topic_builtin_topic_data.durability,
        )?;

        parameter_list_serializer.serialize_parameter_if_not_default::<&DeadlineQosPolicy, _>(
            PID_DEADLINE,
            &self.topic_builtin_topic_data.deadline,
        )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<&LatencyBudgetQosPolicy, _>(
                PID_LATENCY_BUDGET,
                &self.topic_builtin_topic_data.latency_budget,
            )?;
        parameter_list_serializer.serialize_parameter_if_not_default::<&LivelinessQosPolicy, _>(
            PID_LIVELINESS,
            &self.topic_builtin_topic_data.liveliness,
        )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<ReliabilityQosPolicyDataReaderAndTopicsSerialize, _>(
                PID_RELIABILITY,
                &ReliabilityQosPolicyDataReaderAndTopics(&self.topic_builtin_topic_data.reliability),
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<&TransportPriorityQosPolicy, _>(
                PID_TRANSPORT_PRIORITY,
                &self.topic_builtin_topic_data.transport_priority,
            )?;
        parameter_list_serializer.serialize_parameter_if_not_default::<&LifespanQosPolicy, _>(
            PID_LIFESPAN,
            &self.topic_builtin_topic_data.lifespan,
        )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<&DestinationOrderQosPolicy, _>(
                PID_DESTINATION_ORDER,
                &self.topic_builtin_topic_data.destination_order,
            )?;
        parameter_list_serializer.serialize_parameter_if_not_default::<&HistoryQosPolicy, _>(
            PID_HISTORY,
            &self.topic_builtin_topic_data.history,
        )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<&ResourceLimitsQosPolicy, _>(
                PID_RESOURCE_LIMITS,
                &self.topic_builtin_topic_data.resource_limits,
            )?;
        parameter_list_serializer.serialize_parameter_if_not_default::<&OwnershipQosPolicy, _>(
            PID_OWNERSHIP,
            &self.topic_builtin_topic_data.ownership,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default::<&TopicDataQosPolicy, _>(
            PID_TOPIC_DATA,
            &self.topic_builtin_topic_data.topic_data,
        )?;
        parameter_list_serializer.serialize_sentinel()
    }
}

impl DdsDeserialize<'_> for DiscoveredTopicData {
    fn deserialize(buf: &mut &'_ [u8]) -> DdsResult<Self> {
        let param_list = ParameterListDeserializer::read(buf).unwrap();

        let key = param_list.get::<BuiltInTopicKey, _>(PID_ENDPOINT_GUID)?;
        let name = param_list.get::<String, _>(PID_TOPIC_NAME)?;
        let type_name = param_list.get::<String, _>(PID_TYPE_NAME)?;
        let durability = param_list.get_or_default::<DurabilityQosPolicy, _>(PID_DURABILITY)?;
        let deadline = param_list.get_or_default::<DeadlineQosPolicy, _>(PID_DEADLINE)?;
        let latency_budget =
            param_list.get_or_default::<LatencyBudgetQosPolicy, _>(PID_LATENCY_BUDGET)?;
        let liveliness = param_list.get_or_default::<LivelinessQosPolicy, _>(PID_LIVELINESS)?;
        let reliability = param_list
            .get_or_default::<ReliabilityQosPolicyDataReaderAndTopicsDeserialize, _>(
                PID_RELIABILITY,
            )?;
        let transport_priority =
            param_list.get_or_default::<TransportPriorityQosPolicy, _>(PID_TRANSPORT_PRIORITY)?;
        let lifespan = param_list.get_or_default::<LifespanQosPolicy, _>(PID_LIFESPAN)?;
        let ownership = param_list.get_or_default::<OwnershipQosPolicy, _>(PID_OWNERSHIP)?;
        let destination_order =
            param_list.get_or_default::<DestinationOrderQosPolicy, _>(PID_DESTINATION_ORDER)?;
        let history = param_list.get_or_default::<HistoryQosPolicy, _>(PID_HISTORY)?;
        let resource_limits =
            param_list.get_or_default::<ResourceLimitsQosPolicy, _>(PID_RESOURCE_LIMITS)?;
        let topic_data = param_list.get_or_default::<TopicDataQosPolicy, _>(PID_TOPIC_DATA)?;

        Ok(Self {
            topic_builtin_topic_data: TopicBuiltinTopicData {
                key,
                name,
                type_name,
                durability,
                deadline,
                latency_budget,
                liveliness,
                reliability,
                transport_priority,
                lifespan,
                ownership,
                destination_order,
                history,
                resource_limits,
                topic_data,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::qos_policy::{
        DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, HistoryQosPolicy,
        LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy,
        ResourceLimitsQosPolicy, TopicDataQosPolicy, TransportPriorityQosPolicy,
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
    };

    use crate::topic_definition::type_support::LittleEndian;

    use super::*;

    fn to_bytes_le<S: DdsSerialize>(value: &S) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        value.serialize::<_, LittleEndian>(&mut writer).unwrap();
        writer
    }

    #[test]
    fn serialize_all_default() {
        let data = DiscoveredTopicData {
            topic_builtin_topic_data: TopicBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                name: "ab".to_string(),
                type_name: "cd".to_string(),
                durability: DurabilityQosPolicy::default(),
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
        assert_eq!(to_bytes_le(&data), expected);
    }

    #[test]
    fn deserialize_all_default() {
        let expected = DiscoveredTopicData {
            topic_builtin_topic_data: TopicBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                name: "ab".to_string(),
                type_name: "cd".to_string(),
                durability: DurabilityQosPolicy::default(),
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
        ][..];
        let result: DiscoveredTopicData = DdsDeserialize::deserialize(&mut data).unwrap();
        assert_eq!(result, expected);
    }
}
