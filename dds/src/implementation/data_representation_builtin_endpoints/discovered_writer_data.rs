use std::io::Write;

use crate::builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData};
use crate::implementation::rtps::types::{EntityId, Guid, Locator};
use crate::infrastructure::error::DdsResult;
use crate::infrastructure::qos_policy::{
    DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
    LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy,
    PartitionQosPolicy, PresentationQosPolicy, ReliabilityQosPolicy, TopicDataQosPolicy,
    UserDataQosPolicy, DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
};
use crate::topic_definition::type_support::LittleEndian;
use crate::{
    implementation::parameter_list_serde::{
        parameter_list_deserializer::ParameterListDeserializer,
        parameter_list_serializer::ParameterListSerializer,
    },
    topic_definition::type_support::{DdsDeserialize, DdsSerialize, DdsType, Endianness},
};

use super::parameter_id_values::{
    PID_DATA_MAX_SIZE_SERIALIZED, PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY,
    PID_ENDPOINT_GUID, PID_GROUP_DATA, PID_GROUP_ENTITYID, PID_LATENCY_BUDGET, PID_LIFESPAN,
    PID_LIVELINESS, PID_MULTICAST_LOCATOR, PID_OWNERSHIP, PID_PARTICIPANT_GUID, PID_PARTITION,
    PID_PRESENTATION, PID_RELIABILITY, PID_TOPIC_DATA, PID_TOPIC_NAME, PID_TYPE_NAME,
    PID_UNICAST_LOCATOR, PID_USER_DATA,
};

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
pub struct ReliabilityQosPolicyDataWriter<'a>(pub &'a ReliabilityQosPolicy);
impl<'a> Default for ReliabilityQosPolicyDataWriter<'a> {
    fn default() -> Self {
        Self(&DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER)
    }
}
#[derive(Debug, PartialEq, Eq, serde::Serialize, derive_more::From)]
pub struct ReliabilityQosPolicyDataWriterSerialize<'a>(pub &'a ReliabilityQosPolicyDataWriter<'a>);

#[derive(Debug, PartialEq, Eq, serde::Deserialize, derive_more::Into)]
pub struct ReliabilityQosPolicyDataWriterDeserialize(pub ReliabilityQosPolicy);
impl Default for ReliabilityQosPolicyDataWriterDeserialize {
    fn default() -> Self {
        Self(DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct WriterProxy {
    pub remote_writer_guid: Guid,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    pub data_max_size_serialized: Option<i32>,
    pub remote_group_entity_id: EntityId,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DiscoveredWriterData {
    pub writer_proxy: WriterProxy,
    pub publication_builtin_topic_data: PublicationBuiltinTopicData,
}

pub const DCPS_PUBLICATION: &str = "DCPSPublication";

impl DdsType for DiscoveredWriterData {
    fn type_name() -> &'static str {
        "DiscoveredWriterData"
    }

    fn has_key() -> bool {
        true
    }

    fn get_serialized_key<E: Endianness>(&self) -> Vec<u8> {
        self.publication_builtin_topic_data.key.value.to_vec()
    }

    fn set_key_fields_from_serialized_key<E: Endianness>(&mut self, _key: &[u8]) -> DdsResult<()> {
        if Self::has_key() {
            unimplemented!("DdsType with key must provide an implementation for set_key_fields_from_serialized_key")
        }
        Ok(())
    }
}

impl DdsSerialize for DiscoveredWriterData {
    fn serialize<W: Write, E: Endianness>(&self, writer: W) -> DdsResult<()> {
        let mut parameter_list_serializer = ParameterListSerializer::<_, E>::new(writer);
        parameter_list_serializer.serialize_payload_header()?;

        // writer_proxy.remote_writer_guid omitted as of table 9.10

        parameter_list_serializer.serialize_parameter_vector::<&Locator, _>(
            PID_UNICAST_LOCATOR,
            &self.writer_proxy.unicast_locator_list,
        )?;
        parameter_list_serializer.serialize_parameter_vector::<&Locator, _>(
            PID_MULTICAST_LOCATOR,
            &self.writer_proxy.multicast_locator_list,
        )?;
        if let Some(data_max_size_serialized) = &self.writer_proxy.data_max_size_serialized {
            parameter_list_serializer.serialize_parameter::<&i32, _>(
                PID_DATA_MAX_SIZE_SERIALIZED,
                data_max_size_serialized,
            )?;
        }
        parameter_list_serializer.serialize_parameter::<&EntityId, _>(
            PID_GROUP_ENTITYID,
            &self.writer_proxy.remote_group_entity_id,
        )?;
        parameter_list_serializer.serialize_parameter::<&BuiltInTopicKey, _>(
            PID_ENDPOINT_GUID,
            &self.publication_builtin_topic_data.key,
        )?;
        parameter_list_serializer.serialize_parameter::<&BuiltInTopicKey, _>(
            PID_PARTICIPANT_GUID,
            &self.publication_builtin_topic_data.participant_key,
        )?;
        parameter_list_serializer.serialize_parameter::<String, _>(
            PID_TOPIC_NAME,
            &self.publication_builtin_topic_data.topic_name,
        )?;
        parameter_list_serializer.serialize_parameter::<String, _>(
            PID_TYPE_NAME,
            &self.publication_builtin_topic_data.type_name,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default::<&DurabilityQosPolicy, _>(
            PID_DURABILITY,
            &self.publication_builtin_topic_data.durability,
        )?;

        parameter_list_serializer.serialize_parameter_if_not_default::<&DeadlineQosPolicy, _>(
            PID_DEADLINE,
            &self.publication_builtin_topic_data.deadline,
        )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<&LatencyBudgetQosPolicy, _>(
                PID_LATENCY_BUDGET,
                &self.publication_builtin_topic_data.latency_budget,
            )?;
        parameter_list_serializer.serialize_parameter_if_not_default::<&LivelinessQosPolicy, _>(
            PID_LIVELINESS,
            &self.publication_builtin_topic_data.liveliness,
        )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<ReliabilityQosPolicyDataWriterSerialize, _>(
                PID_RELIABILITY,
                &ReliabilityQosPolicyDataWriter(&self.publication_builtin_topic_data.reliability),
            )?;
        parameter_list_serializer.serialize_parameter_if_not_default::<&LifespanQosPolicy, _>(
            PID_LIFESPAN,
            &self.publication_builtin_topic_data.lifespan,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default::<&UserDataQosPolicy, _>(
            PID_USER_DATA,
            &self.publication_builtin_topic_data.user_data,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default::<&OwnershipQosPolicy, _>(
            PID_OWNERSHIP,
            &self.publication_builtin_topic_data.ownership,
        )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<&DestinationOrderQosPolicy, _>(
                PID_DESTINATION_ORDER,
                &self.publication_builtin_topic_data.destination_order,
            )?;
        parameter_list_serializer.serialize_parameter_if_not_default::<&PresentationQosPolicy, _>(
            PID_PRESENTATION,
            &self.publication_builtin_topic_data.presentation,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default::<&PartitionQosPolicy, _>(
            PID_PARTITION,
            &self.publication_builtin_topic_data.partition,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default::<&TopicDataQosPolicy, _>(
            PID_TOPIC_DATA,
            &self.publication_builtin_topic_data.topic_data,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default::<&GroupDataQosPolicy, _>(
            PID_GROUP_DATA,
            &self.publication_builtin_topic_data.group_data,
        )?;
        parameter_list_serializer.serialize_sentinel()
    }
}

impl DdsDeserialize<'_> for DiscoveredWriterData {
    fn deserialize(buf: &mut &'_ [u8]) -> DdsResult<Self> {
        let param_list = ParameterListDeserializer::read(buf).unwrap();

        // writer_proxy
        let unicast_locator_list = param_list.get_list::<Locator, _>(PID_UNICAST_LOCATOR)?;
        let multicast_locator_list = param_list.get_list::<Locator, _>(PID_MULTICAST_LOCATOR)?;
        let data_max_size_serialized = param_list.get::<i32, _>(PID_DATA_MAX_SIZE_SERIALIZED).ok();
        let remote_group_entity_id = param_list.get::<EntityId, _>(PID_GROUP_ENTITYID)?;

        // publication_builtin_topic_data
        let key = param_list.get::<BuiltInTopicKey, BuiltInTopicKey>(PID_ENDPOINT_GUID)?;
        let participant_key = param_list.get::<BuiltInTopicKey, _>(PID_PARTICIPANT_GUID)?;
        let topic_name = param_list.get::<String, _>(PID_TOPIC_NAME)?;
        let type_name = param_list.get::<String, _>(PID_TYPE_NAME)?;
        let durability = param_list.get_or_default::<DurabilityQosPolicy, _>(PID_DURABILITY)?;
        let deadline = param_list.get_or_default::<DeadlineQosPolicy, _>(PID_DEADLINE)?;
        let latency_budget =
            param_list.get_or_default::<LatencyBudgetQosPolicy, _>(PID_LATENCY_BUDGET)?;
        let liveliness = param_list.get_or_default::<LivelinessQosPolicy, _>(PID_LIVELINESS)?;
        let reliability = param_list
            .get_or_default::<ReliabilityQosPolicyDataWriterDeserialize, _>(PID_RELIABILITY)?;
        let lifespan = param_list.get_or_default::<LifespanQosPolicy, _>(PID_LIFESPAN)?;
        let user_data = param_list.get_or_default::<UserDataQosPolicy, _>(PID_USER_DATA)?;
        let ownership = param_list.get_or_default::<OwnershipQosPolicy, _>(PID_OWNERSHIP)?;
        let destination_order =
            param_list.get_or_default::<DestinationOrderQosPolicy, _>(PID_DESTINATION_ORDER)?;
        let presentation =
            param_list.get_or_default::<PresentationQosPolicy, _>(PID_PRESENTATION)?;
        let partition = param_list.get_or_default::<PartitionQosPolicy, _>(PID_PARTITION)?;
        let topic_data = param_list.get_or_default::<TopicDataQosPolicy, _>(PID_TOPIC_DATA)?;
        let group_data = param_list.get_or_default::<GroupDataQosPolicy, _>(PID_GROUP_DATA)?;

        let remote_writer_guid = key.value.into();
        Ok(Self {
            writer_proxy: WriterProxy {
                remote_writer_guid,
                unicast_locator_list,
                multicast_locator_list,
                data_max_size_serialized,
                remote_group_entity_id,
            },
            publication_builtin_topic_data: PublicationBuiltinTopicData {
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
            },
        })
    }

    fn deserialize_key(mut buf: &[u8]) -> DdsResult<Vec<u8>> {
        Ok(Self::deserialize(&mut buf)?.get_serialized_key::<LittleEndian>())
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::rtps::types::{GuidPrefix, EntityKind};
    use crate::infrastructure::qos_policy::{
        DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
        LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy,
        PartitionQosPolicy, PresentationQosPolicy, TopicDataQosPolicy, UserDataQosPolicy,
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
    };
    use crate::topic_definition::type_support::LittleEndian;

    use super::*;

    fn to_bytes_le<S: DdsSerialize>(value: &S) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        value.serialize::<_, LittleEndian>(&mut writer).unwrap();
        writer
    }
    #[test]
    fn serialize_all_default() {
        let data = DiscoveredWriterData {
            writer_proxy: WriterProxy {
                remote_writer_guid: Guid::new(
                    GuidPrefix::from([5; 12]),
                    EntityId::new([11, 12, 13], EntityKind::BuiltInWriterWithKey),
                ),
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                data_max_size_serialized: None,
                remote_group_entity_id: EntityId::new([21, 22, 23], EntityKind::BuiltInReaderGroup),
            },
            publication_builtin_topic_data: PublicationBuiltinTopicData {
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
                reliability: DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
                lifespan: LifespanQosPolicy::default(),
                user_data: UserDataQosPolicy::default(),
                ownership: OwnershipQosPolicy::default(),
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
        let expected = DiscoveredWriterData {
            writer_proxy: WriterProxy {
                // must correspond to publication_builtin_topic_data.key
                remote_writer_guid: Guid::new(
                    GuidPrefix::from([1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0]),
                    EntityId::new([4, 0, 0], EntityKind::UserDefinedUnknown),
                ),
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                data_max_size_serialized: None,
                remote_group_entity_id: EntityId::new([21, 22, 23], EntityKind::BuiltInParticipant),
            },
            publication_builtin_topic_data: PublicationBuiltinTopicData {
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
                reliability: DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
                lifespan: LifespanQosPolicy::default(),
                user_data: UserDataQosPolicy::default(),
                ownership: OwnershipQosPolicy::default(),
                destination_order: DestinationOrderQosPolicy::default(),
                presentation: PresentationQosPolicy::default(),
                partition: PartitionQosPolicy::default(),
                topic_data: TopicDataQosPolicy::default(),
                group_data: GroupDataQosPolicy::default(),
            },
        };

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
