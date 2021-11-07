use std::io::Write;

use rust_dds_api::{builtin_topics::SubscriptionBuiltinTopicData, infrastructure::qos_policy::{DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS, DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy, LatencyBudgetQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, PartitionQosPolicy, PresentationQosPolicy, TimeBasedFilterQosPolicy, TopicDataQosPolicy, UserDataQosPolicy}};
use rust_rtps_pim::{behavior::writer::reader_proxy::RtpsReaderProxy, structure::types::Locator};

use crate::{
    data_serialize_deserialize::ParameterSerializer,
    dds_type::{DdsDeserialize, DdsSerialize, DdsType, Endianness},
};

use super::{dds_serialize_deserialize_impl::{
        BuiltInTopicKeySerialize, DeadlineQosPolicySerialize, DestinationOrderQosPolicySerialize,
        DurabilityQosPolicySerialize, EntityIdSerialize, GroupDataQosPolicySerialize,
        LatencyBudgetQosPolicySerialize, LivelinessQosPolicySerialize, LocatorSerdeSerialize,
        OwnershipQosPolicySerialize, PartitionQosPolicySerialize, PresentationQosPolicySerialize,
        ReliabilityQosPolicySerialize, TimeBasedFilterQosPolicySerialize,
        TopicDataQosPolicySerialize, UserDataQosPolicySerialize,
    }, parameter_id_values::{DEFAULT_EXPECTS_INLINE_QOS, PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY, PID_ENDPOINT_GUID, PID_EXPECTS_INLINE_QOS, PID_GROUP_DATA, PID_GROUP_ENTITYID, PID_LATENCY_BUDGET, PID_MULTICAST_LOCATOR, PID_OWNERSHIP, PID_PARTICIPANT_GUID, PID_PARTITION, PID_PRESENTATION, PID_RELIABILITY, PID_TIME_BASED_FILTER, PID_TOPIC_DATA, PID_TOPIC_NAME, PID_TYPE_NAME, PID_UNICAST_LOCATOR, PID_USER_DATA}};

pub struct SedpDiscoveredReaderData {
    pub reader_proxy: RtpsReaderProxy<Vec<Locator>>,
    pub subscriptions_builtin_topic_data: SubscriptionBuiltinTopicData,
}

impl DdsType for SedpDiscoveredReaderData {
    fn type_name() -> &'static str {
        "SedpDiscoveredReaderData"
    }

    fn has_key() -> bool {
        true
    }
}

impl DdsSerialize for SedpDiscoveredReaderData {
    fn serialize<W: Write, E: Endianness>(
        &self,
        writer: W,
    ) -> rust_dds_api::return_type::DDSResult<()> {
        let mut parameter_list_serializer = ParameterSerializer::<_, E>::new(writer);

        // omitted (as of table 9.10) reader_proxy.remote_reader_guid

        for locator in &self.reader_proxy.unicast_locator_list {
            parameter_list_serializer
                .serialize_parameter(PID_UNICAST_LOCATOR, &LocatorSerdeSerialize(locator))
                .unwrap();
        }
        for locator in &self.reader_proxy.multicast_locator_list {
            parameter_list_serializer
                .serialize_parameter(PID_MULTICAST_LOCATOR, &LocatorSerdeSerialize(locator))
                .unwrap();
        }
        parameter_list_serializer
            .serialize_parameter(
                PID_GROUP_ENTITYID,
                &EntityIdSerialize(&self.reader_proxy.remote_group_entity_id),
            )
            .unwrap();
        if self.reader_proxy.expects_inline_qos != DEFAULT_EXPECTS_INLINE_QOS {
            parameter_list_serializer
                .serialize_parameter(
                    PID_EXPECTS_INLINE_QOS,
                    &self.reader_proxy.expects_inline_qos,
                )
                .unwrap();
        }

        parameter_list_serializer
            .serialize_parameter(
                PID_ENDPOINT_GUID,
                &BuiltInTopicKeySerialize(&self.subscriptions_builtin_topic_data.key),
            )
            .unwrap();
        parameter_list_serializer
            .serialize_parameter(
                PID_PARTICIPANT_GUID,
                &BuiltInTopicKeySerialize(&self.subscriptions_builtin_topic_data.participant_key),
            )
            .unwrap();
        parameter_list_serializer
            .serialize_parameter(
                PID_TOPIC_NAME,
                &self.subscriptions_builtin_topic_data.topic_name,
            )
            .unwrap();
        parameter_list_serializer
            .serialize_parameter(
                PID_TYPE_NAME,
                &self.subscriptions_builtin_topic_data.type_name,
            )
            .unwrap();
        if self.subscriptions_builtin_topic_data.durability != DurabilityQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_DURABILITY,
                    &DurabilityQosPolicySerialize(
                        &self.subscriptions_builtin_topic_data.durability,
                    ),
                )
                .unwrap();
        }
        if self.subscriptions_builtin_topic_data.deadline != DeadlineQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_DEADLINE,
                    &DeadlineQosPolicySerialize(&self.subscriptions_builtin_topic_data.deadline),
                )
                .unwrap();
        }
        if self.subscriptions_builtin_topic_data.latency_budget != LatencyBudgetQosPolicy::default()
        {
            parameter_list_serializer
                .serialize_parameter(
                    PID_LATENCY_BUDGET,
                    &LatencyBudgetQosPolicySerialize(
                        &self.subscriptions_builtin_topic_data.latency_budget,
                    ),
                )
                .unwrap();
        }
        if self.subscriptions_builtin_topic_data.liveliness != LivelinessQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_DURABILITY,
                    &LivelinessQosPolicySerialize(
                        &self.subscriptions_builtin_topic_data.liveliness,
                    ),
                )
                .unwrap();
        }
        if self.subscriptions_builtin_topic_data.reliability
            != DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS
        {
            parameter_list_serializer
                .serialize_parameter(
                    PID_RELIABILITY,
                    &ReliabilityQosPolicySerialize(
                        &self.subscriptions_builtin_topic_data.reliability,
                    ),
                )
                .unwrap();
        }
        if self.subscriptions_builtin_topic_data.ownership != OwnershipQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_OWNERSHIP,
                    &OwnershipQosPolicySerialize(&self.subscriptions_builtin_topic_data.ownership),
                )
                .unwrap();
        }
        if self.subscriptions_builtin_topic_data.destination_order
            != DestinationOrderQosPolicy::default()
        {
            parameter_list_serializer
                .serialize_parameter(
                    PID_DESTINATION_ORDER,
                    &DestinationOrderQosPolicySerialize(
                        &self.subscriptions_builtin_topic_data.destination_order,
                    ),
                )
                .unwrap();
        }
        if self.subscriptions_builtin_topic_data.user_data != UserDataQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_USER_DATA,
                    &UserDataQosPolicySerialize(&self.subscriptions_builtin_topic_data.user_data),
                )
                .unwrap();
        }
        if self.subscriptions_builtin_topic_data.time_based_filter
            != TimeBasedFilterQosPolicy::default()
        {
            parameter_list_serializer
                .serialize_parameter(
                    PID_TIME_BASED_FILTER,
                    &TimeBasedFilterQosPolicySerialize(
                        &self.subscriptions_builtin_topic_data.time_based_filter,
                    ),
                )
                .unwrap();
        }
        if self.subscriptions_builtin_topic_data.presentation != PresentationQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_PRESENTATION,
                    &PresentationQosPolicySerialize(
                        &self.subscriptions_builtin_topic_data.presentation,
                    ),
                )
                .unwrap();
        }
        if self.subscriptions_builtin_topic_data.partition != PartitionQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_PARTITION,
                    &PartitionQosPolicySerialize(&self.subscriptions_builtin_topic_data.partition),
                )
                .unwrap();
        }
        if self.subscriptions_builtin_topic_data.topic_data != TopicDataQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_TOPIC_DATA,
                    &TopicDataQosPolicySerialize(&self.subscriptions_builtin_topic_data.topic_data),
                )
                .unwrap();
        }
        if self.subscriptions_builtin_topic_data.group_data != GroupDataQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_GROUP_DATA,
                    &GroupDataQosPolicySerialize(&self.subscriptions_builtin_topic_data.group_data),
                )
                .unwrap();
        }

        Ok(())
    }
}

impl DdsDeserialize<'_> for SedpDiscoveredReaderData {
    fn deserialize(_buf: &mut &'_ [u8]) -> rust_dds_api::return_type::DDSResult<Self> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use rust_dds_api::{
        dcps_psm::BuiltInTopicKey,
        infrastructure::qos_policy::{
            DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
            LatencyBudgetQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, PartitionQosPolicy,
            PresentationQosPolicy, TimeBasedFilterQosPolicy, TopicDataQosPolicy, UserDataQosPolicy,
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
        let data = SedpDiscoveredReaderData {
            reader_proxy: RtpsReaderProxy {
                remote_reader_guid: Guid::new(
                    GuidPrefix([5; 12]),
                    EntityId {
                        entity_key: [11, 12, 13],
                        entity_kind: 15,
                    },
                ),
                remote_group_entity_id: EntityId {
                    entity_key: [21, 22, 23],
                    entity_kind: 25,
                },
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                expects_inline_qos: DEFAULT_EXPECTS_INLINE_QOS,
            },
            subscriptions_builtin_topic_data: SubscriptionBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [1, 2, 3, 4],
                },
                participant_key: BuiltInTopicKey {
                    value: [6, 7, 8, 9],
                },
                topic_name: "ab".to_string(),
                type_name: "cd".to_string(),
                durability: DurabilityQosPolicy::default(),
                deadline: DeadlineQosPolicy::default(),
                latency_budget: LatencyBudgetQosPolicy::default(),
                liveliness: LivelinessQosPolicy::default(),
                reliability: DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
                user_data: UserDataQosPolicy { value: vec![] },
                ownership: OwnershipQosPolicy::default(),
                time_based_filter: TimeBasedFilterQosPolicy::default(),
                destination_order: DestinationOrderQosPolicy::default(),
                presentation: PresentationQosPolicy::default(),
                partition: PartitionQosPolicy::default(),
                topic_data: TopicDataQosPolicy::default(),
                group_data: GroupDataQosPolicy::default(),
            },
        };

        let expected = vec![
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            21, 22, 23, 25, 0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
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
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        assert_eq!(to_bytes_le(&data), expected);
    }
}
