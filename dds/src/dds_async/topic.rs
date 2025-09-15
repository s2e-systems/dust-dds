use super::{condition::StatusConditionAsync, domain_participant::DomainParticipantAsync};
use crate::{
    dcps::{
        actor::ActorAddress,
        domain_participant_mail::{DomainParticipantMail, TopicServiceMail},
        status_condition::StatusConditionActor,
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{QosKind, TopicQos},
        status::{InconsistentTopicStatus, StatusKind},
    },
    runtime::{ChannelSend, DdsRuntime, OneshotReceive},
    topic_definition::topic_listener::TopicListener,
    xtypes::dynamic_type::DynamicType,
};
use alloc::{string::String, sync::Arc, vec::Vec};

/// Async version of [`Topic`](crate::topic_definition::topic::Topic).
pub struct TopicAsync<R: DdsRuntime> {
    handle: InstanceHandle,
    status_condition_address: ActorAddress<R, StatusConditionActor<R>>,
    type_name: String,
    topic_name: String,
    participant: DomainParticipantAsync<R>,
}

impl<R: DdsRuntime> Clone for TopicAsync<R> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle,
            status_condition_address: self.status_condition_address.clone(),
            type_name: self.type_name.clone(),
            topic_name: self.topic_name.clone(),
            participant: self.participant.clone(),
        }
    }
}

impl<R: DdsRuntime> TopicAsync<R> {
    pub(crate) fn new(
        handle: InstanceHandle,
        status_condition_address: ActorAddress<R, StatusConditionActor<R>>,
        type_name: String,
        topic_name: String,
        participant: DomainParticipantAsync<R>,
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

impl<R: DdsRuntime> TopicAsync<R> {
    /// Async version of [`get_inconsistent_topic_status`](crate::topic_definition::topic::Topic::get_inconsistent_topic_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_inconsistent_topic_status(&self) -> DdsResult<InconsistentTopicStatus> {
        let (reply_sender, reply_receiver) = R::oneshot();
        self.participant
            .participant_address()
            .send(DomainParticipantMail::Topic(
                TopicServiceMail::GetInconsistentTopicStatus {
                    topic_name: self.topic_name.clone(),
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.receive().await?
    }
}

impl<R: DdsRuntime> TopicAsync<R> {
    /// Async version of [`get_participant`](crate::topic_definition::topic::Topic::get_participant).
    #[tracing::instrument(skip(self))]
    pub fn get_participant(&self) -> DomainParticipantAsync<R> {
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

impl<R: DdsRuntime> TopicAsync<R> {
    /// Async version of [`set_qos`](crate::topic_definition::topic::Topic::set_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = R::oneshot();
        self.participant
            .participant_address()
            .send(DomainParticipantMail::Topic(TopicServiceMail::SetQos {
                topic_name: self.topic_name.clone(),
                topic_qos: qos,
                reply_sender,
            }))
            .await?;

        reply_receiver.receive().await?
    }

    /// Async version of [`get_qos`](crate::topic_definition::topic::Topic::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<TopicQos> {
        let (reply_sender, reply_receiver) = R::oneshot();
        self.participant
            .participant_address()
            .send(DomainParticipantMail::Topic(TopicServiceMail::GetQos {
                topic_name: self.topic_name.clone(),
                reply_sender,
            }))
            .await?;

        reply_receiver.receive().await?
    }

    /// Async version of [`get_statuscondition`](crate::topic_definition::topic::Topic::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync<R> {
        StatusConditionAsync::new(
            self.status_condition_address.clone(),
            self.participant.clock_handle().clone(),
        )
    }

    /// Async version of [`get_status_changes`](crate::topic_definition::topic::Topic::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::topic_definition::topic::Topic::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = R::oneshot();
        self.participant
            .participant_address()
            .send(DomainParticipantMail::Topic(TopicServiceMail::Enable {
                topic_name: self.topic_name.clone(),
                reply_sender,
            }))
            .await?;
        reply_receiver.receive().await?
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
        _a_listener: Option<impl TopicListener<R> + Send + 'static>,
        _mask: &[StatusKind],
    ) -> DdsResult<()> {
        todo!()
    }
}

impl<R: DdsRuntime> TopicAsync<R> {
    #[doc(hidden)]
    #[tracing::instrument(skip(self))]
    pub async fn get_type_support(&self) -> DdsResult<Arc<dyn DynamicType + Send + Sync>> {
        let (reply_sender, reply_receiver) = R::oneshot();
        self.participant
            .participant_address()
            .send(DomainParticipantMail::Topic(
                TopicServiceMail::GetTypeSupport {
                    topic_name: self.topic_name.clone(),
                    reply_sender,
                },
            ))
            .await?;

        reply_receiver.receive().await?
    }
}
