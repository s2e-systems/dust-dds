use dust_dds::infrastructure::status::NO_STATUS;
use pyo3::{exceptions::PyTypeError, prelude::*};

use crate::topic_definition::{topic::Topic, type_support::PythonDdsData};

use super::data_writer::DataWriter;

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
    pub fn create_datawriter<'a>(
        &self,
        a_topic: &Topic,
        // qos: QosKind<DataWriterQos>,
        // a_listener: Option<Box<dyn DataWriterListener<'a, Foo = MyDdsData> + Send + 'a>>,
        // mask: &[StatusKind],
    ) -> PyResult<DataWriter> {
        match self.0.create_datawriter::<PythonDdsData>(
            a_topic.as_ref(),
            dust_dds::infrastructure::qos::QosKind::Default,
            None,
            NO_STATUS,
        ) {
            Ok(dw) => Ok(dw.into()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }
}
