use crate::{
    data_representation_builtin_endpoints::discovered_topic_data::{
        DiscoveredTopicData, DCPS_TOPIC,
    },
    implementation::{
        actor::ActorWeakAddress,
        actors::{
            domain_participant_actor::DomainParticipantActor,
            status_condition_actor::StatusConditionActor, topic_actor::TopicActor,
        },
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{QosKind, TopicQos},
        status::{InconsistentTopicStatus, StatusKind},
    },
};

use super::{
    condition::StatusConditionAsync, domain_participant::DomainParticipantAsync,
    topic_listener::TopicListenerAsync,
};

/// Async version of [`Topic`](crate::topic_definition::topic::Topic).
#[derive(Clone)]
pub struct TopicAsync {
    topic_address: ActorWeakAddress<TopicActor>,
    status_condition_address: ActorWeakAddress<StatusConditionActor>,
    type_name: String,
    topic_name: String,
    participant: DomainParticipantAsync,
}

impl TopicAsync {
    pub(crate) fn new(
        topic_address: ActorWeakAddress<TopicActor>,
        status_condition_address: ActorWeakAddress<StatusConditionActor>,
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

    pub(crate) fn topic_address(&self) -> &ActorWeakAddress<TopicActor> {
        &self.topic_address
    }

    pub(crate) fn participant_address(&self) -> &ActorWeakAddress<DomainParticipantActor> {
        self.participant.participant_address()
    }

    pub(crate) fn runtime_handle(&self) -> &tokio::runtime::Handle {
        self.participant.runtime_handle()
    }

    async fn announce_topic(&self) -> DdsResult<()> {
        let builtin_publisher = self.get_participant().get_builtin_publisher().await?;
        if let Some(sedp_topics_announcer) = builtin_publisher
            .lookup_datawriter::<DiscoveredTopicData>(DCPS_TOPIC)
            .await?
        {
            let discovered_topic_data = self
                .topic_address
                .upgrade()?
                .as_discovered_topic_data()
                .await?;
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
        self.topic_address
            .upgrade()?
            .get_inconsistent_topic_status()
            .await?
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
                    .upgrade()?
                    .get_default_topic_qos()
                    .await?
            }
            QosKind::Specific(q) => q,
        };

        let topic = self.topic_address.upgrade()?;
        topic.set_qos(qos).await??;

        if topic.is_enabled().await? {
            self.announce_topic().await?;
        }
        Ok(())
    }

    /// Async version of [`get_qos`](crate::topic_definition::topic::Topic::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<TopicQos> {
        self.topic_address.upgrade()?.get_qos().await
    }

    /// Async version of [`get_statuscondition`](crate::topic_definition::topic::Topic::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(
            self.status_condition_address.clone(),
            self.runtime_handle().clone(),
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
        let topic = self.topic_address.upgrade()?;
        if !topic.is_enabled().await? {
            topic.enable().await?;
            self.announce_topic().await?;
        }

        Ok(())
    }

    /// Async version of [`get_instance_handle`](crate::topic_definition::topic::Topic::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.topic_address.upgrade()?.get_instance_handle().await
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
