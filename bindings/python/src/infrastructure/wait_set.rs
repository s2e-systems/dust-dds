use pyo3::prelude::*;

use super::{condition::StatusCondition, error::into_pyerr, time::Duration};

#[pyclass]
#[derive(Clone)]
pub enum Condition {
    StatusCondition { condition: StatusCondition },
}

impl From<dust_dds::infrastructure::wait_set::Condition> for Condition {
    fn from(value: dust_dds::infrastructure::wait_set::Condition) -> Self {
        match value {
            dust_dds::infrastructure::wait_set::Condition::StatusCondition(c) => {
                Condition::StatusCondition {
                    condition: c.into(),
                }
            }
        }
    }
}

impl From<Condition> for dust_dds::infrastructure::wait_set::Condition {
    fn from(value: Condition) -> Self {
        match value {
            Condition::StatusCondition { condition } => {
                dust_dds::infrastructure::wait_set::Condition::StatusCondition(condition.into())
            }
        }
    }
}

#[pyclass]
#[derive(Default)]
pub struct WaitSet(dust_dds::infrastructure::wait_set::WaitSet);

#[pymethods]
impl WaitSet {
    #[new]
    pub fn new() -> Self {
        Self(dust_dds::infrastructure::wait_set::WaitSet::new())
    }

    pub fn wait(&self, timeout: Duration) -> PyResult<Vec<Condition>> {
        Ok(self
            .0
            .wait(timeout.into())
            .map_err(into_pyerr)?
            .into_iter()
            .map(Condition::from)
            .collect())
    }

    pub fn attach_condition(&mut self, cond: Condition) -> PyResult<()> {
        self.0.attach_condition(cond.into()).map_err(into_pyerr)
    }

    pub fn detach_condition(&self, cond: Condition) -> PyResult<()> {
        self.0.detach_condition(cond.into()).map_err(into_pyerr)
    }

    pub fn get_conditions(&self) -> PyResult<Vec<Condition>> {
        Ok(self
            .0
            .get_conditions()
            .map_err(into_pyerr)?
            .into_iter()
            .map(Condition::from)
            .collect())
    }
}
