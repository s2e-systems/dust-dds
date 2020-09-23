use crate::types::{Duration, LENGTH_UNLIMITED};
use crate::infrastructure::qos_policy:: {
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
    HistoryQosPolicyKind,
};
#[derive(Debug, PartialEq, Clone)]
pub struct TopicQos {
    pub topic_data: TopicDataQosPolicy,
    pub durability: DurabilityQosPolicy,
    pub durability_service: DurabilityServiceQosPolicy,
    pub deadline: DeadlineQosPolicy,
    pub latency_budget: LatencyBudgetQosPolicy,
    pub liveliness: LivelinessQosPolicy,
    pub reliability: ReliabilityQosPolicy,
    pub destination_order: DestinationOrderQosPolicy,
    pub history: HistoryQosPolicy,
    pub resource_limits: ResourceLimitsQosPolicy,
    pub transport_priority: TransportPriorityQosPolicy,
    pub lifespan: LifespanQosPolicy,
    pub ownership: OwnershipQosPolicy,
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

impl TopicQos {
    pub fn is_consistent(&self) -> bool {
        // The setting of RESOURCE_LIMITS max_samples must be consistent with the max_samples_per_instance. For these two
        // values to be consistent they must verify that “max_samples >= max_samples_per_instance.”
        if  self.resource_limits.max_samples_per_instance != LENGTH_UNLIMITED {
            if self.resource_limits.max_samples == LENGTH_UNLIMITED || self.resource_limits.max_samples < self.resource_limits.max_samples_per_instance {
                return false
            }
        }

        // The setting of RESOURCE_LIMITS max_samples_per_instance must be consistent with the HISTORY depth. For these two
        // QoS to be consistent, they must verify that “depth <= max_samples_per_instance.”
        if self.history.kind == HistoryQosPolicyKind::KeepLastHistoryQoS && self.resource_limits.max_samples_per_instance != LENGTH_UNLIMITED {
            if self.history.depth == LENGTH_UNLIMITED || self.history.depth > self.resource_limits.max_samples_per_instance {
                return false
            }
        }

        true
    }
}