use pyo3::prelude::*;

use crate::{infrastructure::status::InconsistentTopicStatus, topic_definition::topic::Topic};

#[derive(Clone)]
pub struct TopicListener(Py<PyAny>);
impl From<Py<PyAny>> for TopicListener {
    fn from(value: Py<PyAny>) -> Self {
        Self(value)
    }
}

impl dust_dds::topic_definition::topic_listener::TopicListener for TopicListener {
    fn on_inconsistent_topic(
        &mut self,
        the_topic: dust_dds::topic_definition::topic::Topic,
        status: dust_dds::infrastructure::status::InconsistentTopicStatus,
    ) {
        let args = (
            Topic::from(the_topic),
            InconsistentTopicStatus::from(status),
        );
        Python::with_gil(|py| {
            self.0
                .bind(py)
                .call_method("on_inconsistent_topic", args, None)
                .unwrap();
        })
    }
}
