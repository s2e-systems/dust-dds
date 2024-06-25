mod domain;
mod infrastructure;
mod publication;
mod subscription;
mod topic_definition;

use pyo3::prelude::*;

/// Dust DDS python bindings
#[pymodule]
fn dust_dds(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<domain::domain_participant_factory::DomainParticipantFactory>()?;
    m.add_class::<topic_definition::type_support::MyDdsData>()?;

    // Add time classes
    m.add_class::<infrastructure::time::Duration>()?;
    m.add_class::<infrastructure::time::DurationKind>()?;

    // Add QosPolicy classes
    m.add_class::<infrastructure::qos_policy::DeadlineQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::DestinationOrderQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::DestinationOrderQosPolicyKind>()?;
    m.add_class::<infrastructure::qos_policy::DurabilityQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::DurabilityQosPolicyKind>()?;
    m.add_class::<infrastructure::qos_policy::EntityFactoryQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::GroupDataQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::HistoryQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::HistoryQosPolicyKind>()?;
    m.add_class::<infrastructure::qos_policy::LatencyBudgetQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::Length>()?;
    m.add_class::<infrastructure::qos_policy::LifespanQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::LivelinessQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::LivelinessQosPolicyKind>()?;
    m.add_class::<infrastructure::qos_policy::OwnershipQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::OwnershipQosPolicyKind>()?;
    m.add_class::<infrastructure::qos_policy::PartitionQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::ReliabilityQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::ReliabilityQosPolicyKind>()?;
    m.add_class::<infrastructure::qos_policy::ResourceLimitsQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::TopicDataQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::TransportPriorityQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::UserDataQosPolicy>()?;

    // Add Qos classes
    m.add_class::<infrastructure::qos::DomainParticipantFactoryQos>()?;
    m.add_class::<infrastructure::qos::DomainParticipantQos>()?;
    m.add_class::<infrastructure::qos::SubscriberQos>()?;
    m.add_class::<infrastructure::qos::PublisherQos>()?;
    m.add_class::<infrastructure::qos::TopicQos>()?;

    Ok(())
}
