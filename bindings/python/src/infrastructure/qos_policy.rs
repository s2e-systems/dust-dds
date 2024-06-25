use pyo3::prelude::*;

use super::time::DurationKind;

#[pyclass(frozen)]
#[derive(Clone, Default)]
pub struct UserDataQosPolicy(dust_dds::infrastructure::qos_policy::UserDataQosPolicy);

impl From<UserDataQosPolicy> for dust_dds::infrastructure::qos_policy::UserDataQosPolicy {
    fn from(value: UserDataQosPolicy) -> Self {
        value.0
    }
}

#[pymethods]
impl UserDataQosPolicy {
    #[new]
    pub fn new(value: Vec<u8>) -> Self {
        Self(dust_dds::infrastructure::qos_policy::UserDataQosPolicy { value })
    }

    #[getter]
    pub fn get_value(&self) -> &[u8] {
        &self.0.value
    }
}

#[pyclass(frozen)]
#[derive(Clone, Default)]
pub struct EntityFactoryQosPolicy(dust_dds::infrastructure::qos_policy::EntityFactoryQosPolicy);

impl From<EntityFactoryQosPolicy> for dust_dds::infrastructure::qos_policy::EntityFactoryQosPolicy {
    fn from(value: EntityFactoryQosPolicy) -> Self {
        value.0
    }
}

#[pymethods]
impl EntityFactoryQosPolicy {
    #[new]
    pub fn new(autoenable_created_entities: bool) -> Self {
        Self(
            dust_dds::infrastructure::qos_policy::EntityFactoryQosPolicy {
                autoenable_created_entities,
            },
        )
    }

    #[getter]
    pub fn get_autoenable_created_entities(&self) -> bool {
        self.0.autoenable_created_entities
    }
}

#[pyclass(frozen)]
#[derive(Clone, Default)]
pub struct TopicDataQosPolicy(dust_dds::infrastructure::qos_policy::TopicDataQosPolicy);

impl From<TopicDataQosPolicy> for dust_dds::infrastructure::qos_policy::TopicDataQosPolicy {
    fn from(value: TopicDataQosPolicy) -> Self {
        value.0
    }
}

#[pymethods]
impl TopicDataQosPolicy {
    #[new]
    pub fn new(value: Vec<u8>) -> Self {
        Self(dust_dds::infrastructure::qos_policy::TopicDataQosPolicy { value })
    }

    #[getter]
    pub fn get_value(&self) -> &[u8] {
        &self.0.value
    }
}

#[derive(Clone, Copy)]
#[pyclass]
pub enum DurabilityQosPolicyKind {
    Volatile,
    TransientLocal,
}

impl From<DurabilityQosPolicyKind>
    for dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind
{
    fn from(value: DurabilityQosPolicyKind) -> Self {
        match value {
            DurabilityQosPolicyKind::Volatile => {
                dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind::Volatile
            }
            DurabilityQosPolicyKind::TransientLocal => {
                dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind::TransientLocal
            }
        }
    }
}

impl From<dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind>
    for DurabilityQosPolicyKind
{
    fn from(value: dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind) -> Self {
        match value {
            dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind::Volatile => {
                DurabilityQosPolicyKind::Volatile
            }
            dust_dds::infrastructure::qos_policy::DurabilityQosPolicyKind::TransientLocal => {
                DurabilityQosPolicyKind::TransientLocal
            }
        }
    }
}

#[pyclass(frozen)]
#[derive(Clone, Default)]
pub struct DurabilityQosPolicy(dust_dds::infrastructure::qos_policy::DurabilityQosPolicy);

impl From<DurabilityQosPolicy> for dust_dds::infrastructure::qos_policy::DurabilityQosPolicy {
    fn from(value: DurabilityQosPolicy) -> Self {
        value.0
    }
}

#[pymethods]
impl DurabilityQosPolicy {
    #[new]
    pub fn new(kind: DurabilityQosPolicyKind) -> Self {
        Self(dust_dds::infrastructure::qos_policy::DurabilityQosPolicy { kind: kind.into() })
    }

    #[getter]
    pub fn get_kind(&self) -> DurabilityQosPolicyKind {
        self.0.kind.into()
    }
}

#[pyclass(frozen)]
#[derive(Clone, Default)]
pub struct DeadlineQosPolicy(dust_dds::infrastructure::qos_policy::DeadlineQosPolicy);

impl From<DeadlineQosPolicy> for dust_dds::infrastructure::qos_policy::DeadlineQosPolicy {
    fn from(value: DeadlineQosPolicy) -> Self {
        value.0
    }
}

#[pymethods]
impl DeadlineQosPolicy {
    #[new]
    pub fn new(period: DurationKind) -> Self {
        Self(dust_dds::infrastructure::qos_policy::DeadlineQosPolicy {
            period: period.into(),
        })
    }

    #[getter]
    pub fn get_period(&self) -> DurationKind {
        self.0.period.into()
    }
}
