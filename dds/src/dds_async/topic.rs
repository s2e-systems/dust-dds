use crate::{
    implementation::{
        actors::{
            domain_participant_actor::DomainParticipantActor,
            status_condition_actor::StatusConditionActor, topic_actor::TopicActor,
        },
        data_representation_builtin_endpoints::discovered_topic_data::DiscoveredTopicData,
        utils::{actor::ActorAddress, instance_handle_from_key::get_instance_handle_from_key},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{QosKind, TopicQos},
        status::{InconsistentTopicStatus, StatusKind},
    },
    topic_definition::type_support::{DdsKey, DdsSerialize},
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

    pub(crate) fn runtime_handle(&self) -> &tokio::runtime::Handle {
        self.participant.runtime_handle()
    }
}

impl TopicAsync {
    /// Async version of [`get_inconsistent_topic_status`](crate::topic_definition::topic::Topic::get_inconsistent_topic_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_inconsistent_topic_status(&self) -> DdsResult<InconsistentTopicStatus> {
        self.topic_address
            .upgrade()?
            .get_inconsistent_topic_status()
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
        let qos = match qos {
            QosKind::Default => {
                self.participant_address()
                    .upgrade()?
                    .get_default_topic_qos()
                    .await
            }
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        self.topic_address.upgrade()?.set_qos(qos).await
    }

    /// Async version of [`get_qos`](crate::topic_definition::topic::Topic::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<TopicQos> {
        Ok(self.topic_address.upgrade()?.get_qos().await)
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
        if !self.topic_address.upgrade()?.is_enabled().await {
            self.topic_address.upgrade()?.enable().await;

            announce_topic(
                self.participant_address(),
                self.topic_address
                    .upgrade()?
                    .as_discovered_topic_data()
                    .await,
            )
            .await?;
        }

        Ok(())
    }

    /// Async version of [`get_instance_handle`](crate::topic_definition::topic::Topic::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.topic_address.upgrade()?.get_instance_handle().await)
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

async fn announce_topic(
    domain_participant: &ActorAddress<DomainParticipantActor>,
    discovered_topic_data: DiscoveredTopicData,
) -> DdsResult<()> {
    let mut serialized_data = Vec::new();
    discovered_topic_data.serialize_data(&mut serialized_data)?;
    let timestamp = domain_participant.upgrade()?.get_current_time().await;
    let builtin_publisher = domain_participant.upgrade()?.get_builtin_publisher().await;
    let data_writer_list = builtin_publisher.upgrade()?.data_writer_list().await;
    for data_writer in data_writer_list {
        if data_writer.upgrade()?.get_type_name().await == "DiscoveredTopicData".to_string() {
            data_writer
                .upgrade()?
                .write_w_timestamp(
                    serialized_data,
                    get_instance_handle_from_key(&discovered_topic_data.get_key()?)?,
                    None,
                    timestamp,
                )
                .await?;

            domain_participant.upgrade()?.send_message().await;
            break;
        }
    }

    Ok(())
}
