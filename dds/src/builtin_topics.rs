use crate::infrastructure::qos_policy::{
    DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
    HistoryQosPolicy, LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy,
    OwnershipQosPolicy, PartitionQosPolicy, PresentationQosPolicy, ReliabilityQosPolicy,
    ResourceLimitsQosPolicy, TimeBasedFilterQosPolicy, TopicDataQosPolicy,
    TransportPriorityQosPolicy, UserDataQosPolicy,
};

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct BuiltInTopicKey {
    pub value: [u8; 16], // Originally in the DDS idl [i32;3]
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParticipantBuiltinTopicData {
    pub key: BuiltInTopicKey,
    pub user_data: UserDataQosPolicy,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TopicBuiltinTopicData {
    pub key: BuiltInTopicKey,
    pub name: String,
    pub type_name: String,
    pub durability: DurabilityQosPolicy,
    pub deadline: DeadlineQosPolicy,
    pub latency_budget: LatencyBudgetQosPolicy,
    pub liveliness: LivelinessQosPolicy,
    pub reliability: ReliabilityQosPolicy,
    pub transport_priority: TransportPriorityQosPolicy,
    pub lifespan: LifespanQosPolicy,
    pub destination_order: DestinationOrderQosPolicy,
    pub history: HistoryQosPolicy,
    pub resource_limits: ResourceLimitsQosPolicy,
    pub ownership: OwnershipQosPolicy,
    pub topic_data: TopicDataQosPolicy,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PublicationBuiltinTopicData {
    pub key: BuiltInTopicKey,
    pub participant_key: BuiltInTopicKey,
    pub topic_name: String,
    pub type_name: String,

    pub durability: DurabilityQosPolicy,
    pub deadline: DeadlineQosPolicy,
    pub latency_budget: LatencyBudgetQosPolicy,
    pub liveliness: LivelinessQosPolicy,
    pub reliability: ReliabilityQosPolicy,
    pub lifespan: LifespanQosPolicy,
    pub user_data: UserDataQosPolicy,
    pub ownership: OwnershipQosPolicy,
    pub destination_order: DestinationOrderQosPolicy,

    pub presentation: PresentationQosPolicy,
    pub partition: PartitionQosPolicy,
    pub topic_data: TopicDataQosPolicy,
    pub group_data: GroupDataQosPolicy,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SubscriptionBuiltinTopicData {
    pub key: BuiltInTopicKey,
    pub participant_key: BuiltInTopicKey,
    pub topic_name: String,
    pub type_name: String,

    pub durability: DurabilityQosPolicy,
    pub deadline: DeadlineQosPolicy,
    pub latency_budget: LatencyBudgetQosPolicy,
    pub liveliness: LivelinessQosPolicy,
    pub reliability: ReliabilityQosPolicy,
    pub ownership: OwnershipQosPolicy,
    pub destination_order: DestinationOrderQosPolicy,
    pub user_data: UserDataQosPolicy,
    pub time_based_filter: TimeBasedFilterQosPolicy,

    pub presentation: PresentationQosPolicy,
    pub partition: PartitionQosPolicy,
    pub topic_data: TopicDataQosPolicy,
    pub group_data: GroupDataQosPolicy,
}
