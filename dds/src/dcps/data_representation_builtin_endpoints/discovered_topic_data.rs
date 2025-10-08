use crate::{
    builtin_topics::{BuiltInTopicKey, TopicBuiltinTopicData},
    dcps::data_representation_builtin_endpoints::parameter_id_values::{
        PID_DATA_REPRESENTATION, PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY,
        PID_ENDPOINT_GUID, PID_HISTORY, PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS,
        PID_OWNERSHIP, PID_RELIABILITY, PID_RESOURCE_LIMITS, PID_TOPIC_DATA, PID_TOPIC_NAME,
        PID_TRANSPORT_PRIORITY, PID_TYPE_NAME,
    },
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
};
use alloc::string::String;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DiscoveredTopicData {
    pub(crate) topic_builtin_topic_data: TopicBuiltinTopicData,
}

impl TypeSupport for DiscoveredTopicData {
    fn get_type() -> dust_dds::xtypes::dynamic_type::DynamicType {
        extern crate alloc;
        let mut builder = dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::create_type(
            dust_dds::xtypes::dynamic_type::TypeDescriptor {
                kind: dust_dds::xtypes::dynamic_type::TypeKind::STRUCTURE,
                name: alloc::string::String::from("TopicBuiltinTopicData"),
                base_type: None,
                discriminator_type: None,
                bound: alloc::vec::Vec::new(),
                element_type: None,
                key_element_type: None,
                extensibility_kind: dust_dds::xtypes::dynamic_type::ExtensibilityKind::Mutable,
                is_nested: false,
            },
        );
        builder
            .add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
                name: alloc::string::String::from("key"),
                id: PID_ENDPOINT_GUID as u32,
                r#type:
                    <BuiltInTopicKey as dust_dds::xtypes::binding::XTypesBinding>::get_dynamic_type(
                    ),
                default_value: None,
                index: 0u32,
                try_construct_kind: dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: true,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
                name: alloc::string::String::from("name"),
                id: PID_TOPIC_NAME as u32,
                r#type: <String as dust_dds::xtypes::binding::XTypesBinding>::get_dynamic_type(),
                default_value: None,
                index: 1u32,
                try_construct_kind: dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
                name: alloc::string::String::from("type_name"),
                id: PID_TYPE_NAME as u32,
                r#type: <String as dust_dds::xtypes::binding::XTypesBinding>::get_dynamic_type(),
                default_value: None,
                index: 2u32,
                try_construct_kind: dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder.add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
            name:alloc::string::String::from("durability"),id:PID_DURABILITY as u32,r#type: <DurabilityQosPolicy as dust_dds::xtypes::binding::XTypesBinding> ::get_dynamic_type(),default_value:Some(<DurabilityQosPolicy as Default> ::default().into()),index:3u32,try_construct_kind:dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,label:alloc::vec::Vec::new(),is_key:false,is_optional:true,is_must_understand:true,is_shared:false,is_default_label:false,
        }).unwrap();
        builder.add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
            name:alloc::string::String::from("deadline"),id:PID_DEADLINE as u32,r#type: <DeadlineQosPolicy as dust_dds::xtypes::binding::XTypesBinding> ::get_dynamic_type(),default_value:Some(<DeadlineQosPolicy as Default> ::default().into()),index:4u32,try_construct_kind:dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,label:alloc::vec::Vec::new(),is_key:false,is_optional:true,is_must_understand:true,is_shared:false,is_default_label:false,
        }).unwrap();
        builder.add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
            name:alloc::string::String::from("latency_budget"),id:PID_LATENCY_BUDGET as u32,r#type: <LatencyBudgetQosPolicy as dust_dds::xtypes::binding::XTypesBinding> ::get_dynamic_type(),default_value:Some(<LatencyBudgetQosPolicy as Default> ::default().into()),index:5u32,try_construct_kind:dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,label:alloc::vec::Vec::new(),is_key:false,is_optional:true,is_must_understand:true,is_shared:false,is_default_label:false,
        }).unwrap();
        builder.add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
            name:alloc::string::String::from("liveliness"),id:PID_LIVELINESS as u32,r#type: <LivelinessQosPolicy as dust_dds::xtypes::binding::XTypesBinding> ::get_dynamic_type(),default_value:Some(<LivelinessQosPolicy as Default> ::default().into()),index:6u32,try_construct_kind:dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,label:alloc::vec::Vec::new(),is_key:false,is_optional:true,is_must_understand:true,is_shared:false,is_default_label:false,
        }).unwrap();
        builder.add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
            name:alloc::string::String::from("reliability"),id:PID_RELIABILITY as u32,r#type: <ReliabilityQosPolicy as dust_dds::xtypes::binding::XTypesBinding> ::get_dynamic_type(),default_value:Some(DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS.into()),index:7u32,try_construct_kind:dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,label:alloc::vec::Vec::new(),is_key:false,is_optional:true,is_must_understand:true,is_shared:false,is_default_label:false,
        }).unwrap();
        builder.add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
            name:alloc::string::String::from("transport_priority"),id:PID_TRANSPORT_PRIORITY as u32,r#type: <TransportPriorityQosPolicy as dust_dds::xtypes::binding::XTypesBinding> ::get_dynamic_type(),default_value:Some(<TransportPriorityQosPolicy as Default> ::default().into()),index:8u32,try_construct_kind:dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,label:alloc::vec::Vec::new(),is_key:false,is_optional:true,is_must_understand:true,is_shared:false,is_default_label:false,
        }).unwrap();
        builder.add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
            name:alloc::string::String::from("lifespan"),id:PID_LIFESPAN as u32,r#type: <LifespanQosPolicy as dust_dds::xtypes::binding::XTypesBinding> ::get_dynamic_type(),default_value:Some(<LifespanQosPolicy as Default> ::default().into()),index:9u32,try_construct_kind:dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,label:alloc::vec::Vec::new(),is_key:false,is_optional:true,is_must_understand:true,is_shared:false,is_default_label:false,
        }).unwrap();
        builder.add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
            name:alloc::string::String::from("destination_order"),id:PID_DESTINATION_ORDER as u32,r#type: <DestinationOrderQosPolicy as dust_dds::xtypes::binding::XTypesBinding> ::get_dynamic_type(),default_value:Some(<DestinationOrderQosPolicy as Default> ::default().into()),index:10u32,try_construct_kind:dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,label:alloc::vec::Vec::new(),is_key:false,is_optional:true,is_must_understand:true,is_shared:false,is_default_label:false,
        }).unwrap();
        builder.add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
            name:alloc::string::String::from("history"),id:PID_HISTORY as u32,r#type: <HistoryQosPolicy as dust_dds::xtypes::binding::XTypesBinding> ::get_dynamic_type(),default_value:Some(<HistoryQosPolicy as Default> ::default().into()),index:11u32,try_construct_kind:dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,label:alloc::vec::Vec::new(),is_key:false,is_optional:true,is_must_understand:true,is_shared:false,is_default_label:false,
        }).unwrap();
        builder.add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
            name:alloc::string::String::from("resource_limits"),id:PID_RESOURCE_LIMITS as u32,r#type: <ResourceLimitsQosPolicy as dust_dds::xtypes::binding::XTypesBinding> ::get_dynamic_type(),default_value:Some(<ResourceLimitsQosPolicy as Default> ::default().into()),index:12u32,try_construct_kind:dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,label:alloc::vec::Vec::new(),is_key:false,is_optional:true,is_must_understand:true,is_shared:false,is_default_label:false,
        }).unwrap();
        builder.add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
            name:alloc::string::String::from("ownership"),id:PID_OWNERSHIP as u32,r#type: <OwnershipQosPolicy as dust_dds::xtypes::binding::XTypesBinding> ::get_dynamic_type(),default_value:Some(<OwnershipQosPolicy as Default> ::default().into()),index:13u32,try_construct_kind:dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,label:alloc::vec::Vec::new(),is_key:false,is_optional:true,is_must_understand:true,is_shared:false,is_default_label:false,
        }).unwrap();
        builder.add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
            name:alloc::string::String::from("topic_data"),id:PID_TOPIC_DATA as u32,r#type: <TopicDataQosPolicy as dust_dds::xtypes::binding::XTypesBinding> ::get_dynamic_type(),default_value:Some(<TopicDataQosPolicy as Default> ::default().into()),index:14u32,try_construct_kind:dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,label:alloc::vec::Vec::new(),is_key:false,is_optional:true,is_must_understand:true,is_shared:false,is_default_label:false,
        }).unwrap();
        builder.add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
            name:alloc::string::String::from("representation"),id:PID_DATA_REPRESENTATION as u32,r#type: <DataRepresentationQosPolicy as dust_dds::xtypes::binding::XTypesBinding> ::get_dynamic_type(),default_value:Some(<DataRepresentationQosPolicy as Default> ::default().into()),index:15u32,try_construct_kind:dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,label:alloc::vec::Vec::new(),is_key:false,is_optional:true,is_must_understand:true,is_shared:false,is_default_label:false,
        }).unwrap();
        builder.build()
    }
    fn create_dynamic_sample(self) -> dust_dds::xtypes::dynamic_type::DynamicData {
        let mut data =
            dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data(Self::get_type());
        data.set_value(
            PID_ENDPOINT_GUID as u32,
            self.topic_builtin_topic_data.key.into(),
        )
        .unwrap();
        data.set_value(
            PID_TOPIC_NAME as u32,
            self.topic_builtin_topic_data.name.into(),
        )
        .unwrap();
        data.set_value(
            PID_TYPE_NAME as u32,
            self.topic_builtin_topic_data.type_name.into(),
        )
        .unwrap();
        data.set_value(
            PID_DURABILITY as u32,
            self.topic_builtin_topic_data.durability.into(),
        )
        .unwrap();
        data.set_value(
            PID_DEADLINE as u32,
            self.topic_builtin_topic_data.deadline.into(),
        )
        .unwrap();
        data.set_value(
            PID_LATENCY_BUDGET as u32,
            self.topic_builtin_topic_data.latency_budget.into(),
        )
        .unwrap();
        data.set_value(
            PID_LIVELINESS as u32,
            self.topic_builtin_topic_data.liveliness.into(),
        )
        .unwrap();
        data.set_value(
            PID_RELIABILITY as u32,
            self.topic_builtin_topic_data.reliability.into(),
        )
        .unwrap();
        data.set_value(
            PID_TRANSPORT_PRIORITY as u32,
            self.topic_builtin_topic_data.transport_priority.into(),
        )
        .unwrap();
        data.set_value(
            PID_LIFESPAN as u32,
            self.topic_builtin_topic_data.lifespan.into(),
        )
        .unwrap();
        data.set_value(
            PID_DESTINATION_ORDER as u32,
            self.topic_builtin_topic_data.destination_order.into(),
        )
        .unwrap();
        data.set_value(
            PID_HISTORY as u32,
            self.topic_builtin_topic_data.history.into(),
        )
        .unwrap();
        data.set_value(
            PID_RESOURCE_LIMITS as u32,
            self.topic_builtin_topic_data.resource_limits.into(),
        )
        .unwrap();
        data.set_value(
            PID_OWNERSHIP as u32,
            self.topic_builtin_topic_data.ownership.into(),
        )
        .unwrap();
        data.set_value(
            PID_TOPIC_DATA as u32,
            self.topic_builtin_topic_data.topic_data.into(),
        )
        .unwrap();
        data.set_value(
            PID_DATA_REPRESENTATION as u32,
            self.topic_builtin_topic_data.representation.into(),
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
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
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
