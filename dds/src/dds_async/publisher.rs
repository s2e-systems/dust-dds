use super::{
    condition::StatusConditionAsync, data_writer::DataWriterAsync,
    domain_participant::DomainParticipantAsync, topic::TopicAsync,
};
use crate::{
    dcps::runtime::{DdsRuntime, OneshotReceive},
    implementation::{
        domain_participant_backend::domain_participant_actor_mail::{
            DomainParticipantMail, PublisherServiceMail,
        },
        listeners::{
            data_writer_listener::DataWriterListenerActor,
            publisher_listener::PublisherListenerActor,
        },
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind, TopicQos},
        status::StatusKind,
        time::Duration,
    },
    publication::{
        data_writer_listener::DataWriterListener, publisher_listener::PublisherListener,
    },
    runtime::{
        actor::{Actor, ActorAddress},
        mpsc::MpscSender,
    },
};

/// Async version of [`Publisher`](crate::publication::publisher::Publisher).
pub struct PublisherAsync<R: DdsRuntime> {
    handle: InstanceHandle,
    status_condition_address: ActorAddress<StatusConditionActor>,
    participant: DomainParticipantAsync<R>,
}

impl<R: DdsRuntime> Clone for PublisherAsync<R> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            status_condition_address: self.status_condition_address.clone(),
            participant: self.participant.clone(),
        }
    }
}

impl<R: DdsRuntime> PublisherAsync<R> {
    pub(crate) fn new(
        handle: InstanceHandle,
        status_condition_address: ActorAddress<StatusConditionActor>,
        participant: DomainParticipantAsync<R>,
    ) -> Self {
        Self {
            handle,
            status_condition_address,
            participant,
        }
    }

    pub(crate) fn participant_address(&self) -> &MpscSender<DomainParticipantMail<R>> {
        self.participant.participant_address()
    }
}

impl<R: DdsRuntime> PublisherAsync<R> {
    /// Async version of [`create_datawriter`](crate::publication::publisher::Publisher::create_datawriter).
    #[tracing::instrument(skip(self, a_topic, a_listener))]
    pub async fn create_datawriter<'a, 'b, Foo>(
        &'a self,
        a_topic: &'a TopicAsync<R>,
        qos: QosKind<DataWriterQos>,
        a_listener: impl DataWriterListener<'b, R, Foo> + Send + 'static,
        mask: &'a [StatusKind],
    ) -> DdsResult<DataWriterAsync<R, Foo>>
    where
        Foo: 'b,
    {
        let topic_name = a_topic.get_name();
        let status_condition = Actor::spawn::<R>(
            StatusConditionActor::default(),
            self.participant.spawner_handle(),
        );
        let writer_status_condition_address = status_condition.address();
        let listener_sender =
            DataWriterListenerActor::spawn(a_listener, self.participant.spawner_handle());
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.participant_address()
            .send(DomainParticipantMail::Publisher(
                PublisherServiceMail::CreateDataWriter {
                    publisher_handle: self.handle,
                    topic_name,
                    qos,
                    status_condition,
                    listener_sender,
                    mask: mask.to_vec(),
                    participant_address: self.participant_address().clone(),
                    reply_sender,
                },
            ))?;
        let guid = reply_receiver.receive().await??;

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
        a_datawriter: &DataWriterAsync<R, Foo>,
    ) -> DdsResult<()> {
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.participant_address()
            .send(DomainParticipantMail::Publisher(
                PublisherServiceMail::DeleteDataWriter {
                    publisher_handle: self.handle,
                    datawriter_handle: a_datawriter.get_instance_handle().await,
                    reply_sender,
                },
            ))?;
        reply_receiver.receive().await?
    }

    /// Async version of [`delete_datawriter`](crate::publication::publisher::Publisher::lookup_datawriter).
    #[tracing::instrument(skip(self))]
    pub async fn lookup_datawriter<Foo>(
        &self,
        topic_name: &str,
    ) -> DdsResult<Option<DataWriterAsync<R, Foo>>> {
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
    pub fn get_participant(&self) -> DomainParticipantAsync<R> {
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
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.participant_address()
            .send(DomainParticipantMail::Publisher(
                PublisherServiceMail::SetDefaultDataWriterQos {
                    publisher_handle: self.handle,
                    qos,
                    reply_sender,
                },
            ))?;
        reply_receiver.receive().await?
    }

    /// Async version of [`get_default_datawriter_qos`](crate::publication::publisher::Publisher::get_default_datawriter_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_datawriter_qos(&self) -> DdsResult<DataWriterQos> {
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.participant_address()
            .send(DomainParticipantMail::Publisher(
                PublisherServiceMail::GetDefaultDataWriterQos {
                    publisher_handle: self.handle,
                    reply_sender,
                },
            ))?;
        reply_receiver.receive().await?
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

impl<R: DdsRuntime> PublisherAsync<R> {
    /// Async version of [`set_qos`](crate::publication::publisher::Publisher::set_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.participant_address()
            .send(DomainParticipantMail::Publisher(
                PublisherServiceMail::SetPublisherQos {
                    publisher_handle: self.handle,
                    qos,
                    reply_sender,
                },
            ))?;
        reply_receiver.receive().await?
    }

    /// Async version of [`get_qos`](crate::publication::publisher::Publisher::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<PublisherQos> {
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.participant_address()
            .send(DomainParticipantMail::Publisher(
                PublisherServiceMail::GetPublisherQos {
                    publisher_handle: self.handle,
                    reply_sender,
                },
            ))?;
        reply_receiver.receive().await?
    }

    /// Async version of [`set_listener`](crate::publication::publisher::Publisher::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: impl PublisherListener<R> + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        let (reply_sender, mut reply_receiver) = R::oneshot();
        let listener_sender =
            PublisherListenerActor::spawn(a_listener, self.participant.spawner_handle());
        self.participant_address()
            .send(DomainParticipantMail::Publisher(
                PublisherServiceMail::SetPublisherListener {
                    publisher_handle: self.handle,
                    listener_sender,
                    mask: mask.to_vec(),
                    reply_sender,
                },
            ))?;
        reply_receiver.receive().await?
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
        todo!()
    }

    /// Async version of [`get_instance_handle`](crate::publication::publisher::Publisher::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> InstanceHandle {
        self.handle
    }
}
