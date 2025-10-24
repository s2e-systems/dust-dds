use crate::{
    dds_async::{domain_participant::DomainParticipantAsync, topic::TopicAsync},
    infrastructure::error::DdsResult,
    runtime::DdsRuntime,
};
use alloc::string::String;

/// Async version of [`Topic`](crate::topic_definition::content_filtered_topic::ContentFilteredTopic).
pub struct ContentFilteredTopicAsync<R: DdsRuntime> {
    name: String,
    topic: TopicAsync<R>,
}

impl<R: DdsRuntime> Clone for ContentFilteredTopicAsync<R> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            topic: self.topic.clone(),
        }
    }
}

impl<R: DdsRuntime> ContentFilteredTopicAsync<R> {
    pub(crate) fn new(name: String, topic: TopicAsync<R>) -> Self {
        Self { name, topic }
    }
}

impl<R: DdsRuntime> ContentFilteredTopicAsync<R> {
    /// Async version of [`get_related_topic`](crate::topic_definition::content_filtered_topic::ContentFilteredTopic::get_related_topic).
    pub fn get_related_topic(&self) -> TopicAsync<R> {
        self.topic.clone()
    }

    /// Async version of [`get_expression_parameters`](crate::topic_definition::content_filtered_topic::ContentFilteredTopic::get_expression_parameters).
    pub async fn get_expression_parameters(&self) -> DdsResult<String> {
        todo!()
    }

    /// Async version of [`set_expression_parameters`](crate::topic_definition::content_filtered_topic::ContentFilteredTopic::set_expression_parameters).
    pub async fn set_expression_parameters(
        &self,
        _expression_parameters: &[String],
    ) -> DdsResult<()> {
        todo!()
    }
}

impl<R: DdsRuntime> ContentFilteredTopicAsync<R> {
    /// Async version of [`get_participant`](crate::topic_definition::content_filtered_topic::ContentFilteredTopic::get_participant).
    #[tracing::instrument(skip(self))]
    pub fn get_participant(&self) -> DomainParticipantAsync<R> {
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
