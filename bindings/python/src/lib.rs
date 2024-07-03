mod builtin_topics;
mod domain;
mod infrastructure;
mod publication;
mod subscription;
mod topic_definition;
mod xtypes;

use infrastructure::qos_policy::{
    DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
    DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
};
use pyo3::prelude::*;
use subscription::sample_info::{
    ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE, NOT_ALIVE_INSTANCE_STATE,
};

/// Dust DDS python bindings
#[pymodule]
fn dust_dds(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<domain::domain_participant_factory::DomainParticipantFactory>()?;
    m.add_class::<domain::domain_participant::DomainParticipant>()?;
    m.add_class::<publication::publisher::Publisher>()?;
    m.add_class::<publication::data_writer::DataWriter>()?;
    m.add_class::<subscription::subscriber::Subscriber>()?;
    m.add_class::<subscription::data_reader::DataReader>()?;
    m.add_class::<topic_definition::topic::Topic>()?;
    m.add_class::<topic_definition::type_support::TypeKind>()?;

    m.add_class::<infrastructure::time::Duration>()?;
    m.add_class::<infrastructure::time::DurationKind>()?;
    m.add_class::<infrastructure::status::StatusKind>()?;
    m.add_class::<infrastructure::wait_set::Condition>()?;
    m.add_class::<infrastructure::wait_set::WaitSet>()?;

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
    m.add_class::<infrastructure::qos_policy::ReaderDataLifecycleQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::ReliabilityQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::ReliabilityQosPolicyKind>()?;
    m.add_class::<infrastructure::qos_policy::ResourceLimitsQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::TimeBasedFilterQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::TopicDataQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::TransportPriorityQosPolicy>()?;
    m.add_class::<infrastructure::qos_policy::UserDataQosPolicy>()?;

    // Add Qos classes
    m.add_class::<infrastructure::qos::DomainParticipantFactoryQos>()?;
    m.add_class::<infrastructure::qos::DomainParticipantQos>()?;
    m.add_class::<infrastructure::qos::SubscriberQos>()?;
    m.add_class::<infrastructure::qos::PublisherQos>()?;
    m.add_class::<infrastructure::qos::TopicQos>()?;
    m.add_class::<infrastructure::qos::DataWriterQos>()?;
    m.add_class::<infrastructure::qos::DataReaderQos>()?;

    m.add_class::<subscription::sample_info::SampleStateKind>()?;
    m.add("ANY_SAMPLE_STATE", ANY_SAMPLE_STATE.to_vec())?;
    m.add_class::<subscription::sample_info::ViewStateKind>()?;
    m.add("ANY_VIEW_STATE", ANY_VIEW_STATE.to_vec())?;
    m.add_class::<subscription::sample_info::InstanceStateKind>()?;
    m.add("ANY_INSTANCE_STATE", ANY_INSTANCE_STATE.to_vec())?;
    m.add(
        "NOT_ALIVE_INSTANCE_STATE",
        NOT_ALIVE_INSTANCE_STATE.to_vec(),
    )?;

    m.add(
        "DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS",
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
    )?;
    m.add(
        "DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER",
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
    )?;

    Ok(())
}
