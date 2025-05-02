use tracing::warn;

use super::{
    condition::StatusConditionAsync, data_reader_listener::DataReaderListenerAsync,
    subscriber::SubscriberAsync, topic::TopicAsync,
};
use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    implementation::{
        any_data_reader_listener::AnyDataReaderListener,
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            domain_participant_actor_mail::{DomainParticipantMail, ReaderServiceMail},
            services::data_reader_service,
        },
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind},
        status::{
            LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
            SampleLostStatus, SampleRejectedStatus, StatusKind, SubscriptionMatchedStatus,
        },
        time::Duration,
    },
    runtime::{actor::ActorAddress, oneshot::oneshot},
    subscription::{
        data_reader::Sample,
        sample_info::{
            InstanceStateKind, SampleStateKind, ViewStateKind, ANY_INSTANCE_STATE, ANY_VIEW_STATE,
        },
    },
};
use std::marker::PhantomData;

/// Async version of [`DataReader`](crate::subscription::data_reader::DataReader).
pub struct DataReaderAsync<Foo> {
    handle: InstanceHandle,
    status_condition_address: ActorAddress<StatusConditionActor>,
    subscriber: SubscriberAsync,
    topic: TopicAsync,
    phantom: PhantomData<Foo>,
}

impl<Foo> DataReaderAsync<Foo> {
    pub(crate) fn new(
        handle: InstanceHandle,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
    ) -> Self {
        Self {
            handle,
            status_condition_address,
            subscriber,
            topic,
            phantom: PhantomData,
        }
    }

    pub(crate) fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        self.subscriber.participant_address()
    }

    pub(crate) fn change_foo_type<T>(self) -> DataReaderAsync<T> {
        DataReaderAsync {
            handle: self.handle,
            status_condition_address: self.status_condition_address,
            subscriber: self.subscriber,
            topic: self.topic,
            phantom: PhantomData,
        }
    }
}

impl<Foo> Clone for DataReaderAsync<Foo> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle,
            status_condition_address: self.status_condition_address.clone(),
            subscriber: self.subscriber.clone(),
            topic: self.topic.clone(),
            phantom: self.phantom,
        }
    }
}

impl<Foo> DataReaderAsync<Foo> {
    /// Async version of [`read`](crate::subscription::data_reader::DataReader::read).
    #[tracing::instrument(skip(self))]
    pub async fn read(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send_actor_mail(data_reader_service::Read {
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                max_samples,
                sample_states: sample_states.to_vec(),
                view_states: view_states.to_vec(),
                instance_states: instance_states.to_vec(),
                specific_instance_handle: None,
                reply_sender,
            })?;
        let samples = reply_receiver.await??;

        Ok(samples
            .into_iter()
            .map(|(data, sample_info)| Sample::new(data, sample_info))
            .collect())
    }

    /// Async version of [`take`](crate::subscription::data_reader::DataReader::take).
    #[tracing::instrument(skip(self))]
    pub async fn take(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send_actor_mail(data_reader_service::Take {
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                max_samples,
                sample_states: sample_states.to_vec(),
                view_states: view_states.to_vec(),
                instance_states: instance_states.to_vec(),
                specific_instance_handle: None,
                reply_sender,
            })?;
        let samples = reply_receiver.await??;

        Ok(samples
            .into_iter()
            .map(|(data, sample_info)| Sample::new(data, sample_info))
            .collect())
    }

    /// Async version of [`read_next_sample`](crate::subscription::data_reader::DataReader::read_next_sample).
    #[tracing::instrument(skip(self))]
    pub async fn read_next_sample(&self) -> DdsResult<Sample<Foo>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send_actor_mail(data_reader_service::Read {
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                max_samples: 1,
                sample_states: vec![SampleStateKind::NotRead],
                view_states: ANY_VIEW_STATE.to_vec(),
                instance_states: ANY_INSTANCE_STATE.to_vec(),
                specific_instance_handle: None,
                reply_sender,
            })?;
        let mut samples = reply_receiver.await??;
        let (data, sample_info) = samples.pop().expect("Would return NoData if empty");
        Ok(Sample::new(data, sample_info))
    }

    /// Async version of [`take_next_sample`](crate::subscription::data_reader::DataReader::take_next_sample).
    #[tracing::instrument(skip(self))]
    pub async fn take_next_sample(&self) -> DdsResult<Sample<Foo>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send_actor_mail(data_reader_service::Take {
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                max_samples: 1,
                sample_states: vec![SampleStateKind::NotRead],
                view_states: ANY_VIEW_STATE.to_vec(),
                instance_states: ANY_INSTANCE_STATE.to_vec(),
                specific_instance_handle: None,
                reply_sender,
            })?;
        let mut samples = reply_receiver.await??;
        let (data, sample_info) = samples.pop().expect("Would return NoData if empty");
        Ok(Sample::new(data, sample_info))
    }

    /// Async version of [`read_instance`](crate::subscription::data_reader::DataReader::read_instance).
    #[tracing::instrument(skip(self))]
    pub async fn read_instance(
        &self,
        max_samples: i32,
        a_handle: InstanceHandle,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send_actor_mail(data_reader_service::Read {
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                max_samples,
                sample_states: sample_states.to_vec(),
                view_states: view_states.to_vec(),
                instance_states: instance_states.to_vec(),
                specific_instance_handle: Some(a_handle),
                reply_sender,
            })?;
        let samples = reply_receiver.await??;
        Ok(samples
            .into_iter()
            .map(|(data, sample_info)| Sample::new(data, sample_info))
            .collect())
    }

    /// Async version of [`take_instance`](crate::subscription::data_reader::DataReader::take_instance).
    #[tracing::instrument(skip(self))]
    pub async fn take_instance(
        &self,
        max_samples: i32,
        a_handle: InstanceHandle,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send_actor_mail(data_reader_service::Take {
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                max_samples,
                sample_states: sample_states.to_vec(),
                view_states: view_states.to_vec(),
                instance_states: instance_states.to_vec(),
                specific_instance_handle: Some(a_handle),
                reply_sender,
            })?;
        let samples = reply_receiver.await??;

        Ok(samples
            .into_iter()
            .map(|(data, sample_info)| Sample::new(data, sample_info))
            .collect())
    }

    /// Async version of [`read_next_instance`](crate::subscription::data_reader::DataReader::read_next_instance).
    #[tracing::instrument(skip(self))]
    pub async fn read_next_instance(
        &self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send_actor_mail(data_reader_service::ReadNextInstance {
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                max_samples,
                previous_handle,
                sample_states: sample_states.to_vec(),
                view_states: view_states.to_vec(),
                instance_states: instance_states.to_vec(),
                reply_sender,
            })?;
        let samples = reply_receiver.await??;
        Ok(samples
            .into_iter()
            .map(|(data, sample_info)| Sample::new(data, sample_info))
            .collect())
    }

    /// Async version of [`take_next_instance`](crate::subscription::data_reader::DataReader::take_next_instance).
    #[tracing::instrument(skip(self))]
    pub async fn take_next_instance(
        &self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send_actor_mail(data_reader_service::TakeNextInstance {
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                max_samples,
                previous_handle,
                sample_states: sample_states.to_vec(),
                view_states: view_states.to_vec(),
                instance_states: instance_states.to_vec(),
                reply_sender,
            })?;
        let samples = reply_receiver.await??;
        Ok(samples
            .into_iter()
            .map(|(data, sample_info)| Sample::new(data, sample_info))
            .collect())
    }

    /// Async version of [`get_key_value`](crate::subscription::data_reader::DataReader::get_key_value).
    #[tracing::instrument(skip(self, _key_holder))]
    pub async fn get_key_value(
        &self,
        _key_holder: &mut Foo,
        _handle: InstanceHandle,
    ) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`lookup_instance`](crate::subscription::data_reader::DataReader::lookup_instance).
    #[tracing::instrument(skip(self, _instance))]
    pub async fn lookup_instance(&self, _instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        todo!()
    }
}

impl<Foo> DataReaderAsync<Foo> {
    /// Async version of [`get_liveliness_changed_status`](crate::subscription::data_reader::DataReader::get_liveliness_changed_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_liveliness_changed_status(&self) -> DdsResult<LivelinessChangedStatus> {
        todo!()
    }

    /// Async version of [`get_requested_deadline_missed_status`](crate::subscription::data_reader::DataReader::get_requested_deadline_missed_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_requested_deadline_missed_status(
        &self,
    ) -> DdsResult<RequestedDeadlineMissedStatus> {
        todo!()
    }

    /// Async version of [`get_requested_incompatible_qos_status`](crate::subscription::data_reader::DataReader::get_requested_incompatible_qos_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_requested_incompatible_qos_status(
        &self,
    ) -> DdsResult<RequestedIncompatibleQosStatus> {
        todo!()
    }

    /// Async version of [`get_sample_lost_status`](crate::subscription::data_reader::DataReader::get_sample_lost_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_sample_lost_status(&self) -> DdsResult<SampleLostStatus> {
        todo!()
    }

    /// Async version of [`get_sample_rejected_status`](crate::subscription::data_reader::DataReader::get_sample_rejected_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_sample_rejected_status(&self) -> DdsResult<SampleRejectedStatus> {
        todo!()
    }

    /// Async version of [`get_subscription_matched_status`](crate::subscription::data_reader::DataReader::get_subscription_matched_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_subscription_matched_status(&self) -> DdsResult<SubscriptionMatchedStatus> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address().send_actor_mail(
            data_reader_service::GetSubscriptionMatchedStatus {
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                reply_sender,
            },
        )?;
        reply_receiver.await?
    }

    /// Async version of [`get_topicdescription`](crate::subscription::data_reader::DataReader::get_topicdescription).
    #[tracing::instrument(skip(self))]
    pub fn get_topicdescription(&self) -> TopicAsync {
        self.topic.clone()
    }

    /// Async version of [`get_subscriber`](crate::subscription::data_reader::DataReader::get_subscriber).
    #[tracing::instrument(skip(self))]
    pub fn get_subscriber(&self) -> SubscriberAsync {
        self.subscriber.clone()
    }

    /// Async version of [`wait_for_historical_data`](crate::subscription::data_reader::DataReader::wait_for_historical_data).
    #[tracing::instrument(skip(self))]
    pub async fn wait_for_historical_data(&self, max_wait: Duration) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send_actor_mail(data_reader_service::WaitForHistoricalData {
                participant_address: self.participant_address().clone(),
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                max_wait,
                reply_sender,
            })?;
        reply_receiver.await?.await
    }

    /// Async version of [`get_matched_publication_data`](crate::subscription::data_reader::DataReader::get_matched_publication_data).
    #[tracing::instrument(skip(self))]
    pub async fn get_matched_publication_data(
        &self,
        publication_handle: InstanceHandle,
    ) -> DdsResult<PublicationBuiltinTopicData> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address().send_actor_mail(
            data_reader_service::GetMatchedPublicationData {
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                publication_handle,
                reply_sender,
            },
        )?;

        reply_receiver.await?
    }

    /// Async version of [`get_matched_publications`](crate::subscription::data_reader::DataReader::get_matched_publications).
    #[tracing::instrument(skip(self))]
    pub async fn get_matched_publications(&self) -> DdsResult<Vec<InstanceHandle>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address().send_actor_mail(
            data_reader_service::GetMatchedPublications {
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                reply_sender,
            },
        )?;
        reply_receiver.await?
    }
}

impl<Foo> DataReaderAsync<Foo> {
    /// Async version of [`set_qos`](crate::subscription::data_reader::DataReader::set_qos).
    pub async fn set_qos(&self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send_actor_mail(data_reader_service::SetQos {
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                qos,
                participant_address: self.participant_address().clone(),
                reply_sender,
            })?;
        reply_receiver.await?
    }

    /// Async version of [`get_qos`](crate::subscription::data_reader::DataReader::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<DataReaderQos> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send_actor_mail(data_reader_service::GetQos {
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                reply_sender,
            })?;
        reply_receiver.await?
    }

    /// Async version of [`get_statuscondition`](crate::subscription::data_reader::DataReader::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(self.status_condition_address.clone())
    }

    /// Async version of [`get_status_changes`](crate::subscription::data_reader::DataReader::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::subscription::data_reader::DataReader::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send_actor_mail(DomainParticipantMail::Reader(ReaderServiceMail::Enable {
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                participant_address: self.participant_address().clone(),
                reply_sender,
            }))?;
        reply_receiver.await?
    }

    /// Async version of [`get_instance_handle`](crate::subscription::data_reader::DataReader::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> InstanceHandle {
        self.handle
    }
}

impl<'a, Foo> DataReaderAsync<Foo>
where
    Foo: 'a,
{
    /// Async version of [`set_listener`](crate::subscription::data_reader::DataReader::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: Option<Box<dyn DataReaderListenerAsync<'a, Foo = Foo> + Send + 'a>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        let listener = a_listener.map::<Box<dyn AnyDataReaderListener>, _>(|l| Box::new(l));
        self.participant_address()
            .send_actor_mail(data_reader_service::SetListener {
                subscriber_handle: self.subscriber.get_instance_handle().await,
                data_reader_handle: self.handle,
                listener,
                listener_mask: mask.to_vec(),
                reply_sender,
            })?;
        reply_receiver.await?
    }
}
