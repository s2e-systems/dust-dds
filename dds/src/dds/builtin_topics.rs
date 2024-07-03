use dust_dds_derive::DdsDeserialize;

use crate::{
    data_representation_builtin_endpoints::parameter_id_values::{
        PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY, PID_ENDPOINT_GUID, PID_GROUP_DATA,
        PID_HISTORY, PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS, PID_OWNERSHIP,
        PID_PARTICIPANT_GUID, PID_PARTITION, PID_PRESENTATION, PID_RELIABILITY,
        PID_RESOURCE_LIMITS, PID_TIME_BASED_FILTER, PID_TOPIC_DATA, PID_TOPIC_NAME,
        PID_TRANSPORT_PRIORITY, PID_TYPE_NAME, PID_TYPE_REPRESENTATION, PID_USER_DATA,
    },
    infrastructure::{
        qos::{DataReaderQos, DataWriterQos, PublisherQos, SubscriberQos, TopicQos},
        qos_policy::{
            DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
            HistoryQosPolicy, LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy,
            OwnershipQosPolicy, PartitionQosPolicy, PresentationQosPolicy, ReliabilityQosPolicy,
            ResourceLimitsQosPolicy, TimeBasedFilterQosPolicy, TopicDataQosPolicy,
            TransportPriorityQosPolicy, UserDataQosPolicy,
            DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
            DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
        },
    },
    serialized_payload::{
        cdr::{deserialize::CdrDeserialize, serialize::CdrSerialize},
        parameter_list::{
            deserialize::ParameterListDeserialize, serialize::ParameterListSerialize,
        },
    },
    topic_definition::type_support::DdsHasKey,
};

/// Structure representing the instance handle (or key) of an entity.
#[derive(Debug, PartialEq, Eq, Clone, CdrSerialize, CdrDeserialize, Default)]
pub struct BuiltInTopicKey {
    /// InstanceHandle value as an array of 16 octets.
    pub value: [u8; 16], // Originally in the DDS idl [i32;3]
}

impl From<[u8; 16]> for BuiltInTopicKey {
    fn from(value: [u8; 16]) -> Self {
        BuiltInTopicKey { value }
    }
}

impl From<BuiltInTopicKey> for [u8; 16] {
    fn from(value: BuiltInTopicKey) -> Self {
        value.value
    }
}

/// Structure representing a discovered [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant).
#[derive(
    Debug, PartialEq, Eq, Clone, ParameterListSerialize, ParameterListDeserialize, DdsDeserialize,
)]
#[dust_dds(format = "PL_CDR_LE")]
pub struct ParticipantBuiltinTopicData {
    #[parameter(id = PID_PARTICIPANT_GUID)]
    key: BuiltInTopicKey,
    #[parameter(id = PID_USER_DATA, default = Default::default())]
    user_data: UserDataQosPolicy,
}

impl ParticipantBuiltinTopicData {
    pub(crate) fn new(key: BuiltInTopicKey, user_data: UserDataQosPolicy) -> Self {
        Self { key, user_data }
    }

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
#[derive(
    Debug, PartialEq, Eq, Clone, ParameterListSerialize, ParameterListDeserialize, DdsDeserialize,
)]
#[dust_dds(format = "PL_CDR_LE")]
pub struct TopicBuiltinTopicData {
    #[parameter(id = PID_ENDPOINT_GUID)]
    key: BuiltInTopicKey,
    #[parameter(id = PID_TOPIC_NAME)]
    name: String,
    #[parameter(id = PID_TYPE_NAME)]
    type_name: String,
    #[parameter(id = PID_DURABILITY, default=Default::default())]
    durability: DurabilityQosPolicy,
    #[parameter(id = PID_DEADLINE, default=Default::default())]
    deadline: DeadlineQosPolicy,
    #[parameter(id = PID_LATENCY_BUDGET, default=Default::default())]
    latency_budget: LatencyBudgetQosPolicy,
    #[parameter(id = PID_LIVELINESS, default=Default::default())]
    liveliness: LivelinessQosPolicy,
    #[parameter(id = PID_RELIABILITY, default=DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS)]
    reliability: ReliabilityQosPolicy,
    #[parameter(id = PID_TRANSPORT_PRIORITY, default=Default::default())]
    transport_priority: TransportPriorityQosPolicy,
    #[parameter(id = PID_LIFESPAN, default=Default::default())]
    lifespan: LifespanQosPolicy,
    #[parameter(id = PID_DESTINATION_ORDER, default=Default::default())]
    destination_order: DestinationOrderQosPolicy,
    #[parameter(id = PID_HISTORY, default=Default::default())]
    history: HistoryQosPolicy,
    #[parameter(id = PID_RESOURCE_LIMITS, default=Default::default())]
    resource_limits: ResourceLimitsQosPolicy,
    #[parameter(id = PID_OWNERSHIP, default=Default::default())]
    ownership: OwnershipQosPolicy,
    #[parameter(id = PID_TOPIC_DATA, default=Default::default())]
    topic_data: TopicDataQosPolicy,
}

impl TopicBuiltinTopicData {
    pub(crate) fn new(
        key: BuiltInTopicKey,
        name: String,
        type_name: String,
        topic_qos: TopicQos,
    ) -> Self {
        Self {
            key,
            name,
            type_name,
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
        }
    }

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
}

impl DdsHasKey for TopicBuiltinTopicData {
    const HAS_KEY: bool = true;
}

/// Structure representing a discovered [`DataWriter`](crate::publication::data_writer::DataWriter).
#[derive(
    Debug, PartialEq, Eq, Clone, ParameterListSerialize, ParameterListDeserialize, DdsDeserialize,
)]
#[dust_dds(format = "PL_CDR_LE")]
pub struct PublicationBuiltinTopicData {
    #[parameter(id = PID_ENDPOINT_GUID)]
    key: BuiltInTopicKey,
    // Default value is a deviation from the standard and is used for interoperability reasons:
    #[parameter(id = PID_PARTICIPANT_GUID, default=BuiltInTopicKey::default())]
    participant_key: BuiltInTopicKey,
    #[parameter(id = PID_TOPIC_NAME)]
    topic_name: String,
    #[parameter(id = PID_TYPE_NAME)]
    type_name: String,
    #[parameter(id = PID_DURABILITY, default=Default::default())]
    durability: DurabilityQosPolicy,
    #[parameter(id = PID_DEADLINE, default=Default::default())]
    deadline: DeadlineQosPolicy,
    #[parameter(id = PID_LATENCY_BUDGET, default=Default::default())]
    latency_budget: LatencyBudgetQosPolicy,
    #[parameter(id = PID_LIVELINESS, default=Default::default())]
    liveliness: LivelinessQosPolicy,
    #[parameter(id = PID_RELIABILITY, default=DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER)]
    reliability: ReliabilityQosPolicy,
    #[parameter(id = PID_LIFESPAN, default=Default::default())]
    lifespan: LifespanQosPolicy,
    #[parameter(id = PID_USER_DATA, default=Default::default())]
    user_data: UserDataQosPolicy,
    #[parameter(id = PID_OWNERSHIP, default=Default::default())]
    ownership: OwnershipQosPolicy,
    #[parameter(id = PID_DESTINATION_ORDER, default=Default::default())]
    destination_order: DestinationOrderQosPolicy,
    #[parameter(id = PID_PRESENTATION, default=Default::default())]
    presentation: PresentationQosPolicy,
    #[parameter(id = PID_PARTITION, default=Default::default())]
    partition: PartitionQosPolicy,
    #[parameter(id = PID_TOPIC_DATA, default=Default::default())]
    topic_data: TopicDataQosPolicy,
    #[parameter(id = PID_GROUP_DATA, default=Default::default())]
    group_data: GroupDataQosPolicy,
    #[parameter(id = PID_TYPE_REPRESENTATION, default=Default::default())]
    xml_type: String,
}

impl PublicationBuiltinTopicData {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        key: BuiltInTopicKey,
        participant_key: BuiltInTopicKey,
        topic_name: String,
        type_name: String,
        data_writer_qos: DataWriterQos,
        publisher_qos: PublisherQos,
        topic_data: TopicDataQosPolicy,
        xml_type: String,
    ) -> Self {
        Self {
            key,
            participant_key,
            topic_name,
            type_name,
            durability: data_writer_qos.durability,
            deadline: data_writer_qos.deadline,
            latency_budget: data_writer_qos.latency_budget,
            liveliness: data_writer_qos.liveliness,
            reliability: data_writer_qos.reliability,
            lifespan: data_writer_qos.lifespan,
            user_data: data_writer_qos.user_data,
            ownership: data_writer_qos.ownership,
            destination_order: data_writer_qos.destination_order,
            presentation: publisher_qos.presentation,
            partition: publisher_qos.partition,
            group_data: publisher_qos.group_data,
            topic_data,
            xml_type,
        }
    }

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
}

impl DdsHasKey for PublicationBuiltinTopicData {
    const HAS_KEY: bool = true;
}

/// Structure representing a discovered [`DataReader`](crate::subscription::data_reader::DataReader).
#[derive(
    Debug, PartialEq, Eq, Clone, ParameterListSerialize, ParameterListDeserialize, DdsDeserialize,
)]
#[dust_dds(format = "PL_CDR_LE")]
pub struct SubscriptionBuiltinTopicData {
    #[parameter(id = PID_ENDPOINT_GUID)]
    key: BuiltInTopicKey,
    // Default value is a deviation from the standard and is used for interoperability reasons:
    #[parameter(id = PID_PARTICIPANT_GUID, default = BuiltInTopicKey::default())]
    participant_key: BuiltInTopicKey, //ParameterWithDefault<PID_PARTICIPANT_GUID, BuiltInTopicKey>,
    #[parameter(id = PID_TOPIC_NAME)]
    topic_name: String,
    #[parameter(id = PID_TYPE_NAME)]
    type_name: String,
    #[parameter(id = PID_DURABILITY, default = Default::default())]
    durability: DurabilityQosPolicy,
    #[parameter(id = PID_DEADLINE, default = Default::default())]
    deadline: DeadlineQosPolicy,
    #[parameter(id = PID_LATENCY_BUDGET, default = Default::default())]
    latency_budget: LatencyBudgetQosPolicy,
    #[parameter(id = PID_LIVELINESS, default = Default::default())]
    liveliness: LivelinessQosPolicy,
    #[parameter(id = PID_RELIABILITY, default = DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS)]
    reliability: ReliabilityQosPolicy,
    #[parameter(id = PID_OWNERSHIP, default = Default::default())]
    ownership: OwnershipQosPolicy,
    #[parameter(id = PID_DESTINATION_ORDER, default = Default::default())]
    destination_order: DestinationOrderQosPolicy,
    #[parameter(id = PID_USER_DATA, default = Default::default())]
    user_data: UserDataQosPolicy,
    #[parameter(id = PID_TIME_BASED_FILTER, default = Default::default())]
    time_based_filter: TimeBasedFilterQosPolicy,
    #[parameter(id = PID_PRESENTATION, default = Default::default())]
    presentation: PresentationQosPolicy,
    #[parameter(id = PID_PARTITION, default = Default::default())]
    partition: PartitionQosPolicy,
    #[parameter(id = PID_TOPIC_DATA, default = Default::default())]
    topic_data: TopicDataQosPolicy,
    #[parameter(id = PID_GROUP_DATA, default = Default::default())]
    group_data: GroupDataQosPolicy,
    #[parameter(id = PID_TYPE_REPRESENTATION, default=Default::default())]
    xml_type: String,
}

impl SubscriptionBuiltinTopicData {
    #[allow(clippy::too_many_arguments)]
    /// Construct a new SubscriptionBuiltinTopicData
    pub fn new(
        key: BuiltInTopicKey,
        participant_key: BuiltInTopicKey,
        topic_name: String,
        type_name: String,
        data_reader_qos: DataReaderQos,
        subscriber_qos: SubscriberQos,
        topic_data: TopicDataQosPolicy,
        xml_type: String,
    ) -> Self {
        Self {
            key,
            participant_key,
            topic_name,
            type_name,
            durability: data_reader_qos.durability,
            deadline: data_reader_qos.deadline,
            latency_budget: data_reader_qos.latency_budget,
            liveliness: data_reader_qos.liveliness,
            reliability: data_reader_qos.reliability,
            ownership: data_reader_qos.ownership,
            destination_order: data_reader_qos.destination_order,
            user_data: data_reader_qos.user_data,
            time_based_filter: data_reader_qos.time_based_filter,
            presentation: subscriber_qos.presentation,
            partition: subscriber_qos.partition,
            group_data: subscriber_qos.group_data,
            topic_data,
            xml_type,
        }
    }

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
}

impl DdsHasKey for SubscriptionBuiltinTopicData {
    const HAS_KEY: bool = true;
}
