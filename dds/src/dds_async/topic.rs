use crate::{
    implementation::{
        actors::{
            data_writer_actor,
            domain_participant_actor::{self, DomainParticipantActor},
            publisher_actor,
            topic_actor::{self, TopicActor},
        },
        data_representation_builtin_endpoints::discovered_topic_data::DiscoveredTopicData,
        utils::{actor::ActorAddress, instance_handle_from_key::get_instance_handle_from_key},
    },
    infrastructure::{
        condition::StatusCondition,
        error::DdsResult,
        instance::InstanceHandle,
        qos::{QosKind, TopicQos},
        status::{InconsistentTopicStatus, StatusKind},
    },
    topic_definition::{
        topic_listener::TopicListener,
        type_support::{DdsKey, DdsSerialize},
    },
};

use super::domain_participant::DomainParticipantAsync;

pub struct TopicAsync {
    topic_address: ActorAddress<TopicActor>,
    participant_address: ActorAddress<DomainParticipantActor>,
    runtime_handle: tokio::runtime::Handle,
}

impl TopicAsync {
    pub(crate) fn new(
        topic_address: ActorAddress<TopicActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        runtime_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            topic_address,
            participant_address,
            runtime_handle,
        }
    }

    pub(crate) fn runtime_handle(&self) -> &tokio::runtime::Handle {
        &self.runtime_handle
    }
}

impl TopicAsync {
    #[tracing::instrument(skip(self))]
    pub async fn get_inconsistent_topic_status(&self) -> DdsResult<InconsistentTopicStatus> {
        self.topic_address
            .send_mail_and_await_reply(topic_actor::get_inconsistent_topic_status::new())
            .await?
    }
}

impl TopicAsync {
    #[tracing::instrument(skip(self))]
    pub async fn get_participant(&self) -> DdsResult<DomainParticipantAsync> {
        Ok(DomainParticipantAsync::new(
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        ))
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_type_name(&self) -> DdsResult<String> {
        self.topic_address
            .send_mail_and_await_reply(topic_actor::get_type_name::new())
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_name(&self) -> DdsResult<String> {
        self.topic_address
            .send_mail_and_await_reply(topic_actor::get_name::new())
            .await
    }
}

impl TopicAsync {
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => {
                self.participant_address
                    .send_mail_and_await_reply(domain_participant_actor::default_topic_qos::new())
                    .await?
            }
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        if self
            .topic_address
            .send_mail_and_await_reply(topic_actor::is_enabled::new())
            .await?
        {
            self.topic_address
                .send_mail_and_await_reply(topic_actor::get_qos::new())
                .await?
                .check_immutability(&qos)?
        }

        self.topic_address
            .send_mail_and_await_reply(topic_actor::set_qos::new(qos))
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<TopicQos> {
        self.topic_address
            .send_mail_and_await_reply(topic_actor::get_qos::new())
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        self.topic_address
            .send_mail_and_await_reply(topic_actor::get_statuscondition::new())
            .await
            .map(StatusCondition::new)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        if !self
            .topic_address
            .send_mail_and_await_reply(topic_actor::is_enabled::new())
            .await?
        {
            self.topic_address
                .send_mail_and_await_reply(topic_actor::enable::new())
                .await?;

            announce_topic(
                &self.participant_address,
                self.topic_address
                    .send_mail_and_await_reply(topic_actor::as_discovered_topic_data::new())
                    .await?,
            )
            .await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.topic_address
            .send_mail_and_await_reply(topic_actor::get_instance_handle::new())
            .await
    }

    #[tracing::instrument(skip(self, _a_listener))]
    pub async fn set_listener(
        &self,
        _a_listener: impl TopicListener + Send + 'static,
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
    let timestamp = domain_participant
        .send_mail_and_await_reply(domain_participant_actor::get_current_time::new())
        .await?;
    let builtin_publisher = domain_participant
        .send_mail_and_await_reply(domain_participant_actor::get_builtin_publisher::new())
        .await?;
    let data_writer_list = builtin_publisher
        .send_mail_and_await_reply(publisher_actor::data_writer_list::new())
        .await?;
    for data_writer in data_writer_list {
        if data_writer
            .send_mail_and_await_reply(data_writer_actor::get_type_name::new())
            .await
            == Ok("DiscoveredTopicData".to_string())
        {
            data_writer
                .send_mail_and_await_reply(data_writer_actor::write_w_timestamp::new(
                    serialized_data,
                    get_instance_handle_from_key(&discovered_topic_data.get_key()?)?,
                    None,
                    timestamp,
                ))
                .await??;

            domain_participant
                .send_mail(domain_participant_actor::send_message::new())
                .await?;
            break;
        }
    }

    Ok(())
}
