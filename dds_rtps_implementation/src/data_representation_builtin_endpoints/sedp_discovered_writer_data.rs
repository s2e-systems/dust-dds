use std::io::Write;

use rust_dds_api::{
    builtin_topics::PublicationBuiltinTopicData,
    infrastructure::qos_policy::{
        DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
        DurabilityServiceQosPolicy, GroupDataQosPolicy, LatencyBudgetQosPolicy, LifespanQosPolicy,
        LivelinessQosPolicy, OwnershipQosPolicy, OwnershipStrengthQosPolicy, PartitionQosPolicy,
        PresentationQosPolicy, TopicDataQosPolicy, UserDataQosPolicy,
    },
    return_type::DDSResult,
};
use rust_rtps_pim::{behavior::reader::writer_proxy::RtpsWriterProxy, structure::types::Locator};

use crate::{
    data_serialize_deserialize::ParameterSerializer,
    dds_type::{DdsDeserialize, DdsSerialize, DdsType, Endianness},
};

use super::{dds_serialize_deserialize_impl::{BuiltInTopicKeySerialize, DeadlineQosPolicySerialize, DestinationOrderQosPolicySerialize, DurabilityQosPolicySerialize, DurabilityServiceQosPolicySerialize, GroupDataQosPolicySerialize, LatencyBudgetQosPolicySerialize, LifespanQosPolicySerialize, LivelinessQosPolicySerialize, OwnershipQosPolicySerialize, OwnershipStrengthQosPolicySerialize, PartitionQosPolicySerialize, PresentationQosPolicySerialize, ReliabilityQosPolicySerialize, TopicDataQosPolicySerialize, UserDataQosPolicySerialize}, parameter_id_values::{PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY, PID_DURABILITY_SERVICE, PID_ENDPOINT_GUID, PID_GROUP_DATA, PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS, PID_OWNERSHIP, PID_OWNERSHIP_STRENGTH, PID_PARTICIPANT_GUID, PID_PARTITION, PID_PRESENTATION, PID_RELIABILITY, PID_TOPIC_DATA, PID_TOPIC_NAME, PID_TYPE_NAME, PID_USER_DATA}};

pub struct SedpDiscoveredWriterData {
    pub writer_proxy: RtpsWriterProxy<Vec<Locator>>,
    pub publication_builtin_topic_data: PublicationBuiltinTopicData,
}

impl DdsType for SedpDiscoveredWriterData {
    fn type_name() -> &'static str {
        "SedpDiscoveredWriterData"
    }

    fn has_key() -> bool {
        true
    }
}

impl DdsSerialize for SedpDiscoveredWriterData {
    fn serialize<W: Write, E: Endianness>(&self, writer: W) -> DDSResult<()> {
        let mut parameter_list_serializer = ParameterSerializer::<_, E>::new(writer);

        parameter_list_serializer
            .serialize_parameter(
                PID_ENDPOINT_GUID,
                &BuiltInTopicKeySerialize(&self.publication_builtin_topic_data.key),
            )
            .unwrap();
        parameter_list_serializer
            .serialize_parameter(
                PID_PARTICIPANT_GUID,
                &BuiltInTopicKeySerialize(&self.publication_builtin_topic_data.participant_key),
            )
            .unwrap();
        parameter_list_serializer
            .serialize_parameter(
                PID_TOPIC_NAME,
                &self.publication_builtin_topic_data.topic_name,
            )
            .unwrap();
        parameter_list_serializer
            .serialize_parameter(
                PID_TYPE_NAME,
                &self.publication_builtin_topic_data.type_name,
            )
            .unwrap();
        if self.publication_builtin_topic_data.durability != DurabilityQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_DURABILITY,
                    &DurabilityQosPolicySerialize(&self.publication_builtin_topic_data.durability),
                )
                .unwrap();
        }
        if self.publication_builtin_topic_data.durability_service
            != DurabilityServiceQosPolicy::default()
        {
            parameter_list_serializer
                .serialize_parameter(
                    PID_DURABILITY_SERVICE,
                    &DurabilityServiceQosPolicySerialize(
                        &self.publication_builtin_topic_data.durability_service,
                    ),
                )
                .unwrap();
        }
        if self.publication_builtin_topic_data.deadline != DeadlineQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_DEADLINE,
                    &DeadlineQosPolicySerialize(&self.publication_builtin_topic_data.deadline),
                )
                .unwrap();
        }
        if self.publication_builtin_topic_data.latency_budget != LatencyBudgetQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_LATENCY_BUDGET,
                    &LatencyBudgetQosPolicySerialize(
                        &self.publication_builtin_topic_data.latency_budget,
                    ),
                )
                .unwrap();
        }
        if self.publication_builtin_topic_data.liveliness != LivelinessQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_LIVELINESS,
                    &LivelinessQosPolicySerialize(&self.publication_builtin_topic_data.liveliness),
                )
                .unwrap();
        }
        parameter_list_serializer
            .serialize_parameter(
                PID_RELIABILITY,
                &ReliabilityQosPolicySerialize(&self.publication_builtin_topic_data.reliability),
            )
            .unwrap();
        if self.publication_builtin_topic_data.lifespan != LifespanQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_LIFESPAN,
                    &LifespanQosPolicySerialize(&self.publication_builtin_topic_data.lifespan),
                )
                .unwrap();
        }
        if self.publication_builtin_topic_data.user_data != UserDataQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_USER_DATA,
                    &UserDataQosPolicySerialize(&self.publication_builtin_topic_data.user_data),
                )
                .unwrap();
        }
        if self.publication_builtin_topic_data.ownership != OwnershipQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_OWNERSHIP,
                    &OwnershipQosPolicySerialize(&self.publication_builtin_topic_data.ownership),
                )
                .unwrap();
        }
        if self.publication_builtin_topic_data.ownership_strength
            != OwnershipStrengthQosPolicy::default()
        {
            parameter_list_serializer
                .serialize_parameter(
                    PID_OWNERSHIP_STRENGTH,
                    &OwnershipStrengthQosPolicySerialize(
                        &self.publication_builtin_topic_data.ownership_strength,
                    ),
                )
                .unwrap();
        }
        if self.publication_builtin_topic_data.destination_order
            != DestinationOrderQosPolicy::default()
        {
            parameter_list_serializer
                .serialize_parameter(
                    PID_DESTINATION_ORDER,
                    &DestinationOrderQosPolicySerialize(
                        &self.publication_builtin_topic_data.destination_order,
                    ),
                )
                .unwrap();
        }
        if self.publication_builtin_topic_data.presentation != PresentationQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_PRESENTATION,
                    &PresentationQosPolicySerialize(
                        &self.publication_builtin_topic_data.presentation,
                    ),
                )
                .unwrap();
        }
        if self.publication_builtin_topic_data.partition != PartitionQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_PARTITION,
                    &PartitionQosPolicySerialize(&self.publication_builtin_topic_data.partition),
                )
                .unwrap();
        }
        if self.publication_builtin_topic_data.topic_data != TopicDataQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_TOPIC_DATA,
                    &TopicDataQosPolicySerialize(&self.publication_builtin_topic_data.topic_data),
                )
                .unwrap();
        }
        if self.publication_builtin_topic_data.group_data != GroupDataQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_GROUP_DATA,
                    &GroupDataQosPolicySerialize(&self.publication_builtin_topic_data.group_data),
                )
                .unwrap();
        }

        Ok(())
    }
}

impl DdsDeserialize<'_> for SedpDiscoveredWriterData {
    fn deserialize(_buf: &mut &'_ [u8]) -> DDSResult<Self> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use rust_dds_api::{
        dcps_psm::{BuiltInTopicKey, Duration},
        infrastructure::qos_policy::{
            DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
            DurabilityServiceQosPolicy, GroupDataQosPolicy, LatencyBudgetQosPolicy,
            LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, OwnershipStrengthQosPolicy,
            PartitionQosPolicy, PresentationQosPolicy, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind, TopicDataQosPolicy, UserDataQosPolicy,
        },
    };
    use rust_rtps_pim::structure::types::{EntityId, Guid, GuidPrefix};

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
        let data = SedpDiscoveredWriterData {
            writer_proxy: RtpsWriterProxy {
                remote_writer_guid: Guid::new(
                    GuidPrefix([5; 12]),
                    EntityId {
                        entity_key: [11, 12, 13],
                        entity_kind: 15,
                    },
                ),
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                data_max_size_serialized: None,
                remote_group_entity_id: EntityId {
                    entity_key: [21, 22, 23],
                    entity_kind: 25,
                },
            },
            publication_builtin_topic_data: PublicationBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [1, 2, 3, 4],
                },
                participant_key: BuiltInTopicKey {
                    value: [6, 7, 8, 9],
                },
                topic_name: "ab".to_string(),
                type_name: "cd".to_string(),
                durability: DurabilityQosPolicy::default(),
                durability_service: DurabilityServiceQosPolicy::default(),
                deadline: DeadlineQosPolicy::default(),
                latency_budget: LatencyBudgetQosPolicy::default(),
                liveliness: LivelinessQosPolicy::default(),
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
                    max_blocking_time: Duration::new(2, 9),
                },
                lifespan: LifespanQosPolicy::default(),
                user_data: UserDataQosPolicy { value: vec![] },
                ownership: OwnershipQosPolicy::default(),
                ownership_strength: OwnershipStrengthQosPolicy::default(),
                destination_order: DestinationOrderQosPolicy::default(),
                presentation: PresentationQosPolicy::default(),
                partition: PartitionQosPolicy::default(),
                topic_data: TopicDataQosPolicy::default(),
                group_data: GroupDataQosPolicy::default(),
            },
        };

        let expected = vec![
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // long,
            2, 0, 0, 0, // long,
            3, 0, 0, 0, // long,
            4, 0, 0, 0, // long,
            0x50, 0x00, 16, 0, //PID_PARTICIPANT_GUID, length
            6, 0, 0, 0, // long,
            7, 0, 0, 0, // long,
            8, 0, 0, 0, // long,
            9, 0, 0, 0, // long,
            0x05, 0x00, 0x08, 0x00, // PID_TOPIC_NAME, Length: 8
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'a', b'b', 0, 0x00, // string + padding (1 byte)
            0x07, 0x00, 0x08, 0x00, // PID_TYPE_NAME, Length: 8
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'c', b'd', 0, 0x00, // string + padding (1 byte)
            0x1a, 0x00, 12, 0, //PID_RELIABILITY, length
            1, 0, 0, 0, // kind
            2, 0, 0, 0, // max_blocking_time: seconds
            9, 0, 0, 0, // max_blocking_time: nanoseconds
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        assert_eq!(to_bytes_le(&data), expected);
    }
}
