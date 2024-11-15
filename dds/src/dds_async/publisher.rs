use super::{
    condition::StatusConditionAsync, data_writer::DataWriterAsync,
    data_writer_listener::DataWriterListenerAsync, domain_participant::DomainParticipantAsync,
    publisher_listener::PublisherListenerAsync, topic::TopicAsync,
};
use crate::{
    implementation::{
        actor::ActorAddress,
        actors::{
            domain_participant_backend::{
                any_data_writer_listener::AnyDataWriterListener,
                domain_participant_actor::DomainParticipantActor, publisher_service,
            },
            status_condition_actor::StatusConditionActor,
        },
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind, TopicQos},
        status::StatusKind,
        time::Duration,
    },
};

/// Async version of [`Publisher`](crate::publication::publisher::Publisher).
#[derive(Clone)]
pub struct PublisherAsync {
    handle: InstanceHandle,
    status_condition_address: ActorAddress<StatusConditionActor>,
    participant: DomainParticipantAsync,
}

impl PublisherAsync {
    pub(crate) fn new(
        handle: InstanceHandle,
        status_condition_address: ActorAddress<StatusConditionActor>,
        participant: DomainParticipantAsync,
    ) -> Self {
        Self {
            handle,
            status_condition_address,
            participant,
        }
    }

    pub(crate) fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        self.participant.participant_address()
    }
}

impl PublisherAsync {
    /// Async version of [`create_datawriter`](crate::publication::publisher::Publisher::create_datawriter).
    #[tracing::instrument(skip(self, a_topic, a_listener))]
    pub async fn create_datawriter<'a, 'b, Foo>(
        &'a self,
        a_topic: &'a TopicAsync,
        qos: QosKind<DataWriterQos>,
        a_listener: Option<Box<dyn DataWriterListenerAsync<'b, Foo = Foo> + Send + 'b>>,
        mask: &'a [StatusKind],
    ) -> DdsResult<DataWriterAsync<Foo>>
    where
        Foo: 'b,
    {
        let topic_name = a_topic.get_name();
        let listener = a_listener.map::<Box<dyn AnyDataWriterListener + Send>, _>(|b| Box::new(b));
        let (guid, writer_status_condition_address) = self
            .participant_address()
            .send_actor_mail(publisher_service::CreateDataWriter {
                publisher_handle: self.handle,
                topic_name,
                qos,
                a_listener: listener,
                mask: mask.to_vec(),
            })?
            .receive_reply()
            .await?;

        Ok(DataWriterAsync::new(
            guid,
            writer_status_condition_address,
            self.clone(),
            a_topic.clone(),
        ))
    }

    /// Async version of [`delete_datawriter`](crate::publication::publisher::Publisher::delete_datawriter).
    #[tracing::instrument(skip(self, a_datawriter))]
    pub async fn delete_datawriter<Foo>(
        &self,
        a_datawriter: &DataWriterAsync<Foo>,
    ) -> DdsResult<()> {
        self.participant_address()
            .send_actor_mail(publisher_service::DeleteDataWriter {
                publisher_handle: self.handle,
                datawriter_handle: a_datawriter.get_instance_handle().await,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`delete_datawriter`](crate::publication::publisher::Publisher::lookup_datawriter).
    #[tracing::instrument(skip(self))]
    pub async fn lookup_datawriter<Foo>(
        &self,
        topic_name: &str,
    ) -> DdsResult<Option<DataWriterAsync<Foo>>> {
        self.participant_address()
            .send_actor_mail(publisher_service::LookupDataWriter {
                publisher_handle: self.handle,
                topic_name: topic_name.to_string(),
            })?
            .receive_reply()
            .await?;
        todo!()
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
        self.participant_address()
            .send_actor_mail(publisher_service::DeleteContainedEntities {
                publisher_handle: self.handle,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`set_default_datawriter_qos`](crate::publication::publisher::Publisher::set_default_datawriter_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_datawriter_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        self.participant_address()
            .send_actor_mail(publisher_service::SetDefaultDataWriterQos {
                publisher_handle: self.handle,
                qos,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_default_datawriter_qos`](crate::publication::publisher::Publisher::get_default_datawriter_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_datawriter_qos(&self) -> DdsResult<DataWriterQos> {
        self.participant_address()
            .send_actor_mail(publisher_service::GetDefaultDataWriterQos {
                publisher_handle: self.handle,
            })?
            .receive_reply()
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
    pub async fn set_qos(&self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        self.participant_address()
            .send_actor_mail(publisher_service::SetQos {
                publisher_handle: self.handle,
                qos,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_qos`](crate::publication::publisher::Publisher::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<PublisherQos> {
        self.participant_address()
            .send_actor_mail(publisher_service::GetQos {
                publisher_handle: self.handle,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`set_listener`](crate::publication::publisher::Publisher::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: Option<Box<dyn PublisherListenerAsync + Send>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.participant_address()
            .send_actor_mail(publisher_service::SetListener {
                publisher_handle: self.handle,
                a_listener,
                mask: mask.to_vec(),
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_statuscondition`](crate::publication::publisher::Publisher::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(self.status_condition_address.clone())
    }

    /// Async version of [`get_status_changes`](crate::publication::publisher::Publisher::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::publication::publisher::Publisher::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        self.participant_address()
            .send_actor_mail(publisher_service::Enable {
                publisher_handle: self.handle,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_instance_handle`](crate::publication::publisher::Publisher::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> InstanceHandle {
        self.handle
    }
}
