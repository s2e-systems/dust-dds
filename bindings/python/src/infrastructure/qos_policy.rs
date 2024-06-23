use pyo3::prelude::*;

#[pyclass]
pub struct UserDataQosPolicy(dust_dds::infrastructure::qos_policy::UserDataQosPolicy);

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

    #[setter]
    pub fn set_value(&mut self, value: Vec<u8>) {
        self.0.value = value;
    }
}

#[pyclass]
pub struct EntityFactoryQosPolicy(dust_dds::infrastructure::qos_policy::EntityFactoryQosPolicy);

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

    #[setter]
    pub fn set_autoenable_created_entities(&mut self, value: bool) {
        self.0.autoenable_created_entities = value;
    }
}
