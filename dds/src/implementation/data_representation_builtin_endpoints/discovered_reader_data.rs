use std::io::Write;

use crate::{
    builtin_topics::{BuiltInTopicKey, SubscriptionBuiltinTopicData},
    implementation::{
        parameter_list_serde::{
            parameter_list_deserializer::ParameterListDeserializer,
            parameter_list_serializer::ParameterListSerializer,
        },
        rtps::types::{EntityId, Guid, Locator},
    },
    infrastructure::{
        error::DdsResult,
        qos_policy::{ReliabilityQosPolicy, DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS},
    },
    topic_definition::type_support::{
        DdsDeserialize, DdsSerialize, DdsSerializedKey, DdsType, Endianness,
    },
};

use super::parameter_id_values::{
    PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY, PID_ENDPOINT_GUID, PID_EXPECTS_INLINE_QOS,
    PID_GROUP_DATA, PID_GROUP_ENTITYID, PID_LATENCY_BUDGET, PID_LIVELINESS, PID_MULTICAST_LOCATOR,
    PID_OWNERSHIP, PID_PARTICIPANT_GUID, PID_PARTITION, PID_PRESENTATION, PID_RELIABILITY,
    PID_TIME_BASED_FILTER, PID_TOPIC_DATA, PID_TOPIC_NAME, PID_TYPE_NAME, PID_UNICAST_LOCATOR,
    PID_USER_DATA,
};

#[derive(Debug, PartialEq, Eq, serde::Serialize, derive_more::From, derive_more::Into)]
pub struct ReliabilityQosPolicyDataReaderAndTopics<'a>(pub &'a ReliabilityQosPolicy);
impl<'a> Default for ReliabilityQosPolicyDataReaderAndTopics<'a> {
    fn default() -> Self {
        Self(&DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS)
    }
}
#[derive(Debug, PartialEq, Eq, serde::Serialize, derive_more::From)]
pub struct ReliabilityQosPolicyDataReaderAndTopicsSerialize<'a>(
    pub &'a ReliabilityQosPolicyDataReaderAndTopics<'a>,
);

#[derive(Debug, PartialEq, Eq, serde::Deserialize, derive_more::Into)]
pub struct ReliabilityQosPolicyDataReaderAndTopicsDeserialize(pub ReliabilityQosPolicy);
impl Default for ReliabilityQosPolicyDataReaderAndTopicsDeserialize {
    fn default() -> Self {
        Self(DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS)
    }
}

pub const DCPS_SUBSCRIPTION: &str = "DCPSSubscription";

#[derive(Debug, PartialEq, Eq)]
pub struct ReaderProxy {
    pub remote_reader_guid: Guid,
    pub remote_group_entity_id: EntityId,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    pub expects_inline_qos: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DiscoveredReaderData {
    pub reader_proxy: ReaderProxy,
    pub subscription_builtin_topic_data: SubscriptionBuiltinTopicData,
}

impl DdsType for DiscoveredReaderData {
    fn type_name() -> &'static str {
        "DiscoveredReaderData"
    }

    fn has_key() -> bool {
        true
    }

    fn get_serialized_key(&self) -> DdsSerializedKey {
        self.subscription_builtin_topic_data
            .key
            .value
            .as_ref()
            .into()
    }

    fn set_key_fields_from_serialized_key(&mut self, _key: &DdsSerializedKey) -> DdsResult<()> {
        if Self::has_key() {
            unimplemented!("DdsType with key must provide an implementation for set_key_fields_from_serialized_key")
        }
        Ok(())
    }
}

impl DdsSerialize for DiscoveredReaderData {
    fn serialize<W: Write, E: Endianness>(&self, writer: W) -> DdsResult<()> {
        let mut parameter_list_serializer = ParameterListSerializer::<_, E>::new(writer);
        parameter_list_serializer.serialize_payload_header()?;
        // reader_proxy.remote_reader_guid omitted as of table 9.10

        parameter_list_serializer.serialize_parameter_vector(
            PID_UNICAST_LOCATOR,
            &self.reader_proxy.unicast_locator_list,
        )?;
        parameter_list_serializer.serialize_parameter_vector(
            PID_MULTICAST_LOCATOR,
            &self.reader_proxy.multicast_locator_list,
        )?;
        parameter_list_serializer.serialize_parameter(
            PID_GROUP_ENTITYID,
            &self.reader_proxy.remote_group_entity_id,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_EXPECTS_INLINE_QOS,
            &self.reader_proxy.expects_inline_qos,
        )?;
        parameter_list_serializer
            .serialize_parameter(PID_ENDPOINT_GUID, &self.subscription_builtin_topic_data.key)?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_PARTICIPANT_GUID,
            &self.subscription_builtin_topic_data.participant_key,
        )?; // Default value is a deviation from the standard and is used for interoperability reasons
        parameter_list_serializer.serialize_parameter(
            PID_TOPIC_NAME,
            &self.subscription_builtin_topic_data.topic_name,
        )?;
        parameter_list_serializer.serialize_parameter(
            PID_TYPE_NAME,
            &self.subscription_builtin_topic_data.type_name,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_DURABILITY,
            &self.subscription_builtin_topic_data.durability,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_DEADLINE,
            &self.subscription_builtin_topic_data.deadline,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_LATENCY_BUDGET,
            &self.subscription_builtin_topic_data.latency_budget,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_DURABILITY,
            &self.subscription_builtin_topic_data.liveliness,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_RELIABILITY,
            &ReliabilityQosPolicyDataReaderAndTopics(
                &self.subscription_builtin_topic_data.reliability,
            ),
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_OWNERSHIP,
            &self.subscription_builtin_topic_data.ownership,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_DESTINATION_ORDER,
            &self.subscription_builtin_topic_data.destination_order,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_USER_DATA,
            &self.subscription_builtin_topic_data.user_data,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_TIME_BASED_FILTER,
            &self.subscription_builtin_topic_data.time_based_filter,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_PRESENTATION,
            &self.subscription_builtin_topic_data.presentation,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_PARTITION,
            &self.subscription_builtin_topic_data.partition,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_TOPIC_DATA,
            &self.subscription_builtin_topic_data.topic_data,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_GROUP_DATA,
            &self.subscription_builtin_topic_data.group_data,
        )?;
        parameter_list_serializer.serialize_sentinel()
    }
}

impl DdsDeserialize<'_> for DiscoveredReaderData {
    fn deserialize(buf: &mut &'_ [u8]) -> DdsResult<Self> {
        let param_list = ParameterListDeserializer::read(buf)?;

        // reader_proxy
        let remote_group_entity_id = param_list.get_or_default(PID_GROUP_ENTITYID)?;
        let unicast_locator_list = param_list.get_list(PID_UNICAST_LOCATOR)?;
        let multicast_locator_list = param_list.get_list(PID_MULTICAST_LOCATOR)?;
        let expects_inline_qos = param_list.get_or_default(PID_MULTICAST_LOCATOR)?;

        // subscription_builtin_topic_data
        let key = param_list.get::<BuiltInTopicKey>(PID_ENDPOINT_GUID)?;
        // Default value is a deviation from the standard and is used for interoperability reasons
        let participant_key = param_list.get_or_default(PID_PARTICIPANT_GUID)?;
        let topic_name = param_list.get(PID_TOPIC_NAME)?;
        let type_name = param_list.get(PID_TYPE_NAME)?;
        let durability = param_list.get_or_default(PID_DURABILITY)?;
        let deadline = param_list.get_or_default(PID_DEADLINE)?;
        let latency_budget = param_list.get_or_default(PID_LATENCY_BUDGET)?;
        let liveliness = param_list.get_or_default(PID_LIVELINESS)?;
        let reliability = param_list
            .get_or_default::<ReliabilityQosPolicyDataReaderAndTopicsDeserialize>(PID_RELIABILITY)?
            .into();
        let user_data = param_list.get_or_default(PID_USER_DATA)?;
        let ownership = param_list.get_or_default(PID_OWNERSHIP)?;
        let destination_order = param_list.get_or_default(PID_DESTINATION_ORDER)?;
        let time_based_filter = param_list.get_or_default(PID_TIME_BASED_FILTER)?;
        let presentation = param_list.get_or_default(PID_PRESENTATION)?;
        let partition = param_list.get_or_default(PID_PARTITION)?;
        let topic_data = param_list.get_or_default(PID_TOPIC_DATA)?;
        let group_data = param_list.get_or_default(PID_GROUP_DATA)?;

        let remote_reader_guid = key.value.into();

        Ok(Self {
            reader_proxy: ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id,
                unicast_locator_list,
                multicast_locator_list,
                expects_inline_qos,
            },
            subscription_builtin_topic_data: SubscriptionBuiltinTopicData {
                key,
                participant_key,
                topic_name,
                type_name,
                durability,
                deadline,
                latency_budget,
                liveliness,
                reliability,
                user_data,
                ownership,
                destination_order,
                time_based_filter,
                presentation,
                partition,
                topic_data,
                group_data,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::parameter_list_serde::serde_remote_rtps_pim::{
        ExpectsInlineQosDeserialize, ExpectsInlineQosSerialize,
    };
    use crate::implementation::rtps::types::{
        EntityKey, GuidPrefix, BUILT_IN_WRITER_WITH_KEY, USER_DEFINED_READER_WITH_KEY,
        USER_DEFINED_UNKNOWN,
    };
    use crate::infrastructure::qos_policy::{
        DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
        LatencyBudgetQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, PartitionQosPolicy,
        PresentationQosPolicy, TimeBasedFilterQosPolicy, TopicDataQosPolicy, UserDataQosPolicy,
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
    };
    use crate::topic_definition::type_support::LittleEndian;

    fn to_bytes_le<S: DdsSerialize>(value: &S) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        value.serialize::<_, LittleEndian>(&mut writer).unwrap();
        writer
    }
    #[test]
    fn serialize_all_default() {
        let data = DiscoveredReaderData {
            reader_proxy: ReaderProxy {
                remote_reader_guid: Guid::new(
                    GuidPrefix::new([5; 12]),
                    EntityId::new(EntityKey::new([11, 12, 13]), USER_DEFINED_READER_WITH_KEY),
                ),
                remote_group_entity_id: EntityId::new(
                    EntityKey::new([21, 22, 23]),
                    BUILT_IN_WRITER_WITH_KEY,
                ),
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                expects_inline_qos: *ExpectsInlineQosSerialize::default().0,
            },
            subscription_builtin_topic_data: SubscriptionBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                participant_key: BuiltInTopicKey {
                    value: [6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 0, 9, 0, 0, 0],
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
            21, 22, 23, 0xc2, //
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // ,
            2, 0, 0, 0, // ,
            3, 0, 0, 0, // ,
            4, 0, 0, 0, // ,
            0x50, 0x00, 16, 0, //PID_PARTICIPANT_GUID, length
            6, 0, 0, 0, // ,
            7, 0, 0, 0, // ,
            8, 0, 0, 0, // ,
            9, 0, 0, 0, // ,
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
        let expected = DiscoveredReaderData {
            reader_proxy: ReaderProxy {
                // must correspond to subscription_builtin_topic_data.key
                remote_reader_guid: Guid::new(
                    GuidPrefix::new([1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0]),
                    EntityId::new(EntityKey::new([4, 0, 0]), USER_DEFINED_UNKNOWN),
                ),
                remote_group_entity_id: EntityId::new(
                    EntityKey::new([21, 22, 23]),
                    BUILT_IN_WRITER_WITH_KEY,
                ),
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                expects_inline_qos: ExpectsInlineQosDeserialize::default().0,
            },
            subscription_builtin_topic_data: SubscriptionBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                participant_key: BuiltInTopicKey {
                    value: [6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 0, 9, 0, 0, 0],
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
            21, 22, 23, 0xc2, // u8[3], u8
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // ,
            2, 0, 0, 0, // ,
            3, 0, 0, 0, // ,
            4, 0, 0, 0, // ,
            0x50, 0x00, 16, 0, //PID_PARTICIPANT_GUID, length
            6, 0, 0, 0, // ,
            7, 0, 0, 0, // ,
            8, 0, 0, 0, // ,
            9, 0, 0, 0, // ,
            0x05, 0x00, 0x08, 0x00, // PID_TOPIC_NAME, Length: 8
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'a', b'b', 0, 0x00, // string + padding (1 byte)
            0x07, 0x00, 0x08, 0x00, // PID_TYPE_NAME, Length: 8
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'c', b'd', 0, 0x00, // string + padding (1 byte)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ][..];
        let result: DiscoveredReaderData = DdsDeserialize::deserialize(&mut data).unwrap();
        assert_eq!(result, expected);
    }
}
