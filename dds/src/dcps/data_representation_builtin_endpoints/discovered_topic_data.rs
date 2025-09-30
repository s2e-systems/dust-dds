use super::{
    parameter_id_values::{
        PID_DATA_REPRESENTATION, PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY,
        PID_ENDPOINT_GUID, PID_HISTORY, PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS,
        PID_OWNERSHIP, PID_RELIABILITY, PID_RESOURCE_LIMITS, PID_TOPIC_DATA, PID_TOPIC_NAME,
        PID_TRANSPORT_PRIORITY, PID_TYPE_NAME,
    },
    payload_serializer_deserializer::parameter_list_serializer::ParameterListCdrSerializer,
};
use crate::{
    builtin_topics::TopicBuiltinTopicData,
    infrastructure::{
        error::DdsResult,
        qos_policy::{
            DataRepresentationQosPolicy, DeadlineQosPolicy, DestinationOrderQosPolicy,
            DurabilityQosPolicy, HistoryQosPolicy, LatencyBudgetQosPolicy, LifespanQosPolicy,
            LivelinessQosPolicy, OwnershipQosPolicy, ReliabilityQosPolicy, ResourceLimitsQosPolicy,
            TopicDataQosPolicy, TransportPriorityQosPolicy,
            DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
        },
        type_support::{DdsDeserialize, DdsSerialize},
    },
    xtypes::dynamic_type::TK_UINT8,
};
use alloc::{string::String, vec, vec::Vec};
use dust_dds::xtypes::dynamic_type::{
    DynamicTypeBuilderFactory, ExtensibilityKind, MemberDescriptor, TryConstructKind,
    TypeDescriptor, XTypesBinding, TK_STRUCTURE,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DiscoveredTopicData {
    pub(crate) topic_builtin_topic_data: TopicBuiltinTopicData,
}

impl dust_dds::infrastructure::type_support::TypeSupport for DiscoveredTopicData {
    fn get_type() -> dust_dds::xtypes::dynamic_type::DynamicType {
        extern crate alloc;
        let mut builder = DynamicTypeBuilderFactory::create_type(TypeDescriptor {
            kind: TK_STRUCTURE,
            name: String::from("DiscoveredTopicData"),
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
                name: String::from("name"),
                id: PID_TOPIC_NAME as u32,
                r#type: <String as XTypesBinding>::get_dynamic_type(),
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
                name: String::from("type_name"),
                id: PID_TYPE_NAME as u32,
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
                name: String::from("durability"),
                id: PID_DURABILITY as u32,
                r#type: <DurabilityQosPolicy as XTypesBinding>::get_dynamic_type(),
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
                name: String::from("deadline"),
                id: PID_DEADLINE as u32,
                r#type: <DeadlineQosPolicy as XTypesBinding>::get_dynamic_type(),
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
                name: String::from("latency_budget"),
                id: PID_LATENCY_BUDGET as u32,
                r#type: <LatencyBudgetQosPolicy as XTypesBinding>::get_dynamic_type(),
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
                name: String::from("liveliness"),
                id: PID_LIVELINESS as u32,
                r#type: <LivelinessQosPolicy as XTypesBinding>::get_dynamic_type(),
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
                name: String::from("reliability"),
                id: PID_RELIABILITY as u32,
                r#type: <ReliabilityQosPolicy as XTypesBinding>::get_dynamic_type(),
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
                name: String::from("transport_priority"),
                id: PID_TRANSPORT_PRIORITY as u32,
                r#type: <TransportPriorityQosPolicy as XTypesBinding>::get_dynamic_type(),
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
                name: String::from("lifespan"),
                id: PID_LIFESPAN as u32,
                r#type: <LifespanQosPolicy as XTypesBinding>::get_dynamic_type(),
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
                name: String::from("history"),
                id: PID_HISTORY as u32,
                r#type: <HistoryQosPolicy as XTypesBinding>::get_dynamic_type(),
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
                name: String::from("resource_limits"),
                id: PID_RESOURCE_LIMITS as u32,
                r#type: <ResourceLimitsQosPolicy as XTypesBinding>::get_dynamic_type(),
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
                name: String::from("ownership"),
                id: PID_OWNERSHIP as u32,
                r#type: <OwnershipQosPolicy as XTypesBinding>::get_dynamic_type(),
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
                name: String::from("topic_data"),
                id: PID_TOPIC_DATA as u32,
                r#type: <TopicDataQosPolicy as XTypesBinding>::get_dynamic_type(),
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
                name: String::from("representation"),
                id: PID_DATA_REPRESENTATION as u32,
                r#type: <DataRepresentationQosPolicy as XTypesBinding>::get_dynamic_type(),
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
        builder.build()
    }

    fn create_dynamic_sample(self) -> crate::xtypes::dynamic_type::DynamicData {
        todo!()
    }
}

impl DdsSerialize for DiscoveredTopicData {
    fn serialize_data(&self) -> DdsResult<Vec<u8>> {
        let mut serializer = ParameterListCdrSerializer::default();
        serializer.write_header()?;

        // topic_builtin_topic_data: TopicBuiltinTopicData:

        serializer.write(PID_ENDPOINT_GUID, &self.topic_builtin_topic_data.key)?;
        serializer.write(PID_TOPIC_NAME, &self.topic_builtin_topic_data.name)?;
        serializer.write(PID_TYPE_NAME, &self.topic_builtin_topic_data.type_name)?;
        serializer.write_with_default(
            PID_DURABILITY,
            &self.topic_builtin_topic_data.durability,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_DEADLINE,
            &self.topic_builtin_topic_data.deadline,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_LATENCY_BUDGET,
            &self.topic_builtin_topic_data.latency_budget,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_LIVELINESS,
            &self.topic_builtin_topic_data.liveliness,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_RELIABILITY,
            &self.topic_builtin_topic_data.reliability,
            &DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
        )?;
        serializer.write_with_default(
            PID_TRANSPORT_PRIORITY,
            &self.topic_builtin_topic_data.transport_priority,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_LIFESPAN,
            &self.topic_builtin_topic_data.lifespan,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_DESTINATION_ORDER,
            &self.topic_builtin_topic_data.destination_order,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_HISTORY,
            &self.topic_builtin_topic_data.history,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_RESOURCE_LIMITS,
            &self.topic_builtin_topic_data.resource_limits,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_OWNERSHIP,
            &self.topic_builtin_topic_data.ownership,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_TOPIC_DATA,
            &self.topic_builtin_topic_data.topic_data,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_DATA_REPRESENTATION,
            &self.topic_builtin_topic_data.representation,
            &Default::default(),
        )?;

        serializer.write_sentinel()?;
        Ok(serializer.writer)
    }
}

impl<'de> DdsDeserialize<'de> for DiscoveredTopicData {
    fn deserialize_data(serialized_data: &'de [u8]) -> DdsResult<Self> {
        Ok(Self {
            topic_builtin_topic_data: TopicBuiltinTopicData::deserialize_data(serialized_data)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{builtin_topics::BuiltInTopicKey, infrastructure::qos::TopicQos};

    #[test]
    fn serialize_all_default() {
        let topic_qos = TopicQos::default();
        let data = DiscoveredTopicData {
            topic_builtin_topic_data: TopicBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                name: "ab".to_string(),
                type_name: "cd".to_string(),
                durability: topic_qos.durability,
                deadline: topic_qos.deadline,
                latency_budget: topic_qos.latency_budget,
                liveliness: topic_qos.liveliness,
                reliability: topic_qos.reliability,
                transport_priority: topic_qos.transport_priority,
                lifespan: topic_qos.lifespan,
                destination_order: topic_qos.destination_order,
                history: topic_qos.history,
                resource_limits: topic_qos.resource_limits,
                ownership: topic_qos.ownership,
                topic_data: topic_qos.topic_data,
                representation: topic_qos.representation,
            },
        };

        let expected = vec![
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // ,
            2, 0, 0, 0, // ,
            3, 0, 0, 0, // ,
            4, 0, 0, 0, // ,
            0x05, 0x00, 8, 0, // PID_TOPIC_NAME, length
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'a', b'b', 0, 0x00, // DomainTag: string + padding (1 byte)
            0x07, 0x00, 8, 0, // PID_TYPE_NAME, length
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'c', b'd', 0, 0x00, // DomainTag: string + padding (1 byte)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        let result = data.serialize_data().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_all_default() {
        let topic_qos = TopicQos::default();
        let expected = DiscoveredTopicData {
            topic_builtin_topic_data: TopicBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0],
                },
                name: "ab".to_string(),
                type_name: "cd".to_string(),
                durability: topic_qos.durability,
                deadline: topic_qos.deadline,
                latency_budget: topic_qos.latency_budget,
                liveliness: topic_qos.liveliness,
                reliability: topic_qos.reliability,
                transport_priority: topic_qos.transport_priority,
                lifespan: topic_qos.lifespan,
                destination_order: topic_qos.destination_order,
                history: topic_qos.history,
                resource_limits: topic_qos.resource_limits,
                ownership: topic_qos.ownership,
                topic_data: topic_qos.topic_data,
                representation: topic_qos.representation,
            },
        };

        let mut data = &[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // ,
            2, 0, 0, 0, // ,
            3, 0, 0, 0, // ,
            4, 0, 0, 0, // ,
            0x05, 0x00, 8, 0, // PID_TOPIC_NAME, length
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'a', b'b', 0, 0x00, // DomainTag: string + padding (1 byte)
            0x07, 0x00, 8, 0, // PID_TYPE_NAME, length
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'c', b'd', 0, 0x00, // DomainTag: string + padding (1 byte)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ][..];
        let result = DiscoveredTopicData::deserialize_data(&mut data).unwrap();
        assert_eq!(result, expected);
    }
}
