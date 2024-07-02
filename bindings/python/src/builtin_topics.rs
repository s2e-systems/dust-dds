use pyo3::prelude::*;

use crate::infrastructure::qos_policy::{
    DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
    HistoryQosPolicy, LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy,
    OwnershipQosPolicy, PartitionQosPolicy, PresentationQosPolicy, ReliabilityQosPolicy,
    ResourceLimitsQosPolicy, TimeBasedFilterQosPolicy, TopicDataQosPolicy,
    TransportPriorityQosPolicy, UserDataQosPolicy,
};

#[pyclass]
#[derive(Clone)]
pub struct BuiltInTopicKey(dust_dds::builtin_topics::BuiltInTopicKey);

impl From<dust_dds::builtin_topics::BuiltInTopicKey> for BuiltInTopicKey {
    fn from(value: dust_dds::builtin_topics::BuiltInTopicKey) -> Self {
        Self(value)
    }
}

#[pymethods]
impl BuiltInTopicKey {
    pub fn get_value(&self) -> [u8; 16] {
        self.0.value
    }
}

#[pyclass]
#[derive(Clone)]
pub struct ParticipantBuiltinTopicData(dust_dds::builtin_topics::ParticipantBuiltinTopicData);

impl From<dust_dds::builtin_topics::ParticipantBuiltinTopicData> for ParticipantBuiltinTopicData {
    fn from(value: dust_dds::builtin_topics::ParticipantBuiltinTopicData) -> Self {
        Self(value)
    }
}

#[pymethods]
impl ParticipantBuiltinTopicData {
    pub fn get_key(&self) -> BuiltInTopicKey {
        self.0.key().clone().into()
    }

    pub fn get_user_data(&self) -> UserDataQosPolicy {
        self.0.user_data().clone().into()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct TopicBuiltinTopicData(dust_dds::builtin_topics::TopicBuiltinTopicData);

impl From<dust_dds::builtin_topics::TopicBuiltinTopicData> for TopicBuiltinTopicData {
    fn from(value: dust_dds::builtin_topics::TopicBuiltinTopicData) -> Self {
        Self(value)
    }
}

#[pymethods]
impl TopicBuiltinTopicData {
    pub fn get_key(&self) -> BuiltInTopicKey {
        self.0.key().clone().into()
    }

    pub fn get_name(&self) -> String {
        self.0.name().to_string()
    }

    pub fn get_type_name(&self) -> String {
        self.0.get_type_name().to_string()
    }

    pub fn get_durability(&self) -> DurabilityQosPolicy {
        self.0.durability().clone().into()
    }

    pub fn get_deadline(&self) -> DeadlineQosPolicy {
        self.0.deadline().clone().into()
    }

    pub fn get_latency_budget(&self) -> LatencyBudgetQosPolicy {
        self.0.latency_budget().clone().into()
    }

    pub fn get_liveliness(&self) -> LivelinessQosPolicy {
        self.0.liveliness().clone().into()
    }

    pub fn get_reliability(&self) -> ReliabilityQosPolicy {
        self.0.reliability().clone().into()
    }

    pub fn get_transport_priority(&self) -> TransportPriorityQosPolicy {
        self.0.transport_priority().clone().into()
    }

    pub fn get_lifespan(&self) -> LifespanQosPolicy {
        self.0.lifespan().clone().into()
    }

    pub fn get_destination_order(&self) -> DestinationOrderQosPolicy {
        self.0.destination_order().clone().into()
    }

    pub fn get_history(&self) -> HistoryQosPolicy {
        self.0.history().clone().into()
    }

    pub fn get_resource_limits(&self) -> ResourceLimitsQosPolicy {
        self.0.resource_limits().clone().into()
    }

    pub fn get_ownership(&self) -> OwnershipQosPolicy {
        self.0.ownership().clone().into()
    }

    pub fn get_topic_data(&self) -> TopicDataQosPolicy {
        self.0.topic_data().clone().into()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PublicationBuiltinTopicData(dust_dds::builtin_topics::PublicationBuiltinTopicData);

impl From<dust_dds::builtin_topics::PublicationBuiltinTopicData> for PublicationBuiltinTopicData {
    fn from(value: dust_dds::builtin_topics::PublicationBuiltinTopicData) -> Self {
        Self(value)
    }
}

#[pymethods]
impl PublicationBuiltinTopicData {
    pub fn get_key(&self) -> BuiltInTopicKey {
        self.0.key().clone().into()
    }

    pub fn participant_key(&self) -> BuiltInTopicKey {
        self.0.participant_key().clone().into()
    }

    pub fn topic_name(&self) -> String {
        self.0.topic_name().to_string()
    }

    pub fn get_type_name(&self) -> String {
        self.0.get_type_name().to_string()
    }

    pub fn get_durability(&self) -> DurabilityQosPolicy {
        self.0.durability().clone().into()
    }

    pub fn get_deadline(&self) -> DeadlineQosPolicy {
        self.0.deadline().clone().into()
    }

    pub fn get_latency_budget(&self) -> LatencyBudgetQosPolicy {
        self.0.latency_budget().clone().into()
    }

    pub fn get_liveliness(&self) -> LivelinessQosPolicy {
        self.0.liveliness().clone().into()
    }

    pub fn get_reliability(&self) -> ReliabilityQosPolicy {
        self.0.reliability().clone().into()
    }

    pub fn get_lifespan(&self) -> LifespanQosPolicy {
        self.0.lifespan().clone().into()
    }

    pub fn get_user_data(&self) -> UserDataQosPolicy {
        self.0.user_data().clone().into()
    }

    pub fn get_ownership(&self) -> OwnershipQosPolicy {
        self.0.ownership().clone().into()
    }

    pub fn get_destination_order(&self) -> DestinationOrderQosPolicy {
        self.0.destination_order().clone().into()
    }

    pub fn get_presentation(&self) -> PresentationQosPolicy {
        self.0.presentation().clone().into()
    }

    pub fn get_partition(&self) -> PartitionQosPolicy {
        self.0.partition().clone().into()
    }

    pub fn get_topic_data(&self) -> TopicDataQosPolicy {
        self.0.topic_data().clone().into()
    }

    pub fn get_group_data(&self) -> GroupDataQosPolicy {
        self.0.group_data().clone().into()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct SubscriptionBuiltinTopicData(dust_dds::builtin_topics::SubscriptionBuiltinTopicData);

impl From<dust_dds::builtin_topics::SubscriptionBuiltinTopicData> for SubscriptionBuiltinTopicData {
    fn from(value: dust_dds::builtin_topics::SubscriptionBuiltinTopicData) -> Self {
        Self(value)
    }
}

#[pymethods]
impl SubscriptionBuiltinTopicData {
    pub fn get_key(&self) -> BuiltInTopicKey {
        self.0.key().clone().into()
    }

    pub fn participant_key(&self) -> BuiltInTopicKey {
        self.0.participant_key().clone().into()
    }

    pub fn topic_name(&self) -> String {
        self.0.topic_name().to_string()
    }

    pub fn get_type_name(&self) -> String {
        self.0.get_type_name().to_string()
    }

    pub fn get_durability(&self) -> DurabilityQosPolicy {
        self.0.durability().clone().into()
    }

    pub fn get_deadline(&self) -> DeadlineQosPolicy {
        self.0.deadline().clone().into()
    }

    pub fn get_latency_budget(&self) -> LatencyBudgetQosPolicy {
        self.0.latency_budget().clone().into()
    }

    pub fn get_liveliness(&self) -> LivelinessQosPolicy {
        self.0.liveliness().clone().into()
    }

    pub fn get_reliability(&self) -> ReliabilityQosPolicy {
        self.0.reliability().clone().into()
    }

    pub fn get_ownership(&self) -> OwnershipQosPolicy {
        self.0.ownership().clone().into()
    }

    pub fn get_destination_order(&self) -> DestinationOrderQosPolicy {
        self.0.destination_order().clone().into()
    }

    pub fn get_user_data(&self) -> UserDataQosPolicy {
        self.0.user_data().clone().into()
    }

    pub fn get_time_based_filter(&self) -> TimeBasedFilterQosPolicy {
        self.0.time_based_filter().clone().into()
    }

    pub fn get_presentation(&self) -> PresentationQosPolicy {
        self.0.presentation().clone().into()
    }

    pub fn get_partition(&self) -> PartitionQosPolicy {
        self.0.partition().clone().into()
    }

    pub fn get_topic_data(&self) -> TopicDataQosPolicy {
        self.0.topic_data().clone().into()
    }

    pub fn get_group_data(&self) -> GroupDataQosPolicy {
        self.0.group_data().clone().into()
    }
}
