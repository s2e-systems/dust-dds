use dust_dds::infrastructure::{qos::QosKind, status::NO_STATUS};
use pyo3::prelude::*;

use crate::{
    infrastructure::{error::into_pyerr, instance::InstanceHandle},
    publication::publisher::Publisher,
    subscription::subscriber::Subscriber,
    topic_definition::{topic::Topic, type_support::MyDdsData},
};

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
    pub fn create_publisher(&self) -> PyResult<Publisher> {
        match self.0.create_publisher(QosKind::Default, None, NO_STATUS) {
            Ok(p) => Ok(p.into()),
            Err(_) => todo!(),
        }
    }

    pub fn delete_publisher(&self, a_publisher: &Publisher) -> PyResult<()> {
        match self.0.delete_publisher(a_publisher.as_ref()) {
            Ok(_) => Ok(()),
            Err(e) => Err(into_pyerr(e)),
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
            Err(e) => Err(into_pyerr(e)),
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
            Ok(t) => Ok(t.map(|t| t.into())),
            Err(_) => todo!(),
        }
    }

    pub fn get_builtin_subscriber(&self) -> Subscriber {
        self.0.get_builtin_subscriber().into()
    }

    pub fn ignore_participant(&self, handle: InstanceHandle) -> PyResult<()> {
        self.0
            .ignore_participant(handle.into())
            .map_err(|e| into_pyerr(e))
    }

    pub fn ignore_topic(&self, handle: InstanceHandle) -> PyResult<()> {
        self.0
            .ignore_topic(handle.into())
            .map_err(|e| into_pyerr(e))
    }

    pub fn ignore_publication(&self, handle: InstanceHandle) -> PyResult<()> {
        self.0
            .ignore_publication(handle.into())
            .map_err(|e| into_pyerr(e))
    }

    pub fn ignore_subscription(&self, handle: InstanceHandle) -> PyResult<()> {
        self.0
            .ignore_subscription(handle.into())
            .map_err(|e| into_pyerr(e))
    }

    pub fn get_domain_id(&self) -> dust_dds::domain::domain_participant_factory::DomainId {
        self.0.get_domain_id()
    }

    pub fn delete_contained_entities(&self) -> PyResult<()> {
        self.0
            .delete_contained_entities()
            .map_err(|e| into_pyerr(e))
    }

    pub fn assert_liveliness(&self) -> PyResult<()> {
        self.0.assert_liveliness().map_err(|e| into_pyerr(e))
    }

    // pub fn set_default_publisher_qos(&self, qos: QosKind<PublisherQos>) -> PyResult<()> {}
    // pub fn get_default_publisher_qos(&self) -> PyResult<PublisherQos> {}
    // pub fn set_default_subscriber_qos(&self, qos: QosKind<SubscriberQos>) -> PyResult<()> {}
    // pub fn get_default_subscriber_qos(&self) -> PyResult<SubscriberQos> {}
    // pub fn set_default_topic_qos(&self, qos: QosKind<TopicQos>) -> PyResult<()> {}
    // pub fn get_default_topic_qos(&self) -> PyResult<TopicQos> {}
    // pub fn get_discovered_participants(&self) -> PyResult<Vec<InstanceHandle>> {}
    // pub fn get_discovered_participant_data(
    //     &self,
    //     participant_handle: InstanceHandle,
    // ) -> PyResult<ParticipantBuiltinTopicData> {
    // }
    // pub fn get_discovered_topics(&self) -> PyResult<Vec<InstanceHandle>> {}
    // pub fn get_discovered_topic_data(
    //     &self,
    //     topic_handle: InstanceHandle,
    // ) -> PyResult<TopicBuiltinTopicData> {
    // }
    // pub fn contains_entity(&self, a_handle: InstanceHandle) -> PyResult<bool> {}
    // pub fn get_current_time(&self) -> PyResult<Time> {}
    // pub fn set_qos(&self, qos: QosKind<DomainParticipantQos>) -> PyResult<()> {}\
    // pub fn get_qos(&self) -> PyResult<DomainParticipantQos> {}
    // pub fn set_listener(
    //     &self,
    //     a_listener: Option<Box<dyn DomainParticipantListener + Send + 'static>>,
    //     mask: &[StatusKind],
    // ) -> PyResult<()> {}
    // pub fn get_statuscondition(&self) -> StatusCondition {}
    // pub fn get_status_changes(&self) -> PyResult<Vec<StatusKind>> {}
    // pub fn enable(&self) -> PyResult<()> {}
    // pub fn get_instance_handle(&self) -> PyResult<InstanceHandle> {}
}
