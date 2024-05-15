use crate::{
    data_representation_builtin_endpoints::discovered_writer_data::DCPS_PUBLICATION,
    implementation::{
        actor::{Actor, ActorWeakAddress},
        actors::{
            any_data_writer_listener::AnyDataWriterListener, data_writer_actor::DataWriterActor,
            domain_participant_actor::DomainParticipantActor, publisher_actor::PublisherActor,
            status_condition_actor::StatusConditionActor,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind, TopicQos},
        status::StatusKind,
        time::Duration,
    },
};

use super::{
    condition::StatusConditionAsync, data_writer::DataWriterAsync,
    data_writer_listener::DataWriterListenerAsync, domain_participant::DomainParticipantAsync,
    publisher_listener::PublisherListenerAsync, topic::TopicAsync,
};

/// Async version of [`Publisher`](crate::publication::publisher::Publisher).
#[derive(Clone)]
pub struct PublisherAsync {
    publisher_address: ActorWeakAddress<PublisherActor>,
    status_condition_address: ActorWeakAddress<StatusConditionActor>,
    participant: DomainParticipantAsync,
}

impl PublisherAsync {
    pub(crate) fn new(
        publisher_address: ActorWeakAddress<PublisherActor>,
        status_condition_address: ActorWeakAddress<StatusConditionActor>,
        participant: DomainParticipantAsync,
    ) -> Self {
        Self {
            publisher_address,
            status_condition_address,
            participant,
        }
    }

    pub(crate) fn participant_address(&self) -> &ActorWeakAddress<DomainParticipantActor> {
        self.participant.participant_address()
    }

    pub(crate) fn publisher_address(&self) -> &ActorWeakAddress<PublisherActor> {
        &self.publisher_address
    }

    pub(crate) fn runtime_handle(&self) -> &tokio::runtime::Handle {
        self.participant.runtime_handle()
    }

    async fn announce_deleted_data_writer(&self, writer: Actor<DataWriterActor>) -> DdsResult<()> {
        let builtin_publisher = self.participant.get_builtin_publisher().await?;
        if let Some(sedp_publications_announcer) = builtin_publisher
            .lookup_datawriter(DCPS_PUBLICATION)
            .await?
        {
            let publisher_qos = self.get_qos().await?;
            let default_unicast_locator_list = self
                .participant_address()
                .upgrade()?
                .get_default_unicast_locator_list()
                .await;
            let default_multicast_locator_list = self
                .participant_address()
                .upgrade()?
                .get_default_multicast_locator_list()
                .await;
            let data = writer
                .as_discovered_writer_data(
                    publisher_qos,
                    default_unicast_locator_list,
                    default_multicast_locator_list,
                )
                .await?;
            sedp_publications_announcer.dispose(&data, None).await?;
        }
        Ok(())
    }
}

impl PublisherAsync {
    /// Async version of [`create_datawriter`](crate::publication::publisher::Publisher::create_datawriter).
    #[tracing::instrument(skip(self, a_topic, a_listener))]
    pub async fn create_datawriter<'a, 'b, Foo>(
        &'a self,
        a_topic: &'a TopicAsync,
        qos: QosKind<DataWriterQos>,
        a_listener: Option<Box<dyn DataWriterListenerAsync<Foo = Foo> + Send + 'b>>,
        mask: &'a [StatusKind],
    ) -> DdsResult<DataWriterAsync<Foo>>
    where
        Foo: 'b,
    {
        let default_unicast_locator_list = self
            .participant
            .participant_address()
            .upgrade()?
            .get_default_unicast_locator_list()
            .await;
        let default_multicast_locator_list = self
            .participant
            .participant_address()
            .upgrade()?
            .get_default_multicast_locator_list()
            .await;
        let data_max_size_serialized = self
            .participant
            .participant_address()
            .upgrade()?
            .data_max_size_serialized()
            .await;

        let listener = a_listener.map::<Box<dyn AnyDataWriterListener + Send>, _>(|b| Box::new(b));
        let has_key = a_topic
            .topic_address()
            .upgrade()?
            .get_type_support()
            .await
            .has_key();
        let data_writer_address = self
            .publisher_address
            .upgrade()?
            .create_datawriter(
                a_topic.topic_address().clone(),
                has_key,
                data_max_size_serialized,
                qos,
                listener,
                mask.to_vec(),
                default_unicast_locator_list,
                default_multicast_locator_list,
                self.participant.runtime_handle().clone(),
            )
            .await?;
        let status_condition = data_writer_address.upgrade()?.get_statuscondition().await;
        let data_writer = DataWriterAsync::new(
            data_writer_address,
            status_condition,
            self.clone(),
            a_topic.clone(),
        );

        if self.publisher_address.upgrade()?.is_enabled().await
            && self
                .publisher_address
                .upgrade()?
                .get_qos()
                .await
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

        let message_sender_actor = self
            .participant
            .participant_address()
            .upgrade()?
            .get_message_sender()
            .await;
        a_datawriter
            .writer_address()
            .upgrade()?
            .send_message(message_sender_actor.clone())
            .await;

        let deleted_writer = self
            .publisher_address
            .upgrade()?
            .delete_datawriter(writer_handle)
            .await?;

        self.announce_deleted_data_writer(deleted_writer).await
    }

    /// Async version of [`delete_datawriter`](crate::publication::publisher::Publisher::lookup_datawriter).
    #[tracing::instrument(skip(self))]
    pub async fn lookup_datawriter<Foo>(
        &self,
        topic_name: &str,
    ) -> DdsResult<Option<DataWriterAsync<Foo>>> {
        if let Some((topic_address, topic_status_condition, type_name)) = self
            .participant
            .participant_address()
            .upgrade()?
            .lookup_topicdescription(topic_name.to_string())
            .await
        {
            let topic = TopicAsync::new(
                topic_address,
                topic_status_condition,
                topic_name.to_string(),
                type_name,
                self.participant.clone(),
            );
            if let Some(dw) = self
                .publisher_address
                .upgrade()?
                .lookup_datawriter(topic_name.to_string())
                .await
            {
                let dw = dw.upgrade()?;
                let status_condition = dw.get_statuscondition().await;
                Ok(Some(DataWriterAsync::new(
                    dw.address(),
                    status_condition,
                    self.clone(),
                    topic,
                )))
            } else {
                Ok(None)
            }
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
    pub fn get_participant(&self) -> DomainParticipantAsync {
        self.participant.clone()
    }

    /// Async version of [`delete_contained_entities`](crate::publication::publisher::Publisher::delete_contained_entities).
    #[tracing::instrument(skip(self))]
    pub async fn delete_contained_entities(&self) -> DdsResult<()> {
        let deleted_writer_actor = self
            .publisher_address
            .upgrade()?
            .drain_data_writer_list()
            .await;
        let message_sender_actor = self
            .participant
            .participant_address()
            .upgrade()?
            .get_message_sender()
            .await;

        for writer_actor in deleted_writer_actor {
            writer_actor
                .send_message(message_sender_actor.clone())
                .await;

            self.announce_deleted_data_writer(writer_actor).await?;
        }
        Ok(())
    }

    /// Async version of [`set_default_datawriter_qos`](crate::publication::publisher::Publisher::set_default_datawriter_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_datawriter_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => {
                self.publisher_address
                    .upgrade()?
                    .get_default_datawriter_qos()
                    .await
            }
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        self.publisher_address
            .upgrade()?
            .set_default_datawriter_qos(qos)
            .await;

        Ok(())
    }

    /// Async version of [`get_default_datawriter_qos`](crate::publication::publisher::Publisher::get_default_datawriter_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_datawriter_qos(&self) -> DdsResult<DataWriterQos> {
        Ok(self
            .publisher_address
            .upgrade()?
            .get_default_datawriter_qos()
            .await)
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
        Ok(self.publisher_address.upgrade()?.get_qos().await)
    }

    /// Async version of [`set_listener`](crate::publication::publisher::Publisher::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: Option<Box<dyn PublisherListenerAsync + Send>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.publisher_address
            .upgrade()?
            .set_listener(
                a_listener,
                mask.to_vec(),
                self.participant.runtime_handle().clone(),
            )
            .await;
        Ok(())
    }

    /// Async version of [`get_statuscondition`](crate::publication::publisher::Publisher::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(
            self.status_condition_address.clone(),
            self.participant.runtime_handle().clone(),
        )
    }

    /// Async version of [`get_status_changes`](crate::publication::publisher::Publisher::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::publication::publisher::Publisher::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        self.publisher_address.upgrade()?.enable().await;
        Ok(())
    }

    /// Async version of [`get_instance_handle`](crate::publication::publisher::Publisher::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self
            .publisher_address
            .upgrade()?
            .get_instance_handle()
            .await)
    }
}
