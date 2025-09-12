use crate::{dds_async::topic::TopicAsync, infrastructure::error::DdsResult, runtime::DdsRuntime};

/// Async version of [`Topic`](crate::topic_definition::content_filtered_topic::ContentFilteredTopic).

pub struct ContentFilteredTopicAsync<R: DdsRuntime> {
    topic: TopicAsync<R>,
}

impl<R: DdsRuntime> Clone for ContentFilteredTopicAsync<R> {
    fn clone(&self) -> Self {
        Self {
            topic: self.topic.clone(),
        }
    }
}

impl<R: DdsRuntime> ContentFilteredTopicAsync<R> {
    pub(crate) fn new(topic: TopicAsync<R>) -> Self {
        Self { topic }
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
        expression_parameters: &[String],
    ) -> DdsResult<()> {
        todo!()
    }
}
