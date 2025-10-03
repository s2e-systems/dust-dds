use super::parameter_id_values::{
    PID_DATA_REPRESENTATION, PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY,
    PID_ENDPOINT_GUID, PID_HISTORY, PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS,
    PID_OWNERSHIP, PID_RELIABILITY, PID_RESOURCE_LIMITS, PID_TOPIC_DATA, PID_TOPIC_NAME,
    PID_TRANSPORT_PRIORITY, PID_TYPE_NAME,
};
use crate::{
    builtin_topics::{BuiltInTopicKey, TopicBuiltinTopicData},
    infrastructure::{
        error::DdsResult,
        qos_policy::{
            DataRepresentationQosPolicy, DeadlineQosPolicy, DestinationOrderQosPolicy,
            DurabilityQosPolicy, HistoryQosPolicy, LatencyBudgetQosPolicy, LifespanQosPolicy,
            LivelinessQosPolicy, OwnershipQosPolicy, ReliabilityQosPolicy, ResourceLimitsQosPolicy,
            TopicDataQosPolicy, TransportPriorityQosPolicy,
            DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
        },
        type_support::{DdsDeserialize, TypeSupport},
    },
    xtypes::dynamic_type::TypeKind,
};
use alloc::string::String;
use dust_dds::xtypes::dynamic_type::{
    DynamicTypeBuilderFactory, ExtensibilityKind, MemberDescriptor, TryConstructKind,
    TypeDescriptor, XTypesBinding,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DiscoveredTopicData {
    pub(crate) topic_builtin_topic_data: TopicBuiltinTopicData,
}

impl TypeSupport for DiscoveredTopicData {
    fn get_type() -> crate::xtypes::dynamic_type::DynamicType {
        let mut builder = DynamicTypeBuilderFactory::create_type(TypeDescriptor {
            kind: TypeKind::STRUCTURE,
            name: String::from("DiscoveredTopicData"),
            base_type: None,
            discriminator_type: None,
            bound: alloc::vec::Vec::new(),
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Mutable,
            is_nested: false,
        });
        builder
            .add_member(MemberDescriptor {
                name: String::from("key"),
                id: PID_ENDPOINT_GUID as u32,
                r#type: <BuiltInTopicKey as TypeSupport>::get_type(),
                default_value: None,
                index: 0,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
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
                default_value: None,
                index: 1,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
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
                default_value: None,
                index: 2,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
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
                r#type: <DurabilityQosPolicy as TypeSupport>::get_type(),
                default_value: Some(
                    DurabilityQosPolicy::const_default()
                        .create_dynamic_sample()
                        .into(),
                ),
                index: 3,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: true,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("deadline"),
                id: PID_DEADLINE as u32,
                r#type: <DeadlineQosPolicy as TypeSupport>::get_type(),
                default_value: Some(
                    DeadlineQosPolicy::const_default()
                        .create_dynamic_sample()
                        .into(),
                ),
                index: 4,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: true,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("latency_budget"),
                id: PID_LATENCY_BUDGET as u32,
                r#type: <LatencyBudgetQosPolicy as TypeSupport>::get_type(),
                default_value: Some(
                    LatencyBudgetQosPolicy::const_default()
                        .create_dynamic_sample()
                        .into(),
                ),
                index: 5,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: true,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("liveliness"),
                id: PID_LIVELINESS as u32,
                r#type: <LivelinessQosPolicy as TypeSupport>::get_type(),
                default_value: Some(
                    LivelinessQosPolicy::const_default()
                        .create_dynamic_sample()
                        .into(),
                ),
                index: 6,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: true,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("reliability"),
                id: PID_RELIABILITY as u32,
                r#type: <ReliabilityQosPolicy as TypeSupport>::get_type(),
                default_value: Some(
                    DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS
                        .create_dynamic_sample()
                        .into(),
                ),
                index: 7,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: true,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("transport_priority"),
                id: PID_TRANSPORT_PRIORITY as u32,
                r#type: <TransportPriorityQosPolicy as TypeSupport>::get_type(),
                default_value: Some(
                    TransportPriorityQosPolicy::const_default()
                        .create_dynamic_sample()
                        .into(),
                ),
                index: 8,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: true,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("lifespan"),
                id: PID_LIFESPAN as u32,
                r#type: <LifespanQosPolicy as TypeSupport>::get_type(),
                default_value: Some(
                    LifespanQosPolicy::const_default()
                        .create_dynamic_sample()
                        .into(),
                ),
                index: 9,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: true,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("destination_order"),
                id: PID_DESTINATION_ORDER as u32,
                r#type: <DestinationOrderQosPolicy as TypeSupport>::get_type(),
                default_value: Some(
                    DestinationOrderQosPolicy::const_default()
                        .create_dynamic_sample()
                        .into(),
                ),
                index: 10,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: true,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("history"),
                id: PID_HISTORY as u32,
                r#type: <HistoryQosPolicy as TypeSupport>::get_type(),
                default_value: Some(
                    HistoryQosPolicy::const_default()
                        .create_dynamic_sample()
                        .into(),
                ),
                index: 11,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: true,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("resource_limits"),
                id: PID_RESOURCE_LIMITS as u32,
                r#type: <ResourceLimitsQosPolicy as TypeSupport>::get_type(),
                default_value: Some(
                    ResourceLimitsQosPolicy::const_default()
                        .create_dynamic_sample()
                        .into(),
                ),
                index: 12,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: true,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("ownership"),
                id: PID_OWNERSHIP as u32,
                r#type: <OwnershipQosPolicy as TypeSupport>::get_type(),
                default_value: Some(
                    OwnershipQosPolicy::const_default()
                        .create_dynamic_sample()
                        .into(),
                ),
                index: 13,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: true,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("topic_data"),
                id: PID_TOPIC_DATA as u32,
                r#type: <TopicDataQosPolicy as TypeSupport>::get_type(),
                default_value: Some(
                    TopicDataQosPolicy::const_default()
                        .create_dynamic_sample()
                        .into(),
                ),
                index: 14,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: true,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("representation"),
                id: PID_DATA_REPRESENTATION as u32,
                r#type: <DataRepresentationQosPolicy as TypeSupport>::get_type(),
                default_value: Some(
                    DataRepresentationQosPolicy::const_default()
                        .create_dynamic_sample()
                        .into(),
                ),
                index: 15,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: true,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();

        builder.build()
    }

    fn create_dynamic_sample(self) -> crate::xtypes::dynamic_type::DynamicData {
        let mut data =
            dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data(Self::get_type());
        data.set_complex_value(
            PID_ENDPOINT_GUID as u32,
            self.topic_builtin_topic_data.key.create_dynamic_sample(),
        )
        .unwrap();
        data.set_string_value(PID_TOPIC_NAME as u32, self.topic_builtin_topic_data.name)
            .unwrap();
        data.set_string_value(
            PID_TYPE_NAME as u32,
            self.topic_builtin_topic_data.type_name,
        )
        .unwrap();
        data.set_complex_value(
            PID_DURABILITY as u32,
            self.topic_builtin_topic_data
                .durability
                .create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(
            PID_DEADLINE as u32,
            self.topic_builtin_topic_data
                .deadline
                .create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(
            PID_LATENCY_BUDGET as u32,
            self.topic_builtin_topic_data
                .latency_budget
                .create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(
            PID_LIVELINESS as u32,
            self.topic_builtin_topic_data
                .liveliness
                .create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(
            PID_RELIABILITY as u32,
            self.topic_builtin_topic_data
                .reliability
                .create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(
            PID_TRANSPORT_PRIORITY as u32,
            self.topic_builtin_topic_data
                .transport_priority
                .create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(
            PID_LIFESPAN as u32,
            self.topic_builtin_topic_data
                .lifespan
                .create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(
            PID_DESTINATION_ORDER as u32,
            self.topic_builtin_topic_data
                .destination_order
                .create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(
            PID_HISTORY as u32,
            self.topic_builtin_topic_data
                .history
                .create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(
            PID_RESOURCE_LIMITS as u32,
            self.topic_builtin_topic_data
                .resource_limits
                .create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(
            PID_OWNERSHIP as u32,
            self.topic_builtin_topic_data
                .ownership
                .create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(
            PID_TOPIC_DATA as u32,
            self.topic_builtin_topic_data
                .topic_data
                .create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(
            PID_DATA_REPRESENTATION as u32,
            self.topic_builtin_topic_data
                .representation
                .create_dynamic_sample(),
        )
        .unwrap();

        data
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
    use crate::{
        builtin_topics::BuiltInTopicKey,
        infrastructure::qos::TopicQos,
        xtypes::{pl_cdr_serializer::PlCdrLeSerializer, serialize::XTypesSerialize},
    };

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
            // 0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x05, 0x00, 8, 0, // PID_TOPIC_NAME, length
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'a', b'b', 0, 0x00, // string + padding (1 byte)
            0x07, 0x00, 8, 0, // PID_TYPE_NAME, length
            3, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'c', b'd', 0, 0x00, // string + padding (1 byte)
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
            1, 0, 0, 0, // ,
            2, 0, 0, 0, // ,
            3, 0, 0, 0, // ,
            4, 0, 0, 0, // ,
               // 0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        let dynamic_data = data.create_dynamic_sample();
        let mut buffer = Vec::new();
        let mut serializer = PlCdrLeSerializer::new(&mut buffer);
        dynamic_data.serialize(&mut serializer).unwrap();

        assert_eq!(buffer, expected);
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
