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
    pub entity_factory: EntityFactoryQosPolicy,
}

/// Enumeration representing the kind of Qos to be used
#[derive(Debug)]
pub enum QosKind<T> {
    Default,
    Specific(T),
}

/// QoS policies applicable to the [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant)
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct DomainParticipantQos {
    pub user_data: UserDataQosPolicy,
    pub entity_factory: EntityFactoryQosPolicy,
}

/// QoS policies applicable to the [`Publisher`](crate::publication::publisher::Publisher)
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

/// QoS policies applicable to the [`DataWriter`](crate::publication::data_writer::DataWriter)
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DataWriterQos {
    pub durability: DurabilityQosPolicy,
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
    pub fn is_consistent(&self) -> DdsResult<()> {
        // The setting of RESOURCE_LIMITS max_samples must be consistent with the max_samples_per_instance. For these two
        // values to be consistent they must verify that “max_samples >= max_samples_per_instanc
        if self.resource_limits.max_samples < self.resource_limits.max_samples_per_instance {
            return Err(DdsError::InconsistentPolicy);
        }

        // The setting of RESOURCE_LIMITS max_samples_per_instance must be consistent with the HISTORY depth. For these two
        // QoS to be consistent, they must verify that “depth <= max_samples_per_instance.”
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

/// QoS policies applicable to the [`Subscriber`](crate::subscription::subscriber::Subscriber)
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

/// QoS policies applicable to the [`DataReader`](crate::subscription::data_reader::DataReader)
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
    pub fn is_consistent(&self) -> DdsResult<()> {
        // The setting of RESOURCE_LIMITS max_samples must be consistent with the max_samples_per_instance. For these two
        // values to be consistent they must verify that “max_samples >= max_samples_per_instance.”
        if self.resource_limits.max_samples < self.resource_limits.max_samples_per_instance {
            return Err(DdsError::InconsistentPolicy);
        }

        // The setting of RESOURCE_LIMITS max_samples_per_instance must be consistent with the HISTORY depth. For these two
        // QoS to be consistent, they must verify that “depth <= max_samples_per_instance.”
        match self.history.kind {
            HistoryQosPolicyKind::KeepLast(depth) => {
                if depth as usize > self.resource_limits.max_samples_per_instance {
                    return Err(DdsError::InconsistentPolicy);
                }
            }
            HistoryQosPolicyKind::KeepAll => (),
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

/// QoS policies applicable to the [`Topic`](crate::topic_definition::topic::Topic)
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TopicQos {
    pub topic_data: TopicDataQosPolicy,
    pub durability: DurabilityQosPolicy,
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
    pub fn is_consistent(&self) -> DdsResult<()> {
        // The setting of RESOURCE_LIMITS max_samples must be consistent with the max_samples_per_instance. For these two
        // values to be consistent they must verify that “max_samples >= max_samples_per_instance.”
        if self.resource_limits.max_samples < self.resource_limits.max_samples_per_instance {
            return Err(DdsError::InconsistentPolicy);
        }

        // The setting of RESOURCE_LIMITS max_samples_per_instance must be consistent with the HISTORY depth. For these two
        // QoS to be consistent, they must verify that “depth <= max_samples_per_instance.”
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
