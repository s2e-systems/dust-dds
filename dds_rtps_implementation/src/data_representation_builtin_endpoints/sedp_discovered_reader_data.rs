use std::io::Write;

use rust_dds_api::{
    builtin_topics::SubscriptionBuiltinTopicData,
    dcps_psm::BuiltInTopicKey,
    infrastructure::qos_policy::{
        DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
        LatencyBudgetQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, PartitionQosPolicy,
        PresentationQosPolicy, TimeBasedFilterQosPolicy, TopicDataQosPolicy, UserDataQosPolicy,
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
    },
};
use rust_rtps_pim::{
    behavior::writer::reader_proxy::RtpsReaderProxy,
    structure::types::{EntityId, Guid, GuidPrefix, Locator},
};

use crate::{
    data_serialize_deserialize::{ParameterList, ParameterSerializer},
    dds_type::{DdsDeserialize, DdsSerialize, DdsType, Endianness},
};

use super::{
    serde_remote_dds_api::{
        BuiltInTopicKeyDeserialize, BuiltInTopicKeySerialize, DeadlineQosPolicySerialize,
        DestinationOrderQosPolicySerialize, DurabilityQosPolicySerialize, GroupDataQosPolicySerialize, LatencyBudgetQosPolicySerialize,
        LivelinessQosPolicySerialize,
        OwnershipQosPolicySerialize, PartitionQosPolicySerialize, PresentationQosPolicySerialize,
        ReliabilityQosPolicySerialize, TimeBasedFilterQosPolicySerialize,
        TopicDataQosPolicySerialize, UserDataQosPolicySerialize,
    },
    serde_remote_rtps_pim::{
        EntityIdDeserialize,
        EntityIdSerialize, LocatorDeserialize, LocatorSerialize,
    },
    parameter_id_values::{
        DEFAULT_EXPECTS_INLINE_QOS, PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY,
        PID_ENDPOINT_GUID, PID_EXPECTS_INLINE_QOS, PID_GROUP_DATA, PID_GROUP_ENTITYID,
        PID_LATENCY_BUDGET, PID_MULTICAST_LOCATOR, PID_OWNERSHIP, PID_PARTICIPANT_GUID,
        PID_PARTITION, PID_PRESENTATION, PID_RELIABILITY, PID_TIME_BASED_FILTER, PID_TOPIC_DATA,
        PID_TOPIC_NAME, PID_TYPE_NAME, PID_UNICAST_LOCATOR, PID_USER_DATA,
    },
};

#[derive(Debug, PartialEq)]
pub struct SedpDiscoveredReaderData {
    pub reader_proxy: RtpsReaderProxy<Vec<Locator>>,
    pub subscription_builtin_topic_data: SubscriptionBuiltinTopicData,
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
        parameter_list_serializer.serialize_payload_header()?;
        // omitted (as of table 9.10) reader_proxy.remote_reader_guid

        for locator in &self.reader_proxy.unicast_locator_list {
            parameter_list_serializer
                .serialize_parameter(PID_UNICAST_LOCATOR, &LocatorSerialize(locator))?;
        }
        for locator in &self.reader_proxy.multicast_locator_list {
            parameter_list_serializer
                .serialize_parameter(PID_MULTICAST_LOCATOR, &LocatorSerialize(locator))?;
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
                )?;
        }

        parameter_list_serializer
            .serialize_parameter(
                PID_ENDPOINT_GUID,
                &BuiltInTopicKeySerialize(&self.subscription_builtin_topic_data.key),
            )
            .unwrap();
        parameter_list_serializer
            .serialize_parameter(
                PID_PARTICIPANT_GUID,
                &BuiltInTopicKeySerialize(&self.subscription_builtin_topic_data.participant_key),
            )
            .unwrap();
        parameter_list_serializer
            .serialize_parameter(
                PID_TOPIC_NAME,
                &self.subscription_builtin_topic_data.topic_name,
            )
            .unwrap();
        parameter_list_serializer
            .serialize_parameter(
                PID_TYPE_NAME,
                &self.subscription_builtin_topic_data.type_name,
            )
            .unwrap();
        if self.subscription_builtin_topic_data.durability != DurabilityQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_DURABILITY,
                    &DurabilityQosPolicySerialize(
                        &self.subscription_builtin_topic_data.durability,
                    ),
                )?;
        }
        if self.subscription_builtin_topic_data.deadline != DeadlineQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_DEADLINE,
                    &DeadlineQosPolicySerialize(&self.subscription_builtin_topic_data.deadline),
                )?;
        }
        if self.subscription_builtin_topic_data.latency_budget != LatencyBudgetQosPolicy::default()
        {
            parameter_list_serializer
                .serialize_parameter(
                    PID_LATENCY_BUDGET,
                    &LatencyBudgetQosPolicySerialize(
                        &self.subscription_builtin_topic_data.latency_budget,
                    ),
                )?;
        }
        if self.subscription_builtin_topic_data.liveliness != LivelinessQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_DURABILITY,
                    &LivelinessQosPolicySerialize(
                        &self.subscription_builtin_topic_data.liveliness,
                    ),
                )?;
        }
        if self.subscription_builtin_topic_data.reliability
            != DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS
        {
            parameter_list_serializer
                .serialize_parameter(
                    PID_RELIABILITY,
                    &ReliabilityQosPolicySerialize(
                        &self.subscription_builtin_topic_data.reliability,
                    ),
                )?;
        }
        if self.subscription_builtin_topic_data.ownership != OwnershipQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_OWNERSHIP,
                    &OwnershipQosPolicySerialize(&self.subscription_builtin_topic_data.ownership),
                )?;
        }
        if self.subscription_builtin_topic_data.destination_order
            != DestinationOrderQosPolicy::default()
        {
            parameter_list_serializer
                .serialize_parameter(
                    PID_DESTINATION_ORDER,
                    &DestinationOrderQosPolicySerialize(
                        &self.subscription_builtin_topic_data.destination_order,
                    ),
                )?;
        }
        if self.subscription_builtin_topic_data.user_data != UserDataQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_USER_DATA,
                    &UserDataQosPolicySerialize(&self.subscription_builtin_topic_data.user_data),
                )?;
        }
        if self.subscription_builtin_topic_data.time_based_filter
            != TimeBasedFilterQosPolicy::default()
        {
            parameter_list_serializer
                .serialize_parameter(
                    PID_TIME_BASED_FILTER,
                    &TimeBasedFilterQosPolicySerialize(
                        &self.subscription_builtin_topic_data.time_based_filter,
                    ),
                )?;
        }
        if self.subscription_builtin_topic_data.presentation != PresentationQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_PRESENTATION,
                    &PresentationQosPolicySerialize(
                        &self.subscription_builtin_topic_data.presentation,
                    ),
                )?;
        }
        if self.subscription_builtin_topic_data.partition != PartitionQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_PARTITION,
                    &PartitionQosPolicySerialize(&self.subscription_builtin_topic_data.partition),
                )?;
        }
        if self.subscription_builtin_topic_data.topic_data != TopicDataQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_TOPIC_DATA,
                    &TopicDataQosPolicySerialize(&self.subscription_builtin_topic_data.topic_data),
                )?;
        }
        if self.subscription_builtin_topic_data.group_data != GroupDataQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_GROUP_DATA,
                    &GroupDataQosPolicySerialize(&self.subscription_builtin_topic_data.group_data),
                )?;
        }
        parameter_list_serializer.serialize_sentinel()?;
        Ok(())
    }
}

fn convert_built_in_topic_key_to_guid(key: &BuiltInTopicKey) -> Guid {
    let v0 = key.value[0].to_le_bytes();
    let v1 = key.value[1].to_le_bytes();
    let v2 = key.value[2].to_le_bytes();
    let v3 = key.value[3].to_le_bytes();
    let prefix = GuidPrefix([
        v0[0], v0[1], v0[2], v0[3], v1[0], v1[1], v1[2], v1[3], v2[0], v2[1], v2[2], v2[3],
    ]);
    let entity_id = EntityId {
        entity_key: [v3[0], v3[1], v3[2]],
        entity_kind: v3[3],
    };
    Guid { prefix, entity_id }
}

impl DdsDeserialize<'_> for SedpDiscoveredReaderData {
    fn deserialize(buf: &mut &'_ [u8]) -> rust_dds_api::return_type::DDSResult<Self> {
        let param_list = ParameterList::read(buf).unwrap();

        // reader_proxy
        let remote_group_entity_id = param_list
            .get::<EntityIdDeserialize>(PID_GROUP_ENTITYID)
            .unwrap()
            .0;
        let unicast_locator_list = param_list
            .get_list::<LocatorDeserialize>(PID_UNICAST_LOCATOR)
            .unwrap()
            .into_iter()
            .map(|l| l.0)
            .collect();
        let multicast_locator_list = param_list
            .get_list::<LocatorDeserialize>(PID_MULTICAST_LOCATOR)
            .unwrap()
            .into_iter()
            .map(|l| l.0)
            .collect();
        let expects_inline_qos = param_list
            .get::<bool>(PID_MULTICAST_LOCATOR)
            .unwrap_or(DEFAULT_EXPECTS_INLINE_QOS);

        // subscription_builtin_topic_data
        let key = param_list
            .get::<BuiltInTopicKeyDeserialize>(PID_ENDPOINT_GUID)
            .unwrap()
            .0;
        let participant_key = param_list
            .get::<BuiltInTopicKeyDeserialize>(PID_PARTICIPANT_GUID)
            .unwrap()
            .0;
        let topic_name = param_list.get(PID_TOPIC_NAME).unwrap();
        let type_name = param_list.get(PID_TYPE_NAME).unwrap();

        // Assembly
        let remote_reader_guid = convert_built_in_topic_key_to_guid(&key);
        let reader_proxy = RtpsReaderProxy {
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
        };
        let subscriptions_builtin_topic_data = SubscriptionBuiltinTopicData {
            key,
            participant_key,
            topic_name,
            type_name,
            durability: DurabilityQosPolicy::default(),
            deadline: DeadlineQosPolicy::default(),
            latency_budget: LatencyBudgetQosPolicy::default(),
            liveliness: LivelinessQosPolicy::default(),
            reliability: DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
            user_data: UserDataQosPolicy::default(),
            ownership: OwnershipQosPolicy::default(),
            destination_order: DestinationOrderQosPolicy::default(),
            time_based_filter: TimeBasedFilterQosPolicy::default(),
            presentation: PresentationQosPolicy::default(),
            partition: PartitionQosPolicy::default(),
            topic_data: TopicDataQosPolicy::default(),
            group_data: GroupDataQosPolicy::default(),
        };
        Ok(Self {
            reader_proxy,
            subscription_builtin_topic_data: subscriptions_builtin_topic_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_dds_api::{
        dcps_psm::BuiltInTopicKey,
        infrastructure::qos_policy::{
            DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
            LatencyBudgetQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, PartitionQosPolicy,
            PresentationQosPolicy, TimeBasedFilterQosPolicy, TopicDataQosPolicy, UserDataQosPolicy,
        },
    };
    use rust_rtps_pim::structure::types::{EntityId, Guid, GuidPrefix};

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
            subscription_builtin_topic_data: SubscriptionBuiltinTopicData {
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

    #[test]
    fn deserialize_all_default() {
        let expected = SedpDiscoveredReaderData {
            reader_proxy: RtpsReaderProxy {
                // must correspond to subscription_builtin_topic_data.key
                remote_reader_guid: Guid::new(
                    GuidPrefix([1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0]),
                    EntityId {
                        entity_key: [4, 0, 0],
                        entity_kind: 0,
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
            subscription_builtin_topic_data: SubscriptionBuiltinTopicData {
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
                user_data: UserDataQosPolicy::default(),
                ownership: OwnershipQosPolicy::default(),
                destination_order: DestinationOrderQosPolicy::default(),
                time_based_filter: TimeBasedFilterQosPolicy::default(),
                presentation: PresentationQosPolicy::default(),
                partition: PartitionQosPolicy::default(),
                topic_data: TopicDataQosPolicy::default(),
                group_data: GroupDataQosPolicy::default(),
            },
        };

        let mut data = &[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            21, 22, 23, 25, // u8[3], u8
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
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ][..];
        let result: SedpDiscoveredReaderData = DdsDeserialize::deserialize(&mut data).unwrap();
        assert_eq!(result, expected);
    }
}
