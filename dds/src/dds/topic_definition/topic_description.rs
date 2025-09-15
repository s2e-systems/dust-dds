use crate::{
    dds_async::topic_description::TopicDescriptionAsync,
    domain::domain_participant::DomainParticipant,
    runtime::DdsRuntime,
    topic_definition::{content_filtered_topic::ContentFilteredTopic, topic::Topic},
};
use alloc::string::String;

/// This class is an enumrator for different topic types.
pub enum TopicDescription<R: DdsRuntime> {
    /// Topic type
    Topic(Topic<R>),
    /// Content filtered topic
    ContentFilteredTopic(ContentFilteredTopic<R>),
}

impl<R: DdsRuntime> Clone for TopicDescription<R> {
    fn clone(&self) -> Self {
        match self {
            Self::Topic(arg0) => Self::Topic(arg0.clone()),
            Self::ContentFilteredTopic(arg0) => Self::ContentFilteredTopic(arg0.clone()),
        }
    }
}

impl<R: DdsRuntime> From<TopicDescriptionAsync<R>> for TopicDescription<R> {
    fn from(value: TopicDescriptionAsync<R>) -> Self {
        match value {
            TopicDescriptionAsync::Topic(t) => TopicDescription::Topic(t.into()),
            TopicDescriptionAsync::ContentFilteredTopic(t) => {
                TopicDescription::ContentFilteredTopic(t.into())
            }
        }
    }
}

impl<R: DdsRuntime> From<TopicDescription<R>> for TopicDescriptionAsync<R> {
    fn from(value: TopicDescription<R>) -> Self {
        match value {
            TopicDescription::Topic(t) => TopicDescriptionAsync::Topic(t.into()),
            TopicDescription::ContentFilteredTopic(t) => {
                TopicDescriptionAsync::ContentFilteredTopic(t.into())
            }
        }
    }
}

/// This implementation block represents the TopicDescription operations for the [`Topic`].
impl<R: DdsRuntime> TopicDescription<R> {
    /// This operation returns the [`DomainParticipant`] to which the [`Topic`] belongs.
    #[tracing::instrument(skip(self))]
    pub fn get_participant(&self) -> DomainParticipant<R> {
        match self {
            Self::Topic(t) => t.get_participant(),
            Self::ContentFilteredTopic(t) => t.get_participant(),
        }
    }

    /// The name of the type used to create the [`Topic`]
    #[tracing::instrument(skip(self))]
    pub fn get_type_name(&self) -> String {
        match self {
            Self::Topic(t) => t.get_type_name(),
            Self::ContentFilteredTopic(t) => t.get_type_name(),
        }
    }

    /// The name used to create the [`Topic`]
    #[tracing::instrument(skip(self))]
    pub fn get_name(&self) -> String {
        match self {
            Self::Topic(t) => t.get_name(),
            Self::ContentFilteredTopic(t) => t.get_name(),
        }
    }
}
