use xtypes::serializer::SerializeMutableStruct;

use crate::{
    builtin_topics::TopicBuiltinTopicData,
    infrastructure::{
        error::DdsResult, qos_policy::DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
    },
    serialized_payload::parameter_list::{
        deserialize::ParameterListDeserialize, serialize::ParameterListSerialize,
    },
    topic_definition::type_support::{DdsDeserialize, DdsHasKey, DdsKey, DdsSerialize, DdsTypeXml},
};

use super::parameter_id_values::{
    PID_DATA_REPRESENTATION, PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY,
    PID_ENDPOINT_GUID, PID_HISTORY, PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS,
    PID_OWNERSHIP, PID_RELIABILITY, PID_RESOURCE_LIMITS, PID_TOPIC_DATA, PID_TOPIC_NAME,
    PID_TRANSPORT_PRIORITY, PID_TYPE_NAME,
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
impl xtypes::serialize::XTypesSerialize for DiscoveredTopicData {
    fn serialize(
        &self,
        serializer: impl xtypes::serialize::XTypesSerializer,
    ) -> Result<(), xtypes::error::XcdrError> {
        let mut p = serializer.serialize_mutable_struct()?;
        p.serialize_field(
            self.topic_builtin_topic_data.key(),
            PID_ENDPOINT_GUID as u16,
            "key",
        )?;
        p.serialize_field(
            &self.topic_builtin_topic_data.name(),
            PID_TOPIC_NAME as u16,
            "name",
        )?;
        p.serialize_field(
            &self.topic_builtin_topic_data.get_type_name(),
            PID_TYPE_NAME as u16,
            "type_name",
        )?;
        if self.topic_builtin_topic_data.durability() != &Default::default() {
            p.serialize_field(
                self.topic_builtin_topic_data.durability(),
                PID_DURABILITY as u16,
                "durability",
            )?;
        }
        if self.topic_builtin_topic_data.deadline() != &Default::default() {
            p.serialize_field(
                self.topic_builtin_topic_data.deadline(),
                PID_DEADLINE as u16,
                "deadline",
            )?;
        }
        if self.topic_builtin_topic_data.latency_budget() != &Default::default() {
            p.serialize_field(
                self.topic_builtin_topic_data.latency_budget(),
                PID_LATENCY_BUDGET as u16,
                "latency_budget",
            )?;
        }
        if self.topic_builtin_topic_data.liveliness() != &Default::default() {
            p.serialize_field(
                self.topic_builtin_topic_data.liveliness(),
                PID_LIVELINESS as u16,
                "liveliness",
            )?;
        }
        if self.topic_builtin_topic_data.reliability()
            != &DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS
        {
            p.serialize_field(
                self.topic_builtin_topic_data.reliability(),
                PID_RELIABILITY as u16,
                "reliability",
            )?;
        }
        if self.topic_builtin_topic_data.transport_priority() != &Default::default() {
            p.serialize_field(
                self.topic_builtin_topic_data.transport_priority(),
                PID_TRANSPORT_PRIORITY as u16,
                "transport_priority",
            )?;
        }
        if self.topic_builtin_topic_data.lifespan() != &Default::default() {
            p.serialize_field(
                self.topic_builtin_topic_data.lifespan(),
                PID_LIFESPAN as u16,
                "lifespan",
            )?;
        }
        if self.topic_builtin_topic_data.destination_order() != &Default::default() {
            p.serialize_field(
                self.topic_builtin_topic_data.destination_order(),
                PID_DESTINATION_ORDER as u16,
                "destination_order",
            )?;
        }
        if self.topic_builtin_topic_data.history() != &Default::default() {
            p.serialize_field(
                self.topic_builtin_topic_data.history(),
                PID_HISTORY as u16,
                "history",
            )?;
        }
        if self.topic_builtin_topic_data.resource_limits() != &Default::default() {
            p.serialize_field(
                self.topic_builtin_topic_data.resource_limits(),
                PID_RESOURCE_LIMITS as u16,
                "resource_limits",
            )?;
        }
        if self.topic_builtin_topic_data.ownership() != &Default::default() {
            p.serialize_field(
                self.topic_builtin_topic_data.ownership(),
                PID_OWNERSHIP as u16,
                "ownership",
            )?;
        }
        if self.topic_builtin_topic_data.topic_data() != &Default::default() {
            p.serialize_field(
                self.topic_builtin_topic_data.topic_data(),
                PID_TOPIC_DATA as u16,
                "topic_data",
            )?;
        }
        if self.topic_builtin_topic_data.representation() != &Default::default() {
            p.serialize_field(
                self.topic_builtin_topic_data.representation(),
                PID_DATA_REPRESENTATION as u16,
                "representation",
            )?;
        }
        p.end()
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
    use super::*;
    use crate::{builtin_topics::BuiltInTopicKey, infrastructure::qos::TopicQos};
    use xtypes::{serialize::XTypesSerialize, xcdr_serializer::Xcdr1LeSerializer};

    fn serialize_v1_le<T: XTypesSerialize, const N: usize>(v: &T) -> [u8; N] {
        let mut buffer = [0; N];
        v.serialize(&mut Xcdr1LeSerializer::new(&mut buffer))
            .unwrap();
        buffer
    }

    #[test]
    fn xtypes_serialize_all_default() {
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

        let expected = [
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // ,
            2, 0, 0, 0, // ,
            3, 0, 0, 0, // ,
            4, 0, 0, 0, // ,
            0x05, 0x00, 7, 0, // PID_TOPIC_NAME, length
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'a', b'b', 0, 0x00, // DomainTag: string + padding (1 byte)
            0x07, 0x00, 7, 0, // PID_TYPE_NAME, length
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'c', b'd', 0, 0x00, // DomainTag: string + padding (1 byte)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        assert_eq!(serialize_v1_le(&data), expected);
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
