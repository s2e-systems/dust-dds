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

#[pyclass]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum DurationKind {
    Finite { duration: Duration },
    Infinite {},
}

impl From<DurationKind> for dust_dds::infrastructure::time::DurationKind {
    fn from(value: DurationKind) -> Self {
        match value {
            DurationKind::Finite { duration } => {
                dust_dds::infrastructure::time::DurationKind::Finite(duration.into())
            }
            DurationKind::Infinite {} => dust_dds::infrastructure::time::DurationKind::Infinite,
        }
    }
}

impl From<dust_dds::infrastructure::time::DurationKind> for DurationKind {
    fn from(value: dust_dds::infrastructure::time::DurationKind) -> Self {
        match value {
            dust_dds::infrastructure::time::DurationKind::Finite(duration) => {
                DurationKind::Finite {
                    duration: duration.into(),
                }
            }
            dust_dds::infrastructure::time::DurationKind::Infinite => DurationKind::Infinite {},
        }
    }
}