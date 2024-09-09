use super::parameter_id_values::{
    DEFAULT_EXPECTS_INLINE_QOS, PID_DATA_REPRESENTATION, PID_DEADLINE, PID_DESTINATION_ORDER,
    PID_DURABILITY, PID_ENDPOINT_GUID, PID_EXPECTS_INLINE_QOS, PID_GROUP_DATA, PID_GROUP_ENTITYID,
    PID_LATENCY_BUDGET, PID_LIVELINESS, PID_MULTICAST_LOCATOR, PID_OWNERSHIP, PID_PARTICIPANT_GUID,
    PID_PARTITION, PID_PRESENTATION, PID_RELIABILITY, PID_TIME_BASED_FILTER, PID_TOPIC_DATA,
    PID_TOPIC_NAME, PID_TYPE_NAME, PID_TYPE_REPRESENTATION, PID_UNICAST_LOCATOR, PID_USER_DATA,
};
use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    implementation::payload_serializer_deserializer::{
        endianness::CdrEndianness, parameter_list_deserializer::ParameterListCdrDeserializer,
        parameter_list_serializer::ParameterListCdrSerializer,
    },
    infrastructure::{
        error::DdsResult, qos_policy::DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
    },
    rtps::types::{EntityId, Guid, Locator},
    topic_definition::type_support::{DdsDeserialize, DdsHasKey, DdsKey, DdsSerialize, DdsTypeXml},
};

pub const DCPS_SUBSCRIPTION: &str = "DCPSSubscription";

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ReaderProxy {
    remote_reader_guid: Guid,
    remote_group_entity_id: EntityId,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    expects_inline_qos: bool,
}

impl ReaderProxy {
    pub fn new(
        remote_reader_guid: Guid,
        remote_group_entity_id: EntityId,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        expects_inline_qos: bool,
    ) -> Self {
        Self {
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
        }
    }

    pub fn remote_reader_guid(&self) -> Guid {
        self.remote_reader_guid
    }

    pub fn remote_group_entity_id(&self) -> EntityId {
        self.remote_group_entity_id
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.unicast_locator_list.as_ref()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.multicast_locator_list.as_ref()
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DiscoveredReaderData {
    subscription_builtin_topic_data: SubscriptionBuiltinTopicData,
    reader_proxy: ReaderProxy,
}

impl DdsSerialize for DiscoveredReaderData {
    fn serialize_data(&self) -> DdsResult<Vec<u8>> {
        let mut serializer = ParameterListCdrSerializer::new();
        serializer.write_header()?;

        // subscription_builtin_topic_data: SubscriptionBuiltinTopicData:
        serializer.write(PID_ENDPOINT_GUID, &self.subscription_builtin_topic_data.key)?;
        // Default value is a deviation from the standard and is used for interoperability reasons:
        serializer.write_with_default(
            PID_PARTICIPANT_GUID,
            &self.subscription_builtin_topic_data.participant_key,
            &Default::default(),
        )?;
        serializer.write(
            PID_TOPIC_NAME,
            &self.subscription_builtin_topic_data.topic_name,
        )?;
        serializer.write(
            PID_TYPE_NAME,
            &self.subscription_builtin_topic_data.type_name,
        )?;
        serializer.write_with_default(
            PID_DURABILITY,
            &self.subscription_builtin_topic_data.durability,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_DEADLINE,
            &self.subscription_builtin_topic_data.deadline,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_LATENCY_BUDGET,
            &self.subscription_builtin_topic_data.latency_budget,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_LIVELINESS,
            &self.subscription_builtin_topic_data.liveliness,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_RELIABILITY,
            &self.subscription_builtin_topic_data.reliability,
            &DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
        )?;
        serializer.write_with_default(
            PID_OWNERSHIP,
            &self.subscription_builtin_topic_data.ownership,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_DESTINATION_ORDER,
            &self.subscription_builtin_topic_data.destination_order,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_USER_DATA,
            &self.subscription_builtin_topic_data.user_data,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_TIME_BASED_FILTER,
            &self.subscription_builtin_topic_data.time_based_filter,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_PRESENTATION,
            &self.subscription_builtin_topic_data.presentation,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_PARTITION,
            &self.subscription_builtin_topic_data.partition,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_TOPIC_DATA,
            &self.subscription_builtin_topic_data.topic_data,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_GROUP_DATA,
            &self.subscription_builtin_topic_data.group_data,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_TYPE_REPRESENTATION,
            &self.subscription_builtin_topic_data.xml_type,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_DATA_REPRESENTATION,
            &self.subscription_builtin_topic_data.representation,
            &Default::default(),
        )?;

        // reader_proxy: ReaderProxy

        // skip serilize:
        // reader_proxy.remote_writer_guid: Guid,
        serializer.write_with_default(
            PID_GROUP_ENTITYID,
            &self.reader_proxy.remote_group_entity_id,
            &Default::default(),
        )?;
        serializer
            .write_collection(PID_UNICAST_LOCATOR, &self.reader_proxy.unicast_locator_list)?;
        serializer.write_collection(
            PID_MULTICAST_LOCATOR,
            &self.reader_proxy.multicast_locator_list,
        )?;
        serializer.write_with_default(
            PID_EXPECTS_INLINE_QOS,
            &self.reader_proxy.expects_inline_qos,
            &DEFAULT_EXPECTS_INLINE_QOS,
        )?;

        serializer.write_sentinel()?;
        Ok(serializer.writer)
    }
}

impl<'de> DdsDeserialize<'de> for DiscoveredReaderData {
    fn deserialize_data(serialized_data: &'de [u8]) -> DdsResult<Self> {
        let pl_deserializer = ParameterListCdrDeserializer::new(serialized_data)?;

        Ok(Self {
            subscription_builtin_topic_data: SubscriptionBuiltinTopicData::deserialize_data(
                serialized_data,
            )?,
            reader_proxy: ReaderProxy {
                remote_reader_guid: pl_deserializer.read(PID_ENDPOINT_GUID)?,
                remote_group_entity_id: pl_deserializer
                    .read_with_default(PID_GROUP_ENTITYID, Default::default())?,
                unicast_locator_list: pl_deserializer.read_collection(PID_UNICAST_LOCATOR)?,
                multicast_locator_list: pl_deserializer.read_collection(PID_MULTICAST_LOCATOR)?,
                expects_inline_qos: pl_deserializer
                    .read_with_default(PID_EXPECTS_INLINE_QOS, DEFAULT_EXPECTS_INLINE_QOS)?,
            },
        })
    }
}

impl DiscoveredReaderData {
    pub fn new(
        reader_proxy: ReaderProxy,
        subscription_builtin_topic_data: SubscriptionBuiltinTopicData,
    ) -> Self {
        Self {
            reader_proxy,
            subscription_builtin_topic_data,
        }
    }

    pub fn reader_proxy(&self) -> &ReaderProxy {
        &self.reader_proxy
    }

    pub fn subscription_builtin_topic_data(&self) -> &SubscriptionBuiltinTopicData {
        &self.subscription_builtin_topic_data
    }
}

impl DdsHasKey for DiscoveredReaderData {
    const HAS_KEY: bool = true;
}

impl DdsKey for DiscoveredReaderData {
    type Key = [u8; 16];

    fn get_key(&self) -> DdsResult<Self::Key> {
        Ok(self.subscription_builtin_topic_data.key().value)
    }

    fn get_key_from_serialized_data(serialized_foo: &[u8]) -> DdsResult<Self::Key> {
        Ok(Self::deserialize_data(serialized_foo)?
            .subscription_builtin_topic_data
            .key()
            .value)
    }
}

impl DdsTypeXml for DiscoveredReaderData {
    fn get_type_xml() -> Option<String> {
        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        builtin_topics::BuiltInTopicKey,
        infrastructure::{
            qos::{DataReaderQos, SubscriberQos},
            qos_policy::TopicDataQosPolicy,
        },
        rtps::types::{
            BUILT_IN_WRITER_WITH_KEY, USER_DEFINED_READER_WITH_KEY, USER_DEFINED_UNKNOWN,
        },
    };

    #[test]
    fn serialize_all_default() {
        let data = DiscoveredReaderData {
            subscription_builtin_topic_data: SubscriptionBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                BuiltInTopicKey {
                    value: [6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 0, 9, 0, 0, 0],
                },
                "ab".to_string(),
                "cd".to_string(),
                DataReaderQos::default(),
                SubscriberQos::default(),
                TopicDataQosPolicy::default(),
                String::default(),
            ),
            reader_proxy: ReaderProxy::new(
                Guid::new(
                    [5; 12],
                    EntityId::new([11, 12, 13], USER_DEFINED_READER_WITH_KEY),
                ),
                EntityId::new([21, 22, 23], BUILT_IN_WRITER_WITH_KEY),
                vec![],
                vec![],
                false,
            ),
        };

        let expected = vec![
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            // SubscriptionBuiltinTopicData:
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
            // ReaderProxy:
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            21, 22, 23, 0xc2, //
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        let result = data.serialize_data().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_all_default() {
        let expected = DiscoveredReaderData {
            reader_proxy: ReaderProxy::new(
                // must correspond to subscription_builtin_topic_data.key
                Guid::new(
                    [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0],
                    EntityId::new([4, 0, 0], USER_DEFINED_UNKNOWN),
                ),
                EntityId::new([21, 22, 23], BUILT_IN_WRITER_WITH_KEY),
                vec![],
                vec![],
                false,
            ),
            subscription_builtin_topic_data: SubscriptionBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                BuiltInTopicKey {
                    value: [6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 0, 9, 0, 0, 0],
                },
                "ab".to_string(),
                "cd".to_string(),
                DataReaderQos::default(),
                SubscriberQos::default(),
                TopicDataQosPolicy::default(),
                String::default(),
            ),
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
        let result = DiscoveredReaderData::deserialize_data(&mut data).unwrap();
        assert_eq!(result, expected);
    }
}
