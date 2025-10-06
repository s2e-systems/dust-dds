use crate::{
    builtin_topics::TopicBuiltinTopicData,
    infrastructure::{
        error::DdsResult,
        type_support::{DdsDeserialize, TypeSupport},
    },
};

#[derive(Debug, PartialEq, Eq, Clone, TypeSupport)]
pub struct DiscoveredTopicData {
    #[dust_dds(key)]
    pub(crate) topic_builtin_topic_data: TopicBuiltinTopicData,
}

impl<'de> DdsDeserialize<'de> for DiscoveredTopicData {
    fn deserialize_data(serialized_data: &'de [u8]) -> DdsResult<Self> {
        Ok(Self {
            topic_builtin_topic_data: TopicBuiltinTopicData::deserialize_data(serialized_data)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        builtin_topics::BuiltInTopicKey,
        infrastructure::qos::TopicQos,
        xtypes::{pl_cdr_serializer::PlCdrLeSerializer, serialize::XTypesSerialize},
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
        };

        let expected = vec![
            // 0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
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
        let dynamic_data = data.create_dynamic_sample();
        let mut buffer = Vec::new();
        let mut serializer = PlCdrLeSerializer::new(&mut buffer);
        dynamic_data.serialize(&mut serializer).unwrap();

        assert_eq!(buffer, expected);
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
        let result = DiscoveredTopicData::deserialize_data(&mut data).unwrap();
        assert_eq!(result, expected);
    }
}
