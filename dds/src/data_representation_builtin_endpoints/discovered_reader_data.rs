use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    infrastructure::error::DdsResult,
    rtps::types::{EntityId, Guid, Locator},
    serialized_payload::parameter_list::{
        deserialize::ParameterListDeserialize, serialize::ParameterListSerialize,
    },
    topic_definition::type_support::{DdsDeserialize, DdsHasKey, DdsKey, DdsSerialize, DdsTypeXml},
};

use super::parameter_id_values::{
    DEFAULT_EXPECTS_INLINE_QOS, PID_ENDPOINT_GUID, PID_EXPECTS_INLINE_QOS, PID_GROUP_ENTITYID,
    PID_MULTICAST_LOCATOR, PID_UNICAST_LOCATOR,
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
    subscription_builtin_topic_data: SubscriptionBuiltinTopicData,
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
