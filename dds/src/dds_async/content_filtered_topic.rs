use crate::{
    dds_async::{domain_participant::DomainParticipantAsync, topic::TopicAsync},
    infrastructure::error::DdsResult,
};
use alloc::{string::String, vec::Vec};

/// Async version of [`Topic`](crate::topic_definition::content_filtered_topic::ContentFilteredTopic).
pub struct ContentFilteredTopicAsync {
    name: String,
    topic: TopicAsync,
}

impl Clone for ContentFilteredTopicAsync {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            topic: self.topic.clone(),
        }
    }
}

impl ContentFilteredTopicAsync {
    pub(crate) fn new(name: String, topic: TopicAsync) -> Self {
        Self { name, topic }
    }
}

impl ContentFilteredTopicAsync {
    /// Async version of [`get_related_topic`](crate::topic_definition::content_filtered_topic::ContentFilteredTopic::get_related_topic).
    pub fn get_related_topic(&self) -> TopicAsync {
        self.topic.clone()
    }

    /// Async version of [`get_expression_parameters`](crate::topic_definition::content_filtered_topic::ContentFilteredTopic::get_expression_parameters).
    pub async fn get_expression_parameters(&self) -> DdsResult<Vec<String>> {
        todo!()
    }

    /// Async version of [`set_expression_parameters`](crate::topic_definition::content_filtered_topic::ContentFilteredTopic::set_expression_parameters).
    pub async fn set_expression_parameters(
        &self,
        _expression_parameters: Vec<String>,
    ) -> DdsResult<()> {
        todo!()
    }
}

impl ContentFilteredTopicAsync {
    /// Async version of [`get_participant`](crate::topic_definition::content_filtered_topic::ContentFilteredTopic::get_participant).
    #[tracing::instrument(skip(self))]
    pub fn get_participant(&self) -> DomainParticipantAsync {
        self.topic.get_participant()
    }

    /// Async version of [`get_type_name`](crate::topic_definition::content_filtered_topic::ContentFilteredTopic::get_type_name).
    #[tracing::instrument(skip(self))]
    pub fn get_type_name(&self) -> String {
        self.topic.get_type_name()
    }

    /// Async version of [`get_name`](crate::topic_definition::content_filtered_topic::ContentFilteredTopic::get_name).
    #[tracing::instrument(skip(self))]
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}
