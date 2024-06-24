use pyo3::prelude::*;

#[pyclass]
pub struct Topic(dust_dds::topic_definition::topic::Topic);

impl AsRef<dust_dds::topic_definition::topic::Topic> for Topic {
    fn as_ref(&self) -> &dust_dds::topic_definition::topic::Topic {
        &self.0
    }
}

impl From<dust_dds::topic_definition::topic::Topic> for Topic {
    fn from(value: dust_dds::topic_definition::topic::Topic) -> Self {
        Self(value)
    }
}

#[pymethods]
impl Topic {}
