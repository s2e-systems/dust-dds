use pyo3::prelude::*;

#[pyclass]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Duration(dust_dds::infrastructure::time::Duration);

impl From<Duration> for dust_dds::infrastructure::time::Duration {
    fn from(value: Duration) -> Self {
        value.0
    }
}

impl From<dust_dds::infrastructure::time::Duration> for Duration {
    fn from(value: dust_dds::infrastructure::time::Duration) -> Self {
        Self(value)
    }
}

#[pymethods]
impl Duration {
    #[new]
    pub fn new(sec: i32, nanosec: u32) -> Self {
        Self(dust_dds::infrastructure::time::Duration::new(sec, nanosec))
    }

    #[getter]
    pub fn get_sec(&self) -> i32 {
        self.0.sec()
    }

    #[getter]
    pub fn get_nanosec(&self) -> u32 {
        self.0.nanosec()
    }
}
