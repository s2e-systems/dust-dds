use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    data_representation_builtin_endpoints::discovered_reader_data::{
        DiscoveredReaderData, DCPS_SUBSCRIPTION,
    },
    implementation::{
        actor::ActorAddress,
        actors::{
            any_data_reader_listener::AnyDataReaderListener,
            data_reader_actor::{self, DataReaderActor},
            domain_participant_actor::{self, DomainParticipantActor},
            status_condition_actor::StatusConditionActor,
            subscriber_actor::{self, SubscriberActor},
            topic_actor,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind},
        status::{
            LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
            SampleLostStatus, SampleRejectedStatus, StatusKind, SubscriptionMatchedStatus,
        },
        time::Duration,
    },
    subscription::{
        data_reader::Sample,
        sample_info::{
            InstanceStateKind, SampleStateKind, ViewStateKind, ANY_INSTANCE_STATE, ANY_VIEW_STATE,
        },
    },
};

use std::marker::PhantomData;

use super::{
    condition::StatusConditionAsync, data_reader_listener::DataReaderListenerAsync,
    subscriber::SubscriberAsync, topic::TopicAsync,
};

/// Async version of [`DataReader`](crate::subscription::data_reader::DataReader).
pub struct DataReaderAsync<Foo> {
    reader_address: ActorAddress<DataReaderActor>,
    status_condition_address: ActorAddress<StatusConditionActor>,
    subscriber: SubscriberAsync,
    topic: TopicAsync,
    phantom: PhantomData<Foo>,
}

impl<Foo> DataReaderAsync<Foo> {
    pub(crate) fn new(
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
    ) -> Self {
        Self {
            reader_address,
            status_condition_address,
            subscriber,
            topic,
            phantom: PhantomData,
        }
    }

    pub(crate) fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        self.subscriber.participant_address()
    }

    pub(crate) fn subscriber_address(&self) -> &ActorAddress<SubscriberActor> {
        self.subscriber.subscriber_address()
    }

    pub(crate) fn reader_address(&self) -> &ActorAddress<DataReaderActor> {
        &self.reader_address
    }

    async fn announce_reader(&self) -> DdsResult<()> {
        let builtin_publisher = self
            .get_subscriber()
            .get_participant()
            .get_builtin_publisher()
            .await?;
        if let Some(sedp_subscriptions_announcer) = builtin_publisher
            .lookup_datawriter::<DiscoveredReaderData>(DCPS_SUBSCRIPTION)
            .await?
        {
            let subscriber_qos = self
                .subscriber_address()
                .send_actor_mail(subscriber_actor::GetQos)?
                .receive_reply()
                .await;
            let default_unicast_locator_list = self
                .participant_address()
                .send_actor_mail(domain_participant_actor::GetDefaultUnicastLocatorList)?
                .receive_reply()
                .await;
            let default_multicast_locator_list = self
                .participant_address()
                .send_actor_mail(domain_participant_actor::GetDefaultMulticastLocatorList)?
                .receive_reply()
                .await;
            let topic_data = self
                .topic
                .topic_address()
                .send_actor_mail(topic_actor::GetQos)?
                .receive_reply()
                .await
                .topic_data;
            let xml_type = self
                .topic
                .topic_address()
                .send_actor_mail(topic_actor::GetTypeSupport)?
                .receive_reply()
                .await
                .xml_type();
            let discovered_reader_data = self
                .reader_address
                .send_actor_mail(data_reader_actor::AsDiscoveredReaderData {
                    subscriber_qos,
                    default_unicast_locator_list,
                    default_multicast_locator_list,
                    topic_data,
                    xml_type,
                })?
                .receive_reply()
                .await?;

            sedp_subscriptions_announcer
                .write(&discovered_reader_data, None)
                .await?;
        }
        Ok(())
    }
}

impl<Foo> Clone for DataReaderAsync<Foo> {
    fn clone(&self) -> Self {
        Self {
            reader_address: self.reader_address.clone(),
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
        let samples = self
            .reader_address
            .send_actor_mail(data_reader_actor::Read {
                max_samples,
                sample_states: sample_states.to_vec(),
                view_states: view_states.to_vec(),
                instance_states: instance_states.to_vec(),
                specific_instance_handle: None,
            })?
            .receive_reply()
            .await?;

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
        let samples = self
            .reader_address
            .send_actor_mail(data_reader_actor::Take {
                max_samples,
                sample_states: sample_states.to_vec(),
                view_states: view_states.to_vec(),
                instance_states: instance_states.to_vec(),
                specific_instance_handle: None,
            })?
            .receive_reply()
            .await?;

        Ok(samples
            .into_iter()
            .map(|(data, sample_info)| Sample::new(data, sample_info))
            .collect())
    }

    /// Async version of [`read_next_sample`](crate::subscription::data_reader::DataReader::read_next_sample).
    #[tracing::instrument(skip(self))]
    pub async fn read_next_sample(&self) -> DdsResult<Sample<Foo>> {
        let mut samples = self
            .reader_address
            .send_actor_mail(data_reader_actor::Read {
                max_samples: 1,
                sample_states: vec![SampleStateKind::NotRead],
                view_states: ANY_VIEW_STATE.to_vec(),
                instance_states: ANY_INSTANCE_STATE.to_vec(),
                specific_instance_handle: None,
            })?
            .receive_reply()
            .await?;
        let (data, sample_info) = samples.pop().expect("Would return NoData if empty");
        Ok(Sample::new(data, sample_info))
    }

    /// Async version of [`take_next_sample`](crate::subscription::data_reader::DataReader::take_next_sample).
    #[tracing::instrument(skip(self))]
    pub async fn take_next_sample(&self) -> DdsResult<Sample<Foo>> {
        let mut samples = self
            .reader_address
            .send_actor_mail(data_reader_actor::Take {
                max_samples: 1,
                sample_states: vec![SampleStateKind::NotRead],
                view_states: ANY_VIEW_STATE.to_vec(),
                instance_states: ANY_INSTANCE_STATE.to_vec(),
                specific_instance_handle: None,
            })?
            .receive_reply()
            .await?;
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
        let samples = self
            .reader_address
            .send_actor_mail(data_reader_actor::Read {
                max_samples,
                sample_states: sample_states.to_vec(),
                view_states: view_states.to_vec(),
                instance_states: instance_states.to_vec(),
                specific_instance_handle: Some(a_handle),
            })?
            .receive_reply()
            .await?;
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
        let samples = self
            .reader_address
            .send_actor_mail(data_reader_actor::Take {
                max_samples,
                sample_states: sample_states.to_vec(),
                view_states: view_states.to_vec(),
                instance_states: instance_states.to_vec(),
                specific_instance_handle: Some(a_handle),
            })?
            .receive_reply()
            .await?;

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
        let samples = self
            .reader_address
            .send_actor_mail(data_reader_actor::ReadNextInstance {
                max_samples,
                previous_handle,
                sample_states: sample_states.to_vec(),
                view_states: view_states.to_vec(),
                instance_states: instance_states.to_vec(),
            })?
            .receive_reply()
            .await?;
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
        let samples = self
            .reader_address
            .send_actor_mail(data_reader_actor::TakeNextInstance {
                max_samples,
                previous_handle,
                sample_states: sample_states.to_vec(),
                view_states: view_states.to_vec(),
                instance_states: instance_states.to_vec(),
            })?
            .receive_reply()
            .await?;
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
        Ok(self
            .reader_address
            .send_actor_mail(data_reader_actor::GetSubscriptionMatchedStatus)?
            .receive_reply()
            .await)
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
        let reader_address = self.reader_address.clone();
        self.subscriber
            .get_participant()
            .timer_handle()
            .timeout(
                max_wait.into(),
                Box::pin(async move {
                    loop {
                        if reader_address
                            .send_actor_mail(data_reader_actor::IsHistoricalDataReceived)?
                            .receive_reply()
                            .await?
                        {
                            return Ok(());
                        }
                    }
                }),
            )
            .await
            .map_err(|_| DdsError::Timeout)?
    }

    /// Async version of [`get_matched_publication_data`](crate::subscription::data_reader::DataReader::get_matched_publication_data).
    #[tracing::instrument(skip(self))]
    pub async fn get_matched_publication_data(
        &self,
        publication_handle: InstanceHandle,
    ) -> DdsResult<PublicationBuiltinTopicData> {
        self.reader_address
            .send_actor_mail(data_reader_actor::GetMatchedPublicationData { publication_handle })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_matched_publications`](crate::subscription::data_reader::DataReader::get_matched_publications).
    #[tracing::instrument(skip(self))]
    pub async fn get_matched_publications(&self) -> DdsResult<Vec<InstanceHandle>> {
        Ok(self
            .reader_address
            .send_actor_mail(data_reader_actor::GetMatchedPublications)?
            .receive_reply()
            .await)
    }
}

impl<Foo> DataReaderAsync<Foo> {
    /// Async version of [`set_qos`](crate::subscription::data_reader::DataReader::set_qos).
    pub async fn set_qos(&self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => {
                self.subscriber_address()
                    .send_actor_mail(subscriber_actor::GetDefaultDatareaderQos)?
                    .receive_reply()
                    .await
            }
            QosKind::Specific(q) => q,
        };

        self.reader_address
            .send_actor_mail(data_reader_actor::SetQos { qos })?
            .receive_reply()
            .await?;
        if self
            .reader_address
            .send_actor_mail(data_reader_actor::IsEnabled)?
            .receive_reply()
            .await
        {
            self.announce_reader().await?;
        }

        Ok(())
    }

    /// Async version of [`get_qos`](crate::subscription::data_reader::DataReader::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<DataReaderQos> {
        Ok(self
            .reader_address
            .send_actor_mail(data_reader_actor::GetQos)?
            .receive_reply()
            .await)
    }

    /// Async version of [`get_statuscondition`](crate::subscription::data_reader::DataReader::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(
            self.status_condition_address.clone(),
            self.subscriber.get_participant().executor_handle().clone(),
            self.subscriber.get_participant().timer_handle().clone(),
        )
    }

    /// Async version of [`get_status_changes`](crate::subscription::data_reader::DataReader::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::subscription::data_reader::DataReader::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        if !self
            .reader_address
            .send_actor_mail(data_reader_actor::IsEnabled)?
            .receive_reply()
            .await
        {
            self.reader_address
                .send_actor_mail(data_reader_actor::Enable)?
                .receive_reply()
                .await;

            self.announce_reader().await?;
        }
        Ok(())
    }

    /// Async version of [`get_instance_handle`](crate::subscription::data_reader::DataReader::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self
            .reader_address
            .send_actor_mail(data_reader_actor::GetInstanceHandle)?
            .receive_reply()
            .await)
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
        self.reader_address
            .send_actor_mail(data_reader_actor::SetListener {
                listener: a_listener
                    .map::<Box<dyn AnyDataReaderListener + Send>, _>(|b| Box::new(b)),
                status_kind: mask.to_vec(),
            })?
            .receive_reply()
            .await
    }
}
