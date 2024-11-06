use super::{
    condition::StatusConditionAsync, domain_participant::DomainParticipantAsync,
    topic_listener::TopicListenerAsync,
};
use crate::{
    implementation::{
        actor::ActorAddress,
        actors::{
            domain_participant_actor::{self},
            status_condition_actor::StatusConditionActor,
        },
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{QosKind, TopicQos},
        status::{InconsistentTopicStatus, StatusKind},
    },
    xtypes::dynamic_type::DynamicType,
};
use std::sync::Arc;

/// Async version of [`Topic`](crate::topic_definition::topic::Topic).
#[derive(Clone)]
pub struct TopicAsync {
    handle: InstanceHandle,
    status_condition_address: ActorAddress<StatusConditionActor>,
    type_name: String,
    topic_name: String,
    participant: DomainParticipantAsync,
}

impl TopicAsync {
    pub(crate) fn new(
        handle: InstanceHandle,
        status_condition_address: ActorAddress<StatusConditionActor>,
        type_name: String,
        topic_name: String,
        participant: DomainParticipantAsync,
    ) -> Self {
        Self {
            handle,
            status_condition_address,
            type_name,
            topic_name,
            participant,
        }
    }
}

impl TopicAsync {
    /// Async version of [`get_inconsistent_topic_status`](crate::topic_definition::topic::Topic::get_inconsistent_topic_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_inconsistent_topic_status(&self) -> DdsResult<InconsistentTopicStatus> {
        self.participant
            .participant_address()
            .send_actor_mail(domain_participant_actor::GetInconsistentTopicStatus {
                topic_name: self.topic_name.clone(),
            })?
            .receive_reply()
            .await
    }
}

impl TopicAsync {
    /// Async version of [`get_participant`](crate::topic_definition::topic::Topic::get_participant).
    #[tracing::instrument(skip(self))]
    pub fn get_participant(&self) -> DomainParticipantAsync {
        self.participant.clone()
    }

    /// Async version of [`get_type_name`](crate::topic_definition::topic::Topic::get_type_name).
    #[tracing::instrument(skip(self))]
    pub fn get_type_name(&self) -> String {
        self.type_name.clone()
    }

    /// Async version of [`get_name`](crate::topic_definition::topic::Topic::get_name).
    #[tracing::instrument(skip(self))]
    pub fn get_name(&self) -> String {
        self.topic_name.clone()
    }
}

impl TopicAsync {
    /// Async version of [`set_qos`](crate::topic_definition::topic::Topic::set_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        self.participant
            .participant_address()
            .send_actor_mail(domain_participant_actor::SetTopicQos {
                topic_name: self.topic_name.clone(),
                topic_qos: qos,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_qos`](crate::topic_definition::topic::Topic::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<TopicQos> {
        self.participant
            .participant_address()
            .send_actor_mail(domain_participant_actor::GetTopicQos {
                topic_name: self.topic_name.clone(),
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_statuscondition`](crate::topic_definition::topic::Topic::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(self.status_condition_address.clone())
    }

    /// Async version of [`get_status_changes`](crate::topic_definition::topic::Topic::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::topic_definition::topic::Topic::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        self.participant
            .participant_address()
            .send_actor_mail(domain_participant_actor::EnableTopic {
                topic_name: self.topic_name.clone(),
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_instance_handle`](crate::topic_definition::topic::Topic::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> InstanceHandle {
        self.handle
    }

    /// Async version of [`set_listener`](crate::topic_definition::topic::Topic::set_listener).
    #[tracing::instrument(skip(self, _a_listener))]
    pub async fn set_listener(
        &self,
        _a_listener: Option<Box<dyn TopicListenerAsync + Send>>,
        _mask: &[StatusKind],
    ) -> DdsResult<()> {
        todo!()
    }
}

impl TopicAsync {
    #[doc(hidden)]
    #[tracing::instrument(skip(self))]
    pub async fn get_type_support(&self) -> DdsResult<Arc<dyn DynamicType + Send + Sync>> {
        self.participant
            .participant_address()
            .send_actor_mail(domain_participant_actor::GetTopicTypeSupport {
                topic_name: self.topic_name.clone(),
            })?
            .receive_reply()
            .await
    }
}
