use crate::{
    dds_async::{
        content_filtered_topic::ContentFilteredTopicAsync,
        domain_participant::DomainParticipantAsync, topic::TopicAsync,
    },
    runtime::DdsRuntime,
};

/// Async version of [`TopicDescription`](crate::topic_definition::topic_description::TopicDescription).
pub enum TopicDescriptionAsync<R: DdsRuntime> {
    /// Async topic
    Topic(TopicAsync<R>),
    /// Content filtered topic
    ContentFilteredTopic(ContentFilteredTopicAsync<R>),
}

impl<R: DdsRuntime> Clone for TopicDescriptionAsync<R> {
    fn clone(&self) -> Self {
        match self {
            Self::Topic(arg0) => Self::Topic(arg0.clone()),
            Self::ContentFilteredTopic(arg0) => Self::ContentFilteredTopic(arg0.clone()),
        }
    }
}

impl<R: DdsRuntime> TopicDescriptionAsync<R> {
    /// Async version of [`get_participant`](crate::topic_definition::topic::Topic::get_participant).
    #[tracing::instrument(skip(self))]
    pub fn get_participant(&self) -> DomainParticipantAsync<R> {
        match self {
            Self::Topic(t) => t.get_participant(),
            Self::ContentFilteredTopic(t) => t.get_participant(),
        }
    }

    /// Async version of [`get_type_name`](crate::topic_definition::topic::Topic::get_type_name).
    #[tracing::instrument(skip(self))]
    pub fn get_type_name(&self) -> String {
        match self {
            Self::Topic(t) => t.get_type_name(),
            Self::ContentFilteredTopic(t) => t.get_type_name(),
        }
    }

    /// Async version of [`get_name`](crate::topic_definition::topic::Topic::get_name).
    #[tracing::instrument(skip(self))]
    pub fn get_name(&self) -> String {
        match self {
            Self::Topic(t) => t.get_name(),
            Self::ContentFilteredTopic(t) => t.get_name(),
        }
    }
}
