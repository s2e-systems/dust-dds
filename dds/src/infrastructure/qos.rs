use crate::{
    dcps_psm::{Duration, LENGTH_UNLIMITED},
    return_type::{DdsError, DdsResult},
};

use super::qos_policy::{
    DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, DurabilityServiceQosPolicy,
    EntityFactoryQosPolicy, GroupDataQosPolicy, HistoryQosPolicy, HistoryQosPolicyKind,
    LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy,
    OwnershipStrengthQosPolicy, PartitionQosPolicy, PresentationQosPolicy,
    ReaderDataLifecycleQosPolicy, ReliabilityQosPolicy, ReliabilityQosPolicyKind,
    ResourceLimitsQosPolicy, TimeBasedFilterQosPolicy, TopicDataQosPolicy,
    TransportPriorityQosPolicy, UserDataQosPolicy, WriterDataLifecycleQosPolicy,
};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct DomainParticipantFactoryQos {
    entity_factory: EntityFactoryQosPolicy,
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct DomainParticipantQos {
    pub user_data: UserDataQosPolicy,
    pub entity_factory: EntityFactoryQosPolicy,
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct PublisherQos {
    pub presentation: PresentationQosPolicy,
    pub partition: PartitionQosPolicy,
    pub group_data: GroupDataQosPolicy,
    pub entity_factory: EntityFactoryQosPolicy,
}

impl PublisherQos {
    pub fn check_immutability(&self, other: &Self) -> DdsResult<()> {
        if self.presentation != other.presentation {
            Err(DdsError::ImmutablePolicy)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
                max_blocking_time: Duration::new(0, 100_000_000 /*100ms*/),
            },
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

impl DataWriterQos {
    pub fn is_consistent(&self) -> DdsResult<()> {
        // The setting of RESOURCE_LIMITS max_samples must be consistent with the max_samples_per_instance. For these two
        // values to be consistent they must verify that “max_samples >= max_samples_per_instance.”
        let limited_samples = self.resource_limits.max_samples != LENGTH_UNLIMITED;
        let samples_per_instance_over_limit = self.resource_limits.max_samples == LENGTH_UNLIMITED
            || self.resource_limits.max_samples_per_instance > self.resource_limits.max_samples;

        if limited_samples && samples_per_instance_over_limit {
            return Err(DdsError::InconsistentPolicy);
        }

        // The setting of RESOURCE_LIMITS max_samples_per_instance must be consistent with the HISTORY depth. For these two
        // QoS to be consistent, they must verify that “depth <= max_samples_per_instance.”
        let limited_samples_per_instance =
            self.resource_limits.max_samples_per_instance != LENGTH_UNLIMITED;
        let history_depth_over_limit = self.history.depth == LENGTH_UNLIMITED
            || self.history.depth > self.resource_limits.max_samples_per_instance;

        if self.history.kind == HistoryQosPolicyKind::KeepLastHistoryQoS
            && limited_samples_per_instance
            && history_depth_over_limit
        {
            return Err(DdsError::InconsistentPolicy);
        }

        Ok(())
    }

    pub fn check_immutability(&self, other: &Self) -> DdsResult<()> {
        if self.durability != other.durability
            || self.liveliness != other.liveliness
            || self.reliability != other.reliability
            || self.destination_order != other.destination_order
            || self.history != other.history
            || self.resource_limits != other.resource_limits
            || self.ownership != other.ownership
            || self.durability_service != other.durability_service
        {
            Err(DdsError::ImmutablePolicy)
        } else {
            Ok(())
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct SubscriberQos {
    pub presentation: PresentationQosPolicy,
    pub partition: PartitionQosPolicy,
    pub group_data: GroupDataQosPolicy,
    pub entity_factory: EntityFactoryQosPolicy,
}

impl SubscriberQos {
    pub fn check_immutability(&self, other: &Self) -> DdsResult<()> {
        if self.presentation != other.presentation {
            Err(DdsError::ImmutablePolicy)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos,
                max_blocking_time: Duration::new(0, 100_000_000 /*100ms*/),
            },
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

impl DataReaderQos {
    pub fn is_consistent(&self) -> DdsResult<()> {
        // The setting of RESOURCE_LIMITS max_samples must be consistent with the max_samples_per_instance. For these two
        // values to be consistent they must verify that “max_samples >= max_samples_per_instance.”
        let limited_samples = self.resource_limits.max_samples != LENGTH_UNLIMITED;
        let samples_per_instance_over_limit = self.resource_limits.max_samples == LENGTH_UNLIMITED
            || self.resource_limits.max_samples_per_instance > self.resource_limits.max_samples;

        if limited_samples && samples_per_instance_over_limit {
            return Err(DdsError::InconsistentPolicy);
        }

        // The setting of RESOURCE_LIMITS max_samples_per_instance must be consistent with the HISTORY depth. For these two
        // QoS to be consistent, they must verify that “depth <= max_samples_per_instance.”
        let limited_samples_per_instance =
            self.resource_limits.max_samples_per_instance != LENGTH_UNLIMITED;
        let history_depth_over_limit = self.history.depth == LENGTH_UNLIMITED
            || self.history.depth > self.resource_limits.max_samples_per_instance;

        if self.history.kind == HistoryQosPolicyKind::KeepLastHistoryQoS
            && limited_samples_per_instance
            && history_depth_over_limit
        {
            return Err(DdsError::InconsistentPolicy);
        }

        // The setting of the DEADLINE policy must be set consistently with that of the TIME_BASED_FILTER. For these two policies
        // to be consistent the settings must be such that “deadline period>= minimum_separation.”
        if self.deadline.period < self.time_based_filter.minimum_separation {
            return Err(DdsError::InconsistentPolicy);
        }

        Ok(())
    }

    pub fn check_immutability(&self, other: &Self) -> DdsResult<()> {
        if self.durability != other.durability
            || self.liveliness != other.liveliness
            || self.reliability != other.reliability
            || self.destination_order != other.destination_order
            || self.history != other.history
            || self.resource_limits != other.resource_limits
            || self.ownership != other.ownership
        {
            Err(DdsError::ImmutablePolicy)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos,
                max_blocking_time: Duration::new(0, 100_000_000 /*100ms*/),
            },
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
    pub fn is_consistent(&self) -> DdsResult<()> {
        // The setting of RESOURCE_LIMITS max_samples must be consistent with the max_samples_per_instance. For these two
        // values to be consistent they must verify that “max_samples >= max_samples_per_instance.”
        let limited_samples = self.resource_limits.max_samples != LENGTH_UNLIMITED;
        let samples_per_instance_over_limit = self.resource_limits.max_samples == LENGTH_UNLIMITED
            || self.resource_limits.max_samples_per_instance > self.resource_limits.max_samples;

        if limited_samples && samples_per_instance_over_limit {
            return Err(DdsError::InconsistentPolicy);
        }

        // The setting of RESOURCE_LIMITS max_samples_per_instance must be consistent with the HISTORY depth. For these two
        // QoS to be consistent, they must verify that “depth <= max_samples_per_instance.”
        let limited_samples_per_instance =
            self.resource_limits.max_samples_per_instance != LENGTH_UNLIMITED;
        let history_depth_over_limit = self.history.depth == LENGTH_UNLIMITED
            || self.history.depth > self.resource_limits.max_samples_per_instance;

        if self.history.kind == HistoryQosPolicyKind::KeepLastHistoryQoS
            && limited_samples_per_instance
            && history_depth_over_limit
        {
            return Err(DdsError::InconsistentPolicy);
        }

        Ok(())
    }

    pub fn check_immutability(&self, other: &Self) -> DdsResult<()> {
        if self.durability != other.durability
            || self.durability_service != other.durability_service
            || self.liveliness != other.liveliness
            || self.reliability != other.reliability
            || self.destination_order != other.destination_order
            || self.history != other.history
            || self.resource_limits != other.resource_limits
            || self.ownership != other.ownership
        {
            Err(DdsError::ImmutablePolicy)
        } else {
            Ok(())
        }
    }
}
