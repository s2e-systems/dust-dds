use dust_dds::infrastructure::status::NO_STATUS;
use pyo3::{exceptions::PyTypeError, prelude::*};

use crate::topic_definition::topic::Topic;

use super::data_reader::DataReader;

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
    pub fn create_datareader<'a>(
        &self,
        a_topic: &Topic,
        // qos: QosKind<DataReaderQos>,
        // a_listener: Option<Box<dyn DataReaderListener<'a, Foo = Foo> + Send + 'a>>,
        // mask: &[StatusKind],
    ) -> PyResult<DataReader> {
        match self.0.create_datareader(
            a_topic.as_ref(),
            dust_dds::infrastructure::qos::QosKind::Default,
            None,
            NO_STATUS,
        ) {
            Ok(dr) => Ok(dr.into()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }
}
