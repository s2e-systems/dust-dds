use crate::{
    builtin_topics::TopicBuiltinTopicData,
    implementation::parameter_list_serde::{
        parameter_list_deserializer::ParameterListDeserializer,
    },
    infrastructure::error::DdsResult,
    topic_definition::type_support::{
        DdsDeserialize, DdsSerializedKey, DdsType, RepresentationType, PL_CDR_LE, RepresentationFormat,
    },
};

pub const DCPS_TOPIC: &str = "DCPSTopic";

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DiscoveredTopicData {
    topic_builtin_topic_data: TopicBuiltinTopicData,
}
impl RepresentationFormat for DiscoveredTopicData {
    const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_LE;
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

impl<'de> DdsDeserialize<'de> for DiscoveredTopicData {
    fn dds_deserialize_parameter_list<E: byteorder::ByteOrder>(
        deserializer: &mut ParameterListDeserializer<'de, E>,
    ) -> DdsResult<Self> {
        Ok(Self {
            topic_builtin_topic_data: DdsDeserialize::dds_deserialize_parameter_list(deserializer)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::builtin_topics::BuiltInTopicKey;
    use crate::implementation::parameter_list_serde::serde_parameter_list_deserializer::dds_deserialize;
    use crate::implementation::parameter_list_serde::serde_parameter_list_serializer::dds_serialize;
    use crate::infrastructure::qos_policy::{
        DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, HistoryQosPolicy,
        LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy,
        ResourceLimitsQosPolicy, TopicDataQosPolicy, TransportPriorityQosPolicy,
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
    };

    use super::*;

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
        let result = dds_serialize(&data).unwrap();
        assert_eq!(result, expected);
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
        let result: DiscoveredTopicData = dds_deserialize(&mut data).unwrap();
        assert_eq!(result, expected);
    }
}
