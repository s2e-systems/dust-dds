use dust_dds::infrastructure::{qos::QosKind, status::NO_STATUS};
use pyo3::prelude::*;

#[pyclass]
pub struct DomainParticipant(dust_dds::domain::domain_participant::DomainParticipant);

impl DomainParticipant {
    pub fn new(
        domain_participant: dust_dds::domain::domain_participant::DomainParticipant,
    ) -> Self {
        Self(domain_participant)
    }
}

#[pymethods]
impl DomainParticipant {
    pub fn create_publisher(&self) -> PyResult<()> {
        match self.0.create_publisher(QosKind::Default, None, NO_STATUS) {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        }
    }
}
