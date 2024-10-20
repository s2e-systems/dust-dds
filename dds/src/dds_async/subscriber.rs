use super::{
    condition::StatusConditionAsync, data_reader::DataReaderAsync,
    data_reader_listener::DataReaderListenerAsync, domain_participant::DomainParticipantAsync,
    subscriber_listener::SubscriberListenerAsync, topic::TopicAsync,
};
use crate::{
    implementation::{
        actor::ActorAddress,
        actors::{
            any_data_reader_listener::AnyDataReaderListener, domain_participant_actor,
            domain_participant_actor::DomainParticipantActor,
        },
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos, TopicQos},
        status::{SampleLostStatus, StatusKind},
    },
};

/// Async version of [`Subscriber`](crate::subscription::subscriber::Subscriber).
#[derive(Clone)]
pub struct SubscriberAsync {
    handle: InstanceHandle,
    participant: DomainParticipantAsync,
}

impl SubscriberAsync {
    pub(crate) fn new(handle: InstanceHandle, participant: DomainParticipantAsync) -> Self {
        Self {
            handle,
            participant,
        }
    }

    pub(crate) fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        self.participant.participant_address()
    }
}

impl SubscriberAsync {
    /// Async version of [`create_datareader`](crate::subscription::subscriber::Subscriber::create_datareader).
    #[tracing::instrument(skip(self, a_topic, a_listener))]
    pub async fn create_datareader<'a, 'b, Foo>(
        &'a self,
        a_topic: &'a TopicAsync,
        qos: QosKind<DataReaderQos>,
        a_listener: Option<Box<(dyn DataReaderListenerAsync<'b, Foo = Foo> + Send + 'b)>>,
        mask: &'a [StatusKind],
    ) -> DdsResult<DataReaderAsync<Foo>>
    where
        Foo: 'b,
    {
        let topic_name = a_topic.get_name();
        let listener = a_listener.map::<Box<dyn AnyDataReaderListener + Send>, _>(|b| Box::new(b));
        let guid = self
            .participant_address()
            .send_actor_mail(domain_participant_actor::CreateUserDefinedDataReader {
                subscriber_handle: self.handle,
                topic_name,
                qos,
                a_listener: listener,
                mask: mask.to_vec(),
                domain_participant_address: self.participant_address().clone(),
            })?
            .receive_reply()
            .await?;

        Ok(DataReaderAsync::new(guid, self.clone(), a_topic.clone()))
    }

    /// Async version of [`delete_datareader`](crate::subscription::subscriber::Subscriber::delete_datareader).
    #[tracing::instrument(skip(self, a_datareader))]
    pub async fn delete_datareader<Foo>(
        &self,
        a_datareader: &DataReaderAsync<Foo>,
    ) -> DdsResult<()> {
        self.participant_address()
            .send_actor_mail(domain_participant_actor::DeleteUserDefinedDataReader {
                handle: a_datareader.get_instance_handle().await,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`lookup_datareader`](crate::subscription::subscriber::Subscriber::lookup_datareader).
    #[tracing::instrument(skip(self))]
    pub async fn lookup_datareader<Foo>(
        &self,
        topic_name: &str,
    ) -> DdsResult<Option<DataReaderAsync<Foo>>> {
        self.participant_address()
            .send_actor_mail(domain_participant_actor::LookupDataReader {
                subscriber_handle: self.handle,
                topic_name: topic_name.to_string(),
            })?
            .receive_reply()
            .await?;
        todo!()
    }

    /// Async version of [`notify_datareaders`](crate::subscription::subscriber::Subscriber::notify_datareaders).
    #[tracing::instrument(skip(self))]
    pub async fn notify_datareaders(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`get_participant`](crate::subscription::subscriber::Subscriber::get_participant).
    #[tracing::instrument(skip(self))]
    pub fn get_participant(&self) -> DomainParticipantAsync {
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
        self.participant_address()
            .send_actor_mail(
                domain_participant_actor::DeleteSubscriberContainedEntities {
                    subscriber_handle: self.handle,
                },
            )?
            .receive_reply()
            .await
    }

    /// Async version of [`set_default_datareader_qos`](crate::subscription::subscriber::Subscriber::set_default_datareader_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_datareader_qos(&self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        self.participant_address()
            .send_actor_mail(domain_participant_actor::SetDefaultDataReaderQos {
                subscriber_handle: self.handle,
                qos,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_default_datareader_qos`](crate::subscription::subscriber::Subscriber::get_default_datareader_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_datareader_qos(&self) -> DdsResult<DataReaderQos> {
        self.participant_address()
            .send_actor_mail(domain_participant_actor::GetDefaultDataReaderQos {
                subscriber_handle: self.handle,
            })?
            .receive_reply()
            .await
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
        self.participant_address()
            .send_actor_mail(domain_participant_actor::SetSubscriberQos {
                subscriber_handle: self.handle,
                qos,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_qos`](crate::subscription::subscriber::Subscriber::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<SubscriberQos> {
        self.participant_address()
            .send_actor_mail(domain_participant_actor::GetSubscriberQos {
                subscriber_handle: self.handle,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`set_listener`](crate::subscription::subscriber::Subscriber::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.participant_address()
            .send_actor_mail(domain_participant_actor::SetSubscriberListener {
                subscriber_handle: self.handle,
                a_listener,
                mask: mask.to_vec(),
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_statuscondition`](crate::subscription::subscriber::Subscriber::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(self.participant_address().clone(), self.handle)
    }

    /// Async version of [`get_status_changes`](crate::subscription::subscriber::Subscriber::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::subscription::subscriber::Subscriber::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        self.participant_address()
            .send_actor_mail(domain_participant_actor::EnableSubscriber {
                subscriber_handle: self.handle,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_instance_handle`](crate::subscription::subscriber::Subscriber::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> InstanceHandle {
        self.handle
    }
}
