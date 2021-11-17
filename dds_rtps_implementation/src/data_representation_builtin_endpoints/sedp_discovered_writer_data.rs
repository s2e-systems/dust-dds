use std::io::Write;

use rust_dds_api::{
    builtin_topics::PublicationBuiltinTopicData,
    dcps_psm::BuiltInTopicKey,
    infrastructure::qos_policy::{
        DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
        DurabilityServiceQosPolicy, GroupDataQosPolicy, LatencyBudgetQosPolicy, LifespanQosPolicy,
        LivelinessQosPolicy, OwnershipQosPolicy, OwnershipStrengthQosPolicy, PartitionQosPolicy,
        PresentationQosPolicy, TopicDataQosPolicy, UserDataQosPolicy,
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
    },
    return_type::DDSResult,
};
use rust_rtps_pim::{
    behavior::reader::writer_proxy::RtpsWriterProxy,
    structure::types::{EntityId, Guid, GuidPrefix, Locator},
};

use crate::{
    data_representation_builtin_endpoints::serde_remote_rtps_pim::EntityIdDeserialize,
    dds_type::{DdsDeserialize, DdsSerialize, DdsType, Endianness},
};

use super::{
    data_serialize_deserialize::{ParameterListDeserializer, ParameterListSerializer},
    serde_remote_rtps_pim::{EntityIdSerialize, LocatorDeserialize, LocatorSerialize},
    parameter_id_values::{
        PID_DATA_MAX_SIZE_SERIALIZED, PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY,
        PID_DURABILITY_SERVICE, PID_ENDPOINT_GUID, PID_GROUP_DATA, PID_GROUP_ENTITYID,
        PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS, PID_MULTICAST_LOCATOR, PID_OWNERSHIP,
        PID_OWNERSHIP_STRENGTH, PID_PARTICIPANT_GUID, PID_PARTITION, PID_PRESENTATION,
        PID_RELIABILITY, PID_TOPIC_DATA, PID_TOPIC_NAME, PID_TYPE_NAME, PID_UNICAST_LOCATOR,
        PID_USER_DATA,
    },
    serde_remote_dds_api::{
        BuiltInTopicKeyDeserialize, BuiltInTopicKeySerialize, DeadlineQosPolicySerialize,
        DestinationOrderQosPolicySerialize, DurabilityQosPolicySerialize,
        DurabilityServiceQosPolicySerialize, GroupDataQosPolicySerialize,
        LatencyBudgetQosPolicySerialize, LifespanQosPolicySerialize, LivelinessQosPolicySerialize,
        OwnershipQosPolicySerialize, OwnershipStrengthQosPolicySerialize,
        PartitionQosPolicySerialize, PresentationQosPolicySerialize, ReliabilityQosPolicySerialize,
        TopicDataQosPolicySerialize, UserDataQosPolicySerialize,
    },
};

#[derive(Debug, PartialEq)]
pub struct SedpDiscoveredWriterData {
    pub writer_proxy: RtpsWriterProxy<Vec<Locator>>,
    pub publication_builtin_topic_data: PublicationBuiltinTopicData,
}

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

        // omitted (as of table 9.10) writer_proxy.remote_writer_guid

        for locator in &self.writer_proxy.unicast_locator_list {
            parameter_list_serializer
                .serialize_parameter(PID_UNICAST_LOCATOR, &LocatorSerialize(locator))?;
        }
        for locator in &self.writer_proxy.multicast_locator_list {
            parameter_list_serializer
                .serialize_parameter(PID_MULTICAST_LOCATOR, &LocatorSerialize(locator))?;
        }
        if let Some(data_max_size_serialized) = &self.writer_proxy.data_max_size_serialized {
            parameter_list_serializer
                .serialize_parameter(PID_DATA_MAX_SIZE_SERIALIZED, data_max_size_serialized)?;
        }
        parameter_list_serializer
            .serialize_parameter(
                PID_GROUP_ENTITYID,
                &EntityIdSerialize(&self.writer_proxy.remote_group_entity_id),
            )?;

        parameter_list_serializer
            .serialize_parameter(
                PID_ENDPOINT_GUID,
                &BuiltInTopicKeySerialize(&self.publication_builtin_topic_data.key),
            )?;
        parameter_list_serializer
            .serialize_parameter(
                PID_PARTICIPANT_GUID,
                &BuiltInTopicKeySerialize(&self.publication_builtin_topic_data.participant_key),
            )?;
        parameter_list_serializer
            .serialize_parameter(
                PID_TOPIC_NAME,
                &self.publication_builtin_topic_data.topic_name,
            )?;
        parameter_list_serializer
            .serialize_parameter(
                PID_TYPE_NAME,
                &self.publication_builtin_topic_data.type_name,
            )?;
        if self.publication_builtin_topic_data.durability != DurabilityQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_DURABILITY,
                    &DurabilityQosPolicySerialize(&self.publication_builtin_topic_data.durability),
                )?;
        }
        if self.publication_builtin_topic_data.durability_service
            != DurabilityServiceQosPolicy::default()
        {
            parameter_list_serializer
                .serialize_parameter(
                    PID_DURABILITY_SERVICE,
                    &DurabilityServiceQosPolicySerialize(
                        &self.publication_builtin_topic_data.durability_service,
                    ),
                )?;
        }
        if self.publication_builtin_topic_data.deadline != DeadlineQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_DEADLINE,
                    &DeadlineQosPolicySerialize(&self.publication_builtin_topic_data.deadline),
                )?;
        }
        if self.publication_builtin_topic_data.latency_budget != LatencyBudgetQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_LATENCY_BUDGET,
                    &LatencyBudgetQosPolicySerialize(
                        &self.publication_builtin_topic_data.latency_budget,
                    ),
                )?;
        }
        if self.publication_builtin_topic_data.liveliness != LivelinessQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_LIVELINESS,
                    &LivelinessQosPolicySerialize(&self.publication_builtin_topic_data.liveliness),
                )?;
        }
        if self.publication_builtin_topic_data.reliability
            != DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER
        {
            parameter_list_serializer
                .serialize_parameter(
                    PID_RELIABILITY,
                    &ReliabilityQosPolicySerialize(
                        &self.publication_builtin_topic_data.reliability,
                    ),
                )?;
        }
        if self.publication_builtin_topic_data.lifespan != LifespanQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_LIFESPAN,
                    &LifespanQosPolicySerialize(&self.publication_builtin_topic_data.lifespan),
                )?;
        }
        if self.publication_builtin_topic_data.user_data != UserDataQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_USER_DATA,
                    &UserDataQosPolicySerialize(&self.publication_builtin_topic_data.user_data),
                )?;
        }
        if self.publication_builtin_topic_data.ownership != OwnershipQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_OWNERSHIP,
                    &OwnershipQosPolicySerialize(&self.publication_builtin_topic_data.ownership),
                )?;
        }
        if self.publication_builtin_topic_data.ownership_strength
            != OwnershipStrengthQosPolicy::default()
        {
            parameter_list_serializer
                .serialize_parameter(
                    PID_OWNERSHIP_STRENGTH,
                    &OwnershipStrengthQosPolicySerialize(
                        &self.publication_builtin_topic_data.ownership_strength,
                    ),
                )?;
        }
        if self.publication_builtin_topic_data.destination_order
            != DestinationOrderQosPolicy::default()
        {
            parameter_list_serializer
                .serialize_parameter(
                    PID_DESTINATION_ORDER,
                    &DestinationOrderQosPolicySerialize(
                        &self.publication_builtin_topic_data.destination_order,
                    ),
                )?;
        }
        if self.publication_builtin_topic_data.presentation != PresentationQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_PRESENTATION,
                    &PresentationQosPolicySerialize(
                        &self.publication_builtin_topic_data.presentation,
                    ),
                )?;
        }
        if self.publication_builtin_topic_data.partition != PartitionQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_PARTITION,
                    &PartitionQosPolicySerialize(&self.publication_builtin_topic_data.partition),
                )?;
        }
        if self.publication_builtin_topic_data.topic_data != TopicDataQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_TOPIC_DATA,
                    &TopicDataQosPolicySerialize(&self.publication_builtin_topic_data.topic_data),
                )?;
        }
        if self.publication_builtin_topic_data.group_data != GroupDataQosPolicy::default() {
            parameter_list_serializer
                .serialize_parameter(
                    PID_GROUP_DATA,
                    &GroupDataQosPolicySerialize(&self.publication_builtin_topic_data.group_data),
                )?;
        }
        parameter_list_serializer.serialize_sentinel()?;

        Ok(())
    }
}

fn convert_built_in_topic_key_to_guid(key: &BuiltInTopicKey) -> Guid {
    let v0 = key.value[0].to_le_bytes();
    let v1 = key.value[1].to_le_bytes();
    let v2 = key.value[2].to_le_bytes();
    let v3 = key.value[3].to_le_bytes();
    let prefix = GuidPrefix([
        v0[0], v0[1], v0[2], v0[3], v1[0], v1[1], v1[2], v1[3], v2[0], v2[1], v2[2], v2[3],
    ]);
    let entity_id = EntityId {
        entity_key: [v3[0], v3[1], v3[2]],
        entity_kind: v3[3],
    };
    Guid { prefix, entity_id }
}

impl DdsDeserialize<'_> for SedpDiscoveredWriterData {
    fn deserialize(buf: &mut &'_ [u8]) -> DDSResult<Self> {
        let param_list = ParameterListDeserializer::read(buf).unwrap();

        // pub remote_writer_guid: Guid,
        // pub unicast_locator_list: L,
        // pub multicast_locator_list: L,
        // pub data_max_size_serialized: Option<i32>,
        // pub remote_group_entity_id: EntityId,

        // pub key: BuiltInTopicKey,
        // pub participant_key: BuiltInTopicKey,
        // pub topic_name: String,
        // pub type_name: String,
        // pub durability: DurabilityQosPolicy,
        // pub durability_service: DurabilityServiceQosPolicy,
        // pub deadline: DeadlineQosPolicy,
        // pub latency_budget: LatencyBudgetQosPolicy,
        // pub liveliness: LivelinessQosPolicy,
        // pub reliability: ReliabilityQosPolicy,
        // pub lifespan: LifespanQosPolicy,
        // pub user_data: UserDataQosPolicy,
        // pub ownership: OwnershipQosPolicy,
        // pub ownership_strength: OwnershipStrengthQosPolicy,
        // pub destination_order: DestinationOrderQosPolicy,
        // pub presentation: PresentationQosPolicy,
        // pub partition: PartitionQosPolicy,
        // pub topic_data: TopicDataQosPolicy,
        // pub group_data: GroupDataQosPolicy,

        // writer_proxy
        let unicast_locator_list = param_list
            .get_list::<LocatorDeserialize, _>(PID_UNICAST_LOCATOR)?;
        let multicast_locator_list = param_list
            .get_list::<LocatorDeserialize, _>(PID_MULTICAST_LOCATOR)?;
        let data_max_size_serialized = param_list.get::<i32, _>(PID_DATA_MAX_SIZE_SERIALIZED).ok();
        let remote_group_entity_id = param_list
            .get::<EntityIdDeserialize, _>(PID_GROUP_ENTITYID)?;

        // publication_builtin_topic_data
        let key = param_list
            .get::<BuiltInTopicKeyDeserialize, _>(PID_ENDPOINT_GUID)?;
        let participant_key = param_list
            .get::<BuiltInTopicKeyDeserialize, _>(PID_PARTICIPANT_GUID)?;
        let topic_name = param_list.get::<String, _>(PID_TOPIC_NAME)?;
        let type_name = param_list.get::<String, _>(PID_TYPE_NAME)?;

        // Assembly
        let remote_writer_guid = convert_built_in_topic_key_to_guid(&key);
        let writer_proxy = RtpsWriterProxy {
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
            data_max_size_serialized,
            remote_group_entity_id,
        };
        let publication_builtin_topic_data = PublicationBuiltinTopicData {
            key,
            participant_key,
            topic_name,
            type_name,
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
        };
        Ok(Self {
            writer_proxy,
            publication_builtin_topic_data,
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
        },
    };
    use rust_rtps_pim::structure::types::{EntityId, Guid, GuidPrefix};

    use super::*;

    fn to_bytes_le<S: DdsSerialize>(value: &S) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        value
            .serialize::<_, crate::dds_type::LittleEndian>(&mut writer)
            .unwrap();
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
                    value: [1, 2, 3, 4],
                },
                participant_key: BuiltInTopicKey {
                    value: [6, 7, 8, 9],
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
            1, 0, 0, 0, // long,
            2, 0, 0, 0, // long,
            3, 0, 0, 0, // long,
            4, 0, 0, 0, // long,
            0x50, 0x00, 16, 0, //PID_PARTICIPANT_GUID, length
            6, 0, 0, 0, // long,
            7, 0, 0, 0, // long,
            8, 0, 0, 0, // long,
            9, 0, 0, 0, // long,
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
                    value: [1, 2, 3, 4],
                },
                participant_key: BuiltInTopicKey {
                    value: [6, 7, 8, 9],
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
            1, 0, 0, 0, // long,
            2, 0, 0, 0, // long,
            3, 0, 0, 0, // long,
            4, 0, 0, 0, // long,
            0x50, 0x00, 16, 0, //PID_PARTICIPANT_GUID, length
            6, 0, 0, 0, // long,
            7, 0, 0, 0, // long,
            8, 0, 0, 0, // long,
            9, 0, 0, 0, // long,
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
