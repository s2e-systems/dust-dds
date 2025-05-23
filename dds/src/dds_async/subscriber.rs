use super::{
    condition::StatusConditionAsync, data_reader::DataReaderAsync,
    domain_participant::DomainParticipantAsync, topic::TopicAsync,
};
use crate::{
    dcps::{
        actor::{Actor, ActorAddress},
        domain_participant_actor_mail::{DomainParticipantMail, SubscriberServiceMail},
        listeners::{
            data_reader_listener::DataReaderListenerActor,
            subscriber_listener::SubscriberListenerActor,
        },
        status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos, TopicQos},
        status::{SampleLostStatus, StatusKind},
    },
    runtime::{ChannelSend, DdsRuntime, OneshotReceive},
    subscription::{
        data_reader_listener::DataReaderListener, subscriber_listener::SubscriberListener,
    },
};
use alloc::{string::String, vec::Vec};

/// Async version of [`Subscriber`](crate::subscription::subscriber::Subscriber).
pub struct SubscriberAsync<R: DdsRuntime> {
    handle: InstanceHandle,
    status_condition_address: ActorAddress<R, StatusConditionActor<R>>,
    participant: DomainParticipantAsync<R>,
}

impl<R: DdsRuntime> Clone for SubscriberAsync<R> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle,
            status_condition_address: self.status_condition_address.clone(),
            participant: self.participant.clone(),
        }
    }
}

impl<R: DdsRuntime> SubscriberAsync<R> {
    pub(crate) fn new(
        handle: InstanceHandle,
        status_condition_address: ActorAddress<R, StatusConditionActor<R>>,
        participant: DomainParticipantAsync<R>,
    ) -> Self {
        Self {
            handle,
            status_condition_address,
            participant,
        }
    }

    pub(crate) fn participant_address(&self) -> &R::ChannelSender<DomainParticipantMail<R>> {
        self.participant.participant_address()
    }
}

impl<R: DdsRuntime> SubscriberAsync<R> {
    /// Async version of [`create_datareader`](crate::subscription::subscriber::Subscriber::create_datareader).
    #[tracing::instrument(skip(self, a_topic, a_listener))]
    pub async fn create_datareader<Foo>(
        &self,
        a_topic: &TopicAsync<R>,
        qos: QosKind<DataReaderQos>,
        a_listener: Option<impl DataReaderListener<R, Foo> + Send + 'static>,
        mask: &[StatusKind],
    ) -> DdsResult<DataReaderAsync<R, Foo>> {
        let status_condition = Actor::spawn(
            StatusConditionActor::default(),
            self.participant.spawner_handle(),
        );
        let reader_status_condition_address = status_condition.address();
        let listener_sender = a_listener
            .map(|l| DataReaderListenerActor::spawn(l, self.participant.spawner_handle()));
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.participant_address()
            .send(DomainParticipantMail::Subscriber(
                SubscriberServiceMail::CreateDataReader {
                    subscriber_handle: self.handle,
                    topic_name: a_topic.get_name(),
                    qos,
                    status_condition,
                    listener_sender,
                    mask: mask.to_vec(),
                    domain_participant_address: self.participant_address().clone(),
                    reply_sender,
                },
            ))
            .await?;
        let guid = reply_receiver.receive().await??;

        Ok(DataReaderAsync::new(
            guid,
            reader_status_condition_address,
            self.clone(),
            a_topic.clone(),
        ))
    }

    /// Async version of [`delete_datareader`](crate::subscription::subscriber::Subscriber::delete_datareader).
    #[tracing::instrument(skip(self, a_datareader))]
    pub async fn delete_datareader<Foo>(
        &self,
        a_datareader: &DataReaderAsync<R, Foo>,
    ) -> DdsResult<()> {
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.participant_address()
            .send(DomainParticipantMail::Subscriber(
                SubscriberServiceMail::DeleteDataReader {
                    subscriber_handle: self.handle,
                    datareader_handle: a_datareader.get_instance_handle().await,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.receive().await?
    }

    /// Async version of [`lookup_datareader`](crate::subscription::subscriber::Subscriber::lookup_datareader).
    #[tracing::instrument(skip(self))]
    pub async fn lookup_datareader<Foo>(
        &self,
        topic_name: &str,
    ) -> DdsResult<Option<DataReaderAsync<R, Foo>>> {
        if let Some(topic) = self.participant.lookup_topicdescription(topic_name).await? {
            let (reply_sender, mut reply_receiver) = R::oneshot();
            self.participant_address()
                .send(DomainParticipantMail::Subscriber(
                    SubscriberServiceMail::LookupDataReader {
                        subscriber_handle: self.handle,
                        topic_name: String::from(topic_name),
                        reply_sender,
                    },
                ))
                .await?;
            if let Some((reader_handle, reader_status_condition_address)) =
                reply_receiver.receive().await??
            {
                Ok(Some(DataReaderAsync::new(
                    reader_handle,
                    reader_status_condition_address,
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

    /// Async version of [`notify_datareaders`](crate::subscription::subscriber::Subscriber::notify_datareaders).
    #[tracing::instrument(skip(self))]
    pub async fn notify_datareaders(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`get_participant`](crate::subscription::subscriber::Subscriber::get_participant).
    #[tracing::instrument(skip(self))]
    pub fn get_participant(&self) -> DomainParticipantAsync<R> {
        self.participant.clone()
    }

    /// Async version of [`get_sample_lost_status`](crate::subscription::subscriber::Subscriber::get_sample_lost_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_sample_lost_status(&self) -> DdsResult<SampleLostStatus> {
        todo!()
    }

    /// Async version of [`delete_contained_entities`](crate::subscription::subscriber::Subscriber::delete_contained_entities).
    #[tracing::instrument(skip(self))]
    pub async fn delete_contained_entities(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`set_default_datareader_qos`](crate::subscription::subscriber::Subscriber::set_default_datareader_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_datareader_qos(&self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.participant_address()
            .send(DomainParticipantMail::Subscriber(
                SubscriberServiceMail::SetDefaultDataReaderQos {
                    subscriber_handle: self.handle,
                    qos,
                    reply_sender,
                },
            ))
            .await?;

        reply_receiver.receive().await?
    }

    /// Async version of [`get_default_datareader_qos`](crate::subscription::subscriber::Subscriber::get_default_datareader_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_datareader_qos(&self) -> DdsResult<DataReaderQos> {
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.participant_address()
            .send(DomainParticipantMail::Subscriber(
                SubscriberServiceMail::GetDefaultDataReaderQos {
                    subscriber_handle: self.handle,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.receive().await?
    }

    /// Async version of [`copy_from_topic_qos`](crate::subscription::subscriber::Subscriber::copy_from_topic_qos).
    #[tracing::instrument]
    pub async fn copy_from_topic_qos(
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`set_qos`](crate::subscription::subscriber::Subscriber::set_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.participant_address()
            .send(DomainParticipantMail::Subscriber(
                SubscriberServiceMail::SetQos {
                    subscriber_handle: self.handle,
                    qos,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.receive().await?
    }

    /// Async version of [`get_qos`](crate::subscription::subscriber::Subscriber::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<SubscriberQos> {
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.participant_address()
            .send(DomainParticipantMail::Subscriber(
                SubscriberServiceMail::GetSubscriberQos {
                    subscriber_handle: self.handle,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.receive().await?
    }

    /// Async version of [`set_listener`](crate::subscription::subscriber::Subscriber::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: Option<impl SubscriberListener<R> + Send + 'static>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        let (reply_sender, mut reply_receiver) = R::oneshot();
        let listener_sender = a_listener
            .map(|l| SubscriberListenerActor::spawn(l, self.participant.spawner_handle()));
        self.participant_address()
            .send(DomainParticipantMail::Subscriber(
                SubscriberServiceMail::SetListener {
                    subscriber_handle: self.handle,
                    listener_sender,
                    mask: mask.to_vec(),
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.receive().await?
    }

    /// Async version of [`get_statuscondition`](crate::subscription::subscriber::Subscriber::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync<R> {
        StatusConditionAsync::new(
            self.status_condition_address.clone(),
            self.participant.clock_handle().clone(),
        )
    }

    /// Async version of [`get_status_changes`](crate::subscription::subscriber::Subscriber::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::subscription::subscriber::Subscriber::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`get_instance_handle`](crate::subscription::subscriber::Subscriber::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> InstanceHandle {
        self.handle
    }
}
