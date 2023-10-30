use dust_dds_derive::{ParameterListDeserialize, ParameterListSerialize};

use crate::{
    cdr::{
        deserialize::CdrDeserialize, parameter_list_deserialize::ParameterListDeserialize,
        parameter_list_deserializer::ParameterListDeserializer, serialize::CdrSerialize,
    },
    implementation::data_representation_builtin_endpoints::parameter_id_values::{
        PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY, PID_ENDPOINT_GUID, PID_GROUP_DATA,
        PID_HISTORY, PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS, PID_OWNERSHIP,
        PID_PARTICIPANT_GUID, PID_PARTITION, PID_PRESENTATION, PID_RELIABILITY,
        PID_RESOURCE_LIMITS, PID_TIME_BASED_FILTER, PID_TOPIC_DATA, PID_TOPIC_NAME,
        PID_TRANSPORT_PRIORITY, PID_TYPE_NAME, PID_USER_DATA,
    },
    infrastructure::qos_policy::{
        DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
        HistoryQosPolicy, LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy,
        OwnershipQosPolicy, PartitionQosPolicy, PresentationQosPolicy, ReliabilityQosPolicy,
        ResourceLimitsQosPolicy, TimeBasedFilterQosPolicy, TopicDataQosPolicy,
        TransportPriorityQosPolicy, UserDataQosPolicy,
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
    },
    topic_definition::type_support::DdsHasKey,
};

#[derive(Debug, PartialEq, Eq, Clone, CdrSerialize, CdrDeserialize, Default)]
pub struct BuiltInTopicKey {
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

#[derive(Debug, PartialEq, Eq, Clone, CdrSerialize, CdrDeserialize)]
pub struct ParticipantBuiltinTopicData {
    key: BuiltInTopicKey,
    user_data: UserDataQosPolicy,
}

impl<'de> ParameterListDeserialize<'de> for ParticipantBuiltinTopicData {
    fn deserialize(
        pl_deserializer: &mut ParameterListDeserializer<'de>,
    ) -> Result<Self, std::io::Error> {
        Ok(Self {
            key: pl_deserializer.read(PID_PARTICIPANT_GUID)?,
            user_data: pl_deserializer.read_with_default(PID_USER_DATA, Default::default())?,
        })
    }
}

impl ParticipantBuiltinTopicData {
    pub fn new(key: BuiltInTopicKey, user_data: UserDataQosPolicy) -> Self {
        Self { key, user_data }
    }

    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key
    }

    pub fn user_data(&self) -> &UserDataQosPolicy {
        &self.user_data
    }
}

impl DdsHasKey for ParticipantBuiltinTopicData {
    const HAS_KEY: bool = true;
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    ParameterListSerialize,
    ParameterListDeserialize,
    CdrSerialize,
    CdrDeserialize,
)]
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        key: BuiltInTopicKey,
        name: String,
        type_name: String,
        durability: DurabilityQosPolicy,
        deadline: DeadlineQosPolicy,
        latency_budget: LatencyBudgetQosPolicy,
        liveliness: LivelinessQosPolicy,
        reliability: ReliabilityQosPolicy,
        transport_priority: TransportPriorityQosPolicy,
        lifespan: LifespanQosPolicy,
        destination_order: DestinationOrderQosPolicy,
        history: HistoryQosPolicy,
        resource_limits: ResourceLimitsQosPolicy,
        ownership: OwnershipQosPolicy,
        topic_data: TopicDataQosPolicy,
    ) -> Self {
        Self {
            key,
            name,
            type_name,
            durability,
            deadline,
            latency_budget,
            liveliness,
            reliability,
            transport_priority,
            lifespan,
            destination_order,
            history,
            resource_limits,
            ownership,
            topic_data,
        }
    }

    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_type_name(&self) -> &str {
        &self.type_name
    }

    pub fn durability(&self) -> &DurabilityQosPolicy {
        &self.durability
    }

    pub fn deadline(&self) -> &DeadlineQosPolicy {
        &self.deadline
    }

    pub fn latency_budget(&self) -> &LatencyBudgetQosPolicy {
        &self.latency_budget
    }

    pub fn liveliness(&self) -> &LivelinessQosPolicy {
        &self.liveliness
    }

    pub fn reliability(&self) -> &ReliabilityQosPolicy {
        &self.reliability
    }

    pub fn transport_priority(&self) -> &TransportPriorityQosPolicy {
        &self.transport_priority
    }

    pub fn lifespan(&self) -> &LifespanQosPolicy {
        &self.lifespan
    }

    pub fn destination_order(&self) -> &DestinationOrderQosPolicy {
        &self.destination_order
    }

    pub fn history(&self) -> &HistoryQosPolicy {
        &self.history
    }

    pub fn resource_limits(&self) -> &ResourceLimitsQosPolicy {
        &self.resource_limits
    }

    pub fn ownership(&self) -> &OwnershipQosPolicy {
        &self.ownership
    }

    pub fn topic_data(&self) -> &TopicDataQosPolicy {
        &self.topic_data
    }
}

impl DdsHasKey for TopicBuiltinTopicData {
    const HAS_KEY: bool = true;
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    ParameterListSerialize,
    ParameterListDeserialize,
    CdrSerialize,
    CdrDeserialize,
)]
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
}

impl PublicationBuiltinTopicData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        key: BuiltInTopicKey,
        participant_key: BuiltInTopicKey,
        topic_name: String,
        type_name: String,
        durability: DurabilityQosPolicy,
        deadline: DeadlineQosPolicy,
        latency_budget: LatencyBudgetQosPolicy,
        liveliness: LivelinessQosPolicy,
        reliability: ReliabilityQosPolicy,
        lifespan: LifespanQosPolicy,
        user_data: UserDataQosPolicy,
        ownership: OwnershipQosPolicy,
        destination_order: DestinationOrderQosPolicy,
        presentation: PresentationQosPolicy,
        partition: PartitionQosPolicy,
        topic_data: TopicDataQosPolicy,
        group_data: GroupDataQosPolicy,
    ) -> Self {
        Self {
            key,
            participant_key,
            topic_name,
            type_name,
            durability,
            deadline,
            latency_budget,
            liveliness,
            reliability,
            lifespan,
            user_data,
            ownership,
            destination_order,
            presentation,
            partition,
            topic_data,
            group_data,
        }
    }

    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key
    }

    pub fn participant_key(&self) -> &BuiltInTopicKey {
        &self.participant_key
    }

    pub fn topic_name(&self) -> &str {
        &self.topic_name
    }

    pub fn get_type_name(&self) -> &str {
        &self.type_name
    }

    pub fn durability(&self) -> &DurabilityQosPolicy {
        &self.durability
    }

    pub fn deadline(&self) -> &DeadlineQosPolicy {
        &self.deadline
    }

    pub fn latency_budget(&self) -> &LatencyBudgetQosPolicy {
        &self.latency_budget
    }

    pub fn liveliness(&self) -> &LivelinessQosPolicy {
        &self.liveliness
    }

    pub fn reliability(&self) -> &ReliabilityQosPolicy {
        &self.reliability
    }

    pub fn lifespan(&self) -> &LifespanQosPolicy {
        &self.lifespan
    }

    pub fn user_data(&self) -> &UserDataQosPolicy {
        &self.user_data
    }

    pub fn ownership(&self) -> &OwnershipQosPolicy {
        &self.ownership
    }

    pub fn destination_order(&self) -> &DestinationOrderQosPolicy {
        &self.destination_order
    }

    pub fn presentation(&self) -> &PresentationQosPolicy {
        &self.presentation
    }

    pub fn partition(&self) -> &PartitionQosPolicy {
        &self.partition
    }

    pub fn topic_data(&self) -> &TopicDataQosPolicy {
        &self.topic_data
    }

    pub fn group_data(&self) -> &GroupDataQosPolicy {
        &self.group_data
    }
}

impl DdsHasKey for PublicationBuiltinTopicData {
    const HAS_KEY: bool = true;
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    ParameterListSerialize,
    ParameterListDeserialize,
    CdrSerialize,
    CdrDeserialize,
)]
pub struct SubscriptionBuiltinTopicData {
    #[parameter(id = PID_ENDPOINT_GUID)]
    key: BuiltInTopicKey,
    // Default value is a deviation from the standard and is used for interoperability reasons:
    #[parameter(id = PID_ENDPOINT_GUID, default = BuiltInTopicKey::default())]
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
}

impl SubscriptionBuiltinTopicData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        key: BuiltInTopicKey,
        participant_key: BuiltInTopicKey,
        topic_name: String,
        type_name: String,
        durability: DurabilityQosPolicy,
        deadline: DeadlineQosPolicy,
        latency_budget: LatencyBudgetQosPolicy,
        liveliness: LivelinessQosPolicy,
        reliability: ReliabilityQosPolicy,
        ownership: OwnershipQosPolicy,
        destination_order: DestinationOrderQosPolicy,
        user_data: UserDataQosPolicy,
        time_based_filter: TimeBasedFilterQosPolicy,
        presentation: PresentationQosPolicy,
        partition: PartitionQosPolicy,
        topic_data: TopicDataQosPolicy,
        group_data: GroupDataQosPolicy,
    ) -> Self {
        Self {
            key,
            participant_key,
            topic_name,
            type_name,
            durability,
            deadline,
            latency_budget,
            liveliness,
            reliability,
            ownership,
            destination_order,
            user_data,
            time_based_filter,
            presentation,
            partition,
            topic_data,
            group_data,
        }
    }

    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key
    }

    pub fn participant_key(&self) -> &BuiltInTopicKey {
        &self.participant_key
    }

    pub fn topic_name(&self) -> &str {
        &self.topic_name
    }

    pub fn get_type_name(&self) -> &str {
        &self.type_name
    }

    pub fn durability(&self) -> &DurabilityQosPolicy {
        &self.durability
    }

    pub fn deadline(&self) -> &DeadlineQosPolicy {
        &self.deadline
    }

    pub fn latency_budget(&self) -> &LatencyBudgetQosPolicy {
        &self.latency_budget
    }

    pub fn liveliness(&self) -> &LivelinessQosPolicy {
        &self.liveliness
    }

    pub fn reliability(&self) -> &ReliabilityQosPolicy {
        &self.reliability
    }

    pub fn ownership(&self) -> &OwnershipQosPolicy {
        &self.ownership
    }

    pub fn destination_order(&self) -> &DestinationOrderQosPolicy {
        &self.destination_order
    }

    pub fn user_data(&self) -> &UserDataQosPolicy {
        &self.user_data
    }

    pub fn time_based_filter(&self) -> &TimeBasedFilterQosPolicy {
        &self.time_based_filter
    }

    pub fn presentation(&self) -> &PresentationQosPolicy {
        &self.presentation
    }

    pub fn partition(&self) -> &PartitionQosPolicy {
        &self.partition
    }

    pub fn topic_data(&self) -> &TopicDataQosPolicy {
        &self.topic_data
    }

    pub fn group_data(&self) -> &GroupDataQosPolicy {
        &self.group_data
    }
}

impl DdsHasKey for SubscriptionBuiltinTopicData {
    const HAS_KEY: bool = true;
}
