use crate::{
    builtin_topics::TopicBuiltinTopicData,
    implementation::parameter_list_serde::{
        parameter_list_deserializer::ParameterListDeserializer,
        parameter_list_serializer::ParameterListSerializer,
    },
    infrastructure::{
        error::DdsResult,
        qos_policy::{ReliabilityQosPolicy, DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS},
    },
    topic_definition::type_support::{
        DdsDeserialize, DdsSerialize, DdsSerializedKey, DdsType, RepresentationType, PL_CDR_LE,
    },
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

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    derive_more::Into,
    derive_more::From,
)]
pub struct ReliabilityQosPolicyDataReaderAndTopicsDeserialize(pub ReliabilityQosPolicy);
impl Default for ReliabilityQosPolicyDataReaderAndTopicsDeserialize {
    fn default() -> Self {
        Self(DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS)
    }
}

pub const DCPS_TOPIC: &str = "DCPSTopic";

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
pub struct DiscoveredTopicData {
    topic_builtin_topic_data: TopicBuiltinTopicData,
}

impl DiscoveredTopicData {
    pub fn new(topic_builtin_topic_data: TopicBuiltinTopicData) -> Self {
        Self {
            topic_builtin_topic_data,
        }
    }

    pub fn topic_builtin_topic_data(&self) -> &TopicBuiltinTopicData {
        &self.topic_builtin_topic_data
    }
}

impl DdsType for DiscoveredTopicData {
    fn type_name() -> &'static str {
        "DiscoveredTopicData"
    }

    fn has_key() -> bool {
        true
    }

    fn get_serialized_key(&self) -> DdsSerializedKey {
        self.topic_builtin_topic_data.key().value.as_ref().into()
    }

    fn set_key_fields_from_serialized_key(&mut self, _key: &DdsSerializedKey) -> DdsResult<()> {
        if Self::has_key() {
            unimplemented!("DdsType with key must provide an implementation for set_key_fields_from_serialized_key")
        }
        Ok(())
    }
}

impl DdsSerialize for DiscoveredTopicData {
    const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_LE;
    fn dds_serialize<W: std::io::Write>(&self, writer: W) -> DdsResult<()> {
        let mut parameter_list_serializer = ParameterListSerializer::new(writer);
        parameter_list_serializer.serialize_payload_header()?;

        parameter_list_serializer
            .serialize_parameter(PID_ENDPOINT_GUID, self.topic_builtin_topic_data.key())?;
        parameter_list_serializer.serialize_parameter(
            PID_TOPIC_NAME,
            &self.topic_builtin_topic_data.name().to_string(),
        )?;
        parameter_list_serializer.serialize_parameter(
            PID_TYPE_NAME,
            &self.topic_builtin_topic_data.get_type_name().to_string(),
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_DURABILITY,
            self.topic_builtin_topic_data.durability(),
        )?;

        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_DEADLINE,
            self.topic_builtin_topic_data.deadline(),
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_LATENCY_BUDGET,
            self.topic_builtin_topic_data.latency_budget(),
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_LIVELINESS,
            self.topic_builtin_topic_data.liveliness(),
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_RELIABILITY,
            &ReliabilityQosPolicyDataReaderAndTopics(self.topic_builtin_topic_data.reliability()),
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_TRANSPORT_PRIORITY,
            self.topic_builtin_topic_data.transport_priority(),
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_LIFESPAN,
            self.topic_builtin_topic_data.lifespan(),
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_DESTINATION_ORDER,
            self.topic_builtin_topic_data.destination_order(),
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_HISTORY,
            self.topic_builtin_topic_data.history(),
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_RESOURCE_LIMITS,
            self.topic_builtin_topic_data.resource_limits(),
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_OWNERSHIP,
            self.topic_builtin_topic_data.ownership(),
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_TOPIC_DATA,
            self.topic_builtin_topic_data.topic_data(),
        )?;
        parameter_list_serializer.serialize_sentinel()
    }
}

impl DdsDeserialize<'_> for DiscoveredTopicData {
    fn deserialize(buf: &mut &'_ [u8]) -> DdsResult<Self> {
        let param_list = ParameterListDeserializer::read(buf)?;

        let key = param_list.get(PID_ENDPOINT_GUID)?;
        let name = param_list.get(PID_TOPIC_NAME)?;
        let type_name = param_list.get(PID_TYPE_NAME)?;
        let durability = param_list.get_or_default(PID_DURABILITY)?;
        let deadline = param_list.get_or_default(PID_DEADLINE)?;
        let latency_budget = param_list.get_or_default(PID_LATENCY_BUDGET)?;
        let liveliness = param_list.get_or_default(PID_LIVELINESS)?;
        let reliability = param_list
            .get_or_default::<ReliabilityQosPolicyDataReaderAndTopicsDeserialize>(PID_RELIABILITY)?
            .into();
        let transport_priority = param_list.get_or_default(PID_TRANSPORT_PRIORITY)?;
        let lifespan = param_list.get_or_default(PID_LIFESPAN)?;
        let ownership = param_list.get_or_default(PID_OWNERSHIP)?;
        let destination_order = param_list.get_or_default(PID_DESTINATION_ORDER)?;
        let history = param_list.get_or_default(PID_HISTORY)?;
        let resource_limits = param_list.get_or_default(PID_RESOURCE_LIMITS)?;
        let topic_data = param_list.get_or_default(PID_TOPIC_DATA)?;

        Ok(Self {
            topic_builtin_topic_data: TopicBuiltinTopicData::new(
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
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::builtin_topics::BuiltInTopicKey;
    use crate::infrastructure::qos_policy::{
        DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, HistoryQosPolicy,
        LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy,
        ResourceLimitsQosPolicy, TopicDataQosPolicy, TransportPriorityQosPolicy,
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
    };

    use super::*;

    fn to_bytes_le<S: DdsSerialize + serde::Serialize>(value: &S) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        value.dds_serialize(&mut writer).unwrap();
        writer
    }

    #[test]
    fn serialize_all_default() {
        let data = DiscoveredTopicData {
            topic_builtin_topic_data: TopicBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                "ab".to_string(),
                "cd".to_string(),
                DurabilityQosPolicy::default(),
                DeadlineQosPolicy::default(),
                LatencyBudgetQosPolicy::default(),
                LivelinessQosPolicy::default(),
                DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
                TransportPriorityQosPolicy::default(),
                LifespanQosPolicy::default(),
                DestinationOrderQosPolicy::default(),
                HistoryQosPolicy::default(),
                ResourceLimitsQosPolicy::default(),
                OwnershipQosPolicy::default(),
                TopicDataQosPolicy::default(),
            ),
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
            topic_builtin_topic_data: TopicBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                "ab".to_string(),
                "cd".to_string(),
                DurabilityQosPolicy::default(),
                DeadlineQosPolicy::default(),
                LatencyBudgetQosPolicy::default(),
                LivelinessQosPolicy::default(),
                DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
                TransportPriorityQosPolicy::default(),
                LifespanQosPolicy::default(),
                DestinationOrderQosPolicy::default(),
                HistoryQosPolicy::default(),
                ResourceLimitsQosPolicy::default(),
                OwnershipQosPolicy::default(),
                TopicDataQosPolicy::default(),
            ),
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
