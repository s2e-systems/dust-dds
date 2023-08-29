use crate::{
    builtin_topics::TopicBuiltinTopicData,
    topic_definition::type_support::{
        DdsGetKey, DdsHasKey, DdsRepresentation, DdsSetKeyFields, Representation,
    },
};

pub const DCPS_TOPIC: &str = "DCPSTopic";

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
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

impl DdsHasKey for DiscoveredTopicData {
    const HAS_KEY: bool = true;
}

impl DdsRepresentation for DiscoveredTopicData {
    const REPRESENTATION: Representation = Representation::PlCdrLe;
}

impl DdsGetKey for DiscoveredTopicData {
    type BorrowedKeyHolder<'a> = [u8; 16];

    fn get_key(&self) -> Self::BorrowedKeyHolder<'_> {
        self.topic_builtin_topic_data.key().value
    }
}

impl DdsSetKeyFields for DiscoveredTopicData {
    type OwningKeyHolder = [u8; 16];

    fn set_key_from_holder(&mut self, _key_holder: Self::OwningKeyHolder) {
        todo!()
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
    use crate::topic_definition::type_support::{
        dds_deserialize_from_bytes, dds_serialize_to_bytes,
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
        let result = dds_serialize_to_bytes(&data).unwrap();
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

        let data = &[
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
        let result = dds_deserialize_from_bytes::<DiscoveredTopicData>(data).unwrap();
        assert_eq!(result, expected);
    }
}
