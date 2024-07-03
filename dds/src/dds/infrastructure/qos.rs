use crate::infrastructure::{
    error::{DdsError, DdsResult},
    time::Duration,
};

use super::{
    qos_policy::{
        DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, EntityFactoryQosPolicy,
        GroupDataQosPolicy, HistoryQosPolicy, HistoryQosPolicyKind, LatencyBudgetQosPolicy,
        LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, PartitionQosPolicy,
        PresentationQosPolicy, ReaderDataLifecycleQosPolicy, ReliabilityQosPolicy,
        ReliabilityQosPolicyKind, ResourceLimitsQosPolicy, TimeBasedFilterQosPolicy,
        TopicDataQosPolicy, TransportPriorityQosPolicy, UserDataQosPolicy,
        WriterDataLifecycleQosPolicy,
    },
    time::DurationKind,
};

/// QoS policies applicable to the [`DomainParticipantFactory`](crate::domain::domain_participant_factory::DomainParticipantFactory)
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct DomainParticipantFactoryQos {
    /// Value of the entity factory QoS policy.
    pub entity_factory: EntityFactoryQosPolicy,
}

/// Enumeration representing the kind of Qos to be used
#[derive(Debug)]
pub enum QosKind<T> {
    /// Default QoS variant
    Default,
    /// Specific QoS variant
    Specific(T),
}

/// QoS policies applicable to the [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant)
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct DomainParticipantQos {
    /// Value of the user data QoS policy.
    pub user_data: UserDataQosPolicy,
    /// Value of the entity factory QoS policy.
    pub entity_factory: EntityFactoryQosPolicy,
}

/// QoS policies applicable to the [`Publisher`](crate::publication::publisher::Publisher)
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct PublisherQos {
    /// Value of the presentation QoS policy.
    pub presentation: PresentationQosPolicy,
    /// Value of the partition QoS policy.
    pub partition: PartitionQosPolicy,
    /// Value of the group data QoS policy.
    pub group_data: GroupDataQosPolicy,
    /// Value of the entity factory QoS policy.
    pub entity_factory: EntityFactoryQosPolicy,
}

/// QoS policies applicable to the [`DataWriter`](crate::publication::data_writer::DataWriter)
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DataWriterQos {
    /// Value of the durability QoS policy.
    pub durability: DurabilityQosPolicy,
    /// Value of the deadline QoS policy.
    pub deadline: DeadlineQosPolicy,
    /// Value of the latency budget QoS policy.
    pub latency_budget: LatencyBudgetQosPolicy,
    /// Value of the liveliness QoS policy.
    pub liveliness: LivelinessQosPolicy,
    /// Value of the reliability QoS policy.
    pub reliability: ReliabilityQosPolicy,
    /// Value of the destination order QoS policy.
    pub destination_order: DestinationOrderQosPolicy,
    /// Value of the history QoS policy.
    pub history: HistoryQosPolicy,
    /// Value of the resource limits QoS policy.
    pub resource_limits: ResourceLimitsQosPolicy,
    /// Value of the transport priority QoS policy.
    pub transport_priority: TransportPriorityQosPolicy,
    /// Value of the lifespan QoS policy.
    pub lifespan: LifespanQosPolicy,
    /// Value of the user_data QoS policy.
    pub user_data: UserDataQosPolicy,
    /// Value of the ownership QoS policy.
    pub ownership: OwnershipQosPolicy,
    /// Value of the writer data lifecycle QoS policy.
    pub writer_data_lifecycle: WriterDataLifecycleQosPolicy,
}

impl Default for DataWriterQos {
    fn default() -> Self {
        Self {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::Reliable,
                max_blocking_time: DurationKind::Finite(Duration::new(
                    0,
                    100_000_000, /*100ms*/
                )),
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
            lifespan: LifespanQosPolicy::default(),
            transport_priority: TransportPriorityQosPolicy::default(),
            writer_data_lifecycle: WriterDataLifecycleQosPolicy::default(),
        }
    }
}

impl DataWriterQos {
    pub(crate) fn is_consistent(&self) -> DdsResult<()> {
        // The setting of RESOURCE_LIMITS max_samples must be consistent with the max_samples_per_instance. For these two
        // values to be consistent they must verify that *max_samples >= max_samples_per_instanc
        if self.resource_limits.max_samples < self.resource_limits.max_samples_per_instance {
            return Err(DdsError::InconsistentPolicy);
        }

        // The setting of RESOURCE_LIMITS max_samples_per_instance must be consistent with the HISTORY depth. For these two
        // QoS to be consistent, they must verify that *depth <= max_samples_per_instance.*
        match self.history.kind {
            HistoryQosPolicyKind::KeepLast(depth) => {
                if depth as usize > self.resource_limits.max_samples_per_instance {
                    Err(DdsError::InconsistentPolicy)
                } else {
                    Ok(())
                }
            }
            HistoryQosPolicyKind::KeepAll => Ok(()),
        }
    }

    pub(crate) fn check_immutability(&self, other: &Self) -> DdsResult<()> {
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

/// QoS policies applicable to the [`Subscriber`](crate::subscription::subscriber::Subscriber)
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct SubscriberQos {
    /// Value of the presentation QoS policy.
    pub presentation: PresentationQosPolicy,
    /// Value of the partition QoS policy.
    pub partition: PartitionQosPolicy,
    /// Value of the group data QoS policy.
    pub group_data: GroupDataQosPolicy,
    /// Value of the entity factory QoS policy.
    pub entity_factory: EntityFactoryQosPolicy,
}

impl SubscriberQos {
    pub(crate) fn check_immutability(&self, other: &Self) -> DdsResult<()> {
        if self.presentation != other.presentation {
            Err(DdsError::ImmutablePolicy)
        } else {
            Ok(())
        }
    }
}

/// QoS policies applicable to the [`DataReader`](crate::subscription::data_reader::DataReader)
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DataReaderQos {
    /// Value of the durability QoS policy.
    pub durability: DurabilityQosPolicy,
    /// Value of the deadline QoS policy.
    pub deadline: DeadlineQosPolicy,
    /// Value of the latency budget QoS policy.
    pub latency_budget: LatencyBudgetQosPolicy,
    /// Value of the liveliness QoS policy.
    pub liveliness: LivelinessQosPolicy,
    /// Value of the reliability QoS policy.
    pub reliability: ReliabilityQosPolicy,
    /// Value of the destination order QoS policy.
    pub destination_order: DestinationOrderQosPolicy,
    /// Value of the history QoS policy.
    pub history: HistoryQosPolicy,
    /// Value of the resource limits QoS policy.
    pub resource_limits: ResourceLimitsQosPolicy,
    /// Value of the user data QoS policy.
    pub user_data: UserDataQosPolicy,
    /// Value of the ownership QoS policy.
    pub ownership: OwnershipQosPolicy,
    /// Value of the time based filter QoS policy.
    pub time_based_filter: TimeBasedFilterQosPolicy,
    /// Value of the reader data lifecycle QoS policy.
    pub reader_data_lifecycle: ReaderDataLifecycleQosPolicy,
}

impl Default for DataReaderQos {
    fn default() -> Self {
        Self {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort,
                max_blocking_time: DurationKind::Finite(Duration::new(
                    0,
                    100_000_000, /*100ms*/
                )),
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
    pub(crate) fn is_consistent(&self) -> DdsResult<()> {
        // The setting of RESOURCE_LIMITS max_samples must be consistent with the max_samples_per_instance. For these two
        // values to be consistent they must verify that *max_samples >= max_samples_per_instance.*
        if self.resource_limits.max_samples < self.resource_limits.max_samples_per_instance {
            return Err(DdsError::InconsistentPolicy);
        }

        // The setting of RESOURCE_LIMITS max_samples_per_instance must be consistent with the HISTORY depth. For these two
        // QoS to be consistent, they must verify that *depth <= max_samples_per_instance.*
        match self.history.kind {
            HistoryQosPolicyKind::KeepLast(depth) => {
                if depth as usize > self.resource_limits.max_samples_per_instance {
                    return Err(DdsError::InconsistentPolicy);
                }
            }
            HistoryQosPolicyKind::KeepAll => (),
        }

        // The setting of the DEADLINE policy must be set consistently with that of the TIME_BASED_FILTER. For these two policies
        // to be consistent the settings must be such that *deadline period>= minimum_separation.*
        if self.deadline.period < self.time_based_filter.minimum_separation {
            return Err(DdsError::InconsistentPolicy);
        }

        Ok(())
    }

    pub(crate) fn check_immutability(&self, other: &Self) -> DdsResult<()> {
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

/// QoS policies applicable to the [`Topic`](crate::topic_definition::topic::Topic)
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TopicQos {
    /// Value of the topic data QoS policy.
    pub topic_data: TopicDataQosPolicy,
    /// Value of the durability QoS policy.
    pub durability: DurabilityQosPolicy,
    /// Value of the deadline QoS policy.
    pub deadline: DeadlineQosPolicy,
    /// Value of the latency budget QoS policy.
    pub latency_budget: LatencyBudgetQosPolicy,
    /// Value of the liveliness QoS policy.
    pub liveliness: LivelinessQosPolicy,
    /// Value of the reliability QoS policy.
    pub reliability: ReliabilityQosPolicy,
    /// Value of the destination order QoS policy.
    pub destination_order: DestinationOrderQosPolicy,
    /// Value of the history QoS policy.
    pub history: HistoryQosPolicy,
    /// Value of the resource limits QoS policy.
    pub resource_limits: ResourceLimitsQosPolicy,
    /// Value of the transport priority QoS policy.
    pub transport_priority: TransportPriorityQosPolicy,
    /// Value of the lifespan QoS policy.
    pub lifespan: LifespanQosPolicy,
    /// Value of the ownership QoS policy.
    pub ownership: OwnershipQosPolicy,
}

impl Default for TopicQos {
    fn default() -> Self {
        Self {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort,
                max_blocking_time: DurationKind::Finite(Duration::new(
                    0,
                    100_000_000, /*100ms*/
                )),
            },
            topic_data: TopicDataQosPolicy::default(),
            durability: DurabilityQosPolicy::default(),
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
    pub(crate) fn is_consistent(&self) -> DdsResult<()> {
        // The setting of RESOURCE_LIMITS max_samples must be consistent with the max_samples_per_instance. For these two
        // values to be consistent they must verify that *max_samples >= max_samples_per_instance.*
        if self.resource_limits.max_samples < self.resource_limits.max_samples_per_instance {
            return Err(DdsError::InconsistentPolicy);
        }

        // The setting of RESOURCE_LIMITS max_samples_per_instance must be consistent with the HISTORY depth. For these two
        // QoS to be consistent, they must verify that *depth <= max_samples_per_instance.*
        match self.history.kind {
            HistoryQosPolicyKind::KeepLast(depth) => {
                if depth as usize > self.resource_limits.max_samples_per_instance {
                    Err(DdsError::InconsistentPolicy)
                } else {
                    Ok(())
                }
            }
            HistoryQosPolicyKind::KeepAll => Ok(()),
        }
    }

    pub(crate) fn check_immutability(&self, other: &Self) -> DdsResult<()> {
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

#[cfg(test)]
mod tests {
    use crate::infrastructure::qos_policy::Length;

    use super::*;

    #[test]
    fn data_writer_qos_consistency() {
        assert_eq!(DataWriterQos::default().is_consistent(), Ok(()));
        assert_eq!(
            DataWriterQos {
                resource_limits: ResourceLimitsQosPolicy {
                    max_samples_per_instance: Length::Limited(2),
                    max_samples: Length::Limited(1),
                    ..Default::default()
                },
                ..Default::default()
            }
            .is_consistent(),
            Err(DdsError::InconsistentPolicy)
        );
        assert_eq!(
            DataWriterQos {
                history: HistoryQosPolicy {
                    kind: HistoryQosPolicyKind::KeepLast(3),
                },
                resource_limits: ResourceLimitsQosPolicy {
                    max_samples_per_instance: Length::Limited(2),
                    ..Default::default()
                },
                ..Default::default()
            }
            .is_consistent(),
            Err(DdsError::InconsistentPolicy)
        );
    }

    #[test]
    fn data_reader_qos_consistency() {
        assert_eq!(DataReaderQos::default().is_consistent(), Ok(()));
        assert_eq!(
            DataReaderQos {
                resource_limits: ResourceLimitsQosPolicy {
                    max_samples_per_instance: Length::Limited(2),
                    max_samples: Length::Limited(1),
                    ..Default::default()
                },
                ..Default::default()
            }
            .is_consistent(),
            Err(DdsError::InconsistentPolicy)
        );
        assert_eq!(
            DataReaderQos {
                history: HistoryQosPolicy {
                    kind: HistoryQosPolicyKind::KeepLast(3),
                },
                resource_limits: ResourceLimitsQosPolicy {
                    max_samples_per_instance: Length::Limited(2),
                    ..Default::default()
                },
                ..Default::default()
            }
            .is_consistent(),
            Err(DdsError::InconsistentPolicy)
        );
    }

    #[test]
    fn topic_qos_consistency() {
        assert_eq!(TopicQos::default().is_consistent(), Ok(()));
        assert_eq!(
            TopicQos {
                resource_limits: ResourceLimitsQosPolicy {
                    max_samples_per_instance: Length::Limited(2),
                    max_samples: Length::Limited(1),
                    ..Default::default()
                },
                ..Default::default()
            }
            .is_consistent(),
            Err(DdsError::InconsistentPolicy)
        );
        assert_eq!(
            TopicQos {
                history: HistoryQosPolicy {
                    kind: HistoryQosPolicyKind::KeepLast(3),
                },
                resource_limits: ResourceLimitsQosPolicy {
                    max_samples_per_instance: Length::Limited(2),
                    ..Default::default()
                },
                ..Default::default()
            }
            .is_consistent(),
            Err(DdsError::InconsistentPolicy)
        );
    }
}
