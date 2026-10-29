use super::{
    condition::StatusConditionAsync, domain_participant::DomainParticipantAsync,
    topic_listener::TopicListener,
};
use crate::{
    dcps::{
        dcps_mail::{
            EnableTopicMail, GetInconsistentTopicStatusMail, GetTopicQosMail,
            GetTopicTypeSupportMail, SetTopicQosMail,
        },
        status_condition::StatusConditionEntity,
    },
    dds_async::topic_description::TopicDescriptionAsync,
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{QosKind, TopicQos},
        status::{InconsistentTopicStatus, StatusKind},
    },
    xtypes::dynamic_type::DynamicType,
};
use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};

/// Async version of [`Topic`](crate::topic_definition::topic::Topic).
#[derive(Clone)]
pub struct TopicAsync {
    handle: InstanceHandle,
    type_name: Arc<str>,
    topic_name: Arc<str>,
    participant: DomainParticipantAsync,
}

impl TopicAsync {
    pub(crate) fn new(
        handle: InstanceHandle,
        type_name: Arc<str>,
        topic_name: Arc<str>,
        participant: DomainParticipantAsync,
    ) -> Self {
        Self {
            handle,
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
            .dcps_sender()
            .execute(GetInconsistentTopicStatusMail {
                participant_handle: self.participant.get_instance_handle(),
                topic_name: self.topic_name.to_string(),
            })
            .await
    }
}

impl TopicDescriptionAsync for TopicAsync {
    fn get_participant(&self) -> DomainParticipantAsync {
        self.participant.clone()
    }

    fn get_type_name(&self) -> String {
        self.type_name.to_string()
    }

    fn get_name(&self) -> String {
        self.topic_name.to_string()
    }
}

impl TopicAsync {
    /// Async version of [`set_qos`](crate::topic_definition::topic::Topic::set_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        self.participant
            .dcps_sender()
            .execute(SetTopicQosMail {
                participant_handle: self.participant.get_instance_handle(),
                topic_name: self.topic_name.to_string(),
                topic_qos: Box::new(qos),
            })
            .await
    }

    /// Async version of [`get_qos`](crate::topic_definition::topic::Topic::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<TopicQos> {
        self.participant
            .dcps_sender()
            .execute(GetTopicQosMail {
                participant_handle: self.participant.get_instance_handle(),
                topic_name: self.topic_name.to_string(),
            })
            .await
    }

    /// Async version of [`get_statuscondition`](crate::topic_definition::topic::Topic::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(
            self.participant.dcps_sender().clone(),
            StatusConditionEntity::Topic {
                participant_handle: self.get_participant().get_instance_handle(),
                topic_handle: self.handle,
            },
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
        self.participant
            .dcps_sender()
            .execute(EnableTopicMail {
                participant_handle: self.participant.get_instance_handle(),
                topic_name: self.topic_name.to_string(),
            })
            .await
    }

    /// Async version of [`get_instance_handle`](crate::topic_definition::topic::Topic::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.handle
    }

    /// Async version of [`set_listener`](crate::topic_definition::topic::Topic::set_listener).
    #[tracing::instrument(skip(self, _a_listener))]
    pub async fn set_listener(
        &self,
        _a_listener: Option<impl TopicListener + Send + 'static>,
        _mask: &[StatusKind],
    ) -> DdsResult<()> {
        todo!()
    }
}

impl TopicAsync {
    #[doc(hidden)]
    #[tracing::instrument(skip(self))]
    pub async fn get_type_support(&self) -> DdsResult<DynamicType<'static>> {
        self.participant
            .dcps_sender()
            .execute(GetTopicTypeSupportMail {
                participant_handle: self.participant.get_instance_handle(),
                topic_name: self.topic_name.to_string(),
            })
            .await
    }
}
