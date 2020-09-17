use crate::dds::types::Duration;
use crate::dds::infrastructure::qos_policy:: {
    TopicDataQosPolicy,
    DurabilityQosPolicy,
    DurabilityServiceQosPolicy,
    DeadlineQosPolicy,
    LatencyBudgetQosPolicy,
    LivelinessQosPolicy,
    ReliabilityQosPolicy,
    DestinationOrderQosPolicy,
    HistoryQosPolicy,
    ResourceLimitsQosPolicy,
    TransportPriorityQosPolicy,
    LifespanQosPolicy,
    OwnershipQosPolicy,
    ReliabilityQosPolicyKind,
};
#[derive(Debug)]
pub struct TopicQos {
    topic_data: TopicDataQosPolicy,
    durability: DurabilityQosPolicy,
    durability_service: DurabilityServiceQosPolicy,
    deadline: DeadlineQosPolicy,
    latency_budget: LatencyBudgetQosPolicy,
    liveliness: LivelinessQosPolicy,
    reliability: ReliabilityQosPolicy,
    destination_order: DestinationOrderQosPolicy,
    history: HistoryQosPolicy,
    resource_limits: ResourceLimitsQosPolicy,
    transport_priority: TransportPriorityQosPolicy,
    lifespan: LifespanQosPolicy,
    ownership: OwnershipQosPolicy,
}

impl Default for TopicQos {
    fn default() -> Self {
        Self {
            reliability: ReliabilityQosPolicy{kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos, max_blocking_time: Duration{sec: 0, nanosec: 100000000 /*100ms*/}},
            topic_data: TopicDataQosPolicy::default(),
            durability: DurabilityQosPolicy::default(),
            durability_service: DurabilityServiceQosPolicy::default(),
            deadline: DeadlineQosPolicy::default(),
            latency_budget: LatencyBudgetQosPolicy::default(), 
            liveliness: LivelinessQosPolicy::default(),
            destination_order: DestinationOrderQosPolicy::default(),
            history: HistoryQosPolicy::default(),
            resource_limits: ResourceLimitsQosPolicy::default(),
            transport_priority: TransportPriorityQosPolicy::default(),
            lifespan: LifespanQosPolicy::default(),
            ownership: OwnershipQosPolicy::default(),
        }
    }
}