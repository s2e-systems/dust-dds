use super::infrastructure::qos_policy::{
    DataRepresentationQosPolicy, DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
    GroupDataQosPolicy, HistoryQosPolicy, LatencyBudgetQosPolicy, LifespanQosPolicy,
    LivelinessQosPolicy, OwnershipQosPolicy, OwnershipStrengthQosPolicy, PartitionQosPolicy,
    PresentationQosPolicy, ReliabilityQosPolicy, ResourceLimitsQosPolicy, TimeBasedFilterQosPolicy,
    TopicDataQosPolicy, TransportPriorityQosPolicy, UserDataQosPolicy,
};
use crate::{
    dcps::data_representation_builtin_endpoints::{
        parameter_id_values::{
            PID_DATA_REPRESENTATION, PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY,
            PID_ENDPOINT_GUID, PID_HISTORY, PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS,
            PID_OWNERSHIP, PID_PARTICIPANT_GUID, PID_RELIABILITY, PID_RESOURCE_LIMITS,
            PID_TOPIC_DATA, PID_TOPIC_NAME, PID_TRANSPORT_PRIORITY, PID_TYPE_NAME,
        },
        payload_serializer_deserializer::parameter_list_serializer::ParameterListCdrSerializer,
    },
    infrastructure::{
        error::DdsResult, qos_policy::DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
        type_support::TypeSupport,
    },
    xtypes::{
        deserialize::XTypesDeserialize,
        dynamic_type::{
            DynamicTypeBuilderFactory, ExtensibilityKind, MemberDescriptor, TryConstructKind,
            TypeDescriptor, TypeKind, XTypesBinding,
        },
        serialize::XTypesSerialize,
    },
};
use alloc::{string::String, vec::Vec};

/// Topic name of the built-in publication discovery topic
pub const DCPS_PUBLICATION: &str = "DCPSPublication";

/// Topic name of the built-in subscription discovery topic
pub const DCPS_SUBSCRIPTION: &str = "DCPSSubscription";

/// Topic name of the built-in topic discovery topic
pub const DCPS_TOPIC: &str = "DCPSTopic";

/// Topic name of the built-in participant discovery topic
pub const DCPS_PARTICIPANT: &str = "DCPSParticipant";

/// Structure representing the instance handle (or key) of an entity.
#[derive(Debug, PartialEq, Eq, Clone, Default, XTypesSerialize, XTypesDeserialize, TypeSupport)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct BuiltInTopicKey {
    /// InstanceHandle value as an array of 16 octets.
    pub value: [u8; 16], // Originally in the DDS idl [i32;3]
}

/// Structure representing a discovered [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant).
#[derive(Debug, PartialEq, Eq, Clone, XTypesSerialize, XTypesDeserialize)]
pub struct ParticipantBuiltinTopicData {
    pub(crate) key: BuiltInTopicKey,
    pub(crate) user_data: UserDataQosPolicy,
}

impl ParticipantBuiltinTopicData {
    /// Get the key value of the discovered participant.
    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key
    }

    /// Get the user data value of the discovered participant.
    pub fn user_data(&self) -> &UserDataQosPolicy {
        &self.user_data
    }
}

/// Structure representing a discovered [`Topic`](crate::topic_definition::topic::Topic).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TopicBuiltinTopicData {
    pub(crate) key: BuiltInTopicKey,
    pub(crate) name: String,
    pub(crate) type_name: String,
    pub(crate) durability: DurabilityQosPolicy,
    pub(crate) deadline: DeadlineQosPolicy,
    pub(crate) latency_budget: LatencyBudgetQosPolicy,
    pub(crate) liveliness: LivelinessQosPolicy,
    pub(crate) reliability: ReliabilityQosPolicy,
    pub(crate) transport_priority: TransportPriorityQosPolicy,
    pub(crate) lifespan: LifespanQosPolicy,
    pub(crate) destination_order: DestinationOrderQosPolicy,
    pub(crate) history: HistoryQosPolicy,
    pub(crate) resource_limits: ResourceLimitsQosPolicy,
    pub(crate) ownership: OwnershipQosPolicy,
    pub(crate) topic_data: TopicDataQosPolicy,
    pub(crate) representation: DataRepresentationQosPolicy,
}

impl TypeSupport for TopicBuiltinTopicData {
    fn get_type() -> crate::xtypes::dynamic_type::DynamicType {
        let mut builder = DynamicTypeBuilderFactory::create_type(TypeDescriptor {
            kind: TypeKind::STRUCTURE,
            name: String::from("TopicBuiltinTopicData"),
            base_type: None,
            discriminator_type: None,
            bound: alloc::vec::Vec::new(),
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        });
        builder
            .add_member(MemberDescriptor {
                name: String::from("key"),
                id: PID_PARTICIPANT_GUID as u32,
                r#type: <BuiltInTopicKey as TypeSupport>::get_type(),
                default_value: String::new(),
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
                default_value: String::new(),
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
                default_value: String::new(),
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
                default_value: String::new(),
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
                default_value: String::new(),
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
                default_value: String::new(),
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
                default_value: String::new(),
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
                default_value: String::new(),
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
                default_value: String::new(),
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
                default_value: String::new(),
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
                default_value: String::new(),
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
                default_value: String::new(),
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
                default_value: String::new(),
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
                default_value: String::new(),
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
                default_value: String::new(),
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
                id: PID_TOPIC_DATA as u32,
                r#type: <DataRepresentationQosPolicy as TypeSupport>::get_type(),
                default_value: String::new(),
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
            PID_PARTICIPANT_GUID as u32,
            self.key.create_dynamic_sample(),
        )
        .unwrap();
        data.set_string_value(PID_TOPIC_NAME as u32, self.name)
            .unwrap();
        data.set_string_value(PID_TYPE_NAME as u32, self.type_name)
            .unwrap();
        data.set_complex_value(
            PID_DURABILITY as u32,
            self.durability.create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(PID_DEADLINE as u32, self.deadline.create_dynamic_sample())
            .unwrap();
        data.set_complex_value(
            PID_LATENCY_BUDGET as u32,
            self.latency_budget.create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(
            PID_LIVELINESS as u32,
            self.liveliness.create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(
            PID_RELIABILITY as u32,
            self.reliability.create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(
            PID_TRANSPORT_PRIORITY as u32,
            self.transport_priority.create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(PID_LIFESPAN as u32, self.lifespan.create_dynamic_sample())
            .unwrap();
        data.set_complex_value(
            PID_DESTINATION_ORDER as u32,
            self.destination_order.create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(PID_HISTORY as u32, self.history.create_dynamic_sample())
            .unwrap();
        data.set_complex_value(
            PID_RESOURCE_LIMITS as u32,
            self.resource_limits.create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(PID_OWNERSHIP as u32, self.ownership.create_dynamic_sample())
            .unwrap();
        data.set_complex_value(
            PID_TOPIC_DATA as u32,
            self.topic_data.create_dynamic_sample(),
        )
        .unwrap();
        data.set_complex_value(
            PID_DATA_REPRESENTATION as u32,
            self.representation.create_dynamic_sample(),
        )
        .unwrap();

        data
    }
}

impl TopicBuiltinTopicData {
    /// Get the key value of the discovered topic.
    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key
    }

    /// Get the name of the discovered topic.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the type name of the discovered topic.
    pub fn get_type_name(&self) -> &str {
        &self.type_name
    }

    /// Get the durability QoS policy of the discovered topic.
    pub fn durability(&self) -> &DurabilityQosPolicy {
        &self.durability
    }

    /// Get the deadline QoS policy of the discovered topic.
    pub fn deadline(&self) -> &DeadlineQosPolicy {
        &self.deadline
    }

    /// Get the latency budget QoS policy of the discovered topic.
    pub fn latency_budget(&self) -> &LatencyBudgetQosPolicy {
        &self.latency_budget
    }

    /// Get the liveliness QoS policy of the discovered topic.
    pub fn liveliness(&self) -> &LivelinessQosPolicy {
        &self.liveliness
    }

    /// Get the reliability QoS policy of the discovered topic.
    pub fn reliability(&self) -> &ReliabilityQosPolicy {
        &self.reliability
    }

    /// Get the transport priority QoS policy of the discovered topic.
    pub fn transport_priority(&self) -> &TransportPriorityQosPolicy {
        &self.transport_priority
    }

    /// Get the lifespan QoS policy of the discovered topic.
    pub fn lifespan(&self) -> &LifespanQosPolicy {
        &self.lifespan
    }

    /// Get the destination order QoS policy of the discovered topic.
    pub fn destination_order(&self) -> &DestinationOrderQosPolicy {
        &self.destination_order
    }

    /// Get the history QoS policy of the discovered topic.
    pub fn history(&self) -> &HistoryQosPolicy {
        &self.history
    }

    /// Get the resource limits QoS policy of the discovered topic.
    pub fn resource_limits(&self) -> &ResourceLimitsQosPolicy {
        &self.resource_limits
    }

    /// Get the ownership QoS policy of the discovered topic.
    pub fn ownership(&self) -> &OwnershipQosPolicy {
        &self.ownership
    }

    /// Get the topic data QoS policy of the discovered topic.
    pub fn topic_data(&self) -> &TopicDataQosPolicy {
        &self.topic_data
    }

    /// Get the data representation QoS policy of the discovered topic.
    pub fn representation(&self) -> &DataRepresentationQosPolicy {
        &self.representation
    }
}

/// Structure representing a discovered [`DataWriter`](crate::publication::data_writer::DataWriter).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PublicationBuiltinTopicData {
    pub(crate) key: BuiltInTopicKey,
    pub(crate) participant_key: BuiltInTopicKey,
    pub(crate) topic_name: String,
    pub(crate) type_name: String,
    pub(crate) durability: DurabilityQosPolicy,
    pub(crate) deadline: DeadlineQosPolicy,
    pub(crate) latency_budget: LatencyBudgetQosPolicy,
    pub(crate) liveliness: LivelinessQosPolicy,
    pub(crate) reliability: ReliabilityQosPolicy,
    pub(crate) lifespan: LifespanQosPolicy,
    pub(crate) user_data: UserDataQosPolicy,
    pub(crate) ownership: OwnershipQosPolicy,
    pub(crate) ownership_strength: OwnershipStrengthQosPolicy,
    pub(crate) destination_order: DestinationOrderQosPolicy,
    pub(crate) presentation: PresentationQosPolicy,
    pub(crate) partition: PartitionQosPolicy,
    pub(crate) topic_data: TopicDataQosPolicy,
    pub(crate) group_data: GroupDataQosPolicy,
    pub(crate) representation: DataRepresentationQosPolicy,
}

impl PublicationBuiltinTopicData {
    /// Get the key value of the discovered writer.
    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key
    }

    /// Get the key value of the parent participant of the discovered writer.
    pub fn participant_key(&self) -> &BuiltInTopicKey {
        &self.participant_key
    }

    /// Get the name of the topic associated with the discovered writer.
    pub fn topic_name(&self) -> &str {
        &self.topic_name
    }

    /// Get the name of the type associated with the discovered writer.
    pub fn get_type_name(&self) -> &str {
        &self.type_name
    }

    /// Get the durability QoS policy of the discovered writer.
    pub fn durability(&self) -> &DurabilityQosPolicy {
        &self.durability
    }

    /// Get the deadline QoS policy of the discovered writer.
    pub fn deadline(&self) -> &DeadlineQosPolicy {
        &self.deadline
    }

    /// Get the latency budget QoS policy of the discovered writer.
    pub fn latency_budget(&self) -> &LatencyBudgetQosPolicy {
        &self.latency_budget
    }

    /// Get the liveliness QoS policy of the discovered writer.
    pub fn liveliness(&self) -> &LivelinessQosPolicy {
        &self.liveliness
    }

    /// Get the reliability QoS policy of the discovered writer.
    pub fn reliability(&self) -> &ReliabilityQosPolicy {
        &self.reliability
    }

    /// Get the lifespan QoS policy of the discovered writer.
    pub fn lifespan(&self) -> &LifespanQosPolicy {
        &self.lifespan
    }

    /// Get the user data QoS policy of the discovered writer.
    pub fn user_data(&self) -> &UserDataQosPolicy {
        &self.user_data
    }

    /// Get the ownership QoS policy of the discovered writer.
    pub fn ownership(&self) -> &OwnershipQosPolicy {
        &self.ownership
    }

    /// Get the ownership strength QoS policy of the discovered writer.
    pub fn ownership_strength(&self) -> &OwnershipStrengthQosPolicy {
        &self.ownership_strength
    }

    /// Get the destination order QoS policy of the discovered writer.
    pub fn destination_order(&self) -> &DestinationOrderQosPolicy {
        &self.destination_order
    }

    /// Get the presentation QoS policy of the discovered writer.
    pub fn presentation(&self) -> &PresentationQosPolicy {
        &self.presentation
    }

    /// Get the partition QoS policy of the discovered writer.
    pub fn partition(&self) -> &PartitionQosPolicy {
        &self.partition
    }

    /// Get the topic data QoS policy of the topic associated with the discovered writer.
    pub fn topic_data(&self) -> &TopicDataQosPolicy {
        &self.topic_data
    }

    /// Get the group data QoS policy of the discovered writer.
    pub fn group_data(&self) -> &GroupDataQosPolicy {
        &self.group_data
    }

    /// Get the data representation QoS policy of the discovered writer.
    pub fn representation(&self) -> &DataRepresentationQosPolicy {
        &self.representation
    }
}

/// Structure representing a discovered [`DataReader`](crate::subscription::data_reader::DataReader).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SubscriptionBuiltinTopicData {
    pub(crate) key: BuiltInTopicKey,
    pub(crate) participant_key: BuiltInTopicKey,
    pub(crate) topic_name: String,
    pub(crate) type_name: String,
    pub(crate) durability: DurabilityQosPolicy,
    pub(crate) deadline: DeadlineQosPolicy,
    pub(crate) latency_budget: LatencyBudgetQosPolicy,
    pub(crate) liveliness: LivelinessQosPolicy,
    pub(crate) reliability: ReliabilityQosPolicy,
    pub(crate) ownership: OwnershipQosPolicy,
    pub(crate) destination_order: DestinationOrderQosPolicy,
    pub(crate) user_data: UserDataQosPolicy,
    pub(crate) time_based_filter: TimeBasedFilterQosPolicy,
    pub(crate) presentation: PresentationQosPolicy,
    pub(crate) partition: PartitionQosPolicy,
    pub(crate) topic_data: TopicDataQosPolicy,
    pub(crate) group_data: GroupDataQosPolicy,
    pub(crate) representation: DataRepresentationQosPolicy,
}

impl SubscriptionBuiltinTopicData {
    /// Get the key value of the discovered reader.
    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key
    }

    /// Get the key value of the parent participant of the discovered reader.
    pub fn participant_key(&self) -> &BuiltInTopicKey {
        &self.participant_key
    }

    /// Get the name of the topic associated with the discovered reader.
    pub fn topic_name(&self) -> &str {
        &self.topic_name
    }

    /// Get the name of the type associated with the discovered reader.
    pub fn get_type_name(&self) -> &str {
        &self.type_name
    }

    /// Get the durability QoS policy of the discovered reader.
    pub fn durability(&self) -> &DurabilityQosPolicy {
        &self.durability
    }

    /// Get the deadline QoS policy of the discovered reader.
    pub fn deadline(&self) -> &DeadlineQosPolicy {
        &self.deadline
    }

    /// Get the latency budget QoS policy of the discovered reader.
    pub fn latency_budget(&self) -> &LatencyBudgetQosPolicy {
        &self.latency_budget
    }

    /// Get the liveliness QoS policy of the discovered reader.
    pub fn liveliness(&self) -> &LivelinessQosPolicy {
        &self.liveliness
    }

    /// Get the reliability QoS policy of the discovered reader.
    pub fn reliability(&self) -> &ReliabilityQosPolicy {
        &self.reliability
    }

    /// Get the ownership QoS policy of the discovered reader.
    pub fn ownership(&self) -> &OwnershipQosPolicy {
        &self.ownership
    }

    /// Get the destination order QoS policy of the discovered reader.
    pub fn destination_order(&self) -> &DestinationOrderQosPolicy {
        &self.destination_order
    }

    /// Get the user data QoS policy of the discovered reader.
    pub fn user_data(&self) -> &UserDataQosPolicy {
        &self.user_data
    }

    /// Get the time based filter QoS policy of the discovered reader.
    pub fn time_based_filter(&self) -> &TimeBasedFilterQosPolicy {
        &self.time_based_filter
    }

    /// Get the presentation QoS policy of the discovered reader.
    pub fn presentation(&self) -> &PresentationQosPolicy {
        &self.presentation
    }

    /// Get the partition QoS policy of the discovered reader.
    pub fn partition(&self) -> &PartitionQosPolicy {
        &self.partition
    }

    /// Get the topic data QoS policy of the topic associated with the discovered reader.
    pub fn topic_data(&self) -> &TopicDataQosPolicy {
        &self.topic_data
    }

    /// Get the group data QoS policy of the discovered reader.
    pub fn group_data(&self) -> &GroupDataQosPolicy {
        &self.group_data
    }

    /// Get the data representation QoS policy of the discovered reader.
    pub fn representation(&self) -> &DataRepresentationQosPolicy {
        &self.representation
    }
}
