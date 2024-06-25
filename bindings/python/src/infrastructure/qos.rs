use pyo3::prelude::*;

use super::qos_policy::{
    DeadlineQosPolicy, DurabilityQosPolicy, EntityFactoryQosPolicy, LatencyBudgetQosPolicy,
    LivelinessQosPolicy, TopicDataQosPolicy, UserDataQosPolicy,
};

#[pyclass]
#[derive(Clone)]
pub struct DomainParticipantFactoryQos(dust_dds::infrastructure::qos::DomainParticipantFactoryQos);

impl From<DomainParticipantFactoryQos>
    for dust_dds::infrastructure::qos::DomainParticipantFactoryQos
{
    fn from(value: DomainParticipantFactoryQos) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos::DomainParticipantFactoryQos>
    for DomainParticipantFactoryQos
{
    fn from(value: dust_dds::infrastructure::qos::DomainParticipantFactoryQos) -> Self {
        Self(value)
    }
}

#[pymethods]
impl DomainParticipantFactoryQos {
    #[new]
    #[pyo3(signature = (entity_factory = EntityFactoryQosPolicy::default()))]
    pub fn new(entity_factory: EntityFactoryQosPolicy) -> Self {
        Self(dust_dds::infrastructure::qos::DomainParticipantFactoryQos {
            entity_factory: entity_factory.into(),
        })
    }
}

#[pyclass]
#[derive(Clone)]
pub struct DomainParticipantQos(dust_dds::infrastructure::qos::DomainParticipantQos);

impl From<DomainParticipantQos> for dust_dds::infrastructure::qos::DomainParticipantQos {
    fn from(value: DomainParticipantQos) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos::DomainParticipantQos> for DomainParticipantQos {
    fn from(value: dust_dds::infrastructure::qos::DomainParticipantQos) -> Self {
        Self(value)
    }
}

#[pymethods]
impl DomainParticipantQos {
    #[new]
    #[pyo3(signature = (user_data= UserDataQosPolicy::default(), entity_factory = EntityFactoryQosPolicy::default()))]
    pub fn new(user_data: UserDataQosPolicy, entity_factory: EntityFactoryQosPolicy) -> Self {
        Self(dust_dds::infrastructure::qos::DomainParticipantQos {
            user_data: user_data.clone().into(),
            entity_factory: entity_factory.into(),
        })
    }
}

#[pyclass]
#[derive(Clone)]
pub struct TopicQos(dust_dds::infrastructure::qos::TopicQos);

impl From<TopicQos> for dust_dds::infrastructure::qos::TopicQos {
    fn from(value: TopicQos) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::qos::TopicQos> for TopicQos {
    fn from(value: dust_dds::infrastructure::qos::TopicQos) -> Self {
        Self(value)
    }
}

#[pymethods]
impl TopicQos {
    #[new]
    #[pyo3(signature = (
        topic_data = TopicDataQosPolicy::default(),
        durability = DurabilityQosPolicy::default(),
        deadline = DeadlineQosPolicy::default(),
        latency_budget = LatencyBudgetQosPolicy::default(),
        liveliness = LivelinessQosPolicy::default(),
    ))]
    pub fn new(
        topic_data: TopicDataQosPolicy,
        durability: DurabilityQosPolicy,
        deadline: DeadlineQosPolicy,
        latency_budget: LatencyBudgetQosPolicy,
        liveliness: LivelinessQosPolicy,
    ) -> Self {
        Self(dust_dds::infrastructure::qos::TopicQos {
            topic_data: topic_data.into(),
            durability: durability.into(),
            deadline: deadline.into(),
            latency_budget: latency_budget.into(),
            liveliness: liveliness.into(),
            reliability: todo!(),
            destination_order: todo!(),
            history: todo!(),
            resource_limits: todo!(),
            transport_priority: todo!(),
            lifespan: todo!(),
            ownership: todo!(),
        })
    }
}
