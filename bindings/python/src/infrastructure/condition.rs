use pyo3::prelude::*;

use super::{error::into_pyerr, status::StatusKind};

#[pyclass]
#[derive(Clone)]
pub struct StatusCondition(dust_dds::infrastructure::condition::StatusCondition);

impl From<dust_dds::infrastructure::condition::StatusCondition> for StatusCondition {
    fn from(value: dust_dds::infrastructure::condition::StatusCondition) -> Self {
        Self(value)
    }
}

impl From<StatusCondition> for dust_dds::infrastructure::condition::StatusCondition {
    fn from(value: StatusCondition) -> Self {
        value.0
    }
}

#[pymethods]
impl StatusCondition {
    pub fn get_enabled_statuses(&self) -> PyResult<Vec<StatusKind>> {
        Ok(self
            .0
            .get_enabled_statuses()
            .map_err(into_pyerr)?
            .into_iter()
            .map(StatusKind::from)
            .collect())
    }

    pub fn set_enabled_statuses(&self, mask: Vec<StatusKind>) -> PyResult<()> {
        let mask: Vec<dust_dds::infrastructure::status::StatusKind> = mask
            .into_iter()
            .map(dust_dds::infrastructure::status::StatusKind::from)
            .collect();
        self.0.set_enabled_statuses(&mask).map_err(into_pyerr)
    }

    pub fn get_trigger_value(&self) -> PyResult<bool> {
        self.0.get_trigger_value().map_err(into_pyerr)
    }
}
