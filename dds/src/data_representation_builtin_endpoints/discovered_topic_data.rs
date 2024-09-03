use crate::{
    builtin_topics::TopicBuiltinTopicData,
    infrastructure::error::DdsResult,
    serialized_payload::parameter_list::{
        deserialize::ParameterListDeserialize, serialize::ParameterListSerialize,
    },
    topic_definition::type_support::{DdsDeserialize, DdsHasKey, DdsKey, DdsSerialize, DdsTypeXml},
};

pub const DCPS_TOPIC: &str = "DCPSTopic";

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    DdsSerialize,
    DdsDeserialize,
    ParameterListSerialize,
    ParameterListDeserialize,
)]
#[dust_dds(format = "PL_CDR_LE")]
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
