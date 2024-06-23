use pyo3::prelude::*;

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
