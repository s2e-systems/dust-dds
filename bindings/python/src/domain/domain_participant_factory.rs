use dust_dds::infrastructure::{qos::QosKind, status::NO_STATUS};
use pyo3::prelude::*;

use crate::infrastructure::{
    error::into_pyerr,
    qos::{DomainParticipantFactoryQos, DomainParticipantQos},
};

use super::domain_participant::DomainParticipant;

#[pyclass]
pub struct DomainParticipantFactory(
    &'static dust_dds::domain::domain_participant_factory::DomainParticipantFactory,
);

#[pymethods]
impl DomainParticipantFactory {
    #[new]
    pub fn get_instance() -> Self {
        Self(dust_dds::domain::domain_participant_factory::DomainParticipantFactory::get_instance())
    }

    pub fn create_participant(
        &self,
        domain_id: i32,
        qos: Option<DomainParticipantQos>,
    ) -> PyResult<DomainParticipant> {
        let qos = match qos {
            Some(q) => dust_dds::infrastructure::qos::QosKind::Specific(q.into()),
            None => dust_dds::infrastructure::qos::QosKind::Default,
        };

        match self.0.create_participant(domain_id, qos, None, NO_STATUS) {
            Ok(dp) => Ok(dp.into()),
            Err(e) => Err(into_pyerr(e)),
        }
    }

    pub fn delete_participant(&self, a_participant: &DomainParticipant) -> PyResult<()> {
        self.0
            .delete_participant(a_participant.as_ref())
            .map_err(into_pyerr)
    }

    pub fn lookup_participant(&self, domain_id: i32) -> PyResult<Option<DomainParticipant>> {
        match self.0.lookup_participant(domain_id) {
            Ok(dp) => Ok(dp.map(DomainParticipant::from)),
            Err(e) => Err(into_pyerr(e)),
        }
    }

    pub fn set_default_participant_qos(&self, qos: Option<DomainParticipantQos>) -> PyResult<()> {
        match qos {
            Some(q) => self
                .0
                .set_default_participant_qos(QosKind::Specific(q.into())),
            None => self.0.set_default_participant_qos(QosKind::Default),
        }
        .map_err(into_pyerr)
    }

    pub fn get_default_participant_qos(&self) -> PyResult<DomainParticipantQos> {
        match self.0.get_default_participant_qos() {
            Ok(q) => Ok(q.into()),
            Err(e) => Err(into_pyerr(e)),
        }
    }

    pub fn set_qos(&self, qos: Option<DomainParticipantFactoryQos>) -> PyResult<()> {
        match qos {
            Some(q) => self.0.set_qos(QosKind::Specific(q.into())),
            None => self.0.set_qos(QosKind::Default),
        }
        .map_err(into_pyerr)
    }

    pub fn get_qos(&self) -> PyResult<DomainParticipantFactoryQos> {
        match self.0.get_qos() {
            Ok(q) => Ok(q.into()),
            Err(e) => Err(into_pyerr(e)),
        }
    }
}
