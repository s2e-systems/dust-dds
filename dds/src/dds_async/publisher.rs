use crate::{
    implementation::{
        actors::{
            domain_participant_actor::{self, DomainParticipantActor},
            publisher_actor::{self, PublisherActor},
            topic_actor,
        },
        utils::actor::ActorAddress,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind, TopicQos},
        status::StatusKind,
        time::Duration,
    },
    publication::{
        data_writer_listener::DataWriterListener, publisher_listener::PublisherListener,
    },
};

use super::{
    condition::StatusConditionAsync, data_writer::DataWriterAsync,
    domain_participant::DomainParticipantAsync, topic::TopicAsync,
};

/// Async version of [`Publisher`](crate::publication::publisher::Publisher).
#[derive(Clone)]
pub struct PublisherAsync {
    publisher_address: ActorAddress<PublisherActor>,
    participant: DomainParticipantAsync,
}

impl PublisherAsync {
    pub(crate) fn new(
        publisher_address: ActorAddress<PublisherActor>,
        participant: DomainParticipantAsync,
    ) -> Self {
        Self {
            publisher_address,
            participant,
        }
    }

    pub(crate) fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        &self.participant.participant_address()
    }

    pub(crate) fn publisher_address(&self) -> &ActorAddress<PublisherActor> {
        &self.publisher_address
    }

    pub(crate) fn runtime_handle(&self) -> &tokio::runtime::Handle {
        &self.participant.runtime_handle()
    }
}

impl PublisherAsync {
    /// Async version of [`create_datawriter`](crate::publication::publisher::Publisher::create_datawriter).
    #[tracing::instrument(skip(self, a_topic, a_listener))]
    pub async fn create_datawriter<Foo>(
        &self,
        a_topic: &TopicAsync,
        qos: QosKind<DataWriterQos>,
        a_listener: impl DataWriterListener<Foo = Foo> + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<DataWriterAsync<Foo>> {
        let type_name = a_topic.get_type_name().await;
        let type_support = self
            .participant
            .participant_address()
            .send_mail_and_await_reply(domain_participant_actor::get_type_support::new(
                type_name.clone(),
            ))
            .await?
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(format!(
                    "Type with name {} not registered with parent domain participant",
                    type_name
                ))
            })?;

        let default_unicast_locator_list = self
            .participant
            .participant_address()
            .send_mail_and_await_reply(
                domain_participant_actor::get_default_unicast_locator_list::new(),
            )
            .await?;
        let default_multicast_locator_list = self
            .participant
            .participant_address()
            .send_mail_and_await_reply(
                domain_participant_actor::get_default_multicast_locator_list::new(),
            )
            .await?;
        let data_max_size_serialized = self
            .participant
            .participant_address()
            .send_mail_and_await_reply(domain_participant_actor::data_max_size_serialized::new())
            .await?;

        let listener = Box::new(a_listener);
        let has_key = type_support.has_key();
        let data_writer_address = self
            .publisher_address
            .send_mail_and_await_reply(publisher_actor::create_datawriter::new(
                a_topic.get_type_name().await,
                a_topic.get_name().await,
                has_key,
                data_max_size_serialized,
                qos,
                listener,
                mask.to_vec(),
                default_unicast_locator_list,
                default_multicast_locator_list,
                self.participant.runtime_handle().clone(),
                a_topic.topic_address().clone(),
            ))
            .await??;

        let data_writer = DataWriterAsync::new(data_writer_address, self.clone(), a_topic.clone());

        if self
            .publisher_address
            .send_mail_and_await_reply(publisher_actor::is_enabled::new())
            .await?
            && self
                .publisher_address
                .send_mail_and_await_reply(publisher_actor::get_qos::new())
                .await?
                .entity_factory
                .autoenable_created_entities
        {
            data_writer.enable().await?
        }

        Ok(data_writer)
    }

    /// Async version of [`delete_datawriter`](crate::publication::publisher::Publisher::delete_datawriter).
    #[tracing::instrument(skip(self, a_datawriter))]
    pub async fn delete_datawriter<Foo>(
        &self,
        a_datawriter: &DataWriterAsync<Foo>,
    ) -> DdsResult<()> {
        let writer_handle = a_datawriter.get_instance_handle().await?;
        if self
            .publisher_address
            .send_mail_and_await_reply(publisher_actor::get_instance_handle::new())
            .await?
            != a_datawriter
                .get_publisher()
                .await
                .get_instance_handle()
                .await?
        {
            return Err(DdsError::PreconditionNotMet(
                "Data writer can only be deleted from its parent publisher".to_string(),
            ));
        }

        self.publisher_address
            .send_mail_and_await_reply(publisher_actor::datawriter_delete::new(writer_handle))
            .await?;

        self.participant
            .participant_address()
            .send_mail_and_await_reply(domain_participant_actor::announce_deleted_data_writer::new(
                writer_handle,
            ))
            .await?
    }

    /// Async version of [`delete_datawriter`](crate::publication::publisher::Publisher::lookup_datawriter).
    #[tracing::instrument(skip(self))]
    pub async fn lookup_datawriter<Foo>(
        &self,
        topic_name: &str,
    ) -> DdsResult<Option<DataWriterAsync<Foo>>> {
        if let Some(topic_address) = self
            .participant
            .participant_address()
            .send_mail_and_await_reply(domain_participant_actor::lookup_topicdescription::new(
                topic_name.to_string(),
            ))
            .await?
        {
            let type_name = topic_address
                .send_mail_and_await_reply(topic_actor::get_type_name::new())
                .await?;
            let topic = TopicAsync::new(
                topic_address,
                topic_name.to_string(),
                type_name,
                self.participant.clone(),
            );
            Ok(self
                .publisher_address
                .send_mail_and_await_reply(publisher_actor::lookup_datawriter::new(
                    topic_name.to_string(),
                ))
                .await?
                .map(|dw| DataWriterAsync::new(dw, self.clone(), topic)))
        } else {
            Err(DdsError::BadParameter)
        }
    }

    /// Async version of [`delete_datawriter`](crate::publication::publisher::Publisher::suspend_publications).
    #[tracing::instrument(skip(self))]
    pub async fn suspend_publications(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`delete_datawriter`](crate::publication::publisher::Publisher::resume_publications).
    #[tracing::instrument(skip(self))]
    pub async fn resume_publications(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`delete_datawriter`](crate::publication::publisher::Publisher::begin_coherent_changes).
    #[tracing::instrument(skip(self))]
    pub async fn begin_coherent_changes(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`delete_datawriter`](crate::publication::publisher::Publisher::end_coherent_changes).
    #[tracing::instrument(skip(self))]
    pub async fn end_coherent_changes(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`delete_datawriter`](crate::publication::publisher::Publisher::wait_for_acknowledgments).
    #[tracing::instrument(skip(self))]
    pub async fn wait_for_acknowledgments(&self, _max_wait: Duration) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`get_participant`](crate::publication::publisher::Publisher::get_participant).
    #[tracing::instrument(skip(self))]
    pub async fn get_participant(&self) -> DomainParticipantAsync {
        self.participant.clone()
    }

    /// Async version of [`delete_contained_entities`](crate::publication::publisher::Publisher::delete_contained_entities).
    #[tracing::instrument(skip(self))]
    pub async fn delete_contained_entities(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`set_default_datawriter_qos`](crate::publication::publisher::Publisher::set_default_datawriter_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_datawriter_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => {
                self.publisher_address
                    .send_mail_and_await_reply(publisher_actor::get_default_datawriter_qos::new())
                    .await?
            }
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        self.publisher_address
            .send_mail_and_await_reply(publisher_actor::set_default_datawriter_qos::new(qos))
            .await
    }

    /// Async version of [`get_default_datawriter_qos`](crate::publication::publisher::Publisher::get_default_datawriter_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_datawriter_qos(&self) -> DdsResult<DataWriterQos> {
        self.publisher_address
            .send_mail_and_await_reply(publisher_actor::get_default_datawriter_qos::new())
            .await
    }

    /// Async version of [`copy_from_topic_qos`](crate::publication::publisher::Publisher::copy_from_topic_qos).
    #[tracing::instrument(skip(self))]
    pub async fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        todo!()
    }
}

impl PublisherAsync {
    /// Async version of [`set_qos`](crate::publication::publisher::Publisher::set_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, _qos: QosKind<PublisherQos>) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`get_qos`](crate::publication::publisher::Publisher::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<PublisherQos> {
        self.publisher_address
            .send_mail_and_await_reply(publisher_actor::get_qos::new())
            .await
    }

    /// Async version of [`set_listener`](crate::publication::publisher::Publisher::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: impl PublisherListener + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.publisher_address
            .send_mail_and_await_reply(publisher_actor::set_listener::new(
                Box::new(a_listener),
                mask.to_vec(),
                self.participant.runtime_handle().clone(),
            ))
            .await
    }

    /// Async version of [`get_statuscondition`](crate::publication::publisher::Publisher::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub async fn get_statuscondition(&self) -> DdsResult<StatusConditionAsync> {
        todo!()
    }

    /// Async version of [`get_status_changes`](crate::publication::publisher::Publisher::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::publication::publisher::Publisher::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        self.publisher_address
            .send_mail_and_await_reply(publisher_actor::enable::new())
            .await
    }

    /// Async version of [`get_instance_handle`](crate::publication::publisher::Publisher::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.publisher_address
            .send_mail_and_await_reply(publisher_actor::get_instance_handle::new())
            .await
    }
}
