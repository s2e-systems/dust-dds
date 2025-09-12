use crate::{dds_async::topic::TopicAsync, runtime::DdsRuntime};

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
