use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct TopicDescription(dust_dds::topic_definition::topic_description::TopicDescription);

impl From<dust_dds::topic_definition::topic_description::TopicDescription> for TopicDescription {
    fn from(value: dust_dds::topic_definition::topic_description::TopicDescription) -> Self {
        Self(value)
    }
}

impl AsRef<dust_dds::topic_definition::topic_description::TopicDescription> for TopicDescription {
    fn as_ref(&self) -> &dust_dds::topic_definition::topic_description::TopicDescription {
        &self.0
    }
}
