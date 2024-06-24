use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct InstanceHandle(dust_dds::infrastructure::instance::InstanceHandle);

impl From<dust_dds::infrastructure::instance::InstanceHandle> for InstanceHandle {
    fn from(value: dust_dds::infrastructure::instance::InstanceHandle) -> Self {
        Self(value)
    }
}

impl From<InstanceHandle> for dust_dds::infrastructure::instance::InstanceHandle {
    fn from(value: InstanceHandle) -> Self {
        value.0
    }
}
