use crate::{
    dds_async::{
        condition::StatusConditionAsync, domain_participant::DomainParticipantAsync,
        topic::TopicAsync,
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{QosKind, TopicQos},
        status::StatusKind,
    },
    runtime::DdsRuntime,
    topic_definition::topic_listener::TopicListener,
};

/// Async version of [`TopicDescription`](crate::topic_definition::topic_description::TopicDescription).
pub enum TopicDescriptionAsync<R: DdsRuntime> {
    /// Async topic
    Topic(TopicAsync<R>),
}

impl<R: DdsRuntime> Clone for TopicDescriptionAsync<R> {
    fn clone(&self) -> Self {
        match self {
            Self::Topic(arg0) => Self::Topic(arg0.clone()),
        }
    }
}

impl<R: DdsRuntime> TopicDescriptionAsync<R> {
    /// Async version of [`get_participant`](crate::topic_definition::topic::Topic::get_participant).
    #[tracing::instrument(skip(self))]
    pub fn get_participant(&self) -> DomainParticipantAsync<R> {
        match self {
            TopicDescriptionAsync::Topic(topic_async) => topic_async.get_participant(),
        }
    }

    /// Async version of [`get_type_name`](crate::topic_definition::topic::Topic::get_type_name).
    #[tracing::instrument(skip(self))]
    pub fn get_type_name(&self) -> String {
        match self {
            TopicDescriptionAsync::Topic(topic_async) => topic_async.get_type_name(),
        }
    }

    /// Async version of [`get_name`](crate::topic_definition::topic::Topic::get_name).
    #[tracing::instrument(skip(self))]
    pub fn get_name(&self) -> String {
        match self {
            TopicDescriptionAsync::Topic(topic_async) => topic_async.get_name(),
        }
    }
}

impl<R: DdsRuntime> TopicDescriptionAsync<R> {
    /// Async version of [`set_qos`](crate::topic_definition::topic::Topic::set_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        match self {
            TopicDescriptionAsync::Topic(topic_async) => topic_async.set_qos(qos).await,
        }
    }

    /// Async version of [`get_qos`](crate::topic_definition::topic::Topic::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<TopicQos> {
        match self {
            TopicDescriptionAsync::Topic(topic_async) => topic_async.get_qos().await,
        }
    }

    /// Async version of [`get_statuscondition`](crate::topic_definition::topic::Topic::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync<R> {
        match self {
            TopicDescriptionAsync::Topic(topic_async) => topic_async.get_statuscondition(),
        }
    }

    /// Async version of [`get_status_changes`](crate::topic_definition::topic::Topic::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::topic_definition::topic::Topic::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        match self {
            TopicDescriptionAsync::Topic(topic_async) => topic_async.enable().await,
        }
    }

    /// Async version of [`get_instance_handle`](crate::topic_definition::topic::Topic::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> InstanceHandle {
        match self {
            TopicDescriptionAsync::Topic(topic_async) => topic_async.get_instance_handle().await,
        }
    }

    /// Async version of [`set_listener`](crate::topic_definition::topic::Topic::set_listener).
    #[tracing::instrument(skip(self, _a_listener))]
    pub async fn set_listener(
        &self,
        _a_listener: Option<impl TopicListener<R> + Send + 'static>,
        _mask: &[StatusKind],
    ) -> DdsResult<()> {
        todo!()
    }
}
