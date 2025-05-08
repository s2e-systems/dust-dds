use dust_dds::listener::NoOpListener;
use pyo3::prelude::*;

use crate::{
    domain::domain_participant::DomainParticipant,
    infrastructure::{
        condition::StatusCondition,
        error::into_pyerr,
        instance::InstanceHandle,
        qos::TopicQos,
        status::{InconsistentTopicStatus, StatusKind},
    },
};

use super::topic_listener::TopicListener;

#[pyclass]
pub struct Topic(dust_dds::topic_definition::topic::Topic<dust_dds::runtime::StdRuntime>);

impl AsRef<dust_dds::topic_definition::topic::Topic<dust_dds::runtime::StdRuntime>> for Topic {
    fn as_ref(&self) -> &dust_dds::topic_definition::topic::Topic<dust_dds::runtime::StdRuntime> {
        &self.0
    }
}

impl From<dust_dds::topic_definition::topic::Topic<dust_dds::runtime::StdRuntime>> for Topic {
    fn from(
        value: dust_dds::topic_definition::topic::Topic<dust_dds::runtime::StdRuntime>,
    ) -> Self {
        Self(value)
    }
}

impl From<dust_dds::dds_async::topic::TopicAsync<dust_dds::runtime::StdRuntime>> for Topic {
    fn from(value: dust_dds::dds_async::topic::TopicAsync<dust_dds::runtime::StdRuntime>) -> Self {
        Self(dust_dds::topic_definition::topic::Topic::from(value))
    }
}

#[pymethods]
impl Topic {
    pub fn get_inconsistent_topic_status(&self) -> PyResult<InconsistentTopicStatus> {
        Ok(self
            .0
            .get_inconsistent_topic_status()
            .map_err(into_pyerr)?
            .into())
    }

    pub fn get_participant(&self) -> DomainParticipant {
        self.0.get_participant().into()
    }

    pub fn get_type_name(&self) -> String {
        self.0.get_type_name()
    }

    pub fn get_name(&self) -> String {
        self.0.get_name()
    }

    pub fn set_qos(&self, qos: Option<TopicQos>) -> PyResult<()> {
        let qos = match qos {
            Some(q) => dust_dds::infrastructure::qos::QosKind::Specific(q.into()),
            None => dust_dds::infrastructure::qos::QosKind::Default,
        };
        self.0.set_qos(qos).map_err(into_pyerr)
    }

    pub fn get_qos(&self) -> PyResult<TopicQos> {
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
        let mask: Vec<dust_dds::infrastructure::status::StatusKind> = mask
            .into_iter()
            .map(dust_dds::infrastructure::status::StatusKind::from)
            .collect();
        match a_listener {
            Some(l) => self.0.set_listener(TopicListener::from(l), &mask),
            None => self.0.set_listener(NoOpListener, &mask),
        }
        .map_err(into_pyerr)
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

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.0.get_instance_handle().into()
    }
}
