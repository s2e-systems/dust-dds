use super::{
    parameter_id_values::{
        PID_DATA_REPRESENTATION, PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY,
        PID_ENDPOINT_GUID, PID_HISTORY, PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS,
        PID_OWNERSHIP, PID_RELIABILITY, PID_RESOURCE_LIMITS, PID_TOPIC_DATA, PID_TOPIC_NAME,
        PID_TRANSPORT_PRIORITY, PID_TYPE_NAME,
    },
    payload_serializer_deserializer::parameter_list_serializer::ParameterListCdrSerializer,
};
use crate::{
    builtin_topics::TopicBuiltinTopicData,
    infrastructure::{
        error::DdsResult, qos_policy::DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
    },
    topic_definition::type_support::{DdsDeserialize, DdsSerialize, TypeSupport},
    xtypes::type_object::TypeIdentifier,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DiscoveredTopicData {
    pub(crate) topic_builtin_topic_data: TopicBuiltinTopicData,
}
impl TypeSupport for DiscoveredTopicData {
    fn get_type_name() -> &'static str {
        "DiscoveredTopicData"
    }

    fn get_type() -> impl crate::xtypes::dynamic_type::DynamicType {
        dust_dds::xtypes::type_object::CompleteTypeObject::TkStructure {
            struct_type: dust_dds::xtypes::type_object::CompleteStructType {
                struct_flags: dust_dds::xtypes::type_object::StructTypeFlag {
                    is_final: false,
                    is_appendable: false,
                    is_mutable: true,
                    is_nested: false,
                    is_autoid_hash: false,
                },
                header: dust_dds::xtypes::type_object::CompleteStructHeader {
                    base_type: dust_dds::xtypes::type_object::TypeIdentifier::TkNone,
                    detail: dust_dds::xtypes::type_object::CompleteTypeDetail {
                        ann_builtin: None,
                        ann_custom: None,
                        type_name: "DiscoveredTopicData".to_string(),
                    },
                },
                member_seq: vec![dust_dds::xtypes::type_object::CompleteStructMember {
                    common: dust_dds::xtypes::type_object::CommonStructMember {
                        member_id: 0x5Au32,
                        member_flags: dust_dds::xtypes::type_object::StructMemberFlag {
                            try_construct:
                                dust_dds::xtypes::dynamic_type::TryConstructKind::Discard,
                            is_external: false,
                            is_optional: false,
                            is_must_undestand: true,
                            is_key: true,
                        },
                        member_type_id: dust_dds::xtypes::type_object::TypeIdentifier::TkUint32Type,
                    },
                    detail: dust_dds::xtypes::type_object::CompleteMemberDetail {
                        name: "value".to_string(),
                        ann_builtin: None,
                        ann_custom: None,
                    },
                }],
            },
        }
    }
}

impl DdsSerialize for DiscoveredTopicData {
    fn serialize_data(&self) -> DdsResult<Vec<u8>> {
        let mut serializer = ParameterListCdrSerializer::default();
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
        Ok(Self {
            topic_builtin_topic_data: TopicBuiltinTopicData::deserialize_data(serialized_data)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{builtin_topics::BuiltInTopicKey, infrastructure::qos::TopicQos};

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
