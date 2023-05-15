use byteorder::ByteOrder;

use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    implementation::{
        parameter_list_serde::{
            parameter_list_deserializer::ParameterListDeserializer, parameter::{Parameter, ParameterVector, ParameterWithDefault},
        },
        rtps::types::{EntityId, ExpectsInlineQos, Guid, Locator},
    },
    infrastructure::error::DdsResult,
    topic_definition::type_support::{
        DdsDeserialize, DdsSerializedKey, DdsType, RepresentationType, PL_CDR_LE, RepresentationFormat,
    },
};

use super::parameter_id_values::{
    PID_ENDPOINT_GUID, PID_EXPECTS_INLINE_QOS, PID_GROUP_ENTITYID, PID_MULTICAST_LOCATOR,
    PID_UNICAST_LOCATOR,
};

pub const DCPS_SUBSCRIPTION: &str = "DCPSSubscription";

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReaderProxy {
    #[serde(skip_serializing)]
    remote_reader_guid: Parameter<PID_ENDPOINT_GUID, Guid>,
    remote_group_entity_id: Parameter<PID_GROUP_ENTITYID, EntityId>,
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
        self.remote_reader_guid.0
    }

    pub fn remote_group_entity_id(&self) -> EntityId {
        self.remote_group_entity_id.0
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.unicast_locator_list.0.as_ref()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.multicast_locator_list.0.as_ref()
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos.0.into()
    }
}


impl<'de> DdsDeserialize<'de> for ReaderProxy {
    fn dds_deserialize_parameter_list<E: ByteOrder>(
        deserializer: &mut ParameterListDeserializer<'de, E>,
    ) -> DdsResult<Self> {
        // Ok(Self {
        //     remote_reader_guid: deserializer.get(PID_ENDPOINT_GUID)?,
        //     remote_group_entity_id: deserializer.get_or_default(PID_GROUP_ENTITYID)?,
        //     unicast_locator_list: deserializer.get_list(PID_UNICAST_LOCATOR)?,
        //     multicast_locator_list: deserializer.get_list(PID_MULTICAST_LOCATOR)?,
        //     expects_inline_qos: deserializer.get_or_default(PID_MULTICAST_LOCATOR)?,
        // })
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DiscoveredReaderData {
    reader_proxy: ReaderProxy,
    subscription_builtin_topic_data: SubscriptionBuiltinTopicData,
}

impl RepresentationFormat for DiscoveredReaderData {
    const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_LE;
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
    fn type_name() -> &'static str {
        "DiscoveredReaderData"
    }

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


impl<'de> DdsDeserialize<'de> for DiscoveredReaderData {
    fn dds_deserialize_parameter_list<E: ByteOrder>(
        deserializer: &mut ParameterListDeserializer<'de, E>,
    ) -> DdsResult<Self> {
        Ok(Self {
            reader_proxy: DdsDeserialize::dds_deserialize_parameter_list(deserializer)?,
            subscription_builtin_topic_data: DdsDeserialize::dds_deserialize_parameter_list(
                deserializer,
            )?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtin_topics::BuiltInTopicKey;
    use crate::implementation::parameter_list_serde::serde_parameter_list_serializer::dds_serialize;
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

    #[test]
    fn serialize_all_default() {
        let data = DiscoveredReaderData{
            reader_proxy: ReaderProxy::new(
                Guid::new(
                    GuidPrefix::new([5; 12]),
                    EntityId::new(EntityKey::new([11, 12, 13]), USER_DEFINED_READER_WITH_KEY),
                ),
                EntityId::new(
                    EntityKey::new([21, 22, 23]),
                    BUILT_IN_WRITER_WITH_KEY,
                ),
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
        let result = dds_serialize(&data).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_all_default() {
        let expected = DiscoveredReaderData {
            reader_proxy: ReaderProxy::new(
                // must correspond to subscription_builtin_topic_data.key
                Guid::new(
                    GuidPrefix::new([1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0]),
                    EntityId::new(EntityKey::new([4, 0, 0]), USER_DEFINED_UNKNOWN),
                ),
                EntityId::new(
                    EntityKey::new([21, 22, 23]),
                    BUILT_IN_WRITER_WITH_KEY,
                ),
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
        let result: DiscoveredReaderData = DdsDeserialize::dds_deserialize(&mut data).unwrap();
        assert_eq!(result, expected);
    }
}
