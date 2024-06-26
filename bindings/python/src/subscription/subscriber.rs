use pyo3::{exceptions::PyTypeError, prelude::*};

use crate::{
    infrastructure::{qos::DataReaderQos, status::StatusKind},
    topic_definition::{topic::Topic, type_support::MyDdsData},
};

use super::{data_reader::DataReader, data_reader_listener::DataReaderListener};

#[pyclass]
pub struct Subscriber(dust_dds::subscription::subscriber::Subscriber);

impl From<dust_dds::subscription::subscriber::Subscriber> for Subscriber {
    fn from(value: dust_dds::subscription::subscriber::Subscriber) -> Self {
        Self(value)
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
    pub fn create_datareader<'a>(
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

        let mask: Vec<dust_dds::infrastructure::status::StatusKind> =
            mask.into_iter().map(|m| m.into()).collect();

        let listener: Option<
            Box<
                dyn dust_dds::subscription::data_reader_listener::DataReaderListener<
                        Foo = MyDdsData,
                    > + Send,
            >,
        > = match a_listener {
            Some(l) => Some(Box::new(DataReaderListener::from(l))),
            None => None,
        };

        match self
            .0
            .create_datareader(a_topic.as_ref(), qos, listener, &mask)
        {
            Ok(dr) => Ok(dr.into()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }
}
