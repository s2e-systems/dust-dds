use crate::{
    infrastructure::qos_policy::{
        DataRepresentationQosPolicy, DeadlineQosPolicy, DestinationOrderQosPolicy,
        DurabilityQosPolicy, GroupDataQosPolicy, HistoryQosPolicy, LatencyBudgetQosPolicy,
        LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, OwnershipStrengthQosPolicy,
        PartitionQosPolicy, PresentationQosPolicy, ReliabilityQosPolicy, ResourceLimitsQosPolicy,
        TimeBasedFilterQosPolicy, TopicDataQosPolicy, TransportPriorityQosPolicy,
        UserDataQosPolicy,
    },
    topic_definition::type_support::DdsHasKey,
    xtypes::{deserialize::XTypesDeserialize, serialize::XTypesSerialize},
};

/// Structure representing the instance handle (or key) of an entity.
#[derive(Debug, PartialEq, Eq, Clone, Default, XTypesSerialize, XTypesDeserialize)]
pub struct BuiltInTopicKey {
    /// InstanceHandle value as an array of 16 octets.
    pub value: [u8; 16], // Originally in the DDS idl [i32;3]
}

/// Structure representing a discovered [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant).
#[derive(Debug, PartialEq, Eq, Clone)]
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

impl DdsHasKey for ParticipantBuiltinTopicData {
    const HAS_KEY: bool = true;
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

impl DdsHasKey for TopicBuiltinTopicData {
    const HAS_KEY: bool = true;
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
    pub(crate) xml_type: String,
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

    /// Get the XML type representation of the discovered writer.
    /// Note: This is only available if matched with a Dust DDS reader which transmits this information as part of the discovery.
    pub fn xml_type(&self) -> &str {
        &self.xml_type
    }

    /// Get the data representation QoS policy of the discovered writer.
    pub fn representation(&self) -> &DataRepresentationQosPolicy {
        &self.representation
    }
}

impl DdsHasKey for PublicationBuiltinTopicData {
    const HAS_KEY: bool = true;
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
    pub(crate) xml_type: String,
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

    /// Get the XML type representation of the discovered reader.
    /// Note: This is only available if matched with a DustDDS writer which transmits this information as part of the discovery.
    pub fn xml_type(&self) -> &str {
        &self.xml_type
    }

    /// Get the data representation QoS policy of the discovered reader.
    pub fn representation(&self) -> &DataRepresentationQosPolicy {
        &self.representation
    }
}

impl DdsHasKey for SubscriptionBuiltinTopicData {
    const HAS_KEY: bool = true;
}
