use dust_dds::infrastructure::{qos::QosKind, status::NO_STATUS};
use pyo3::{exceptions::PyTypeError, prelude::*};

use crate::{
    publication::publisher::Publisher,
    subscription::subscriber::Subscriber,
    topic_definition::{topic::Topic, type_support::MyDdsData},
};

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

    pub fn delete_publisher(&self, a_publisher: &Publisher) -> PyResult<()> {
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

    pub fn delete_subscriber(&self, a_subscriber: &Subscriber) -> PyResult<()> {
        match self.0.delete_subscriber(a_subscriber.as_ref()) {
            Ok(_) => Ok(()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }

    pub fn create_topic(&self, topic_name: String, type_name: String) -> PyResult<Topic> {
        match self.0.create_topic::<MyDdsData>(
            &topic_name,
            &type_name,
            QosKind::Default,
            None,
            NO_STATUS,
        ) {
            Ok(t) => Ok(t.into()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }

    pub fn delete_topic(&self, a_topic: &Topic) -> PyResult<()> {
        match self.0.delete_topic(a_topic.as_ref()) {
            Ok(_) => Ok(()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }
}
