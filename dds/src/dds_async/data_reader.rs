use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    implementation::{
        actors::{
            data_reader_actor::{self, DataReaderActor},
            data_writer_actor,
            domain_participant_actor::{self, DomainParticipantActor},
            publisher_actor,
            subscriber_actor::{self, SubscriberActor},
            topic_actor::{self, TopicActor},
        },
        data_representation_builtin_endpoints::discovered_reader_data::DiscoveredReaderData,
        utils::{actor::ActorAddress, instance_handle_from_key::get_instance_handle_from_key},
    },
    infrastructure::{
        condition::StatusCondition,
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
        subscriber::Subscriber,
    },
    topic_definition::{
        topic::Topic,
        type_support::{DdsKey, DdsSerialize},
    },
};

use std::marker::PhantomData;

pub struct DataReaderAsync<Foo> {
    reader_address: ActorAddress<DataReaderActor>,
    subscriber_address: ActorAddress<SubscriberActor>,
    participant_address: ActorAddress<DomainParticipantActor>,
    runtime_handle: tokio::runtime::Handle,
    phantom: PhantomData<Foo>,
}

impl<Foo> DataReaderAsync<Foo> {
    pub(crate) fn new(
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        runtime_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            reader_address,
            subscriber_address,
            participant_address,
            runtime_handle,
            phantom: PhantomData,
        }
    }

    pub(crate) fn runtime_handle(&self) -> &tokio::runtime::Handle {
        &self.runtime_handle
    }

    async fn topic_address(&self) -> ActorAddress<TopicActor> {
        let user_defined_topic_list = self
            .participant_address
            .send_mail_and_await_reply(domain_participant_actor::get_user_defined_topic_list::new())
            .await
            .expect("should never fail");
        for topic in user_defined_topic_list {
            if topic
                .send_mail_and_await_reply(topic_actor::get_type_name::new())
                .await
                == self
                    .reader_address
                    .send_mail_and_await_reply(data_reader_actor::get_type_name::new())
                    .await
                && topic
                    .send_mail_and_await_reply(topic_actor::get_name::new())
                    .await
                    == self
                        .reader_address
                        .send_mail_and_await_reply(data_reader_actor::get_topic_name::new())
                        .await
            {
                return topic;
            }
        }
        panic!("Should always exist");
    }
}

impl<Foo> Clone for DataReaderAsync<Foo> {
    fn clone(&self) -> Self {
        Self {
            reader_address: self.reader_address.clone(),
            subscriber_address: self.subscriber_address.clone(),
            participant_address: self.participant_address.clone(),
            runtime_handle: self.runtime_handle.clone(),
            phantom: self.phantom,
        }
    }
}

impl<Foo> DataReaderAsync<Foo> {
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

    #[tracing::instrument(skip(self, _key_holder))]
    pub async fn get_key_value(
        &self,
        _key_holder: &mut Foo,
        _handle: InstanceHandle,
    ) -> DdsResult<()> {
        todo!()
    }

    #[tracing::instrument(skip(self, _instance))]
    pub async fn lookup_instance(&self, _instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        todo!()
    }
}

impl<Foo> DataReaderAsync<Foo> {
    #[tracing::instrument(skip(self))]
    pub async fn get_liveliness_changed_status(&self) -> DdsResult<LivelinessChangedStatus> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_requested_deadline_missed_status(
        &self,
    ) -> DdsResult<RequestedDeadlineMissedStatus> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_requested_incompatible_qos_status(
        &self,
    ) -> DdsResult<RequestedIncompatibleQosStatus> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_sample_lost_status(&self) -> DdsResult<SampleLostStatus> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_sample_rejected_status(&self) -> DdsResult<SampleRejectedStatus> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_subscription_matched_status(&self) -> DdsResult<SubscriptionMatchedStatus> {
        self.reader_address
            .send_mail_and_await_reply(data_reader_actor::get_subscription_matched_status::new())
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_topicdescription(&self) -> DdsResult<Topic> {
        Ok(Topic::new(
            self.topic_address().await,
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        ))
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_subscriber(&self) -> DdsResult<Subscriber> {
        Ok(Subscriber::new(
            self.subscriber_address.clone(),
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        ))
    }

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

    #[tracing::instrument(skip(self))]
    pub async fn get_matched_publications(&self) -> DdsResult<Vec<InstanceHandle>> {
        self.reader_address
            .send_mail_and_await_reply(data_reader_actor::get_matched_publications::new())
            .await
    }
}

impl<Foo> DataReaderAsync<Foo> {
    pub async fn set_qos(&self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        let q = match qos {
            QosKind::Default => {
                self.subscriber_address
                    .send_mail_and_await_reply(subscriber_actor::get_default_datareader_qos::new())
                    .await?
            }
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };
        self.reader_address
            .send_mail_and_await_reply(data_reader_actor::set_qos::new(q))
            .await??;

        if self
            .reader_address
            .send_mail_and_await_reply(data_reader_actor::is_enabled::new())
            .await?
        {
            let type_name = self
                .reader_address
                .send_mail_and_await_reply(data_reader_actor::get_type_name::new())
                .await?;
            let type_support = self
                .participant_address
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
            announce_data_reader(
                &self.participant_address,
                self.reader_address
                    .send_mail_and_await_reply(data_reader_actor::as_discovered_reader_data::new(
                        TopicQos::default(),
                        self.subscriber_address
                            .send_mail_and_await_reply(subscriber_actor::get_qos::new())
                            .await?,
                        self.participant_address
                            .send_mail_and_await_reply(
                                domain_participant_actor::get_default_unicast_locator_list::new(),
                            )
                            .await?,
                        self.participant_address
                            .send_mail_and_await_reply(
                                domain_participant_actor::get_default_multicast_locator_list::new(),
                            )
                            .await?,
                        type_support.xml_type(),
                    ))
                    .await?,
            )
            .await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<DataReaderQos> {
        self.reader_address
            .send_mail_and_await_reply(data_reader_actor::get_qos::new())
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        self.reader_address
            .send_mail_and_await_reply(data_reader_actor::get_statuscondition::new())
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
            .reader_address
            .send_mail_and_await_reply(data_reader_actor::is_enabled::new())
            .await?
        {
            self.reader_address
                .send_mail_and_await_reply(data_reader_actor::enable::new())
                .await?;
        }

        let type_name = self
            .reader_address
            .send_mail_and_await_reply(data_reader_actor::get_type_name::new())
            .await?;
        let type_support = self
            .participant_address
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

        announce_data_reader(
            &self.participant_address,
            self.reader_address
                .send_mail_and_await_reply(data_reader_actor::as_discovered_reader_data::new(
                    TopicQos::default(),
                    self.subscriber_address
                        .send_mail_and_await_reply(subscriber_actor::get_qos::new())
                        .await?,
                    self.participant_address
                        .send_mail_and_await_reply(
                            domain_participant_actor::get_default_unicast_locator_list::new(),
                        )
                        .await?,
                    self.participant_address
                        .send_mail_and_await_reply(
                            domain_participant_actor::get_default_multicast_locator_list::new(),
                        )
                        .await?,
                    type_support.xml_type(),
                ))
                .await?,
        )
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.reader_address
            .send_mail_and_await_reply(data_reader_actor::get_instance_handle::new())
            .await
    }

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
                self.runtime_handle.clone(),
            ))
            .await
    }
}

async fn announce_data_reader(
    domain_participant: &ActorAddress<DomainParticipantActor>,
    discovered_reader_data: DiscoveredReaderData,
) -> DdsResult<()> {
    let mut serialized_data = Vec::new();
    discovered_reader_data.serialize_data(&mut serialized_data)?;
    let timestamp = domain_participant
        .send_mail_and_await_reply(domain_participant_actor::get_current_time::new())
        .await?;

    let builtin_publisher = domain_participant
        .send_mail_and_await_reply(domain_participant_actor::get_builtin_publisher::new())
        .await?;
    let data_writer_list = builtin_publisher
        .send_mail_and_await_reply(publisher_actor::data_writer_list::new())
        .await?;
    for dw in data_writer_list {
        if dw
            .send_mail_and_await_reply(data_writer_actor::get_type_name::new())
            .await
            == Ok("DiscoveredReaderData".to_string())
        {
            let instance_handle = get_instance_handle_from_key(&discovered_reader_data.get_key()?)?;
            dw.send_mail_and_await_reply(data_writer_actor::write_w_timestamp::new(
                serialized_data,
                instance_handle,
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
