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
            PID_ENDPOINT_GUID, PID_GROUP_DATA, PID_HISTORY, PID_LATENCY_BUDGET, PID_LIFESPAN,
            PID_LIVELINESS, PID_OWNERSHIP, PID_OWNERSHIP_STRENGTH, PID_PARTICIPANT_GUID,
            PID_PARTITION, PID_PRESENTATION, PID_RELIABILITY, PID_RESOURCE_LIMITS,
            PID_TIME_BASED_FILTER, PID_TOPIC_DATA, PID_TOPIC_NAME, PID_TRANSPORT_PRIORITY,
            PID_TYPE_NAME, PID_USER_DATA,
        },
        payload_serializer_deserializer::parameter_list_serializer::ParameterListCdrSerializer,
    },
    infrastructure::{
        error::DdsResult,
        qos_policy::{
            DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
            DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
        },
        type_support::{DdsSerialize, TypeSupport},
    },
    xtypes::{
        deserialize::XTypesDeserialize,
        dynamic_type::{
            DynamicTypeBuilderFactory, ExtensibilityKind, MemberDescriptor, TryConstructKind,
            TypeDescriptor, XTypesBinding, TK_STRUCTURE,
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
        extern crate alloc;
        let mut builder = DynamicTypeBuilderFactory::create_type(TypeDescriptor {
            kind: TK_STRUCTURE,
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
                name: String::from("data"),
                id: 777,
                r#type: DynamicTypeBuilderFactory::create_array_type(
                    u8::get_dynamic_type(),
                    vec![u32::MAX],
                )
                .build(),
                default_value: String::new(),
                index: 777,
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
        fn serialize_data(this: &TopicBuiltinTopicData) -> DdsResult<Vec<u8>> {
            let mut serializer = ParameterListCdrSerializer::default();
            serializer.write_header()?;

            // topic_builtin_topic_data: TopicBuiltinTopicData:

            serializer.write(PID_ENDPOINT_GUID, &this.key)?;
            serializer.write(PID_TOPIC_NAME, &this.name)?;
            serializer.write(PID_TYPE_NAME, &this.type_name)?;
            serializer.write_with_default(PID_DURABILITY, &this.durability, &Default::default())?;
            serializer.write_with_default(PID_DEADLINE, &this.deadline, &Default::default())?;
            serializer.write_with_default(
                PID_LATENCY_BUDGET,
                &this.latency_budget,
                &Default::default(),
            )?;
            serializer.write_with_default(PID_LIVELINESS, &this.liveliness, &Default::default())?;
            serializer.write_with_default(
                PID_RELIABILITY,
                &this.reliability,
                &DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
            )?;
            serializer.write_with_default(
                PID_TRANSPORT_PRIORITY,
                &this.transport_priority,
                &Default::default(),
            )?;
            serializer.write_with_default(PID_LIFESPAN, &this.lifespan, &Default::default())?;
            serializer.write_with_default(
                PID_DESTINATION_ORDER,
                &this.destination_order,
                &Default::default(),
            )?;
            serializer.write_with_default(PID_HISTORY, &this.history, &Default::default())?;
            serializer.write_with_default(
                PID_RESOURCE_LIMITS,
                &this.resource_limits,
                &Default::default(),
            )?;
            serializer.write_with_default(PID_OWNERSHIP, &this.ownership, &Default::default())?;
            serializer.write_with_default(PID_TOPIC_DATA, &this.topic_data, &Default::default())?;
            serializer.write_with_default(
                PID_DATA_REPRESENTATION,
                &this.representation,
                &Default::default(),
            )?;

            serializer.write_sentinel()?;
            Ok(serializer.writer)
        }
        let mut data =
            dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data(Self::get_type());
        data.set_uint8_values(777, serialize_data(&self).unwrap())
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

impl DdsSerialize for PublicationBuiltinTopicData {
    fn serialize_data(&self) -> DdsResult<Vec<u8>> {
        let mut serializer = ParameterListCdrSerializer::default();
        serializer.write_header()?;

        // dds_publication_data: PublicationBuiltinTopicData:

        serializer.write(PID_ENDPOINT_GUID, &self.key)?;
        // Default value is a deviation from the standard and is used for interoperability reasons:
        serializer.write_with_default(
            PID_PARTICIPANT_GUID,
            &self.participant_key,
            &Default::default(),
        )?;
        serializer.write(PID_TOPIC_NAME, &self.topic_name)?;
        serializer.write(PID_TYPE_NAME, &self.type_name)?;
        serializer.write_with_default(PID_DURABILITY, &self.durability, &Default::default())?;
        serializer.write_with_default(PID_DEADLINE, &self.deadline, &Default::default())?;
        serializer.write_with_default(
            PID_LATENCY_BUDGET,
            &self.latency_budget,
            &Default::default(),
        )?;
        serializer.write_with_default(PID_LIVELINESS, &self.liveliness, &Default::default())?;
        serializer.write_with_default(
            PID_RELIABILITY,
            &self.reliability,
            &DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
        )?;
        serializer.write_with_default(PID_LIFESPAN, &self.lifespan, &Default::default())?;
        serializer.write_with_default(PID_USER_DATA, &self.user_data, &Default::default())?;
        serializer.write_with_default(PID_OWNERSHIP, &self.ownership, &Default::default())?;
        serializer.write_with_default(
            PID_OWNERSHIP_STRENGTH,
            &self.ownership_strength,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_DESTINATION_ORDER,
            &self.destination_order,
            &Default::default(),
        )?;
        serializer.write_with_default(PID_PRESENTATION, &self.presentation, &Default::default())?;
        serializer.write_with_default(PID_PARTITION, &self.partition, &Default::default())?;
        serializer.write_with_default(PID_TOPIC_DATA, &self.topic_data, &Default::default())?;
        serializer.write_with_default(PID_GROUP_DATA, &self.group_data, &Default::default())?;

        serializer.write_with_default(
            PID_DATA_REPRESENTATION,
            &self.representation,
            &Default::default(),
        )?;

        serializer.write_sentinel()?;
        Ok(serializer.writer)
    }
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

impl DdsSerialize for SubscriptionBuiltinTopicData {
    fn serialize_data(&self) -> DdsResult<Vec<u8>> {
        let mut serializer = ParameterListCdrSerializer::default();
        serializer.write_header()?;

        // subscription_builtin_topic_data: SubscriptionBuiltinTopicData:
        serializer.write(PID_ENDPOINT_GUID, &self.key)?;
        // Default value is a deviation from the standard and is used for interoperability reasons:
        serializer.write_with_default(
            PID_PARTICIPANT_GUID,
            &self.participant_key,
            &Default::default(),
        )?;
        serializer.write(PID_TOPIC_NAME, &self.topic_name)?;
        serializer.write(PID_TYPE_NAME, &self.type_name)?;
        serializer.write_with_default(PID_DURABILITY, &self.durability, &Default::default())?;
        serializer.write_with_default(PID_DEADLINE, &self.deadline, &Default::default())?;
        serializer.write_with_default(
            PID_LATENCY_BUDGET,
            &self.latency_budget,
            &Default::default(),
        )?;
        serializer.write_with_default(PID_LIVELINESS, &self.liveliness, &Default::default())?;
        serializer.write_with_default(
            PID_RELIABILITY,
            &self.reliability,
            &DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
        )?;
        serializer.write_with_default(PID_OWNERSHIP, &self.ownership, &Default::default())?;
        serializer.write_with_default(
            PID_DESTINATION_ORDER,
            &self.destination_order,
            &Default::default(),
        )?;
        serializer.write_with_default(PID_USER_DATA, &self.user_data, &Default::default())?;
        serializer.write_with_default(
            PID_TIME_BASED_FILTER,
            &self.time_based_filter,
            &Default::default(),
        )?;
        serializer.write_with_default(PID_PRESENTATION, &self.presentation, &Default::default())?;
        serializer.write_with_default(PID_PARTITION, &self.partition, &Default::default())?;
        serializer.write_with_default(PID_TOPIC_DATA, &self.topic_data, &Default::default())?;
        serializer.write_with_default(PID_GROUP_DATA, &self.group_data, &Default::default())?;
        serializer.write_with_default(
            PID_DATA_REPRESENTATION,
            &self.representation,
            &Default::default(),
        )?;

        serializer.write_sentinel()?;
        Ok(serializer.writer)
    }
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
