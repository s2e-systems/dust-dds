use std::io::Write;

use rust_dds_api::{
    builtin_topics::PublicationBuiltinTopicData, dcps_psm::BuiltInTopicKey, return_type::DDSResult,
};
use crate::dds_type::{DdsDeserialize, DdsSerialize, DdsType, Endianness};
use rust_rtps_pim::structure::types::{EntityId, Guid, Locator};

use super::{
    parameter_id_values::{
        PID_DATA_MAX_SIZE_SERIALIZED, PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY,
        PID_DURABILITY_SERVICE, PID_ENDPOINT_GUID, PID_GROUP_DATA, PID_GROUP_ENTITYID,
        PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS, PID_MULTICAST_LOCATOR, PID_OWNERSHIP,
        PID_OWNERSHIP_STRENGTH, PID_PARTICIPANT_GUID, PID_PARTITION, PID_PRESENTATION,
        PID_RELIABILITY, PID_TOPIC_DATA, PID_TOPIC_NAME, PID_TYPE_NAME, PID_UNICAST_LOCATOR,
        PID_USER_DATA,
    },
    parameter_list_deserializer::ParameterListDeserializer,
    parameter_list_serializer::ParameterListSerializer,
    serde_remote_dds_api::{
        BuiltInTopicKeyDeserialize, BuiltInTopicKeySerialize, DeadlineQosPolicyDeserialize,
        DeadlineQosPolicySerialize, DestinationOrderQosPolicyDeserialize,
        DestinationOrderQosPolicySerialize, DurabilityQosPolicyDeserialize,
        DurabilityQosPolicySerialize, DurabilityServiceQosPolicyDeserialize,
        DurabilityServiceQosPolicySerialize, GroupDataQosPolicyDeserialize,
        GroupDataQosPolicySerialize, LatencyBudgetQosPolicyDeserialize,
        LatencyBudgetQosPolicySerialize, LifespanQosPolicyDeserialize, LifespanQosPolicySerialize,
        LivelinessQosPolicyDeserialize, LivelinessQosPolicySerialize,
        OwnershipQosPolicyDeserialize, OwnershipQosPolicySerialize,
        OwnershipStrengthQosPolicyDeserialize, OwnershipStrengthQosPolicySerialize,
        PartitionQosPolicyDeserialize, PartitionQosPolicySerialize,
        PresentationQosPolicyDeserialize, PresentationQosPolicySerialize,
        ReliabilityQosPolicyDataWriter, ReliabilityQosPolicyDataWriterDeserialize,
        ReliabilityQosPolicyDataWriterSerialize, TopicDataQosPolicyDeserialize,
        TopicDataQosPolicySerialize, UserDataQosPolicyDeserialize, UserDataQosPolicySerialize,
    },
    serde_remote_rtps_pim::{
        EntityIdDeserialize, EntityIdSerialize, LocatorDeserialize, LocatorSerialize,
    },
};

#[derive(Debug, PartialEq)]
pub struct RtpsWriterProxy {
    pub remote_writer_guid: Guid,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    pub data_max_size_serialized: Option<i32>,
    pub remote_group_entity_id: EntityId,
}

#[derive(Debug, PartialEq)]
pub struct SedpDiscoveredWriterData {
    pub writer_proxy: RtpsWriterProxy,
    pub publication_builtin_topic_data: PublicationBuiltinTopicData,
}

pub const DCPS_PUBLICATION: &'static str = "DCPSPublication";

impl DdsType for SedpDiscoveredWriterData {
    fn type_name() -> &'static str {
        "SedpDiscoveredWriterData"
    }

    fn has_key() -> bool {
        true
    }
}

impl DdsSerialize for SedpDiscoveredWriterData {
    fn serialize<W: Write, E: Endianness>(&self, writer: W) -> DDSResult<()> {
        let mut parameter_list_serializer = ParameterListSerializer::<_, E>::new(writer);
        parameter_list_serializer.serialize_payload_header()?;

        // writer_proxy.remote_writer_guid omitted as of table 9.10

        parameter_list_serializer.serialize_parameter_vector::<LocatorSerialize, _>(
            PID_UNICAST_LOCATOR,
            &self.writer_proxy.unicast_locator_list,
        )?;
        parameter_list_serializer.serialize_parameter_vector::<LocatorSerialize, _>(
            PID_MULTICAST_LOCATOR,
            &self.writer_proxy.multicast_locator_list,
        )?;
        if let Some(data_max_size_serialized) = &self.writer_proxy.data_max_size_serialized {
            parameter_list_serializer.serialize_parameter::<&i32, _>(
                PID_DATA_MAX_SIZE_SERIALIZED,
                data_max_size_serialized,
            )?;
        }
        parameter_list_serializer.serialize_parameter::<EntityIdSerialize, _>(
            PID_GROUP_ENTITYID,
            &self.writer_proxy.remote_group_entity_id,
        )?;
        parameter_list_serializer.serialize_parameter::<BuiltInTopicKeySerialize, _>(
            PID_ENDPOINT_GUID,
            &self.publication_builtin_topic_data.key,
        )?;
        parameter_list_serializer.serialize_parameter::<BuiltInTopicKeySerialize, _>(
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
        parameter_list_serializer
            .serialize_parameter_if_not_default::<DurabilityQosPolicySerialize, _>(
                PID_DURABILITY,
                &self.publication_builtin_topic_data.durability,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<DurabilityServiceQosPolicySerialize, _>(
                PID_DURABILITY_SERVICE,
                &self.publication_builtin_topic_data.durability_service,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<DeadlineQosPolicySerialize, _>(
                PID_DEADLINE,
                &self.publication_builtin_topic_data.deadline,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<LatencyBudgetQosPolicySerialize, _>(
                PID_LATENCY_BUDGET,
                &self.publication_builtin_topic_data.latency_budget,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<LivelinessQosPolicySerialize, _>(
                PID_LIVELINESS,
                &self.publication_builtin_topic_data.liveliness,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<ReliabilityQosPolicyDataWriterSerialize, _>(
                PID_RELIABILITY,
                &ReliabilityQosPolicyDataWriter(&self.publication_builtin_topic_data.reliability),
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<LifespanQosPolicySerialize, _>(
                PID_LIFESPAN,
                &self.publication_builtin_topic_data.lifespan,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<UserDataQosPolicySerialize, _>(
                PID_USER_DATA,
                &self.publication_builtin_topic_data.user_data,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<OwnershipQosPolicySerialize, _>(
                PID_OWNERSHIP,
                &self.publication_builtin_topic_data.ownership,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<OwnershipStrengthQosPolicySerialize, _>(
                PID_OWNERSHIP_STRENGTH,
                &self.publication_builtin_topic_data.ownership_strength,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<DestinationOrderQosPolicySerialize, _>(
                PID_DESTINATION_ORDER,
                &self.publication_builtin_topic_data.destination_order,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<PresentationQosPolicySerialize, _>(
                PID_PRESENTATION,
                &self.publication_builtin_topic_data.presentation,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<PartitionQosPolicySerialize, _>(
                PID_PARTITION,
                &self.publication_builtin_topic_data.partition,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<TopicDataQosPolicySerialize, _>(
                PID_TOPIC_DATA,
                &self.publication_builtin_topic_data.topic_data,
            )?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<GroupDataQosPolicySerialize, _>(
                PID_GROUP_DATA,
                &self.publication_builtin_topic_data.group_data,
            )?;
        parameter_list_serializer.serialize_sentinel()
    }
}

impl DdsDeserialize<'_> for SedpDiscoveredWriterData {
    fn deserialize(buf: &mut &'_ [u8]) -> DDSResult<Self> {
        let param_list = ParameterListDeserializer::read(buf).unwrap();

        // writer_proxy
        let unicast_locator_list =
            param_list.get_list::<LocatorDeserialize, _>(PID_UNICAST_LOCATOR)?;
        let multicast_locator_list =
            param_list.get_list::<LocatorDeserialize, _>(PID_MULTICAST_LOCATOR)?;
        let data_max_size_serialized = param_list.get::<i32, _>(PID_DATA_MAX_SIZE_SERIALIZED).ok();
        let remote_group_entity_id =
            param_list.get::<EntityIdDeserialize, _>(PID_GROUP_ENTITYID)?;

        // publication_builtin_topic_data
        let key =
            param_list.get::<BuiltInTopicKeyDeserialize, BuiltInTopicKey>(PID_ENDPOINT_GUID)?;
        let participant_key =
            param_list.get::<BuiltInTopicKeyDeserialize, _>(PID_PARTICIPANT_GUID)?;
        let topic_name = param_list.get::<String, _>(PID_TOPIC_NAME)?;
        let type_name = param_list.get::<String, _>(PID_TYPE_NAME)?;
        let durability =
            param_list.get_or_default::<DurabilityQosPolicyDeserialize, _>(PID_DURABILITY)?;
        let durability_service = param_list
            .get_or_default::<DurabilityServiceQosPolicyDeserialize, _>(PID_DURABILITY_SERVICE)?;
        let deadline =
            param_list.get_or_default::<DeadlineQosPolicyDeserialize, _>(PID_DEADLINE)?;
        let latency_budget = param_list
            .get_or_default::<LatencyBudgetQosPolicyDeserialize, _>(PID_LATENCY_BUDGET)?;
        let liveliness =
            param_list.get_or_default::<LivelinessQosPolicyDeserialize, _>(PID_LIVELINESS)?;
        let reliability = param_list
            .get_or_default::<ReliabilityQosPolicyDataWriterDeserialize, _>(PID_RELIABILITY)?;
        let lifespan =
            param_list.get_or_default::<LifespanQosPolicyDeserialize, _>(PID_LIFESPAN)?;
        let user_data =
            param_list.get_or_default::<UserDataQosPolicyDeserialize, _>(PID_USER_DATA)?;
        let ownership =
            param_list.get_or_default::<OwnershipQosPolicyDeserialize, _>(PID_OWNERSHIP)?;
        let ownership_strength = param_list
            .get_or_default::<OwnershipStrengthQosPolicyDeserialize, _>(PID_OWNERSHIP_STRENGTH)?;
        let destination_order = param_list
            .get_or_default::<DestinationOrderQosPolicyDeserialize, _>(PID_DESTINATION_ORDER)?;
        let presentation =
            param_list.get_or_default::<PresentationQosPolicyDeserialize, _>(PID_PRESENTATION)?;
        let partition =
            param_list.get_or_default::<PartitionQosPolicyDeserialize, _>(PID_PARTITION)?;
        let topic_data =
            param_list.get_or_default::<TopicDataQosPolicyDeserialize, _>(PID_TOPIC_DATA)?;
        let group_data =
            param_list.get_or_default::<GroupDataQosPolicyDeserialize, _>(PID_GROUP_DATA)?;

        let remote_writer_guid = key.value.into();
        Ok(Self {
            writer_proxy: RtpsWriterProxy {
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
                durability_service,
                deadline,
                latency_budget,
                liveliness,
                reliability,
                lifespan,
                user_data,
                ownership,
                ownership_strength,
                destination_order,
                presentation,
                partition,
                topic_data,
                group_data,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use rust_dds_api::{
        dcps_psm::BuiltInTopicKey,
        infrastructure::qos_policy::{
            DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
            DurabilityServiceQosPolicy, GroupDataQosPolicy, LatencyBudgetQosPolicy,
            LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, OwnershipStrengthQosPolicy,
            PartitionQosPolicy, PresentationQosPolicy, TopicDataQosPolicy, UserDataQosPolicy,
            DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
        },
    };
    use crate::dds_type::LittleEndian;
    use rust_rtps_pim::structure::types::{EntityId, Guid, GuidPrefix};

    use super::*;

    fn to_bytes_le<S: DdsSerialize>(value: &S) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        value.serialize::<_, LittleEndian>(&mut writer).unwrap();
        writer
    }
    #[test]
    fn serialize_all_default() {
        let data = SedpDiscoveredWriterData {
            writer_proxy: RtpsWriterProxy {
                remote_writer_guid: Guid::new(
                    GuidPrefix([5; 12]),
                    EntityId {
                        entity_key: [11, 12, 13],
                        entity_kind: 15,
                    },
                ),
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                data_max_size_serialized: None,
                remote_group_entity_id: EntityId {
                    entity_key: [21, 22, 23],
                    entity_kind: 25,
                },
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
                durability_service: DurabilityServiceQosPolicy::default(),
                deadline: DeadlineQosPolicy::default(),
                latency_budget: LatencyBudgetQosPolicy::default(),
                liveliness: LivelinessQosPolicy::default(),
                reliability: DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
                lifespan: LifespanQosPolicy::default(),
                user_data: UserDataQosPolicy::default(),
                ownership: OwnershipQosPolicy::default(),
                ownership_strength: OwnershipStrengthQosPolicy::default(),
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
            21, 22, 23, 25, // u8[3], u8
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
        let expected = SedpDiscoveredWriterData {
            writer_proxy: RtpsWriterProxy {
                // must correspond to publication_builtin_topic_data.key
                remote_writer_guid: Guid::new(
                    GuidPrefix([1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0]),
                    EntityId {
                        entity_key: [4, 0, 0],
                        entity_kind: 0,
                    },
                ),
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                data_max_size_serialized: None,
                remote_group_entity_id: EntityId {
                    entity_key: [21, 22, 23],
                    entity_kind: 25,
                },
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
                durability_service: DurabilityServiceQosPolicy::default(),
                deadline: DeadlineQosPolicy::default(),
                latency_budget: LatencyBudgetQosPolicy::default(),
                liveliness: LivelinessQosPolicy::default(),
                reliability: DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
                lifespan: LifespanQosPolicy::default(),
                user_data: UserDataQosPolicy::default(),
                ownership: OwnershipQosPolicy::default(),
                ownership_strength: OwnershipStrengthQosPolicy::default(),
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
            21, 22, 23, 25, // u8[3], u8
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
        let result: SedpDiscoveredWriterData = DdsDeserialize::deserialize(&mut data).unwrap();
        assert_eq!(result, expected);
    }
}
