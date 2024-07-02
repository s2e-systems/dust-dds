use pyo3::prelude::*;

use crate::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    infrastructure::{
        condition::StatusCondition,
        error::into_pyerr,
        instance::InstanceHandle,
        qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos},
        status::StatusKind,
        time::Time,
    },
    publication::{publisher::Publisher, publisher_listener::PublisherListener},
    subscription::{subcriber_listener::SubscriberListener, subscriber::Subscriber},
    topic_definition::{
        topic::Topic, topic_listener::TopicListener, type_support::PythonTypeRepresentation,
    },
};

use super::domain_participant_listener::DomainParticipantListener;

#[pyclass]
pub struct DomainParticipant(dust_dds::domain::domain_participant::DomainParticipant);

impl From<dust_dds::domain::domain_participant::DomainParticipant> for DomainParticipant {
    fn from(value: dust_dds::domain::domain_participant::DomainParticipant) -> Self {
        Self(value)
    }
}

impl AsRef<dust_dds::domain::domain_participant::DomainParticipant> for DomainParticipant {
    fn as_ref(&self) -> &dust_dds::domain::domain_participant::DomainParticipant {
        &self.0
    }
}

#[pymethods]
impl DomainParticipant {
    #[pyo3(signature = (qos=None, a_listener=None, mask=Vec::new()))]
    pub fn create_publisher(
        &self,
        qos: Option<PublisherQos>,
        a_listener: Option<Py<PyAny>>,
        mask: Vec<StatusKind>,
    ) -> PyResult<Publisher> {
        let qos = match qos {
            Some(q) => dust_dds::infrastructure::qos::QosKind::Specific(q.into()),
            None => dust_dds::infrastructure::qos::QosKind::Default,
        };

        let listener: Option<
            Box<dyn dust_dds::publication::publisher_listener::PublisherListener + Send>,
        > = match a_listener {
            Some(l) => Some(Box::new(PublisherListener::from(l))),
            None => None,
        };
        let mask: Vec<dust_dds::infrastructure::status::StatusKind> = mask
            .into_iter()
            .map(dust_dds::infrastructure::status::StatusKind::from)
            .collect();

        match self.0.create_publisher(qos, listener, &mask) {
            Ok(p) => Ok(p.into()),
            Err(e) => Err(into_pyerr(e)),
        }
    }

    pub fn delete_publisher(&self, a_publisher: &Publisher) -> PyResult<()> {
        match self.0.delete_publisher(a_publisher.as_ref()) {
            Ok(_) => Ok(()),
            Err(e) => Err(into_pyerr(e)),
        }
    }

    #[pyo3(signature = (qos=None, a_listener=None, mask=Vec::new()))]
    pub fn create_subscriber(
        &self,
        qos: Option<SubscriberQos>,
        a_listener: Option<Py<PyAny>>,
        mask: Vec<StatusKind>,
    ) -> PyResult<Subscriber> {
        let qos = match qos {
            Some(q) => dust_dds::infrastructure::qos::QosKind::Specific(q.into()),
            None => dust_dds::infrastructure::qos::QosKind::Default,
        };

        let listener: Option<
            Box<dyn dust_dds::subscription::subscriber_listener::SubscriberListener + Send>,
        > = match a_listener {
            Some(l) => Some(Box::new(SubscriberListener::from(l))),
            None => None,
        };
        let mask: Vec<dust_dds::infrastructure::status::StatusKind> = mask
            .into_iter()
            .map(dust_dds::infrastructure::status::StatusKind::from)
            .collect();

        match self.0.create_subscriber(qos, listener, &mask) {
            Ok(s) => Ok(s.into()),
            Err(e) => Err(into_pyerr(e)),
        }
    }

    pub fn delete_subscriber(&self, a_subscriber: &Subscriber) -> PyResult<()> {
        match self.0.delete_subscriber(a_subscriber.as_ref()) {
            Ok(_) => Ok(()),
            Err(e) => Err(into_pyerr(e)),
        }
    }

    #[pyo3(signature = (topic_name, type_, qos = None, a_listener = None, mask = Vec::new() ))]
    pub fn create_topic(
        &self,
        topic_name: String,
        type_: Py<PyAny>,
        qos: Option<TopicQos>,
        a_listener: Option<Py<PyAny>>,
        mask: Vec<StatusKind>,
    ) -> PyResult<Topic> {
        let qos = match qos {
            Some(q) => dust_dds::infrastructure::qos::QosKind::Specific(q.into()),
            None => dust_dds::infrastructure::qos::QosKind::Default,
        };

        let listener: Option<
            Box<dyn dust_dds::topic_definition::topic_listener::TopicListener + Send>,
        > = match a_listener {
            Some(l) => Some(Box::new(TopicListener::from(l))),
            None => None,
        };
        let mask: Vec<dust_dds::infrastructure::status::StatusKind> = mask
            .into_iter()
            .map(dust_dds::infrastructure::status::StatusKind::from)
            .collect();

        let type_name = Python::with_gil(|py| type_.getattr(py, "__name__"))?.to_string();

        let dynamic_type_representation = Box::new(PythonTypeRepresentation::from(type_));
        match self.0.create_dynamic_topic(
            &topic_name,
            &type_name,
            qos,
            listener,
            &mask,
            dynamic_type_representation,
        ) {
            Ok(t) => Ok(t.into()),
            Err(e) => Err(into_pyerr(e)),
        }
    }

    pub fn delete_topic(&self, a_topic: &Topic) -> PyResult<()> {
        match self.0.delete_topic(a_topic.as_ref()) {
            Ok(_) => Ok(()),
            Err(e) => Err(into_pyerr(e)),
        }
    }

    pub fn lookup_topicdescription(&self, topic_name: String) -> PyResult<Option<Topic>> {
        match self.0.lookup_topicdescription(&topic_name) {
            Ok(t) => Ok(t.map(Topic::from)),
            Err(e) => Err(into_pyerr(e)),
        }
    }

    pub fn get_builtin_subscriber(&self) -> Subscriber {
        self.0.get_builtin_subscriber().into()
    }

    pub fn ignore_participant(&self, handle: InstanceHandle) -> PyResult<()> {
        self.0.ignore_participant(handle.into()).map_err(into_pyerr)
    }

    pub fn ignore_topic(&self, handle: InstanceHandle) -> PyResult<()> {
        self.0.ignore_topic(handle.into()).map_err(into_pyerr)
    }

    pub fn ignore_publication(&self, handle: InstanceHandle) -> PyResult<()> {
        self.0.ignore_publication(handle.into()).map_err(into_pyerr)
    }

    pub fn ignore_subscription(&self, handle: InstanceHandle) -> PyResult<()> {
        self.0
            .ignore_subscription(handle.into())
            .map_err(into_pyerr)
    }

    pub fn get_domain_id(&self) -> i32 {
        self.0.get_domain_id()
    }

    pub fn delete_contained_entities(&self) -> PyResult<()> {
        self.0.delete_contained_entities().map_err(into_pyerr)
    }

    pub fn assert_liveliness(&self) -> PyResult<()> {
        self.0.assert_liveliness().map_err(into_pyerr)
    }

    pub fn set_default_publisher_qos(&self, qos: Option<PublisherQos>) -> PyResult<()> {
        let qos = match qos {
            Some(q) => dust_dds::infrastructure::qos::QosKind::Specific(q.into()),
            None => dust_dds::infrastructure::qos::QosKind::Default,
        };
        self.0.set_default_publisher_qos(qos).map_err(into_pyerr)
    }

    pub fn get_default_publisher_qos(&self) -> PyResult<PublisherQos> {
        Ok(self
            .0
            .get_default_publisher_qos()
            .map_err(into_pyerr)?
            .into())
    }

    pub fn set_default_subscriber_qos(&self, qos: Option<SubscriberQos>) -> PyResult<()> {
        let qos = match qos {
            Some(q) => dust_dds::infrastructure::qos::QosKind::Specific(q.into()),
            None => dust_dds::infrastructure::qos::QosKind::Default,
        };
        self.0.set_default_subscriber_qos(qos).map_err(into_pyerr)
    }

    pub fn get_default_subscriber_qos(&self) -> PyResult<SubscriberQos> {
        Ok(self
            .0
            .get_default_subscriber_qos()
            .map_err(into_pyerr)?
            .into())
    }

    pub fn set_default_topic_qos(&self, qos: Option<TopicQos>) -> PyResult<()> {
        let qos = match qos {
            Some(q) => dust_dds::infrastructure::qos::QosKind::Specific(q.into()),
            None => dust_dds::infrastructure::qos::QosKind::Default,
        };
        self.0.set_default_topic_qos(qos).map_err(into_pyerr)
    }

    pub fn get_default_topic_qos(&self) -> PyResult<TopicQos> {
        Ok(self.0.get_default_topic_qos().map_err(into_pyerr)?.into())
    }

    pub fn get_discovered_participants(&self) -> PyResult<Vec<InstanceHandle>> {
        Ok(self
            .0
            .get_discovered_participants()
            .map_err(into_pyerr)?
            .into_iter()
            .map(InstanceHandle::from)
            .collect())
    }

    pub fn get_discovered_participant_data(
        &self,
        participant_handle: InstanceHandle,
    ) -> PyResult<ParticipantBuiltinTopicData> {
        Ok(self
            .0
            .get_discovered_participant_data(participant_handle.into())
            .map_err(into_pyerr)?
            .into())
    }

    pub fn get_discovered_topics(&self) -> PyResult<Vec<InstanceHandle>> {
        Ok(self
            .0
            .get_discovered_topics()
            .map_err(into_pyerr)?
            .into_iter()
            .map(InstanceHandle::from)
            .collect())
    }

    pub fn get_discovered_topic_data(
        &self,
        topic_handle: InstanceHandle,
    ) -> PyResult<TopicBuiltinTopicData> {
        Ok(self
            .0
            .get_discovered_topic_data(topic_handle.into())
            .map_err(into_pyerr)?
            .into())
    }

    pub fn contains_entity(&self, a_handle: InstanceHandle) -> PyResult<bool> {
        self.0.contains_entity(a_handle.into()).map_err(into_pyerr)
    }

    pub fn get_current_time(&self) -> PyResult<Time> {
        match self.0.get_current_time() {
            Ok(t) => Ok(t.into()),
            Err(e) => Err(into_pyerr(e)),
        }
    }

    pub fn set_qos(&self, qos: Option<DomainParticipantQos>) -> PyResult<()> {
        let qos = match qos {
            Some(q) => dust_dds::infrastructure::qos::QosKind::Specific(q.into()),
            None => dust_dds::infrastructure::qos::QosKind::Default,
        };
        self.0.set_qos(qos).map_err(into_pyerr)
    }

    pub fn get_qos(&self) -> PyResult<DomainParticipantQos> {
        match self.0.get_qos() {
            Ok(q) => Ok(q.into()),
            Err(e) => Err(into_pyerr(e)),
        }
    }

    #[pyo3(signature = (a_listener = None, mask = Vec::new()))]
    pub fn set_listener(
        &self,
        a_listener: Option<Py<PyAny>>,
        mask: Vec<StatusKind>,
    ) -> PyResult<()> {
        let listener: Option<
            Box<
                dyn dust_dds::domain::domain_participant_listener::DomainParticipantListener + Send,
            >,
        > = match a_listener {
            Some(l) => Some(Box::new(DomainParticipantListener::from(l))),
            None => None,
        };
        let mask: Vec<dust_dds::infrastructure::status::StatusKind> = mask
            .into_iter()
            .map(dust_dds::infrastructure::status::StatusKind::from)
            .collect();
        self.0.set_listener(listener, &mask).map_err(into_pyerr)
    }

    pub fn get_statuscondition(&self) -> StatusCondition {
        self.0.get_statuscondition().into()
    }

    pub fn get_status_changes(&self) -> PyResult<Vec<StatusKind>> {
        Ok(self
            .0
            .get_status_changes()
            .map_err(into_pyerr)?
            .into_iter()
            .map(StatusKind::from)
            .collect())
    }

    pub fn enable(&self) -> PyResult<()> {
        self.0.enable().map_err(into_pyerr)
    }

    pub fn get_instance_handle(&self) -> PyResult<InstanceHandle> {
        Ok(self.0.get_instance_handle().map_err(into_pyerr)?.into())
    }
}
