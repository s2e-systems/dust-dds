use crate::{
    implementation::{
        data_representation_builtin_endpoints::parameter_id_values::{
            PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY, PID_ENDPOINT_GUID, PID_GROUP_DATA,
            PID_HISTORY, PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS, PID_OWNERSHIP,
            PID_PARTICIPANT_GUID, PID_PARTITION, PID_PRESENTATION, PID_RELIABILITY,
            PID_RESOURCE_LIMITS, PID_TIME_BASED_FILTER, PID_TOPIC_DATA, PID_TOPIC_NAME,
            PID_TRANSPORT_PRIORITY, PID_TYPE_NAME, PID_USER_DATA,
        },
        parameter_list_serde::parameter::{Parameter, ParameterWithDefault},
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
    topic_definition::type_support::DdsType,
};

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    derive_more::From,
    derive_more::Into,
)]
pub struct ReliabilityQosPolicyTopics(ReliabilityQosPolicy);
impl Default for ReliabilityQosPolicyTopics {
    fn default() -> Self {
        Self(DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize, Default)]
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

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParticipantBuiltinTopicData {
    key: Parameter<PID_PARTICIPANT_GUID, BuiltInTopicKey>,
    user_data: ParameterWithDefault<PID_USER_DATA, UserDataQosPolicy>,
}

impl ParticipantBuiltinTopicData {
    pub fn new(key: BuiltInTopicKey, user_data: UserDataQosPolicy) -> Self {
        Self {
            key: key.into(),
            user_data: user_data.into(),
        }
    }

    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key.0
    }

    pub fn user_data(&self) -> &UserDataQosPolicy {
        &self.user_data.0
    }
}

impl DdsType for ParticipantBuiltinTopicData {
    fn type_name() -> &'static str {
        "ParticipantBuiltinTopicData"
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct TopicBuiltinTopicData {
    key: Parameter<PID_ENDPOINT_GUID, BuiltInTopicKey>,
    name: Parameter<PID_TOPIC_NAME, String>,
    type_name: Parameter<PID_TYPE_NAME, String>,
    durability: ParameterWithDefault<PID_DURABILITY, DurabilityQosPolicy>,
    deadline: ParameterWithDefault<PID_DEADLINE, DeadlineQosPolicy>,
    latency_budget: ParameterWithDefault<PID_LATENCY_BUDGET, LatencyBudgetQosPolicy>,
    liveliness: ParameterWithDefault<PID_LIVELINESS, LivelinessQosPolicy>,
    reliability: ParameterWithDefault<PID_RELIABILITY, ReliabilityQosPolicyTopics>,
    transport_priority: ParameterWithDefault<PID_TRANSPORT_PRIORITY, TransportPriorityQosPolicy>,
    lifespan: ParameterWithDefault<PID_LIFESPAN, LifespanQosPolicy>,
    destination_order: ParameterWithDefault<PID_DESTINATION_ORDER, DestinationOrderQosPolicy>,
    history: ParameterWithDefault<PID_HISTORY, HistoryQosPolicy>,
    resource_limits: ParameterWithDefault<PID_RESOURCE_LIMITS, ResourceLimitsQosPolicy>,
    ownership: ParameterWithDefault<PID_OWNERSHIP, OwnershipQosPolicy>,
    topic_data: ParameterWithDefault<PID_TOPIC_DATA, TopicDataQosPolicy>,
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
            key: key.into(),
            name: name.into(),
            type_name: type_name.into(),
            durability: durability.into(),
            deadline: deadline.into(),
            latency_budget: latency_budget.into(),
            liveliness: liveliness.into(),
            reliability: ReliabilityQosPolicyTopics::from(reliability).into(),
            transport_priority: transport_priority.into(),
            lifespan: lifespan.into(),
            destination_order: destination_order.into(),
            history: history.into(),
            resource_limits: resource_limits.into(),
            ownership: ownership.into(),
            topic_data: topic_data.into(),
        }
    }

    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key.0
    }

    pub fn name(&self) -> &str {
        self.name.0.as_ref()
    }

    pub fn get_type_name(&self) -> &str {
        self.type_name.0.as_ref()
    }

    pub fn durability(&self) -> &DurabilityQosPolicy {
        &self.durability.0
    }

    pub fn deadline(&self) -> &DeadlineQosPolicy {
        &self.deadline.0
    }

    pub fn latency_budget(&self) -> &LatencyBudgetQosPolicy {
        &self.latency_budget.0
    }

    pub fn liveliness(&self) -> &LivelinessQosPolicy {
        &self.liveliness.0
    }

    pub fn reliability(&self) -> &ReliabilityQosPolicy {
        &self.reliability.0 .0
    }

    pub fn transport_priority(&self) -> &TransportPriorityQosPolicy {
        &self.transport_priority.0
    }

    pub fn lifespan(&self) -> &LifespanQosPolicy {
        &self.lifespan.0
    }

    pub fn destination_order(&self) -> &DestinationOrderQosPolicy {
        &self.destination_order.0
    }

    pub fn history(&self) -> &HistoryQosPolicy {
        &self.history.0
    }

    pub fn resource_limits(&self) -> &ResourceLimitsQosPolicy {
        &self.resource_limits.0
    }

    pub fn ownership(&self) -> &OwnershipQosPolicy {
        &self.ownership.0
    }

    pub fn topic_data(&self) -> &TopicDataQosPolicy {
        &self.topic_data.0
    }
}

impl DdsType for TopicBuiltinTopicData {
    fn type_name() -> &'static str {
        "TopicBuiltinTopicData"
    }
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    derive_more::Into,
    derive_more::From,
)]
struct ReliabilityQosPolicyDataWriter(ReliabilityQosPolicy);
impl Default for ReliabilityQosPolicyDataWriter {
    fn default() -> Self {
        Self(DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER)
    }
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    derive_more::Into,
    derive_more::From,
)]
struct ReliabilityQosPolicyDataReader(ReliabilityQosPolicy);
impl Default for ReliabilityQosPolicyDataReader {
    fn default() -> Self {
        Self(DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct PublicationBuiltinTopicData {
    key: Parameter<PID_ENDPOINT_GUID, BuiltInTopicKey>,
    // Default value is a deviation from the standard and is used for interoperability reasons:
    participant_key: ParameterWithDefault<PID_PARTICIPANT_GUID, BuiltInTopicKey>,
    topic_name: Parameter<PID_TOPIC_NAME, String>,
    type_name: Parameter<PID_TYPE_NAME, String>,
    durability: ParameterWithDefault<PID_DURABILITY, DurabilityQosPolicy>,
    deadline: ParameterWithDefault<PID_DEADLINE, DeadlineQosPolicy>,
    latency_budget: ParameterWithDefault<PID_LATENCY_BUDGET, LatencyBudgetQosPolicy>,
    liveliness: ParameterWithDefault<PID_LIVELINESS, LivelinessQosPolicy>,
    reliability: ParameterWithDefault<PID_RELIABILITY, ReliabilityQosPolicyDataWriter>,
    lifespan: ParameterWithDefault<PID_LIFESPAN, LifespanQosPolicy>,
    user_data: ParameterWithDefault<PID_USER_DATA, UserDataQosPolicy>,
    ownership: ParameterWithDefault<PID_OWNERSHIP, OwnershipQosPolicy>,
    destination_order: ParameterWithDefault<PID_DESTINATION_ORDER, DestinationOrderQosPolicy>,
    presentation: ParameterWithDefault<PID_PRESENTATION, PresentationQosPolicy>,
    partition: ParameterWithDefault<PID_PARTITION, PartitionQosPolicy>,
    topic_data: ParameterWithDefault<PID_TOPIC_DATA, TopicDataQosPolicy>,
    group_data: ParameterWithDefault<PID_GROUP_DATA, GroupDataQosPolicy>,
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
            key: key.into(),
            participant_key: participant_key.into(),
            topic_name: topic_name.into(),
            type_name: type_name.into(),
            durability: durability.into(),
            deadline: deadline.into(),
            latency_budget: latency_budget.into(),
            liveliness: liveliness.into(),
            reliability: ReliabilityQosPolicyDataWriter::from(reliability).into(),
            lifespan: lifespan.into(),
            user_data: user_data.into(),
            ownership: ownership.into(),
            destination_order: destination_order.into(),
            presentation: presentation.into(),
            partition: partition.into(),
            topic_data: topic_data.into(),
            group_data: group_data.into(),
        }
    }

    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key.0
    }

    pub fn participant_key(&self) -> &BuiltInTopicKey {
        &self.participant_key.0
    }

    pub fn topic_name(&self) -> &str {
        self.topic_name.0.as_ref()
    }

    pub fn get_type_name(&self) -> &str {
        self.type_name.0.as_ref()
    }

    pub fn durability(&self) -> &DurabilityQosPolicy {
        &self.durability.0
    }

    pub fn deadline(&self) -> &DeadlineQosPolicy {
        &self.deadline.0
    }

    pub fn latency_budget(&self) -> &LatencyBudgetQosPolicy {
        &self.latency_budget.0
    }

    pub fn liveliness(&self) -> &LivelinessQosPolicy {
        &self.liveliness.0
    }

    pub fn reliability(&self) -> &ReliabilityQosPolicy {
        &self.reliability.0 .0
    }

    pub fn lifespan(&self) -> &LifespanQosPolicy {
        &self.lifespan.0
    }

    pub fn user_data(&self) -> &UserDataQosPolicy {
        &self.user_data.0
    }

    pub fn ownership(&self) -> &OwnershipQosPolicy {
        &self.ownership.0
    }

    pub fn destination_order(&self) -> &DestinationOrderQosPolicy {
        &self.destination_order.0
    }

    pub fn presentation(&self) -> &PresentationQosPolicy {
        &self.presentation.0
    }

    pub fn partition(&self) -> &PartitionQosPolicy {
        &self.partition.0
    }

    pub fn topic_data(&self) -> &TopicDataQosPolicy {
        &self.topic_data.0
    }

    pub fn group_data(&self) -> &GroupDataQosPolicy {
        &self.group_data.0
    }
}

impl DdsType for PublicationBuiltinTopicData {
    fn type_name() -> &'static str {
        "PublicationBuiltinTopicData"
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubscriptionBuiltinTopicData {
    key: Parameter<PID_ENDPOINT_GUID, BuiltInTopicKey>,
    // Default value is a deviation from the standard and is used for interoperability reasons:
    participant_key: ParameterWithDefault<PID_PARTICIPANT_GUID, BuiltInTopicKey>,
    topic_name: Parameter<PID_TOPIC_NAME, String>,
    type_name: Parameter<PID_TYPE_NAME, String>,

    durability: ParameterWithDefault<PID_DURABILITY, DurabilityQosPolicy>,
    deadline: ParameterWithDefault<PID_DEADLINE, DeadlineQosPolicy>,
    latency_budget: ParameterWithDefault<PID_LATENCY_BUDGET, LatencyBudgetQosPolicy>,
    liveliness: ParameterWithDefault<PID_LIVELINESS, LivelinessQosPolicy>,
    reliability: ParameterWithDefault<PID_RELIABILITY, ReliabilityQosPolicyDataReader>,
    ownership: ParameterWithDefault<PID_OWNERSHIP, OwnershipQosPolicy>,
    destination_order: ParameterWithDefault<PID_DESTINATION_ORDER, DestinationOrderQosPolicy>,
    user_data: ParameterWithDefault<PID_USER_DATA, UserDataQosPolicy>,
    time_based_filter: ParameterWithDefault<PID_TIME_BASED_FILTER, TimeBasedFilterQosPolicy>,

    presentation: ParameterWithDefault<PID_PRESENTATION, PresentationQosPolicy>,
    partition: ParameterWithDefault<PID_PARTITION, PartitionQosPolicy>,
    topic_data: ParameterWithDefault<PID_TOPIC_DATA, TopicDataQosPolicy>,
    group_data: ParameterWithDefault<PID_GROUP_DATA, GroupDataQosPolicy>,
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
            key: Parameter(key),
            participant_key: participant_key.into(),
            topic_name: topic_name.into(),
            type_name: type_name.into(),
            durability: durability.into(),
            deadline: deadline.into(),
            latency_budget: latency_budget.into(),
            liveliness: liveliness.into(),
            reliability: ReliabilityQosPolicyDataReader::from(reliability).into(),
            ownership: ownership.into(),
            destination_order: destination_order.into(),
            user_data: user_data.into(),
            time_based_filter: time_based_filter.into(),
            presentation: presentation.into(),
            partition: partition.into(),
            topic_data: topic_data.into(),
            group_data: group_data.into(),
        }
    }

    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key.0
    }

    pub fn participant_key(&self) -> &BuiltInTopicKey {
        &self.participant_key.0
    }

    pub fn topic_name(&self) -> &str {
        self.topic_name.0.as_ref()
    }

    pub fn get_type_name(&self) -> &str {
        self.type_name.0.as_ref()
    }

    pub fn durability(&self) -> &DurabilityQosPolicy {
        &self.durability.0
    }

    pub fn deadline(&self) -> &DeadlineQosPolicy {
        &self.deadline.0
    }

    pub fn latency_budget(&self) -> &LatencyBudgetQosPolicy {
        &self.latency_budget.0
    }

    pub fn liveliness(&self) -> &LivelinessQosPolicy {
        &self.liveliness.0
    }

    pub fn reliability(&self) -> &ReliabilityQosPolicy {
        &self.reliability.0 .0
    }

    pub fn ownership(&self) -> &OwnershipQosPolicy {
        &self.ownership.0
    }

    pub fn destination_order(&self) -> &DestinationOrderQosPolicy {
        &self.destination_order.0
    }

    pub fn user_data(&self) -> &UserDataQosPolicy {
        &self.user_data.0
    }

    pub fn time_based_filter(&self) -> &TimeBasedFilterQosPolicy {
        &self.time_based_filter.0
    }

    pub fn presentation(&self) -> &PresentationQosPolicy {
        &self.presentation.0
    }

    pub fn partition(&self) -> &PartitionQosPolicy {
        &self.partition.0
    }

    pub fn topic_data(&self) -> &TopicDataQosPolicy {
        &self.topic_data.0
    }

    pub fn group_data(&self) -> &GroupDataQosPolicy {
        &self.group_data.0
    }
}

impl DdsType for SubscriptionBuiltinTopicData {
    fn type_name() -> &'static str {
        "SubscriptionBuiltinTopicData"
    }
}
