use super::{
    parameter_id_values::{
        DEFAULT_EXPECTS_INLINE_QOS, PID_DATA_REPRESENTATION, PID_DEADLINE, PID_DESTINATION_ORDER,
        PID_DURABILITY, PID_ENDPOINT_GUID, PID_EXPECTS_INLINE_QOS, PID_GROUP_DATA,
        PID_GROUP_ENTITYID, PID_LATENCY_BUDGET, PID_LIVELINESS, PID_MULTICAST_LOCATOR,
        PID_OWNERSHIP, PID_PARTICIPANT_GUID, PID_PARTITION, PID_PRESENTATION, PID_RELIABILITY,
        PID_TIME_BASED_FILTER, PID_TOPIC_DATA, PID_TOPIC_NAME, PID_TYPE_NAME, PID_UNICAST_LOCATOR,
        PID_USER_DATA,
    },
    payload_serializer_deserializer::{
        parameter_list_deserializer::ParameterListCdrDeserializer,
        parameter_list_serializer::ParameterListCdrSerializer,
    },
};
use crate::{
    builtin_topics::{BuiltInTopicKey, SubscriptionBuiltinTopicData},
    infrastructure::{
        error::DdsResult,
        qos_policy::{
            DataRepresentationQosPolicy, DeadlineQosPolicy, DestinationOrderQosPolicy,
            DurabilityQosPolicy, GroupDataQosPolicy, LatencyBudgetQosPolicy, LivelinessQosPolicy,
            OwnershipQosPolicy, PartitionQosPolicy, PresentationQosPolicy, ReliabilityQosPolicy,
            TimeBasedFilterQosPolicy, TopicDataQosPolicy, UserDataQosPolicy,
            DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
        },
        type_support::{DdsDeserialize, DdsSerialize},
    },
    transport::types::{EntityId, Guid, Locator},
    xtypes::dynamic_type::TK_UINT8,
};
use alloc::{string::String, vec::Vec};
use dust_dds::{
    infrastructure::type_support::TypeSupport,
    xtypes::dynamic_type::{
        XTypesBinding, DynamicTypeBuilderFactory, ExtensibilityKind, MemberDescriptor,
        TryConstructKind, TypeDescriptor, TK_STRUCTURE,
    },
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ReaderProxy {
    pub remote_reader_guid: Guid,
    pub remote_group_entity_id: EntityId,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    pub expects_inline_qos: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DiscoveredReaderData {
    pub(crate) dds_subscription_data: SubscriptionBuiltinTopicData,
    pub(crate) reader_proxy: ReaderProxy,
}

impl TypeSupport for DiscoveredReaderData {
    fn get_type() -> dust_dds::xtypes::dynamic_type::DynamicType {
        extern crate alloc;
        let mut builder = DynamicTypeBuilderFactory::create_type(TypeDescriptor {
            kind: TK_STRUCTURE,
            name: String::from("DiscoveredReaderData"),
            base_type: None,
            discriminator_type: None,
            bound: Vec::new(),
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Mutable,
            is_nested: false,
        });
        builder
            .add_member(MemberDescriptor {
                name: String::from("key"),
                id: PID_ENDPOINT_GUID as u32,
                r#type: DynamicTypeBuilderFactory::create_array_type(
                    DynamicTypeBuilderFactory::get_primitive_type(TK_UINT8),
                    vec![16],
                )
                .build(),
                default_value: String::new(),
                index: 0u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: true,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("participant_key"),
                id: PID_PARTICIPANT_GUID as u32,
                r#type: <BuiltInTopicKey as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 1u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("topic_name"),
                id: PID_TOPIC_NAME as u32,
                r#type: <String as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 2u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("type_name"),
                id: PID_TYPE_NAME as u32,
                r#type: <String as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 3u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("durability"),
                id: PID_DURABILITY as u32,
                r#type: <DurabilityQosPolicy as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 4u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("deadline"),
                id: PID_DEADLINE as u32,
                r#type: <DeadlineQosPolicy as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 5u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("latency_budget"),
                id: PID_LATENCY_BUDGET as u32,
                r#type: <LatencyBudgetQosPolicy as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 6u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("liveliness"),
                id: PID_LIVELINESS as u32,
                r#type: <LivelinessQosPolicy as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 7u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("reliability"),
                id: PID_RELIABILITY as u32,
                r#type: <ReliabilityQosPolicy as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 8u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("ownership"),
                id: PID_OWNERSHIP as u32,
                r#type: <OwnershipQosPolicy as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 9u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("destination_order"),
                id: PID_DESTINATION_ORDER as u32,
                r#type: <DestinationOrderQosPolicy as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 10u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("user_data"),
                id: PID_USER_DATA as u32,
                r#type: <UserDataQosPolicy as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 11u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("time_based_filter"),
                id: PID_TIME_BASED_FILTER as u32,
                r#type: <TimeBasedFilterQosPolicy as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 12u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("presentation"),
                id: PID_PRESENTATION as u32,
                r#type: <PresentationQosPolicy as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 13u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("partition"),
                id: PID_PARTITION as u32,
                r#type: <PartitionQosPolicy as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 14u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("topic_data"),
                id: PID_TOPIC_DATA as u32,
                r#type: <TopicDataQosPolicy as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 15u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("group_data"),
                id: PID_GROUP_DATA as u32,
                r#type: <GroupDataQosPolicy as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 16u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("representation"),
                id: PID_DATA_REPRESENTATION as u32,
                r#type: <DataRepresentationQosPolicy as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 17u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("remote_reader_guid"),
                id: PID_ENDPOINT_GUID as u32,
                r#type: <Guid as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 18u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("remote_group_entity_id"),
                id: PID_GROUP_ENTITYID as u32,
                r#type: <EntityId as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 19u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("unicast_locator_list"),
                id: PID_UNICAST_LOCATOR as u32,
                r#type: <Vec<Locator> as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 20u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("multicast_locator_list"),
                id: PID_MULTICAST_LOCATOR as u32,
                r#type: <Vec<Locator> as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 21u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("expects_inline_qos"),
                id: PID_EXPECTS_INLINE_QOS as u32,
                r#type: <bool as XTypesBinding>::get_dynamic_type(),
                default_value: String::new(),
                index: 22u32,
                try_construct_kind: TryConstructKind::UseDefault,
                label: Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder.build()
    }
}

impl DdsSerialize for DiscoveredReaderData {
    fn serialize_data(&self) -> DdsResult<Vec<u8>> {
        let mut serializer = ParameterListCdrSerializer::default();
        serializer.write_header()?;

        // subscription_builtin_topic_data: SubscriptionBuiltinTopicData:
        serializer.write(PID_ENDPOINT_GUID, &self.dds_subscription_data.key)?;
        // Default value is a deviation from the standard and is used for interoperability reasons:
        serializer.write_with_default(
            PID_PARTICIPANT_GUID,
            &self.dds_subscription_data.participant_key,
            &Default::default(),
        )?;
        serializer.write(PID_TOPIC_NAME, &self.dds_subscription_data.topic_name)?;
        serializer.write(PID_TYPE_NAME, &self.dds_subscription_data.type_name)?;
        serializer.write_with_default(
            PID_DURABILITY,
            &self.dds_subscription_data.durability,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_DEADLINE,
            &self.dds_subscription_data.deadline,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_LATENCY_BUDGET,
            &self.dds_subscription_data.latency_budget,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_LIVELINESS,
            &self.dds_subscription_data.liveliness,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_RELIABILITY,
            &self.dds_subscription_data.reliability,
            &DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
        )?;
        serializer.write_with_default(
            PID_OWNERSHIP,
            &self.dds_subscription_data.ownership,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_DESTINATION_ORDER,
            &self.dds_subscription_data.destination_order,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_USER_DATA,
            &self.dds_subscription_data.user_data,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_TIME_BASED_FILTER,
            &self.dds_subscription_data.time_based_filter,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_PRESENTATION,
            &self.dds_subscription_data.presentation,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_PARTITION,
            &self.dds_subscription_data.partition,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_TOPIC_DATA,
            &self.dds_subscription_data.topic_data,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_GROUP_DATA,
            &self.dds_subscription_data.group_data,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_DATA_REPRESENTATION,
            &self.dds_subscription_data.representation,
            &Default::default(),
        )?;

        // reader_proxy: ReaderProxy

        // skip serilize:
        // reader_proxy.remote_writer_guid: Guid,
        serializer.write_with_default(
            PID_GROUP_ENTITYID,
            &self.reader_proxy.remote_group_entity_id,
            &Default::default(),
        )?;
        serializer
            .write_collection(PID_UNICAST_LOCATOR, &self.reader_proxy.unicast_locator_list)?;
        serializer.write_collection(
            PID_MULTICAST_LOCATOR,
            &self.reader_proxy.multicast_locator_list,
        )?;
        serializer.write_with_default(
            PID_EXPECTS_INLINE_QOS,
            &self.reader_proxy.expects_inline_qos,
            &DEFAULT_EXPECTS_INLINE_QOS,
        )?;

        serializer.write_sentinel()?;
        Ok(serializer.writer)
    }
}

impl<'de> DdsDeserialize<'de> for SubscriptionBuiltinTopicData {
    fn deserialize_data(serialized_data: &'de [u8]) -> DdsResult<Self> {
        let pl_deserializer = ParameterListCdrDeserializer::new(serialized_data)?;

        Ok(Self {
            key: pl_deserializer.read(PID_ENDPOINT_GUID)?,
            // Default value is a deviation from the standard and is used for interoperability reasons:
            participant_key: pl_deserializer
                .read_with_default(PID_PARTICIPANT_GUID, Default::default())?,
            topic_name: pl_deserializer.read(PID_TOPIC_NAME)?,
            type_name: pl_deserializer.read(PID_TYPE_NAME)?,
            durability: pl_deserializer.read_with_default(PID_DURABILITY, Default::default())?,
            deadline: pl_deserializer.read_with_default(PID_DEADLINE, Default::default())?,
            latency_budget: pl_deserializer
                .read_with_default(PID_LATENCY_BUDGET, Default::default())?,
            liveliness: pl_deserializer.read_with_default(PID_LIVELINESS, Default::default())?,
            reliability: pl_deserializer.read_with_default(
                PID_RELIABILITY,
                DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
            )?,
            ownership: pl_deserializer.read_with_default(PID_OWNERSHIP, Default::default())?,
            destination_order: pl_deserializer
                .read_with_default(PID_DESTINATION_ORDER, Default::default())?,
            user_data: pl_deserializer.read_with_default(PID_USER_DATA, Default::default())?,
            time_based_filter: pl_deserializer
                .read_with_default(PID_TIME_BASED_FILTER, Default::default())?,
            presentation: pl_deserializer
                .read_with_default(PID_PRESENTATION, Default::default())?,
            partition: pl_deserializer.read_with_default(PID_PARTITION, Default::default())?,
            topic_data: pl_deserializer.read_with_default(PID_TOPIC_DATA, Default::default())?,
            group_data: pl_deserializer.read_with_default(PID_GROUP_DATA, Default::default())?,
            representation: pl_deserializer
                .read_with_default(PID_DATA_REPRESENTATION, Default::default())?,
        })
    }
}

impl<'de> DdsDeserialize<'de> for DiscoveredReaderData {
    fn deserialize_data(serialized_data: &'de [u8]) -> DdsResult<Self> {
        let pl_deserializer = ParameterListCdrDeserializer::new(serialized_data)?;

        Ok(Self {
            dds_subscription_data: SubscriptionBuiltinTopicData::deserialize_data(serialized_data)?,
            reader_proxy: ReaderProxy {
                remote_reader_guid: pl_deserializer.read(PID_ENDPOINT_GUID)?,
                remote_group_entity_id: pl_deserializer
                    .read_with_default(PID_GROUP_ENTITYID, Default::default())?,
                unicast_locator_list: pl_deserializer.read_collection(PID_UNICAST_LOCATOR)?,
                multicast_locator_list: pl_deserializer.read_collection(PID_MULTICAST_LOCATOR)?,
                expects_inline_qos: pl_deserializer
                    .read_with_default(PID_EXPECTS_INLINE_QOS, DEFAULT_EXPECTS_INLINE_QOS)?,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        builtin_topics::BuiltInTopicKey,
        transport::types::{
            EntityId, Guid, BUILT_IN_WRITER_WITH_KEY, USER_DEFINED_READER_WITH_KEY,
            USER_DEFINED_UNKNOWN,
        },
    };

    #[test]
    fn serialize_all_default() {
        let data = DiscoveredReaderData {
            dds_subscription_data: SubscriptionBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                participant_key: BuiltInTopicKey {
                    value: [6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 0, 9, 0, 0, 0],
                },
                topic_name: "ab".to_string(),
                type_name: "cd".to_string(),
                durability: Default::default(),
                deadline: Default::default(),
                latency_budget: Default::default(),
                liveliness: Default::default(),
                reliability: DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
                ownership: Default::default(),
                destination_order: Default::default(),
                user_data: Default::default(),
                time_based_filter: Default::default(),
                presentation: Default::default(),
                partition: Default::default(),
                topic_data: Default::default(),
                group_data: Default::default(),
                representation: Default::default(),
            },
            reader_proxy: ReaderProxy {
                remote_reader_guid: Guid::new(
                    [5; 12],
                    EntityId::new([11, 12, 13], USER_DEFINED_READER_WITH_KEY),
                ),
                remote_group_entity_id: EntityId::new([21, 22, 23], BUILT_IN_WRITER_WITH_KEY),
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                expects_inline_qos: false,
            },
        };

        let expected = vec![
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            // SubscriptionBuiltinTopicData:
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
            // ReaderProxy:
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            21, 22, 23, 0xc2, //
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        let result = data.serialize_data().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_all_default() {
        let expected = DiscoveredReaderData {
            reader_proxy: ReaderProxy {
                remote_reader_guid: Guid::new(
                    [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0],
                    EntityId::new([4, 0, 0], USER_DEFINED_UNKNOWN),
                ),
                remote_group_entity_id: EntityId::new([21, 22, 23], BUILT_IN_WRITER_WITH_KEY),
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                expects_inline_qos: false,
            },
            dds_subscription_data: SubscriptionBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                participant_key: BuiltInTopicKey {
                    value: [6, 0, 0, 0, 7, 0, 0, 0, 8, 0, 0, 0, 9, 0, 0, 0],
                },
                topic_name: "ab".to_string(),
                type_name: "cd".to_string(),
                durability: Default::default(),
                deadline: Default::default(),
                latency_budget: Default::default(),
                liveliness: Default::default(),
                reliability: DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
                ownership: Default::default(),
                destination_order: Default::default(),
                user_data: Default::default(),
                time_based_filter: Default::default(),
                presentation: Default::default(),
                partition: Default::default(),
                topic_data: Default::default(),
                group_data: Default::default(),
                representation: Default::default(),
            },
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
