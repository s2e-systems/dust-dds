use crate::{
    dds_async::content_filtered_topic::ContentFilteredTopicAsync,
    domain::domain_participant::DomainParticipant, infrastructure::error::DdsResult,
    runtime::DdsRuntime, topic_definition::topic::Topic,
};

/// [`ContentFilteredTopic`] describes a more sophisticated subscription that indicates the subscriber does not want to necessarily see
/// all values of each instance published under the [`Topic`]. Rather, it wants to see only the values whose contents satisfy certain
/// criteria. This class therefore can be used to request content-based subscriptions.
pub struct ContentFilteredTopic<R: DdsRuntime> {
    topic: ContentFilteredTopicAsync<R>,
}

impl<R: DdsRuntime> Clone for ContentFilteredTopic<R> {
    fn clone(&self) -> Self {
        Self {
            topic: self.topic.clone(),
        }
    }
}

impl<R: DdsRuntime> From<ContentFilteredTopicAsync<R>> for ContentFilteredTopic<R> {
    fn from(value: ContentFilteredTopicAsync<R>) -> Self {
        Self { topic: value }
    }
}

impl<R: DdsRuntime> From<ContentFilteredTopic<R>> for ContentFilteredTopicAsync<R> {
    fn from(value: ContentFilteredTopic<R>) -> Self {
        value.topic
    }
}

impl<R: DdsRuntime> ContentFilteredTopic<R> {
    /// This operation returns the [`Topic`] associated with the ContentFilteredTopic. That is, the
    /// [`Topic`] specified when the [`ContentFilteredTopic`] was created.
    pub fn get_related_topic(&self) -> Topic<R> {
        self.topic.get_related_topic().clone().into()
    }

    /// This operation returns the expression_parameters associated with the [`ContentFilteredTopic`]. That is, the parameters specified
    /// on the last successful call to set_expression_parameters, or if set_expression_parameters was never called, the parameters
    /// specified when the [`ContentFilteredTopic`] was created.
    pub fn get_expression_parameters(&self) -> DdsResult<String> {
        R::block_on(self.topic.get_expression_parameters())
    }

    /// This operation changes the expression_parameters associated with the  [`ContentFilteredTopic`].
    pub fn set_expression_parameters(&self, expression_parameters: &[String]) -> DdsResult<()> {
        R::block_on(self.topic.set_expression_parameters(expression_parameters))
    }
}

/// This implementation block represents the TopicDescription operations for the [`Topic`].
impl<R: DdsRuntime> ContentFilteredTopic<R> {
    /// This operation returns the [`DomainParticipant`] to which the [`Topic`] belongs.
    #[tracing::instrument(skip(self))]
    pub fn get_participant(&self) -> DomainParticipant<R> {
        DomainParticipant::new(self.topic.get_participant())
    }

    /// The name of the type used to create the [`Topic`]
    #[tracing::instrument(skip(self))]
    pub fn get_type_name(&self) -> String {
        self.topic.get_type_name()
    }

    /// The name used to create the [`Topic`]
    #[tracing::instrument(skip(self))]
    pub fn get_name(&self) -> String {
        self.topic.get_name()
    }
}
