use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    cdr::{
        deserialize::CdrDeserialize,
        deserializer::CdrDeserializer,
        error::CdrResult,
        representation::{CdrRepresentation, CdrRepresentationKind},
        serialize::CdrSerialize,
        serializer::CdrSerializer,
    },
    implementation::{
        parameter_list_serde::parameter::{Parameter, ParameterVector, ParameterWithDefault},
        rtps::types::{EntityId, Guid, Locator},
    },
    infrastructure::error::DdsResult,
    topic_definition::type_support::{
        DdsDeserialize, DdsGetKeyFromFoo, DdsGetKeyFromSerializedData, DdsHasKey, DdsSerializedKey,
    },
};

use super::parameter_id_values::{
    PID_DATA_MAX_SIZE_SERIALIZED, PID_ENDPOINT_GUID, PID_GROUP_ENTITYID, PID_MULTICAST_LOCATOR,
    PID_UNICAST_LOCATOR,
};
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WriterProxy {
    remote_writer_guid: Parameter<PID_ENDPOINT_GUID, Guid>,
    remote_group_entity_id: ParameterWithDefault<PID_GROUP_ENTITYID, EntityId>,
    unicast_locator_list: ParameterVector<PID_UNICAST_LOCATOR, Locator>,
    multicast_locator_list: ParameterVector<PID_MULTICAST_LOCATOR, Locator>,
    data_max_size_serialized: ParameterWithDefault<PID_DATA_MAX_SIZE_SERIALIZED, i32>,
}

impl CdrSerialize for WriterProxy {
    fn serialize(&self, serializer: &mut CdrSerializer) -> CdrResult<()> {
        // remote_writer_guid omitted as of Table 9.10 - Omitted Builtin Endpoint Parameters
        self.remote_group_entity_id.serialize(serializer)?;
        self.unicast_locator_list.serialize(serializer)?;
        self.multicast_locator_list.serialize(serializer)?;
        self.data_max_size_serialized.serialize(serializer)?;
        Ok(())
    }
}

impl<'de> CdrDeserialize<'de> for WriterProxy {
    fn deserialize(deserializer: &mut CdrDeserializer<'de>) -> CdrResult<Self> {
        todo!()
    }
}

impl WriterProxy {
    pub fn new(
        remote_writer_guid: Guid,
        remote_group_entity_id: EntityId,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        data_max_size_serialized: Option<i32>,
    ) -> Self {
        Self {
            remote_writer_guid: remote_writer_guid.into(),
            remote_group_entity_id: remote_group_entity_id.into(),
            unicast_locator_list: unicast_locator_list.into(),
            multicast_locator_list: multicast_locator_list.into(),
            data_max_size_serialized: data_max_size_serialized.unwrap_or_default().into(),
        }
    }

    pub fn remote_writer_guid(&self) -> Guid {
        *self.remote_writer_guid.as_ref()
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

    pub fn data_max_size_serialized(&self) -> Option<i32> {
        Some(*self.data_max_size_serialized.as_ref())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, CdrSerialize, CdrDeserialize)]
pub struct DiscoveredWriterData {
    dds_publication_data: PublicationBuiltinTopicData,
    writer_proxy: WriterProxy,
}

impl DiscoveredWriterData {
    pub fn new(
        dds_publication_data: PublicationBuiltinTopicData,
        writer_proxy: WriterProxy,
    ) -> Self {
        Self {
            dds_publication_data,
            writer_proxy,
        }
    }

    pub fn dds_publication_data(&self) -> &PublicationBuiltinTopicData {
        &self.dds_publication_data
    }

    pub fn writer_proxy(&self) -> &WriterProxy {
        &self.writer_proxy
    }
}

pub const DCPS_PUBLICATION: &str = "DCPSPublication";

impl DdsHasKey for DiscoveredWriterData {
    const HAS_KEY: bool = true;
}

impl CdrRepresentation for DiscoveredWriterData {
    const REPRESENTATION: CdrRepresentationKind = CdrRepresentationKind::PlCdrLe;
}

impl DdsGetKeyFromFoo for DiscoveredWriterData {
    fn get_key_from_foo(&self) -> DdsResult<DdsSerializedKey> {
        Ok(self.dds_publication_data.key().value.to_vec().into())
    }
}

impl DdsGetKeyFromSerializedData for DiscoveredWriterData {
    fn get_key_from_serialized_data(serialized_data: &[u8]) -> DdsResult<DdsSerializedKey> {
        Ok(Self::deserialize_data(serialized_data)?
            .dds_publication_data
            .key()
            .value
            .to_vec()
            .into())
    }
}

#[cfg(test)]
mod tests {
    use crate::builtin_topics::BuiltInTopicKey;
    use crate::implementation::rtps::types::{
        BUILT_IN_PARTICIPANT, BUILT_IN_READER_GROUP, BUILT_IN_WRITER_WITH_KEY, USER_DEFINED_UNKNOWN,
    };
    use crate::infrastructure::qos_policy::{
        DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
        LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy,
        PartitionQosPolicy, PresentationQosPolicy, TopicDataQosPolicy, UserDataQosPolicy,
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
    };
    use crate::topic_definition::type_support::DdsSerializeData;

    use super::*;

    #[test]
    fn serialize_all_default() {
        let data = DiscoveredWriterData::new(
            PublicationBuiltinTopicData::new(
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
                DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
                LifespanQosPolicy::default(),
                UserDataQosPolicy::default(),
                OwnershipQosPolicy::default(),
                DestinationOrderQosPolicy::default(),
                PresentationQosPolicy::default(),
                PartitionQosPolicy::default(),
                TopicDataQosPolicy::default(),
                GroupDataQosPolicy::default(),
            ),
            WriterProxy::new(
                Guid::new(
                    [5; 12],
                    EntityId::new([11, 12, 13], BUILT_IN_WRITER_WITH_KEY),
                ),
                EntityId::new([21, 22, 23], BUILT_IN_READER_GROUP),
                vec![],
                vec![],
                None,
            ),
        );

        let expected = vec![
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
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
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            21, 22, 23, 0xc9, // u8[3], u8
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        let result = data.serialize_data().unwrap();
        assert_eq!(result, expected.into());
    }

    #[test]
    fn deserialize_all_default() {
        let expected = DiscoveredWriterData::new(
            PublicationBuiltinTopicData::new(
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
                DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
                LifespanQosPolicy::default(),
                UserDataQosPolicy::default(),
                OwnershipQosPolicy::default(),
                DestinationOrderQosPolicy::default(),
                PresentationQosPolicy::default(),
                PartitionQosPolicy::default(),
                TopicDataQosPolicy::default(),
                GroupDataQosPolicy::default(),
            ),
            WriterProxy::new(
                // must correspond to publication_builtin_topic_data.key
                Guid::new(
                    [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0],
                    EntityId::new([4, 0, 0], USER_DEFINED_UNKNOWN),
                ),
                EntityId::new([21, 22, 23], BUILT_IN_PARTICIPANT),
                vec![],
                vec![],
                None,
            ),
        );

        let mut data = &[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            21, 22, 23, 0xc1, // u8[3], u8
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
        let result = DiscoveredWriterData::deserialize_data(&mut data).unwrap();
        assert_eq!(result, expected);
    }
}
