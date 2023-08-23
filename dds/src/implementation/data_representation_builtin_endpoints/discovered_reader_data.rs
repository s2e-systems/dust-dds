use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    implementation::{
        parameter_list_serde::parameter::{Parameter, ParameterVector, ParameterWithDefault},
        rtps::types::{EntityId, Guid, Locator},
    },
    infrastructure::error::DdsResult,
    topic_definition::type_support::{DdsSerializedKey, DdsType, RepresentationType, PL_CDR_LE},
};

use super::parameter_id_values::{
    DEFAULT_EXPECTS_INLINE_QOS, PID_ENDPOINT_GUID, PID_EXPECTS_INLINE_QOS, PID_GROUP_ENTITYID,
    PID_MULTICAST_LOCATOR, PID_UNICAST_LOCATOR,
};

pub const DCPS_SUBSCRIPTION: &str = "DCPSSubscription";

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    derive_more::From,
    derive_more::AsRef,
    serde::Serialize,
    serde::Deserialize,
)]
struct ExpectsInlineQos(bool);
impl Default for ExpectsInlineQos {
    fn default() -> Self {
        Self(DEFAULT_EXPECTS_INLINE_QOS)
    }
}
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReaderProxy {
    #[serde(skip_serializing)]
    remote_reader_guid: Parameter<PID_ENDPOINT_GUID, Guid>,
    remote_group_entity_id: ParameterWithDefault<PID_GROUP_ENTITYID, EntityId>,
    unicast_locator_list: ParameterVector<PID_UNICAST_LOCATOR, Locator>,
    multicast_locator_list: ParameterVector<PID_MULTICAST_LOCATOR, Locator>,
    expects_inline_qos: ParameterWithDefault<PID_EXPECTS_INLINE_QOS, ExpectsInlineQos>,
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
            remote_reader_guid: remote_reader_guid.into(),
            remote_group_entity_id: remote_group_entity_id.into(),
            unicast_locator_list: unicast_locator_list.into(),
            multicast_locator_list: multicast_locator_list.into(),
            expects_inline_qos: ExpectsInlineQos::from(expects_inline_qos).into(),
        }
    }

    pub fn remote_reader_guid(&self) -> Guid {
        *self.remote_reader_guid.as_ref()
    }

    pub fn remote_group_entity_id(&self) -> EntityId {
        *self.remote_group_entity_id.as_ref()
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.unicast_locator_list.as_ref()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.multicast_locator_list.as_ref()
    }

    pub fn expects_inline_qos(&self) -> bool {
        *self.expects_inline_qos.as_ref().as_ref()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
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

impl DdsType for DiscoveredReaderData {
    const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_LE;

    fn has_key() -> bool {
        true
    }

    fn get_serialized_key(&self) -> DdsSerializedKey {
        self.subscription_builtin_topic_data
            .key()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtin_topics::BuiltInTopicKey;
    use crate::implementation::rtps::types::{
        BUILT_IN_WRITER_WITH_KEY, USER_DEFINED_READER_WITH_KEY, USER_DEFINED_UNKNOWN,
    };
    use crate::infrastructure::qos_policy::{
        DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
        LatencyBudgetQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, PartitionQosPolicy,
        PresentationQosPolicy, TimeBasedFilterQosPolicy, TopicDataQosPolicy, UserDataQosPolicy,
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
    };
    use crate::topic_definition::type_support::{DdsDeserialize, DdsSerialize};

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
                DurabilityQosPolicy::default(),
                DeadlineQosPolicy::default(),
                LatencyBudgetQosPolicy::default(),
                LivelinessQosPolicy::default(),
                DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
                OwnershipQosPolicy::default(),
                DestinationOrderQosPolicy::default(),
                UserDataQosPolicy { value: vec![] },
                TimeBasedFilterQosPolicy::default(),
                PresentationQosPolicy::default(),
                PartitionQosPolicy::default(),
                TopicDataQosPolicy::default(),
                GroupDataQosPolicy::default(),
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
        let result = data.dds_serialize().unwrap();
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
                DurabilityQosPolicy::default(),
                DeadlineQosPolicy::default(),
                LatencyBudgetQosPolicy::default(),
                LivelinessQosPolicy::default(),
                DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
                OwnershipQosPolicy::default(),
                DestinationOrderQosPolicy::default(),
                UserDataQosPolicy::default(),
                TimeBasedFilterQosPolicy::default(),
                PresentationQosPolicy::default(),
                PartitionQosPolicy::default(),
                TopicDataQosPolicy::default(),
                GroupDataQosPolicy::default(),
            ),
        };

        let data = &[
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
        let result = DiscoveredReaderData::dds_deserialize(data).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_reader_proxy() {
        let expected = ReaderProxy::new(
            Guid::new(
                [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0],
                EntityId::new([4, 0, 0], USER_DEFINED_UNKNOWN),
            ),
            EntityId::new([21, 22, 23], BUILT_IN_WRITER_WITH_KEY),
            vec![],
            vec![],
            false,
        );

        let data = &[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x5a, 0x00, 16,
            0, //PID_ENDPOINT_GUID, length (SubscriptionBuiltinTopicData::key) used for remote_reader_guid
            1, 0, 0, 0, // ,
            2, 0, 0, 0, // ,
            3, 0, 0, 0, // ,
            4, 0, 0, 0, // ,
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID (remote_group_entity_id)
            21, 22, 23, 0xc2, // u8[3], u8
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ][..];
        let result = ReaderProxy::dds_deserialize(data).unwrap();
        assert_eq!(result, expected);
    }
}
