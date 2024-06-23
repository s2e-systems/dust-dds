use dust_dds::{
    domain::domain_participant_factory::DomainId,
    infrastructure::{qos::QosKind, status::NO_STATUS},
};
use pyo3::{exceptions::PyTypeError, prelude::*};

use super::domain_participant::DomainParticipant;

#[pyclass]
pub struct DomainParticipantFactory(
    &'static dust_dds::domain::domain_participant_factory::DomainParticipantFactory,
);

#[pymethods]
impl DomainParticipantFactory {
    pub fn create_participant(&self, domain_id: DomainId) -> PyResult<DomainParticipant> {
        match self
            .0
            .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        {
            Ok(dp) => Ok(DomainParticipant::new(dp)),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }

    #[staticmethod]
    pub fn get_instance() -> Self {
        Self(dust_dds::domain::domain_participant_factory::DomainParticipantFactory::get_instance())
    }
}
