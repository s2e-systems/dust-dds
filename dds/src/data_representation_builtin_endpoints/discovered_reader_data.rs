use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    infrastructure::{
        error::DdsResult, qos_policy::DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
    },
    rtps::types::{EntityId, Guid, Locator},
    serialized_payload::parameter_list::{
        deserialize::ParameterListDeserialize, serialize::ParameterListSerialize,
    },
    topic_definition::type_support::{DdsDeserialize, DdsHasKey, DdsKey, DdsSerialize, DdsTypeXml},
};
use xtypes::{
    deserialize::XTypesDeserialize,
    deserializer::{DeserializeMutableStruct, XTypesDeserializer},
    error::XcdrError,
    serialize::XTypesSerialize,
    serializer::{SerializeMutableStruct, XTypesSerializer},
};

use super::parameter_id_values::{
    DEFAULT_EXPECTS_INLINE_QOS, PID_DATA_REPRESENTATION, PID_DEADLINE, PID_DESTINATION_ORDER,
    PID_DURABILITY, PID_ENDPOINT_GUID, PID_EXPECTS_INLINE_QOS, PID_GROUP_DATA, PID_GROUP_ENTITYID,
    PID_LATENCY_BUDGET, PID_LIVELINESS, PID_MULTICAST_LOCATOR, PID_OWNERSHIP, PID_PARTICIPANT_GUID,
    PID_PARTITION, PID_PRESENTATION, PID_RELIABILITY, PID_TIME_BASED_FILTER, PID_TOPIC_DATA,
    PID_TOPIC_NAME, PID_TYPE_NAME, PID_TYPE_REPRESENTATION, PID_UNICAST_LOCATOR, PID_USER_DATA,
};

pub const DCPS_SUBSCRIPTION: &str = "DCPSSubscription";

#[derive(Debug, PartialEq, Eq, Clone, ParameterListSerialize, ParameterListDeserialize)]
pub struct ReaderProxy {
    #[parameter(id = PID_ENDPOINT_GUID, skip_serialize)]
    remote_reader_guid: Guid,
    #[parameter(id = PID_GROUP_ENTITYID, default=Default::default())]
    remote_group_entity_id: EntityId,
    #[parameter(id = PID_UNICAST_LOCATOR, collection)]
    unicast_locator_list: Vec<Locator>,
    #[parameter(id = PID_MULTICAST_LOCATOR, collection)]
    multicast_locator_list: Vec<Locator>,
    #[parameter(id = PID_EXPECTS_INLINE_QOS, default=DEFAULT_EXPECTS_INLINE_QOS)]
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
pub struct DiscoveredReaderData {
    reader_proxy: ReaderProxy,
    dds_subscription_data: SubscriptionBuiltinTopicData,
}

impl XTypesSerialize for DiscoveredReaderData {
    fn serialize(&self, serializer: impl XTypesSerializer) -> Result<(), XcdrError> {
        let mut m = serializer.serialize_mutable_struct()?;
        if self.reader_proxy.remote_group_entity_id != Default::default() {
            m.serialize_field(
                &self.reader_proxy.remote_group_entity_id,
                PID_GROUP_ENTITYID as u16,
                "remote_group_entity_id",
            )?;
        }
        for unicast_locator in &self.reader_proxy.unicast_locator_list {
            m.serialize_field(
                unicast_locator,
                PID_UNICAST_LOCATOR as u16,
                "unicast_locator_list",
            )?;
        }
        for multicast_locator in &self.reader_proxy.multicast_locator_list {
            m.serialize_field(
                multicast_locator,
                PID_MULTICAST_LOCATOR as u16,
                "multicast_locator_list",
            )?;
        }
        if self.reader_proxy.expects_inline_qos != DEFAULT_EXPECTS_INLINE_QOS {
            m.serialize_field(
                &self.reader_proxy.expects_inline_qos,
                DEFAULT_EXPECTS_INLINE_QOS as u16,
                "remote_group_entity_id",
            )?;
        }

        m.serialize_field(
            &self.dds_subscription_data.key,
            PID_ENDPOINT_GUID as u16,
            "key",
        )?;
        if self.dds_subscription_data.participant_key != Default::default() {
            m.serialize_field(
                &self.dds_subscription_data.participant_key,
                PID_PARTICIPANT_GUID as u16,
                "participant_key",
            )?;
        }
        m.serialize_field(
            &self.dds_subscription_data.topic_name.as_str(),
            PID_TOPIC_NAME as u16,
            "topic_name",
        )?;
        m.serialize_field(
            &self.dds_subscription_data.type_name.as_str(),
            PID_TYPE_NAME as u16,
            "type_name",
        )?;
        if self.dds_subscription_data.durability != Default::default() {
            m.serialize_field(
                &self.dds_subscription_data.durability,
                PID_DURABILITY as u16,
                "durability",
            )?;
        }
        if self.dds_subscription_data.deadline != Default::default() {
            m.serialize_field(
                &self.dds_subscription_data.deadline,
                PID_DEADLINE as u16,
                "deadline",
            )?;
        }
        if self.dds_subscription_data.latency_budget != Default::default() {
            m.serialize_field(
                &self.dds_subscription_data.latency_budget,
                PID_LATENCY_BUDGET as u16,
                "latency_budget",
            )?;
        }
        if self.dds_subscription_data.liveliness != Default::default() {
            m.serialize_field(
                &self.dds_subscription_data.liveliness,
                PID_LIVELINESS as u16,
                "liveliness",
            )?;
        }
        if self.dds_subscription_data.reliability
            != DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS
        {
            m.serialize_field(
                &self.dds_subscription_data.reliability,
                PID_RELIABILITY as u16,
                "reliability",
            )?;
        }
        if self.dds_subscription_data.ownership != Default::default() {
            m.serialize_field(
                &self.dds_subscription_data.ownership,
                PID_OWNERSHIP as u16,
                "ownership",
            )?;
        }
        if self.dds_subscription_data.destination_order != Default::default() {
            m.serialize_field(
                &self.dds_subscription_data.destination_order,
                PID_DESTINATION_ORDER as u16,
                "destination_order",
            )?;
        }
        if self.dds_subscription_data.user_data != Default::default() {
            m.serialize_field(
                &self.dds_subscription_data.user_data,
                PID_USER_DATA as u16,
                "user_data",
            )?;
        }
        if self.dds_subscription_data.time_based_filter != Default::default() {
            m.serialize_field(
                &self.dds_subscription_data.time_based_filter,
                PID_TIME_BASED_FILTER as u16,
                "time_based_filter",
            )?;
        }
        if self.dds_subscription_data.presentation != Default::default() {
            m.serialize_field(
                &self.dds_subscription_data.presentation,
                PID_PRESENTATION as u16,
                "presentation",
            )?;
        }
        if self.dds_subscription_data.partition != Default::default() {
            m.serialize_field(
                &self.dds_subscription_data.partition,
                PID_PARTITION as u16,
                "partition",
            )?;
        }
        if self.dds_subscription_data.topic_data != Default::default() {
            m.serialize_field(
                &self.dds_subscription_data.topic_data,
                PID_TOPIC_DATA as u16,
                "topic_data",
            )?;
        }
        if self.dds_subscription_data.group_data != Default::default() {
            m.serialize_field(
                &self.dds_subscription_data.group_data,
                PID_GROUP_DATA as u16,
                "group_data",
            )?;
        }
        if !self.dds_subscription_data.xml_type.is_empty() {
            m.serialize_field(
                &self.dds_subscription_data.xml_type.as_str(),
                PID_TYPE_REPRESENTATION as u16,
                "xml_type",
            )?;
        }
        if self.dds_subscription_data.representation != Default::default() {
            m.serialize_field(
                &self.dds_subscription_data.representation,
                PID_DATA_REPRESENTATION as u16,
                "representation",
            )?;
        }
        m.end()
    }
}

impl<'de> XTypesDeserialize<'de> for DiscoveredReaderData {
    fn deserialize(deserializer: impl XTypesDeserializer<'de>) -> Result<Self, XcdrError> {
        let mut m = deserializer.deserialize_mutable_struct()?;
        let mut unicast_locator_list = Vec::new();
        while let Some(unicast_locator) =
            m.deserialize_optional_field(PID_UNICAST_LOCATOR as u16, "unicast_locator_list")?
        {
            unicast_locator_list.push(unicast_locator);
        }
        let mut multicast_locator_list = Vec::new();
        while let Some(multicast_locator) =
            m.deserialize_optional_field(PID_MULTICAST_LOCATOR as u16, "multicast_locator_list")?
        {
            multicast_locator_list.push(multicast_locator);
        }

        Ok(Self {
            reader_proxy: ReaderProxy {
                remote_reader_guid: m
                    .deserialize_field(PID_ENDPOINT_GUID as u16, "remote_reader_guid")?,
                remote_group_entity_id: m
                    .deserialize_optional_field(
                        PID_GROUP_ENTITYID as u16,
                        "remote_group_entity_id",
                    )?
                    .unwrap_or_default(),
                unicast_locator_list,
                multicast_locator_list,
                expects_inline_qos: m
                    .deserialize_optional_field(
                        PID_EXPECTS_INLINE_QOS as u16,
                        "expects_inline_qos",
                    )?
                    .unwrap_or(DEFAULT_EXPECTS_INLINE_QOS),
            },
            dds_subscription_data: SubscriptionBuiltinTopicData {
                key: m.deserialize_field(PID_ENDPOINT_GUID as u16, "key")?,
                participant_key: m
                    .deserialize_optional_field(PID_PARTICIPANT_GUID as u16, "participant_key")?
                    .unwrap_or_default(),
                topic_name: m
                    .deserialize_field::<&str>(PID_TOPIC_NAME as u16, "topic_name")?
                    .to_owned(),
                type_name: m
                    .deserialize_field::<&str>(PID_TYPE_NAME as u16, "type_name")?
                    .to_owned(),
                durability: m
                    .deserialize_optional_field(PID_DURABILITY as u16, "durability")?
                    .unwrap_or_default(),
                deadline: m
                    .deserialize_optional_field(PID_DEADLINE as u16, "deadline")?
                    .unwrap_or_default(),
                latency_budget: m
                    .deserialize_optional_field(PID_LATENCY_BUDGET as u16, "latency_budget")?
                    .unwrap_or_default(),
                liveliness: m
                    .deserialize_optional_field(PID_LIVELINESS as u16, "liveliness")?
                    .unwrap_or_default(),
                reliability: m
                    .deserialize_optional_field(PID_RELIABILITY as u16, "reliability")?
                    .unwrap_or(DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS),
                ownership: m
                    .deserialize_optional_field(PID_OWNERSHIP as u16, "ownership")?
                    .unwrap_or_default(),
                destination_order: m
                    .deserialize_optional_field(PID_DESTINATION_ORDER as u16, "destination_order")?
                    .unwrap_or_default(),
                user_data: m
                    .deserialize_optional_field(PID_USER_DATA as u16, "user_data")?
                    .unwrap_or_default(),
                time_based_filter: m
                    .deserialize_optional_field(PID_TIME_BASED_FILTER as u16, "time_based_filter")?
                    .unwrap_or_default(),
                presentation: m
                    .deserialize_optional_field(PID_PRESENTATION as u16, "presentation")?
                    .unwrap_or_default(),
                partition: m
                    .deserialize_optional_field(PID_PARTITION as u16, "partition")?
                    .unwrap_or_default(),
                topic_data: m
                    .deserialize_optional_field(PID_TOPIC_DATA as u16, "topic_data")?
                    .unwrap_or_default(),
                group_data: m
                    .deserialize_optional_field(PID_GROUP_DATA as u16, "group_data")?
                    .unwrap_or_default(),
                xml_type: m
                    .deserialize_optional_field::<&str>(PID_TYPE_REPRESENTATION as u16, "xml_type")?
                    .unwrap_or_default()
                    .to_owned(),
                representation: m
                    .deserialize_optional_field(PID_DATA_REPRESENTATION as u16, "representation")?
                    .unwrap_or_default(),
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
            dds_subscription_data: subscription_builtin_topic_data,
        }
    }

    pub fn reader_proxy(&self) -> &ReaderProxy {
        &self.reader_proxy
    }

    pub fn subscription_builtin_topic_data(&self) -> &SubscriptionBuiltinTopicData {
        &self.dds_subscription_data
    }
}

impl DdsHasKey for DiscoveredReaderData {
    const HAS_KEY: bool = true;
}

impl DdsKey for DiscoveredReaderData {
    type Key = [u8; 16];

    fn get_key(&self) -> DdsResult<Self::Key> {
        Ok(self.dds_subscription_data.key().value)
    }

    fn get_key_from_serialized_data(serialized_foo: &[u8]) -> DdsResult<Self::Key> {
        Ok(Self::deserialize_data(serialized_foo)?
            .dds_subscription_data
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
    use xtypes::{xcdr_deserializer::Xcdr1LeDeserializer, xcdr_serializer::Xcdr1LeSerializer};

    fn serialize_v1_le<T: XTypesSerialize, const N: usize>(v: &T) -> [u8; N] {
        let mut buffer = [0; N];
        v.serialize(&mut Xcdr1LeSerializer::new(&mut buffer))
            .unwrap();
        buffer
    }

    #[test]
    fn xtypes_serialize_all_default() {
        let data = DiscoveredReaderData {
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
            dds_subscription_data: SubscriptionBuiltinTopicData::new(
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

        let expected = [
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
            0x05, 0x00, 0x07, 0x00, // PID_TOPIC_NAME, Length
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'a', b'b', 0, 0x00, // string + padding (1 byte)
            0x07, 0x00, 0x07, 0x00, // PID_TYPE_NAME, Length
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'c', b'd', 0, 0x00, // string + padding (1 byte)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];

        assert_eq!(serialize_v1_le(&data), expected);
    }

    #[test]
    fn xtypes_deserialize_all_default() {
        let expected = Ok(DiscoveredReaderData {
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
            dds_subscription_data: SubscriptionBuiltinTopicData::new(
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
        });

        let data = [
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
            0x05, 0x00, 0x07, 0x00, // PID_TOPIC_NAME, Length
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'a', b'b', 0, 0x00, // string + padding (1 byte)
            0x07, 0x00, 0x07, 0x00, // PID_TYPE_NAME, Length
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'c', b'd', 0, 0x00, // string + padding (1 byte)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        assert_eq!(
            XTypesDeserialize::deserialize(&mut Xcdr1LeDeserializer::new(&data)),
            expected
        );
    }

    #[test]
    fn serialize_all_default() {
        let data = DiscoveredReaderData {
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
            dds_subscription_data: SubscriptionBuiltinTopicData::new(
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
            dds_subscription_data: SubscriptionBuiltinTopicData::new(
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
