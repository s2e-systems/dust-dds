use pyo3::{exceptions::PyTypeError, prelude::*};

use crate::{
    domain::domain_participant::DomainParticipant,
    infrastructure::{
        condition::StatusCondition,
        error::into_pyerr,
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos},
        status::StatusKind,
        time::Duration,
    },
    topic_definition::{topic::Topic, type_support::PythonDdsData},
};

use super::{
    data_writer::DataWriter, data_writer_listener::DataWriterListener,
    publisher_listener::PublisherListener,
};

#[pyclass]
pub struct Publisher(dust_dds::publication::publisher::Publisher);

impl From<dust_dds::publication::publisher::Publisher> for Publisher {
    fn from(value: dust_dds::publication::publisher::Publisher) -> Self {
        Self(value)
    }
}

impl AsRef<dust_dds::publication::publisher::Publisher> for Publisher {
    fn as_ref(&self) -> &dust_dds::publication::publisher::Publisher {
        &self.0
    }
}

#[pymethods]
impl Publisher {
    #[pyo3(signature = (a_topic, qos = None, a_listener = None, mask = Vec::new()))]
    pub fn create_datawriter(
        &self,
        a_topic: &Topic,
        qos: Option<DataWriterQos>,
        a_listener: Option<Py<PyAny>>,
        mask: Vec<StatusKind>,
    ) -> PyResult<DataWriter> {
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
                dyn dust_dds::publication::data_writer_listener::DataWriterListener<
                        Foo = PythonDdsData,
                    > + Send,
            >,
        > = match a_listener {
            Some(l) => Some(Box::new(DataWriterListener::from(l))),
            None => None,
        };
        match self
            .0
            .create_datawriter::<PythonDdsData>(a_topic.as_ref(), qos, listener, &mask)
        {
            Ok(dw) => Ok(dw.into()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }

    pub fn delete_datawriter(&self, a_datawriter: &DataWriter) -> PyResult<()> {
        self.0
            .delete_datawriter(a_datawriter.as_ref())
            .map_err(into_pyerr)
    }

    pub fn lookup_datawriter(&self, topic_name: &str) -> PyResult<Option<DataWriter>> {
        Ok(self
            .0
            .lookup_datawriter(topic_name)
            .map_err(into_pyerr)?
            .map(DataWriter::from))
    }

    pub fn suspend_publications(&self) -> PyResult<()> {
        self.0.suspend_publications().map_err(into_pyerr)
    }

    pub fn resume_publications(&self) -> PyResult<()> {
        self.0.resume_publications().map_err(into_pyerr)
    }

    pub fn begin_coherent_changes(&self) -> PyResult<()> {
        self.0.begin_coherent_changes().map_err(into_pyerr)
    }

    pub fn end_coherent_changes(&self) -> PyResult<()> {
        self.0.end_coherent_changes().map_err(into_pyerr)
    }

    pub fn wait_for_acknowledgments(&self, max_wait: Duration) -> PyResult<()> {
        self.0
            .wait_for_acknowledgments(max_wait.into())
            .map_err(into_pyerr)
    }

    pub fn get_participant(&self) -> DomainParticipant {
        self.0.get_participant().into()
    }

    pub fn delete_contained_entities(&self) -> PyResult<()> {
        self.0.delete_contained_entities().map_err(into_pyerr)
    }

    pub fn set_default_datawriter_qos(&self, qos: Option<DataWriterQos>) -> PyResult<()> {
        let qos = match qos {
            Some(q) => dust_dds::infrastructure::qos::QosKind::Specific(q.into()),
            None => dust_dds::infrastructure::qos::QosKind::Default,
        };
        self.0.set_default_datawriter_qos(qos).map_err(into_pyerr)
    }

    pub fn get_default_datawriter_qos(&self) -> PyResult<DataWriterQos> {
        Ok(self
            .0
            .get_default_datawriter_qos()
            .map_err(into_pyerr)?
            .into())
    }

    pub fn set_qos(&self, qos: Option<PublisherQos>) -> PyResult<()> {
        let qos = match qos {
            Some(q) => dust_dds::infrastructure::qos::QosKind::Specific(q.into()),
            None => dust_dds::infrastructure::qos::QosKind::Default,
        };
        self.0.set_qos(qos).map_err(into_pyerr)
    }

    pub fn get_qos(&self) -> PyResult<PublisherQos> {
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
            Box<dyn dust_dds::publication::publisher_listener::PublisherListener + Send>,
        > = match a_listener {
            Some(l) => Some(Box::new(PublisherListener::from(l))),
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
