use std::sync::Arc;

use crate::{
    data_representation_builtin_endpoints::discovered_topic_data::{
        DiscoveredTopicData, DCPS_TOPIC,
    },
    implementation::{
        actor::ActorAddress,
        actors::{
            domain_participant_actor::{self, DomainParticipantActor},
            status_condition_actor::StatusConditionActor,
            topic_actor::{self, TopicActor},
        },
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{QosKind, TopicQos},
        status::{InconsistentTopicStatus, StatusKind},
    },
    topic_definition::type_support::DynamicTypeInterface,
};

use super::{
    condition::StatusConditionAsync, domain_participant::DomainParticipantAsync,
    topic_listener::TopicListenerAsync,
};

/// Async version of [`Topic`](crate::topic_definition::topic::Topic).
#[derive(Clone)]
pub struct TopicAsync {
    topic_address: ActorAddress<TopicActor>,
    status_condition_address: ActorAddress<StatusConditionActor>,
    type_name: String,
    topic_name: String,
    participant: DomainParticipantAsync,
}

impl TopicAsync {
    pub(crate) fn new(
        topic_address: ActorAddress<TopicActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        type_name: String,
        topic_name: String,
        participant: DomainParticipantAsync,
    ) -> Self {
        Self {
            topic_address,
            status_condition_address,
            type_name,
            topic_name,
            participant,
        }
    }

    pub(crate) fn topic_address(&self) -> &ActorAddress<TopicActor> {
        &self.topic_address
    }

    pub(crate) fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        self.participant.participant_address()
    }

    async fn announce_topic(&self) -> DdsResult<()> {
        let builtin_publisher = self.get_participant().get_builtin_publisher().await?;
        if let Some(sedp_topics_announcer) = builtin_publisher
            .lookup_datawriter::<DiscoveredTopicData>(DCPS_TOPIC)
            .await?
        {
            let discovered_topic_data = self
                .topic_address
                .send_actor_mail(topic_actor::AsDiscoveredTopicData)?
                .receive_reply()
                .await;
            sedp_topics_announcer
                .write(&discovered_topic_data, None)
                .await?;
        }
        Ok(())
    }
}

impl TopicAsync {
    /// Async version of [`get_inconsistent_topic_status`](crate::topic_definition::topic::Topic::get_inconsistent_topic_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_inconsistent_topic_status(&self) -> DdsResult<InconsistentTopicStatus> {
        Ok(self
            .topic_address
            .send_actor_mail(topic_actor::GetInconsistentTopicStatus)?
            .receive_reply()
            .await)
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
        let qos = match qos {
            QosKind::Default => {
                self.participant_address()
                    .send_actor_mail(domain_participant_actor::GetDefaultTopicQos)?
                    .receive_reply()
                    .await
            }
            QosKind::Specific(q) => q,
        };

        self.topic_address
            .send_actor_mail(topic_actor::SetQos { qos })?
            .receive_reply()
            .await?;

        if self
            .topic_address
            .send_actor_mail(topic_actor::IsEnabled)?
            .receive_reply()
            .await
        {
            self.announce_topic().await?;
        }
        Ok(())
    }

    /// Async version of [`get_qos`](crate::topic_definition::topic::Topic::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<TopicQos> {
        Ok(self
            .topic_address
            .send_actor_mail(topic_actor::GetQos)?
            .receive_reply()
            .await)
    }

    /// Async version of [`get_statuscondition`](crate::topic_definition::topic::Topic::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(
            self.status_condition_address.clone(),
            self.participant.executor_handle().clone(),
            self.participant.timer_handle().clone(),
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
        if !self
            .topic_address
            .send_actor_mail(topic_actor::IsEnabled)?
            .receive_reply()
            .await
        {
            self.topic_address
                .send_actor_mail(topic_actor::Enable)?
                .receive_reply()
                .await;
            self.announce_topic().await?;
        }

        Ok(())
    }

    /// Async version of [`get_instance_handle`](crate::topic_definition::topic::Topic::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self
            .topic_address
            .send_actor_mail(topic_actor::GetInstanceHandle)?
            .receive_reply()
            .await)
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
    pub async fn get_type_support(&self) -> DdsResult<Arc<dyn DynamicTypeInterface>> {
        Ok(self
            .topic_address
            .send_actor_mail(topic_actor::GetTypeSupport)?
            .receive_reply()
            .await)
    }
}
