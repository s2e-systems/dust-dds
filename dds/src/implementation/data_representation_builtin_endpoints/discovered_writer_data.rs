use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    implementation::{
        parameter_list_serde::parameter::{Parameter, ParameterVector, ParameterWithDefault},
        rtps::types::{EntityId, Guid, Locator},
    },
    topic_definition::type_support::{
        DdsGetKey, DdsRepresentation, DdsHasKey, RepresentationType, PL_CDR_LE,
    },
};

use super::parameter_id_values::{
    PID_DATA_MAX_SIZE_SERIALIZED, PID_ENDPOINT_GUID, PID_GROUP_ENTITYID, PID_MULTICAST_LOCATOR,
    PID_UNICAST_LOCATOR,
};
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct WriterProxy {
    // remote_writer_guid omitted as of Table 9.10 - Omitted Builtin Endpoint Parameters
    #[serde(skip_serializing)]
    remote_writer_guid: Parameter<PID_ENDPOINT_GUID, Guid>,
    remote_group_entity_id: ParameterWithDefault<PID_GROUP_ENTITYID, EntityId>,
    unicast_locator_list: ParameterVector<PID_UNICAST_LOCATOR, Locator>,
    multicast_locator_list: ParameterVector<PID_MULTICAST_LOCATOR, Locator>,
    data_max_size_serialized: ParameterWithDefault<PID_DATA_MAX_SIZE_SERIALIZED, Option<i32>>,
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
            data_max_size_serialized: data_max_size_serialized.into(),
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
        *self.data_max_size_serialized.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
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
    fn has_key() -> bool {
        true
    }
}

impl DdsRepresentation for DiscoveredWriterData {
    const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_LE;
}

impl DdsGetKey for DiscoveredWriterData {
    type BorrowedKeyHolder<'a> = [u8; 16];
    type OwningKeyHolder = [u8; 16];

    fn get_key(&self) -> Self::BorrowedKeyHolder<'_> {
        self.dds_publication_data.key().value
    }

    fn set_key_from_holder(&mut self, _key_holder: Self::OwningKeyHolder) {
        todo!()
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
    use crate::topic_definition::type_support::{
        dds_deserialize_from_bytes, dds_serialize_to_bytes,
    };

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
        let result = dds_serialize_to_bytes(&data).unwrap();
        assert_eq!(result, expected);
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

        let data = &[
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
        let result = dds_deserialize_from_bytes::<DiscoveredWriterData>(data).unwrap();
        assert_eq!(result, expected);
    }
}
