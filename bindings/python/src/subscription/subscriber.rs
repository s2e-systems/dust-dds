use dust_dds::listener::NoOpListener;
use pyo3::{exceptions::PyTypeError, prelude::*};

use crate::{
    domain::domain_participant::DomainParticipant,
    infrastructure::{
        condition::StatusCondition,
        error::into_pyerr,
        instance::InstanceHandle,
        qos::{DataReaderQos, SubscriberQos},
        status::{SampleLostStatus, StatusKind},
    },
    topic_definition::topic::Topic,
};

use super::{
    data_reader::DataReader, data_reader_listener::DataReaderListener,
    subcriber_listener::SubscriberListener,
};

#[pyclass]
pub struct Subscriber(dust_dds::subscription::subscriber::Subscriber);

impl From<dust_dds::subscription::subscriber::Subscriber> for Subscriber {
    fn from(value: dust_dds::subscription::subscriber::Subscriber) -> Self {
        Self(value)
    }
}

impl From<dust_dds::dds_async::subscriber::SubscriberAsync> for Subscriber {
    fn from(value: dust_dds::dds_async::subscriber::SubscriberAsync) -> Self {
        Self(dust_dds::subscription::subscriber::Subscriber::from(value))
    }
}

impl AsRef<dust_dds::subscription::subscriber::Subscriber> for Subscriber {
    fn as_ref(&self) -> &dust_dds::subscription::subscriber::Subscriber {
        &self.0
    }
}

#[pymethods]
impl Subscriber {
    #[pyo3(signature = (a_topic, qos = None, a_listener = None, mask = Vec::new()))]
    pub fn create_datareader(
        &self,
        a_topic: &Topic,
        qos: Option<DataReaderQos>,
        a_listener: Option<Py<PyAny>>,
        mask: Vec<StatusKind>,
    ) -> PyResult<DataReader> {
        let qos = match qos {
            Some(q) => dust_dds::infrastructure::qos::QosKind::Specific(q.into()),
            None => dust_dds::infrastructure::qos::QosKind::Default,
        };

        let mask: Vec<dust_dds::infrastructure::status::StatusKind> = mask
            .into_iter()
            .map(dust_dds::infrastructure::status::StatusKind::from)
            .collect();

        let r = match a_listener {
            Some(l) => {
                self.0
                    .create_datareader(a_topic.as_ref(), qos, DataReaderListener::from(l), &mask)
            }
            None => self
                .0
                .create_datareader(a_topic.as_ref(), qos, NoOpListener, &mask),
        };

        match r {
            Ok(dr) => Ok(dr.into()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }

    pub fn delete_datareader(&self, a_datareader: &DataReader) -> PyResult<()> {
        self.0
            .delete_datareader(a_datareader.as_ref())
            .map_err(into_pyerr)
    }

    pub fn lookup_datareader(&self, topic_name: &str) -> PyResult<Option<DataReader>> {
        Ok(self
            .0
            .lookup_datareader(topic_name)
            .map_err(into_pyerr)?
            .map(DataReader::from))
    }

    pub fn notify_datareaders(&self) -> PyResult<()> {
        self.0.notify_datareaders().map_err(into_pyerr)
    }

    pub fn get_participant(&self) -> DomainParticipant {
        self.0.get_participant().into()
    }

    pub fn get_sample_lost_status(&self) -> PyResult<SampleLostStatus> {
        Ok(self.0.get_sample_lost_status().map_err(into_pyerr)?.into())
    }

    pub fn delete_contained_entities(&self) -> PyResult<()> {
        self.0.delete_contained_entities().map_err(into_pyerr)
    }

    pub fn set_default_datareader_qos(&self, qos: Option<DataReaderQos>) -> PyResult<()> {
        let qos = match qos {
            Some(q) => dust_dds::infrastructure::qos::QosKind::Specific(q.into()),
            None => dust_dds::infrastructure::qos::QosKind::Default,
        };
        self.0.set_default_datareader_qos(qos).map_err(into_pyerr)
    }

    pub fn get_default_datareader_qos(&self) -> PyResult<DataReaderQos> {
        Ok(self
            .0
            .get_default_datareader_qos()
            .map_err(into_pyerr)?
            .into())
    }

    // pub fn copy_from_topic_qos(
    //     _a_datareader_qos: &mut DataReaderQos,
    //     _a_topic_qos: &TopicQos,
    // ) -> DdsResult<()> {
    //     )
    // }

    pub fn set_qos(&self, qos: Option<SubscriberQos>) -> PyResult<()> {
        let qos = match qos {
            Some(q) => dust_dds::infrastructure::qos::QosKind::Specific(q.into()),
            None => dust_dds::infrastructure::qos::QosKind::Default,
        };
        self.0.set_qos(qos).map_err(into_pyerr)
    }

    pub fn get_qos(&self) -> PyResult<SubscriberQos> {
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
            Some(l) => self.0.set_listener(SubscriberListener::from(l), &mask),
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
