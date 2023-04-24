use std::io::Write;

use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData},
    implementation::{
        parameter_list_serde::{
            parameter_list_deserializer::ParameterListDeserializer,
            parameter_list_serializer::ParameterListSerializer,
        },
        rtps::types::{EntityId, Guid, Locator},
    },
    infrastructure::{
        error::DdsResult,
        qos_policy::{
            DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
            LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy,
            PartitionQosPolicy, PresentationQosPolicy, ReliabilityQosPolicy, TopicDataQosPolicy,
            UserDataQosPolicy, DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
        },
    },
    topic_definition::type_support::{
        DdsDeserialize, DdsSerialize, DdsSerializedKey, DdsType, RepresentationType, PL_CDR_LE,
    },
};

use super::parameter_id_values::{
    PID_DATA_MAX_SIZE_SERIALIZED, PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY,
    PID_ENDPOINT_GUID, PID_GROUP_DATA, PID_GROUP_ENTITYID, PID_LATENCY_BUDGET, PID_LIFESPAN,
    PID_LIVELINESS, PID_MULTICAST_LOCATOR, PID_OWNERSHIP, PID_PARTICIPANT_GUID, PID_PARTITION,
    PID_PRESENTATION, PID_RELIABILITY, PID_TOPIC_DATA, PID_TOPIC_NAME, PID_TYPE_NAME,
    PID_UNICAST_LOCATOR, PID_USER_DATA,
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DiscoveredWriterData {
    // WriterProxy:
    remote_writer_guid: Guid,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    data_max_size_serialized: DataMaxSizeSerialized,
    remote_group_entity_id: EntityId,
    key: BuiltInTopicKey,
    participant_key: BuiltInTopicKey,
    topic_name: String,
    type_name: String,
    durability: DurabilityQosPolicy,
    deadline: DeadlineQosPolicy,
    latency_budget: LatencyBudgetQosPolicy,
    liveliness: LivelinessQosPolicy,
    reliability: ReliabilityQosPolicyDataWriter,
    lifespan: LifespanQosPolicy,
    user_data: UserDataQosPolicy,
    ownership: OwnershipQosPolicy,
    destination_order: DestinationOrderQosPolicy,
    presentation: PresentationQosPolicy,
    partition: PartitionQosPolicy,
    topic_data: TopicDataQosPolicy,
    group_data: GroupDataQosPolicy,
}

impl DiscoveredWriterData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        remote_writer_guid: Guid,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        data_max_size_serialized: Option<i32>,
        remote_group_entity_id: EntityId,
        key: BuiltInTopicKey,
        participant_key: BuiltInTopicKey,
        topic_name: String,
        type_name: String,
        durability: DurabilityQosPolicy,
        deadline: DeadlineQosPolicy,
        latency_budget: LatencyBudgetQosPolicy,
        liveliness: LivelinessQosPolicy,
        reliability: ReliabilityQosPolicy,
        lifespan: LifespanQosPolicy,
        user_data: UserDataQosPolicy,
        ownership: OwnershipQosPolicy,
        destination_order: DestinationOrderQosPolicy,
        presentation: PresentationQosPolicy,
        partition: PartitionQosPolicy,
        topic_data: TopicDataQosPolicy,
        group_data: GroupDataQosPolicy,
    ) -> Self {
        Self {
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
            data_max_size_serialized: data_max_size_serialized.into(),
            remote_group_entity_id,
            key,
            participant_key,
            topic_name,
            type_name,
            durability,
            deadline,
            latency_budget,
            liveliness,
            reliability: reliability.into(),
            lifespan,
            user_data,
            ownership,
            destination_order,
            presentation,
            partition,
            topic_data,
            group_data,
        }
    }

    pub fn remote_writer_guid(&self) -> Guid {
        self.remote_writer_guid
    }
    pub fn unicast_locator_list(&self) -> &[Locator] {
        &self.unicast_locator_list
    }
    pub fn multicast_locator_list(&self) -> &[Locator] {
        &self.multicast_locator_list
    }
    pub fn data_max_size_serialized(&self) -> Option<i32> {
        (&self.data_max_size_serialized).into()
    }
    pub fn remote_group_entity_id(&self) -> EntityId {
        self.remote_group_entity_id
    }

    pub fn publication_builtin_topic_data(self) -> PublicationBuiltinTopicData {
        PublicationBuiltinTopicData::new(
            self.key,
            self.participant_key,
            self.topic_name,
            self.type_name,
            self.durability,
            self.deadline,
            self.latency_budget,
            self.liveliness,
            self.reliability.into(),
            self.lifespan,
            self.user_data,
            self.ownership,
            self.destination_order,
            self.presentation,
            self.partition,
            self.topic_data,
            self.group_data,
        )
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
        self.key.value.as_ref().into()
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
    fn dds_serialize<W: Write>(&self, writer: W) -> DdsResult<()> {
        let mut parameter_list_serializer = ParameterListSerializer::new(writer);
        parameter_list_serializer.serialize_payload_header()?;

        // remote_writer_guid omitted as of Table 9.10 - Omitted Builtin Endpoint Parameters
        parameter_list_serializer
            .serialize_parameter_vector(PID_UNICAST_LOCATOR, &self.unicast_locator_list)?;
        parameter_list_serializer
            .serialize_parameter_vector(PID_MULTICAST_LOCATOR, &self.multicast_locator_list)?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_DATA_MAX_SIZE_SERIALIZED,
            &self.data_max_size_serialized,
        )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default(PID_GROUP_ENTITYID, &self.remote_group_entity_id)?;
        parameter_list_serializer.serialize_parameter(PID_ENDPOINT_GUID, &self.key)?;
        // Default value is a deviation from the standard and is used for interoperability reasons:
        parameter_list_serializer
            .serialize_parameter_if_not_default(PID_PARTICIPANT_GUID, &self.participant_key)?;
        parameter_list_serializer.serialize_parameter(PID_TOPIC_NAME, &self.topic_name)?;
        parameter_list_serializer.serialize_parameter(PID_TYPE_NAME, &self.type_name)?;
        parameter_list_serializer
            .serialize_parameter_if_not_default(PID_DURABILITY, &self.durability)?;
        parameter_list_serializer
            .serialize_parameter_if_not_default(PID_DEADLINE, &self.deadline)?;
        parameter_list_serializer
            .serialize_parameter_if_not_default(PID_LATENCY_BUDGET, &self.latency_budget)?;
        parameter_list_serializer
            .serialize_parameter_if_not_default(PID_LIVELINESS, &self.liveliness)?;
        parameter_list_serializer
            .serialize_parameter_if_not_default(PID_RELIABILITY, &self.reliability)?;
        parameter_list_serializer
            .serialize_parameter_if_not_default(PID_LIFESPAN, &self.lifespan)?;
        parameter_list_serializer
            .serialize_parameter_if_not_default(PID_USER_DATA, &self.user_data)?;
        parameter_list_serializer
            .serialize_parameter_if_not_default(PID_OWNERSHIP, &self.ownership)?;
        parameter_list_serializer
            .serialize_parameter_if_not_default(PID_DESTINATION_ORDER, &self.destination_order)?;
        parameter_list_serializer
            .serialize_parameter_if_not_default(PID_PRESENTATION, &self.presentation)?;
        parameter_list_serializer
            .serialize_parameter_if_not_default(PID_PARTITION, &self.partition)?;
        parameter_list_serializer
            .serialize_parameter_if_not_default(PID_TOPIC_DATA, &self.topic_data)?;
        parameter_list_serializer
            .serialize_parameter_if_not_default(PID_GROUP_DATA, &self.group_data)?;
        parameter_list_serializer.serialize_sentinel()
    }
}

impl DdsDeserialize<'_> for DiscoveredWriterData {
    fn deserialize(buf: &mut &'_ [u8]) -> DdsResult<Self> {
        let param_list = ParameterListDeserializer::read(buf)?;

        //writer_proxy
        let unicast_locator_list = param_list.get_list(PID_UNICAST_LOCATOR)?;
        let multicast_locator_list = param_list.get_list(PID_MULTICAST_LOCATOR)?;
        let data_max_size_serialized = param_list.get_or_default(PID_DATA_MAX_SIZE_SERIALIZED)?;
        let remote_group_entity_id = param_list.get_or_default(PID_GROUP_ENTITYID)?;

        // publication_builtin_topic_data
        let key = param_list.get::<BuiltInTopicKey>(PID_ENDPOINT_GUID)?;
        // Default value is a deviation from the standard and is used for interoperability reasons:
        let participant_key = param_list.get_or_default(PID_PARTICIPANT_GUID)?;
        let topic_name = param_list.get(PID_TOPIC_NAME)?;
        let type_name = param_list.get(PID_TYPE_NAME)?;
        let durability = param_list.get_or_default(PID_DURABILITY)?;
        let deadline = param_list.get_or_default(PID_DEADLINE)?;
        let latency_budget = param_list.get_or_default(PID_LATENCY_BUDGET)?;
        let liveliness = param_list.get_or_default(PID_LIVELINESS)?;
        let reliability = param_list.get_or_default(PID_RELIABILITY)?;
        let lifespan = param_list.get_or_default(PID_LIFESPAN)?;
        let user_data = param_list.get_or_default(PID_USER_DATA)?;
        let ownership = param_list.get_or_default(PID_OWNERSHIP)?;
        let destination_order = param_list.get_or_default(PID_DESTINATION_ORDER)?;
        let presentation = param_list.get_or_default(PID_PRESENTATION)?;
        let partition = param_list.get_or_default(PID_PARTITION)?;
        let topic_data = param_list.get_or_default(PID_TOPIC_DATA)?;
        let group_data = param_list.get_or_default(PID_GROUP_DATA)?;

        let remote_writer_guid = key.value.into();
        Ok(Self {
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
            data_max_size_serialized,
            remote_group_entity_id,
            key,
            participant_key,
            topic_name,
            type_name,
            durability,
            deadline,
            latency_budget,
            liveliness,
            reliability,
            lifespan,
            user_data,
            ownership,
            destination_order,
            presentation,
            partition,
            topic_data,
            group_data,
        })
    }
}

#[cfg(test)]
mod tests {
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

    fn to_bytes_le<S: DdsSerialize>(value: &S) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        value.dds_serialize(&mut writer).unwrap();
        writer
    }
    #[test]
    fn serialize_all_default() {
        let data = DiscoveredWriterData::new(
            Guid::new(
                GuidPrefix::new([5; 12]),
                EntityId::new(EntityKey::new([11, 12, 13]), BUILT_IN_WRITER_WITH_KEY),
            ),
            vec![],
            vec![],
            None,
            EntityId::new(EntityKey::new([21, 22, 23]), BUILT_IN_READER_GROUP),
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
        );

        let expected = vec![
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            21, 22, 23, 0xc9, // u8[3], u8
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
        let expected = DiscoveredWriterData::new(
            // must correspond to publication_builtin_topic_data.key
            Guid::new(
                GuidPrefix::new([1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0]),
                EntityId::new(EntityKey::new([4, 0, 0]), USER_DEFINED_UNKNOWN),
            ),
            vec![],
            vec![],
            None,
            EntityId::new(EntityKey::new([21, 22, 23]), BUILT_IN_PARTICIPANT),
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
        let result: DiscoveredWriterData = DdsDeserialize::deserialize(&mut data).unwrap();
        assert_eq!(result, expected);
    }
}
