use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    implementation::{
        actors::{
            data_reader_actor::{self, DataReaderActor},
            domain_participant_actor::{self, DomainParticipantActor},
            subscriber_actor::{self, SubscriberActor},
        },
        utils::actor::ActorAddress,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, TopicQos},
        status::{
            LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
            SampleLostStatus, SampleRejectedStatus, StatusKind, SubscriptionMatchedStatus,
        },
        time::Duration,
    },
    subscription::{
        data_reader::Sample,
        data_reader_listener::DataReaderListener,
        sample_info::{
            InstanceStateKind, SampleStateKind, ViewStateKind, ANY_INSTANCE_STATE, ANY_VIEW_STATE,
        },
    },
};

use std::marker::PhantomData;

use super::{condition::StatusConditionAsync, subscriber::SubscriberAsync, topic::TopicAsync};

/// Async version of [`DataReader`](crate::subscription::data_reader::DataReader).
pub struct DataReaderAsync<Foo> {
    reader_address: ActorAddress<DataReaderActor>,
    subscriber: SubscriberAsync,
    topic: TopicAsync,
    phantom: PhantomData<Foo>,
}

impl<Foo> DataReaderAsync<Foo> {
    pub(crate) fn new(
        reader_address: ActorAddress<DataReaderActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
    ) -> Self {
        Self {
            reader_address,
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

    pub(crate) fn runtime_handle(&self) -> &tokio::runtime::Handle {
        self.subscriber.runtime_handle()
    }

    async fn announce_reader(&self) -> DdsResult<()> {
        let type_name = self
            .reader_address
            .send_mail_and_await_reply(data_reader_actor::get_type_name::new())
            .await?;
        let type_support = self
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
        let discovered_reader_data = self
            .reader_address
            .send_mail_and_await_reply(data_reader_actor::as_discovered_reader_data::new(
                TopicQos::default(),
                self.subscriber_address()
                    .send_mail_and_await_reply(subscriber_actor::get_qos::new())
                    .await?,
                self.participant_address()
                    .send_mail_and_await_reply(
                        domain_participant_actor::get_default_unicast_locator_list::new(),
                    )
                    .await?,
                self.participant_address()
                    .send_mail_and_await_reply(
                        domain_participant_actor::get_default_multicast_locator_list::new(),
                    )
                    .await?,
                type_support.xml_type(),
            ))
            .await?;
        self.participant_address()
            .send_mail(
                domain_participant_actor::announce_created_or_modified_data_reader::new(
                    discovered_reader_data,
                ),
            )
            .await
    }
}

impl<Foo> Clone for DataReaderAsync<Foo> {
    fn clone(&self) -> Self {
        Self {
            reader_address: self.reader_address.clone(),
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
            .send_mail_and_await_reply(data_reader_actor::read::new(
                max_samples,
                sample_states.to_vec(),
                view_states.to_vec(),
                instance_states.to_vec(),
                None,
            ))
            .await??;

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
            .send_mail_and_await_reply(data_reader_actor::take::new(
                max_samples,
                sample_states.to_vec(),
                view_states.to_vec(),
                instance_states.to_vec(),
                None,
            ))
            .await??;

        Ok(samples
            .into_iter()
            .map(|(data, sample_info)| Sample::new(data, sample_info))
            .collect())
    }

    /// Async version of [`read_next_sample`](crate::subscription::data_reader::DataReader::read_next_sample).
    #[tracing::instrument(skip(self))]
    pub async fn read_next_sample(&self) -> DdsResult<Sample<Foo>> {
        let mut samples = {
            self.reader_address
                .send_mail_and_await_reply(data_reader_actor::read::new(
                    1,
                    vec![SampleStateKind::NotRead],
                    ANY_VIEW_STATE.to_vec(),
                    ANY_INSTANCE_STATE.to_vec(),
                    None,
                ))
                .await??
        };
        let (data, sample_info) = samples.pop().expect("Would return NoData if empty");
        Ok(Sample::new(data, sample_info))
    }

    /// Async version of [`take_next_sample`](crate::subscription::data_reader::DataReader::take_next_sample).
    #[tracing::instrument(skip(self))]
    pub async fn take_next_sample(&self) -> DdsResult<Sample<Foo>> {
        let mut samples = self
            .reader_address
            .send_mail_and_await_reply(data_reader_actor::take::new(
                1,
                vec![SampleStateKind::NotRead],
                ANY_VIEW_STATE.to_vec(),
                ANY_INSTANCE_STATE.to_vec(),
                None,
            ))
            .await??;
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
            .send_mail_and_await_reply(data_reader_actor::read::new(
                max_samples,
                sample_states.to_vec(),
                view_states.to_vec(),
                instance_states.to_vec(),
                Some(a_handle),
            ))
            .await??;
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
            .send_mail_and_await_reply(data_reader_actor::take::new(
                max_samples,
                sample_states.to_vec(),
                view_states.to_vec(),
                instance_states.to_vec(),
                Some(a_handle),
            ))
            .await??;
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
            .send_mail_and_await_reply(data_reader_actor::read_next_instance::new(
                max_samples,
                previous_handle,
                sample_states.to_vec(),
                view_states.to_vec(),
                instance_states.to_vec(),
            ))
            .await??;
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
            .send_mail_and_await_reply(data_reader_actor::take_next_instance::new(
                max_samples,
                previous_handle,
                sample_states.to_vec(),
                view_states.to_vec(),
                instance_states.to_vec(),
            ))
            .await??;
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
        self.reader_address
            .send_mail_and_await_reply(data_reader_actor::get_subscription_matched_status::new())
            .await
    }

    /// Async version of [`get_topicdescription`](crate::subscription::data_reader::DataReader::get_topicdescription).
    #[tracing::instrument(skip(self))]
    pub async fn get_topicdescription(&self) -> TopicAsync {
        self.topic.clone()
    }

    /// Async version of [`get_subscriber`](crate::subscription::data_reader::DataReader::get_subscriber).
    #[tracing::instrument(skip(self))]
    pub async fn get_subscriber(&self) -> SubscriberAsync {
        self.subscriber.clone()
    }

    /// Async version of [`wait_for_historical_data`](crate::subscription::data_reader::DataReader::wait_for_historical_data).
    #[tracing::instrument(skip(self))]
    pub async fn wait_for_historical_data(&self, max_wait: Duration) -> DdsResult<()> {
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < std::time::Duration::from(max_wait) {
            if self
                .reader_address
                .send_mail_and_await_reply(data_reader_actor::is_historical_data_received::new())
                .await??
            {
                return Ok(());
            }
        }

        Err(DdsError::Timeout)
    }

    /// Async version of [`get_matched_publication_data`](crate::subscription::data_reader::DataReader::get_matched_publication_data).
    #[tracing::instrument(skip(self))]
    pub async fn get_matched_publication_data(
        &self,
        publication_handle: InstanceHandle,
    ) -> DdsResult<PublicationBuiltinTopicData> {
        self.reader_address
            .send_mail_and_await_reply(data_reader_actor::get_matched_publication_data::new(
                publication_handle,
            ))
            .await?
    }

    /// Async version of [`get_matched_publications`](crate::subscription::data_reader::DataReader::get_matched_publications).
    #[tracing::instrument(skip(self))]
    pub async fn get_matched_publications(&self) -> DdsResult<Vec<InstanceHandle>> {
        self.reader_address
            .send_mail_and_await_reply(data_reader_actor::get_matched_publications::new())
            .await
    }
}

impl<Foo> DataReaderAsync<Foo> {
    /// Async version of [`set_qos`](crate::subscription::data_reader::DataReader::set_qos).
    pub async fn set_qos(&self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        let q = match qos {
            QosKind::Default => {
                self.subscriber_address()
                    .send_mail_and_await_reply(subscriber_actor::get_default_datareader_qos::new())
                    .await?
            }
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        if self
            .reader_address
            .send_mail_and_await_reply(data_reader_actor::is_enabled::new())
            .await?
        {
            let current_qos = self.get_qos().await?;
            q.check_immutability(&current_qos)?;
            self.reader_address
                .send_mail_and_await_reply(data_reader_actor::set_qos::new(q))
                .await?;

            self.announce_reader().await?;
        } else {
            self.reader_address
                .send_mail_and_await_reply(data_reader_actor::set_qos::new(q))
                .await?;
        }

        Ok(())
    }

    /// Async version of [`get_qos`](crate::subscription::data_reader::DataReader::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<DataReaderQos> {
        self.reader_address
            .send_mail_and_await_reply(data_reader_actor::get_qos::new())
            .await
    }

    /// Async version of [`get_statuscondition`](crate::subscription::data_reader::DataReader::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub async fn get_statuscondition(&self) -> DdsResult<StatusConditionAsync> {
        self.reader_address
            .send_mail_and_await_reply(data_reader_actor::get_statuscondition::new())
            .await
            .map(|c| StatusConditionAsync::new(c, self.runtime_handle().clone()))
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
            .send_mail_and_await_reply(data_reader_actor::is_enabled::new())
            .await?
        {
            self.reader_address
                .send_mail_and_await_reply(data_reader_actor::enable::new())
                .await?;

            self.announce_reader().await?;
        }
        Ok(())
    }

    /// Async version of [`get_instance_handle`](crate::subscription::data_reader::DataReader::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.reader_address
            .send_mail_and_await_reply(data_reader_actor::get_instance_handle::new())
            .await
    }

    /// Async version of [`set_listener`](crate::subscription::data_reader::DataReader::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: impl DataReaderListener<Foo = Foo> + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.reader_address
            .send_mail_and_await_reply(data_reader_actor::set_listener::new(
                Box::new(a_listener),
                mask.to_vec(),
                self.runtime_handle().clone(),
            ))
            .await
    }
}
