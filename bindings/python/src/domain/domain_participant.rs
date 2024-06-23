use dust_dds::infrastructure::{qos::QosKind, status::NO_STATUS};
use pyo3::{exceptions::PyTypeError, prelude::*};

use crate::{publication::publisher::Publisher, subscription::subscriber::Subscriber};

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
    pub fn create_publisher(&self) -> PyResult<Publisher> {
        match self.0.create_publisher(QosKind::Default, None, NO_STATUS) {
            Ok(p) => Ok(p.into()),
            Err(_) => todo!(),
        }
    }

    pub fn delete_publisher(&self, a_publisher: PyRef<'_, Publisher>) -> PyResult<()> {
        match self.0.delete_publisher(a_publisher.as_ref()) {
            Ok(_) => Ok(()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }

    pub fn create_subscriber(&self) -> PyResult<Subscriber> {
        match self.0.create_subscriber(QosKind::Default, None, NO_STATUS) {
            Ok(s) => Ok(s.into()),
            Err(_) => todo!(),
        }
    }

    pub fn delete_subscriber(&self, a_subscriber: PyRef<'_, Subscriber>) -> PyResult<()> {
        match self.0.delete_subscriber(a_subscriber.as_ref()) {
            Ok(_) => Ok(()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }
}
