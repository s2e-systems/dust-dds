use super::parameter_id_values::{
    PID_DATA_REPRESENTATION, PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY,
    PID_ENDPOINT_GUID, PID_HISTORY, PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS,
    PID_OWNERSHIP, PID_RELIABILITY, PID_RESOURCE_LIMITS, PID_TOPIC_DATA, PID_TOPIC_NAME,
    PID_TRANSPORT_PRIORITY, PID_TYPE_NAME,
};
use crate::{
    builtin_topics::TopicBuiltinTopicData,
    implementation::payload_serializer_deserializer::{
        endianness::CdrEndianness, parameter_list_deserializer::ParameterListCdrDeserializer,
        parameter_list_serializer::ParameterListCdrSerializer,
    },
    infrastructure::{
        error::DdsResult, qos_policy::DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
    },
    serialized_payload::parameter_list::deserialize::ParameterListDeserialize,
    topic_definition::type_support::{DdsDeserialize, DdsHasKey, DdsKey, DdsSerialize, DdsTypeXml},
};

pub const DCPS_TOPIC: &str = "DCPSTopic";

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DiscoveredTopicData {
    topic_builtin_topic_data: TopicBuiltinTopicData,
}

impl DdsSerialize for DiscoveredTopicData {
    fn serialize_data(&self) -> DdsResult<Vec<u8>> {
        let mut serializer = ParameterListCdrSerializer::new();
        serializer.write_header()?;

        // topic_builtin_topic_data: TopicBuiltinTopicData:

        serializer.write(PID_ENDPOINT_GUID, &self.topic_builtin_topic_data.key)?;
        serializer.write(PID_TOPIC_NAME, &self.topic_builtin_topic_data.name)?;
        serializer.write(PID_TYPE_NAME, &self.topic_builtin_topic_data.type_name)?;
        serializer.write_with_default(
            PID_DURABILITY,
            &self.topic_builtin_topic_data.durability,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_DEADLINE,
            &self.topic_builtin_topic_data.deadline,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_LATENCY_BUDGET,
            &self.topic_builtin_topic_data.latency_budget,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_LIVELINESS,
            &self.topic_builtin_topic_data.liveliness,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_RELIABILITY,
            &self.topic_builtin_topic_data.reliability,
            &DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
        )?;
        serializer.write_with_default(
            PID_TRANSPORT_PRIORITY,
            &self.topic_builtin_topic_data.transport_priority,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_LIFESPAN,
            &self.topic_builtin_topic_data.lifespan,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_DESTINATION_ORDER,
            &self.topic_builtin_topic_data.destination_order,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_HISTORY,
            &self.topic_builtin_topic_data.history,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_RESOURCE_LIMITS,
            &self.topic_builtin_topic_data.resource_limits,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_OWNERSHIP,
            &self.topic_builtin_topic_data.ownership,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_TOPIC_DATA,
            &self.topic_builtin_topic_data.topic_data,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_DATA_REPRESENTATION,
            &self.topic_builtin_topic_data.representation,
            &Default::default(),
        )?;

        serializer.write_sentinel()?;
        Ok(serializer.writer)
    }
}

impl<'de> DdsDeserialize<'de> for DiscoveredTopicData {
    fn deserialize_data(serialized_data: &'de [u8]) -> DdsResult<Self> {
        let pl_deserializer =
            ParameterListCdrDeserializer::new(serialized_data, CdrEndianness::LittleEndian);
        Ok(Self {
            topic_builtin_topic_data: TopicBuiltinTopicData {
                key: pl_deserializer.read(PID_ENDPOINT_GUID)?,
                name: pl_deserializer.read(PID_TOPIC_NAME)?,
                type_name: pl_deserializer.read(PID_TYPE_NAME)?,
                durability: pl_deserializer
                    .read_with_default(PID_DURABILITY, Default::default())?,
                deadline: pl_deserializer.read_with_default(PID_DEADLINE, Default::default())?,
                latency_budget: pl_deserializer
                    .read_with_default(PID_LATENCY_BUDGET, Default::default())?,
                liveliness: pl_deserializer
                    .read_with_default(PID_LIVELINESS, Default::default())?,
                reliability: pl_deserializer.read_with_default(
                    PID_RELIABILITY,
                    DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
                )?,
                transport_priority: pl_deserializer
                    .read_with_default(PID_TRANSPORT_PRIORITY, Default::default())?,
                lifespan: pl_deserializer.read_with_default(PID_LIFESPAN, Default::default())?,
                destination_order: pl_deserializer
                    .read_with_default(PID_DESTINATION_ORDER, Default::default())?,
                history: pl_deserializer.read_with_default(PID_HISTORY, Default::default())?,
                resource_limits: pl_deserializer
                    .read_with_default(PID_RESOURCE_LIMITS, Default::default())?,
                ownership: pl_deserializer.read_with_default(PID_OWNERSHIP, Default::default())?,
                topic_data: pl_deserializer
                    .read_with_default(PID_TOPIC_DATA, Default::default())?,
                representation: pl_deserializer
                    .read_with_default(PID_DATA_REPRESENTATION, Default::default())?,
            },
        })
    }
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

impl DdsKey for DiscoveredTopicData {
    type Key = [u8; 16];

    fn get_key(&self) -> DdsResult<Self::Key> {
        Ok(self.topic_builtin_topic_data.key().value)
    }

    fn get_key_from_serialized_data(serialized_foo: &[u8]) -> DdsResult<Self::Key> {
        Ok(Self::deserialize_data(serialized_foo)?
            .topic_builtin_topic_data
            .key()
            .value)
    }
}

impl DdsTypeXml for DiscoveredTopicData {
    fn get_type_xml() -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{builtin_topics::BuiltInTopicKey, infrastructure::qos::TopicQos};

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
                TopicQos::default(),
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
        let result = data.serialize_data().unwrap();
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
                TopicQos::default(),
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
        let result = DiscoveredTopicData::deserialize_data(&mut data).unwrap();
        assert_eq!(result, expected);
    }
}
