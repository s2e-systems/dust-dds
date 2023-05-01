use std::io::Write;

use byteorder::ByteOrder;

use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    implementation::{
        parameter_list_serde::{
            parameter_list_deserializer::ParameterListDeserializer,
            parameter_list_serializer::ParameterListSerializer,
        },
        rtps::types::{EntityId, Guid, Locator},
    },
    infrastructure::{
        error::DdsResult,
        qos_policy::{ReliabilityQosPolicy, DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER},
    },
    topic_definition::type_support::{
        DdsDeserialize, DdsSerialize, DdsSerializedKey, DdsType, RepresentationType, PL_CDR_LE,
    },
};

use super::parameter_id_values::{
    PID_DATA_MAX_SIZE_SERIALIZED, PID_ENDPOINT_GUID, PID_GROUP_ENTITYID, PID_MULTICAST_LOCATOR,
    PID_UNICAST_LOCATOR,
};

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    derive_more::Into,
    derive_more::From,
)]
struct ReliabilityQosPolicyDataWriter(ReliabilityQosPolicy);
impl Default for ReliabilityQosPolicyDataWriter {
    fn default() -> Self {
        Self(DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER)
    }
}

const DEFAULT_DATA_MAX_SIZE_SERIALIZED: i32 = 0;
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
struct DataMaxSizeSerialized(i32);
impl Default for DataMaxSizeSerialized {
    fn default() -> Self {
        Self(DEFAULT_DATA_MAX_SIZE_SERIALIZED)
    }
}
impl From<Option<i32>> for DataMaxSizeSerialized {
    fn from(v: Option<i32>) -> Self {
        if let Some(data_max_size_serialized) = v {
            DataMaxSizeSerialized(data_max_size_serialized)
        } else {
            DataMaxSizeSerialized(DEFAULT_DATA_MAX_SIZE_SERIALIZED)
        }
    }
}
impl From<&DataMaxSizeSerialized> for Option<i32> {
    fn from(v: &DataMaxSizeSerialized) -> Self {
        if v.0 != DEFAULT_DATA_MAX_SIZE_SERIALIZED {
            Some(v.0)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct WriterProxy {
    remote_writer_guid: Guid,
    remote_group_entity_id: EntityId,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    data_max_size_serialized: Option<i32>,
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
            remote_writer_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            data_max_size_serialized,
        }
    }

    pub fn remote_writer_guid(&self) -> Guid {
        self.remote_writer_guid
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

    pub fn data_max_size_serialized(&self) -> Option<i32> {
        self.data_max_size_serialized
    }
}

impl DdsSerialize for WriterProxy {
    const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_LE;
    fn dds_serialize_parameter_list<W: Write>(
        &self,
        serializer: &mut ParameterListSerializer<W>,
    ) -> DdsResult<()> {
        // remote_writer_guid omitted as of Table 9.10 - Omitted Builtin Endpoint Parameters
        serializer
            .serialize_parameter_if_not_default(PID_GROUP_ENTITYID, &self.remote_group_entity_id)?;
        serializer.serialize_parameter_vector(PID_UNICAST_LOCATOR, &self.unicast_locator_list)?;
        serializer
            .serialize_parameter_vector(PID_MULTICAST_LOCATOR, &self.multicast_locator_list)?;
        serializer.serialize_parameter_if_not_default(
            PID_DATA_MAX_SIZE_SERIALIZED,
            &self.data_max_size_serialized,
        )
    }
}

impl<'de> DdsDeserialize<'de> for WriterProxy {
    fn dds_deserialize_parameter_list<E: ByteOrder>(
        deserializer: &mut ParameterListDeserializer<'de, E>,
    ) -> DdsResult<Self> {
        Ok(Self {
            remote_writer_guid: deserializer.get(PID_ENDPOINT_GUID)?,
            remote_group_entity_id: deserializer.get_or_default(PID_GROUP_ENTITYID)?,
            unicast_locator_list: deserializer.get_list(PID_UNICAST_LOCATOR)?,
            multicast_locator_list: deserializer.get_list(PID_MULTICAST_LOCATOR)?,
            data_max_size_serialized: deserializer.get_or_default(PID_DATA_MAX_SIZE_SERIALIZED)?,
        })
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

impl DdsType for DiscoveredWriterData {
    fn type_name() -> &'static str {
        "DiscoveredWriterData"
    }

    fn has_key() -> bool {
        true
    }

    fn get_serialized_key(&self) -> DdsSerializedKey {
        self.dds_publication_data.key().value.as_ref().into()
    }

    fn set_key_fields_from_serialized_key(&mut self, _key: &DdsSerializedKey) -> DdsResult<()> {
        if Self::has_key() {
            unimplemented!("DdsType with key must provide an implementation for set_key_fields_from_serialized_key")
        }
        Ok(())
    }
}

impl DdsSerialize for DiscoveredWriterData {
    const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_LE;

    fn dds_serialize_parameter_list<W: Write>(
        &self,
        serializer: &mut ParameterListSerializer<W>,
    ) -> DdsResult<()> {
        self.dds_publication_data
            .dds_serialize_parameter_list(serializer)?;
        self.writer_proxy.dds_serialize_parameter_list(serializer)
    }
}

impl<'de> DdsDeserialize<'de> for DiscoveredWriterData {
    fn dds_deserialize_parameter_list<E: ByteOrder>(
        deserializer: &mut ParameterListDeserializer<'de, E>,
    ) -> DdsResult<Self> {
        Ok(Self {
            dds_publication_data: DdsDeserialize::dds_deserialize_parameter_list(deserializer)?,
            writer_proxy: DdsDeserialize::dds_deserialize_parameter_list(deserializer)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::builtin_topics::BuiltInTopicKey;
    use crate::implementation::rtps::types::{
        EntityKey, GuidPrefix, BUILT_IN_PARTICIPANT, BUILT_IN_READER_GROUP,
        BUILT_IN_WRITER_WITH_KEY, USER_DEFINED_UNKNOWN,
    };
    use crate::infrastructure::qos_policy::{
        DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
        LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy,
        PartitionQosPolicy, PresentationQosPolicy, TopicDataQosPolicy, UserDataQosPolicy,
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
    };

    use super::*;

    fn to_bytes_le<S: DdsSerialize + serde::Serialize>(value: &S) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        value.dds_serialize(&mut writer).unwrap();
        writer
    }
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
                DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER.into(),
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
                    GuidPrefix::new([5; 12]),
                    EntityId::new(EntityKey::new([11, 12, 13]), BUILT_IN_WRITER_WITH_KEY),
                ),
                EntityId::new(EntityKey::new([21, 22, 23]), BUILT_IN_READER_GROUP),
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
        assert_eq!(to_bytes_le(&data), expected);
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
                DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER.into(),
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
                    GuidPrefix::new([1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0]),
                    EntityId::new(EntityKey::new([4, 0, 0]), USER_DEFINED_UNKNOWN),
                ),
                EntityId::new(EntityKey::new([21, 22, 23]), BUILT_IN_PARTICIPANT),
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
        let result: DiscoveredWriterData = DdsDeserialize::dds_deserialize(&mut data).unwrap();
        assert_eq!(result, expected);
    }
}
