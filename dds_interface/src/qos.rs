use crate::types::{Duration, LENGTH_UNLIMITED};
use crate::qos_policy::{
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
    UserDataQosPolicy,
    OwnershipQosPolicy,
    OwnershipStrengthQosPolicy,
    WriterDataLifecycleQosPolicy,
    ReliabilityQosPolicyKind,
    HistoryQosPolicyKind,
    EntityFactoryQosPolicy,
    PresentationQosPolicy,
    PartitionQosPolicy,
    GroupDataQosPolicy,
    TimeBasedFilterQosPolicy,
    ReaderDataLifecycleQosPolicy,
    TopicDataQosPolicy,
};

pub trait QosList: Default {
    fn is_consistent(&self) -> bool;
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct DomainParticipantQos {
    pub user_data: UserDataQosPolicy,
    pub entity_factory: EntityFactoryQosPolicy,
}

impl QosList for DomainParticipantQos {
    fn is_consistent(&self) -> bool {
        true
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PublisherQos {
    pub presentation: PresentationQosPolicy,
    pub partition: PartitionQosPolicy,
    pub group_data: GroupDataQosPolicy,
    pub entity_factory: EntityFactoryQosPolicy,
}

impl QosList for PublisherQos {
    fn is_consistent(&self) -> bool {
        true
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct DataWriterQos {
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
    pub user_data: UserDataQosPolicy,
    pub ownership: OwnershipQosPolicy,
    pub ownership_strength: OwnershipStrengthQosPolicy,
    pub writer_data_lifecycle: WriterDataLifecycleQosPolicy,
}

impl Default for DataWriterQos {
    fn default() -> Self {
        Self {
            reliability: ReliabilityQosPolicy{kind: ReliabilityQosPolicyKind::ReliableReliabilityQos, max_blocking_time: Duration{sec: 0, nanosec: 100000000 /*100ms*/}},
            durability: DurabilityQosPolicy::default(),
            deadline: DeadlineQosPolicy::default(),
            latency_budget: LatencyBudgetQosPolicy::default(), 
            liveliness: LivelinessQosPolicy::default(),
            destination_order: DestinationOrderQosPolicy::default(),
            history: HistoryQosPolicy::default(),
            resource_limits: ResourceLimitsQosPolicy::default(),
            user_data: UserDataQosPolicy::default(),
            ownership: OwnershipQosPolicy::default(),
            durability_service: DurabilityServiceQosPolicy::default(),
            lifespan: LifespanQosPolicy::default(),
            ownership_strength: OwnershipStrengthQosPolicy::default(),
            transport_priority: TransportPriorityQosPolicy::default(),
            writer_data_lifecycle: WriterDataLifecycleQosPolicy::default(),
        }
    }
}

impl QosList for DataWriterQos {
    fn is_consistent(&self) -> bool {
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

#[derive(Default, Debug, PartialEq, Clone)]
pub struct SubscriberQos {
    pub presentation: PresentationQosPolicy,
    pub partition: PartitionQosPolicy,
    pub group_data: GroupDataQosPolicy,
    pub entity_factory: EntityFactoryQosPolicy,
}

impl QosList for SubscriberQos {
    fn is_consistent(&self) -> bool {
        true
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct DataReaderQos {
    pub durability: DurabilityQosPolicy,
    pub deadline: DeadlineQosPolicy,
    pub latency_budget: LatencyBudgetQosPolicy, 
    pub liveliness: LivelinessQosPolicy,
    pub reliability: ReliabilityQosPolicy,
    pub destination_order: DestinationOrderQosPolicy,
    pub history: HistoryQosPolicy,
    pub resource_limits: ResourceLimitsQosPolicy,
    pub user_data: UserDataQosPolicy,
    pub ownership: OwnershipQosPolicy,
    pub time_based_filter: TimeBasedFilterQosPolicy,
    pub reader_data_lifecycle: ReaderDataLifecycleQosPolicy,
}

impl Default for DataReaderQos {
    fn default() -> Self {
        Self {
            reliability: ReliabilityQosPolicy{kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos, max_blocking_time: Duration{sec: 0, nanosec: 100000000 /*100ms*/}},
            durability: DurabilityQosPolicy::default(),
            deadline: DeadlineQosPolicy::default(),
            latency_budget: LatencyBudgetQosPolicy::default(), 
            liveliness: LivelinessQosPolicy::default(),
            destination_order: DestinationOrderQosPolicy::default(),
            history: HistoryQosPolicy::default(),
            resource_limits: ResourceLimitsQosPolicy::default(),
            user_data: UserDataQosPolicy::default(),
            ownership: OwnershipQosPolicy::default(),
            time_based_filter: TimeBasedFilterQosPolicy::default(),
            reader_data_lifecycle: ReaderDataLifecycleQosPolicy::default(),
        }
    }
}

impl QosList for DataReaderQos {
    fn is_consistent(&self) -> bool {
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

        // The setting of the DEADLINE policy must be set consistently with that of the TIME_BASED_FILTER. For these two policies
        // to be consistent the settings must be such that “deadline period>= minimum_separation.”
        if self.deadline.period < self.time_based_filter.minimum_separation {
            return false
        }

        true
    }
}

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

impl QosList for TopicQos {
    fn is_consistent(&self) -> bool {
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