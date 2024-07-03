use dust_dds::infrastructure::qos::QosKind;
use pyo3::prelude::*;

use crate::infrastructure::{
    error::into_pyerr,
    qos::{DomainParticipantFactoryQos, DomainParticipantQos},
    status::StatusKind,
};

use super::{
    domain_participant::DomainParticipant, domain_participant_listener::DomainParticipantListener,
};

#[pyclass]
pub struct DomainParticipantFactory(
    &'static dust_dds::domain::domain_participant_factory::DomainParticipantFactory,
);

#[pymethods]
impl DomainParticipantFactory {
    #[staticmethod]
    pub fn get_instance() -> DomainParticipantFactory {
        Self(dust_dds::domain::domain_participant_factory::DomainParticipantFactory::get_instance())
    }

    #[pyo3(signature = (domain_id, qos = None, a_listener = None, mask = Vec::new()))]
    pub fn create_participant(
        &self,
        domain_id: i32,
        qos: Option<DomainParticipantQos>,
        a_listener: Option<Py<PyAny>>,
        mask: Vec<StatusKind>,
    ) -> PyResult<DomainParticipant> {
        let qos = match qos {
            Some(q) => dust_dds::infrastructure::qos::QosKind::Specific(q.into()),
            None => dust_dds::infrastructure::qos::QosKind::Default,
        };

        let mask: Vec<dust_dds::infrastructure::status::StatusKind> = mask
            .into_iter()
            .map(dust_dds::infrastructure::status::StatusKind::from)
            .collect();

        let listener: Option<
            Box<
                dyn dust_dds::domain::domain_participant_listener::DomainParticipantListener + Send,
            >,
        > = match a_listener {
            Some(l) => Some(Box::new(DomainParticipantListener::from(l))),
            None => None,
        };

        match self.0.create_participant(domain_id, qos, listener, &mask) {
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
