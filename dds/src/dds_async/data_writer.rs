use std::marker::PhantomData;

use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    data_representation_builtin_endpoints::discovered_writer_data::{
        DiscoveredWriterData, DCPS_PUBLICATION,
    },
    implementation::{
        actor::ActorAddress,
        actors::{
            any_data_writer_listener::AnyDataWriterListener,
            data_writer_actor::{self, DataWriterActor},
            domain_participant_actor::{self, DomainParticipantActor},
            publisher_actor::{self, PublisherActor},
            status_condition_actor::StatusConditionActor,
            topic_actor,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, QosKind},
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, StatusKind,
        },
        time::{Duration, Time},
    },
    topic_definition::type_support::DdsSerialize,
};

use super::{
    condition::StatusConditionAsync, data_writer_listener::DataWriterListenerAsync,
    publisher::PublisherAsync, topic::TopicAsync,
};

/// Async version of [`DataWriter`](crate::publication::data_writer::DataWriter).
pub struct DataWriterAsync<Foo> {
    writer_address: ActorAddress<DataWriterActor>,
    status_condition_address: ActorAddress<StatusConditionActor>,
    publisher: PublisherAsync,
    topic: TopicAsync,
    phantom: PhantomData<Foo>,
}

impl<Foo> Clone for DataWriterAsync<Foo> {
    fn clone(&self) -> Self {
        Self {
            writer_address: self.writer_address.clone(),
            status_condition_address: self.status_condition_address.clone(),
            publisher: self.publisher.clone(),
            topic: self.topic.clone(),
            phantom: self.phantom,
        }
    }
}

impl<Foo> DataWriterAsync<Foo> {
    pub(crate) fn new(
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
    ) -> Self {
        Self {
            writer_address,
            status_condition_address,
            publisher,
            topic,
            phantom: PhantomData,
        }
    }

    pub(crate) fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        self.publisher.participant_address()
    }

    pub(crate) fn publisher_address(&self) -> &ActorAddress<PublisherActor> {
        self.publisher.publisher_address()
    }

    pub(crate) fn writer_address(&self) -> &ActorAddress<DataWriterActor> {
        &self.writer_address
    }

    async fn announce_writer(&self) -> DdsResult<()> {
        let builtin_publisher = self
            .get_publisher()
            .get_participant()
            .get_builtin_publisher()
            .await?;
        if let Some(sedp_publications_announcer) = builtin_publisher
            .lookup_datawriter::<DiscoveredWriterData>(DCPS_PUBLICATION)
            .await?
        {
            let publisher_qos = self.get_publisher().get_qos().await?;
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
            let discovered_writer_data = self
                .writer_address
                .send_actor_mail(data_writer_actor::AsDiscoveredWriterData {
                    publisher_qos,
                    default_unicast_locator_list,
                    default_multicast_locator_list,
                    topic_data,
                    xml_type,
                })?
                .receive_reply()
                .await?;
            sedp_publications_announcer
                .write(&discovered_writer_data, None)
                .await?;
        }
        Ok(())
    }
}

impl<Foo> DataWriterAsync<Foo>
where
    Foo: DdsSerialize,
{
    /// Async version of [`register_instance`](crate::publication::data_writer::DataWriter::register_instance).
    #[tracing::instrument(skip(self, instance))]
    pub async fn register_instance(&self, instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        let timestamp = self
            .participant_address()
            .send_actor_mail(domain_participant_actor::GetCurrentTime)?
            .receive_reply()
            .await;
        self.register_instance_w_timestamp(instance, timestamp)
            .await
    }

    /// Async version of [`register_instance_w_timestamp`](crate::publication::data_writer::DataWriter::register_instance_w_timestamp).
    #[tracing::instrument(skip(self, instance))]
    pub async fn register_instance_w_timestamp(
        &self,
        instance: &Foo,
        timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        let type_support = self
            .topic
            .topic_address()
            .send_actor_mail(topic_actor::GetTypeSupport)?
            .receive_reply()
            .await;

        let serialized_data = instance.serialize_data()?;
        let instance_handle = type_support.instance_handle_from_serialized_foo(&serialized_data)?;

        self.writer_address
            .send_actor_mail(data_writer_actor::RegisterInstanceWTimestamp {
                instance_handle,
                timestamp,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`unregister_instance`](crate::publication::data_writer::DataWriter::unregister_instance).
    #[tracing::instrument(skip(self, instance))]
    pub async fn unregister_instance(
        &self,
        instance: &Foo,
        handle: Option<InstanceHandle>,
    ) -> DdsResult<()> {
        let timestamp = self
            .participant_address()
            .send_actor_mail(domain_participant_actor::GetCurrentTime)?
            .receive_reply()
            .await;
        self.unregister_instance_w_timestamp(instance, handle, timestamp)
            .await
    }

    /// Async version of [`unregister_instance_w_timestamp`](crate::publication::data_writer::DataWriter::unregister_instance_w_timestamp).
    #[tracing::instrument(skip(self, instance))]
    pub async fn unregister_instance_w_timestamp(
        &self,
        instance: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let type_support = self
            .topic
            .topic_address()
            .send_actor_mail(topic_actor::GetTypeSupport)?
            .receive_reply()
            .await;
        let has_key = type_support.has_key();
        if has_key {
            let instance_handle = match handle {
                Some(h) => {
                    if let Some(stored_handle) = self.lookup_instance(instance).await? {
                        if stored_handle == h {
                            Ok(h)
                        } else {
                            Err(DdsError::PreconditionNotMet(
                                "Handle does not match instance".to_string(),
                            ))
                        }
                    } else {
                        Err(DdsError::BadParameter)
                    }
                }
                None => {
                    if let Some(stored_handle) = self.lookup_instance(instance).await? {
                        Ok(stored_handle)
                    } else {
                        Err(DdsError::PreconditionNotMet(
                            "Instance not registered with this DataWriter".to_string(),
                        ))
                    }
                }
            }?;

            let serialized_foo = instance.serialize_data()?;
            let instance_serialized_key = type_support
                .get_serialized_key_from_serialized_foo(&serialized_foo)?
                .into();

            let message_sender_actor = self
                .participant_address()
                .send_actor_mail(domain_participant_actor::GetMessageSender)?
                .receive_reply()
                .await;
            let now = self
                .participant_address()
                .send_actor_mail(domain_participant_actor::GetCurrentTime)?
                .receive_reply()
                .await;

            self.writer_address
                .send_actor_mail(data_writer_actor::UnregisterInstanceWTimestamp {
                    instance_serialized_key,
                    handle: instance_handle,
                    timestamp,
                    message_sender_actor,
                    now,
                    data_writer_address: self.writer_address.clone(),
                    executor_handle: self.publisher.get_participant().executor_handle().clone(),
                    timer_handle: self.publisher.get_participant().timer_handle().clone(),
                })?
                .receive_reply()
                .await
        } else {
            Err(DdsError::IllegalOperation)
        }
    }

    /// Async version of [`get_key_value`](crate::publication::data_writer::DataWriter::get_key_value).
    #[tracing::instrument(skip(self, _key_holder))]
    pub async fn get_key_value(
        &self,
        _key_holder: &mut Foo,
        _handle: InstanceHandle,
    ) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`lookup_instance`](crate::publication::data_writer::DataWriter::lookup_instance).
    #[tracing::instrument(skip(self, instance))]
    pub async fn lookup_instance(&self, instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        let type_support = self
            .topic
            .topic_address()
            .send_actor_mail(topic_actor::GetTypeSupport)?
            .receive_reply()
            .await;

        let serialized_foo = instance.serialize_data()?;
        let instance_handle = type_support.instance_handle_from_serialized_foo(&serialized_foo)?;

        self.writer_address
            .send_actor_mail(data_writer_actor::LookupInstance { instance_handle })?
            .receive_reply()
            .await
    }

    /// Async version of [`write`](crate::publication::data_writer::DataWriter::write).
    #[tracing::instrument(skip(self, data))]
    pub async fn write(&self, data: &Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        let timestamp = self
            .participant_address()
            .send_actor_mail(domain_participant_actor::GetCurrentTime)?
            .receive_reply()
            .await;
        self.write_w_timestamp(data, handle, timestamp).await
    }

    /// Async version of [`write_w_timestamp`](crate::publication::data_writer::DataWriter::write_w_timestamp).
    #[tracing::instrument(skip(self, data))]
    pub async fn write_w_timestamp(
        &self,
        data: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let type_support = self
            .topic
            .topic_address()
            .send_actor_mail(topic_actor::GetTypeSupport)?
            .receive_reply()
            .await;

        let serialized_data = data.serialize_data()?;
        let key = type_support.instance_handle_from_serialized_foo(&serialized_data)?;

        let message_sender_actor = self
            .participant_address()
            .send_actor_mail(domain_participant_actor::GetMessageSender)?
            .receive_reply()
            .await;
        let now = self
            .participant_address()
            .send_actor_mail(domain_participant_actor::GetCurrentTime)?
            .receive_reply()
            .await;
        self.writer_address
            .send_actor_mail(data_writer_actor::WriteWTimestamp {
                serialized_data: serialized_data.into(),
                instance_handle: key,
                _handle: handle,
                timestamp,
                message_sender_actor,
                now,
                data_writer_address: self.writer_address.clone(),
                executor_handle: self.publisher.get_participant().executor_handle().clone(),
                timer_handle: self.publisher.get_participant().timer_handle().clone(),
            })?
            .receive_reply()
            .await?;

        Ok(())
    }

    /// Async version of [`dispose`](crate::publication::data_writer::DataWriter::dispose).
    #[tracing::instrument(skip(self, data))]
    pub async fn dispose(&self, data: &Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        let timestamp = self
            .participant_address()
            .send_actor_mail(domain_participant_actor::GetCurrentTime)?
            .receive_reply()
            .await;
        self.dispose_w_timestamp(data, handle, timestamp).await
    }

    /// Async version of [`dispose_w_timestamp`](crate::publication::data_writer::DataWriter::dispose_w_timestamp).
    #[tracing::instrument(skip(self, data))]
    pub async fn dispose_w_timestamp(
        &self,
        data: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let instance_handle = match handle {
            Some(h) => {
                if let Some(stored_handle) = self.lookup_instance(data).await? {
                    if stored_handle == h {
                        Ok(h)
                    } else {
                        Err(DdsError::PreconditionNotMet(
                            "Handle does not match instance".to_string(),
                        ))
                    }
                } else {
                    Err(DdsError::BadParameter)
                }
            }
            None => {
                if let Some(stored_handle) = self.lookup_instance(data).await? {
                    Ok(stored_handle)
                } else {
                    Err(DdsError::PreconditionNotMet(
                        "Instance not registered with this DataWriter".to_string(),
                    ))
                }
            }
        }?;

        let type_support = self
            .topic
            .topic_address()
            .send_actor_mail(topic_actor::GetTypeSupport)?
            .receive_reply()
            .await;

        let serialized_foo = data.serialize_data()?;
        let key = type_support.get_serialized_key_from_serialized_foo(&serialized_foo)?;
        let message_sender_actor = self
            .participant_address()
            .send_actor_mail(domain_participant_actor::GetMessageSender)?
            .receive_reply()
            .await;
        let now = self
            .participant_address()
            .send_actor_mail(domain_participant_actor::GetCurrentTime)?
            .receive_reply()
            .await;
        self.writer_address
            .send_actor_mail(data_writer_actor::DisposeWTimestamp {
                instance_serialized_key: key.into(),
                handle: instance_handle,
                timestamp,
                message_sender_actor,
                now,
                data_writer_address: self.writer_address.clone(),
                executor_handle: self.publisher.get_participant().executor_handle().clone(),
                timer_handle: self.publisher.get_participant().timer_handle().clone(),
            })?
            .receive_reply()
            .await
    }
}

impl<Foo> DataWriterAsync<Foo> {
    /// Async version of [`wait_for_acknowledgments`](crate::publication::data_writer::DataWriter::wait_for_acknowledgments).
    #[tracing::instrument(skip(self))]
    pub async fn wait_for_acknowledgments(&self, max_wait: Duration) -> DdsResult<()> {
        let writer_address = self.writer_address.clone();
        self.publisher
            .get_participant()
            .timer_handle()
            .timeout(
                max_wait.into(),
                Box::pin(async move {
                    loop {
                        if writer_address
                            .send_actor_mail(data_writer_actor::AreAllChangesAcknowledge)?
                            .receive_reply()
                            .await
                        {
                            return Ok(());
                        }
                    }
                }),
            )
            .await
            .map_err(|_| DdsError::Timeout)?
    }

    /// Async version of [`get_liveliness_lost_status`](crate::publication::data_writer::DataWriter::get_liveliness_lost_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_liveliness_lost_status(&self) -> DdsResult<LivelinessLostStatus> {
        todo!()
    }

    /// Async version of [`get_offered_deadline_missed_status`](crate::publication::data_writer::DataWriter::get_offered_deadline_missed_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_offered_deadline_missed_status(
        &self,
    ) -> DdsResult<OfferedDeadlineMissedStatus> {
        todo!()
    }

    /// Async version of [`get_offered_incompatible_qos_status`](crate::publication::data_writer::DataWriter::get_offered_incompatible_qos_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_offered_incompatible_qos_status(
        &self,
    ) -> DdsResult<OfferedIncompatibleQosStatus> {
        todo!()
    }

    /// Async version of [`get_publication_matched_status`](crate::publication::data_writer::DataWriter::get_publication_matched_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_publication_matched_status(&self) -> DdsResult<PublicationMatchedStatus> {
        Ok(self
            .writer_address
            .send_actor_mail(data_writer_actor::GetPublicationMatchedStatus)?
            .receive_reply()
            .await)
    }

    /// Async version of [`get_topic`](crate::publication::data_writer::DataWriter::get_topic).
    #[tracing::instrument(skip(self))]
    pub fn get_topic(&self) -> TopicAsync {
        self.topic.clone()
    }

    /// Async version of [`get_publisher`](crate::publication::data_writer::DataWriter::get_publisher).
    #[tracing::instrument(skip(self))]
    pub fn get_publisher(&self) -> PublisherAsync {
        self.publisher.clone()
    }

    /// Async version of [`assert_liveliness`](crate::publication::data_writer::DataWriter::assert_liveliness).
    #[tracing::instrument(skip(self))]
    pub async fn assert_liveliness(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`get_matched_subscription_data`](crate::publication::data_writer::DataWriter::get_matched_subscription_data).
    #[tracing::instrument(skip(self))]
    pub async fn get_matched_subscription_data(
        &self,
        subscription_handle: InstanceHandle,
    ) -> DdsResult<SubscriptionBuiltinTopicData> {
        self.writer_address
            .send_actor_mail(data_writer_actor::GetMatchedSubscriptionData {
                handle: subscription_handle,
            })?
            .receive_reply()
            .await
            .ok_or(DdsError::BadParameter)
    }

    /// Async version of [`get_matched_subscriptions`](crate::publication::data_writer::DataWriter::get_matched_subscriptions).
    #[tracing::instrument(skip(self))]
    pub async fn get_matched_subscriptions(&self) -> DdsResult<Vec<InstanceHandle>> {
        Ok(self
            .writer_address
            .send_actor_mail(data_writer_actor::GetMatchedSubscriptions)?
            .receive_reply()
            .await)
    }
}

impl<Foo> DataWriterAsync<Foo> {
    /// Async version of [`set_qos`](crate::publication::data_writer::DataWriter::set_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => {
                self.publisher_address()
                    .send_actor_mail(publisher_actor::GetDefaultDatawriterQos)?
                    .receive_reply()
                    .await
            }
            QosKind::Specific(q) => q,
        };

        self.writer_address
            .send_actor_mail(data_writer_actor::SetQos { qos })?
            .receive_reply()
            .await?;
        if self
            .writer_address
            .send_actor_mail(data_writer_actor::IsEnabled)?
            .receive_reply()
            .await
        {
            self.announce_writer().await?;
        }

        Ok(())
    }

    /// Async version of [`get_qos`](crate::publication::data_writer::DataWriter::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<DataWriterQos> {
        Ok(self
            .writer_address
            .send_actor_mail(data_writer_actor::GetQos)?
            .receive_reply()
            .await)
    }

    /// Async version of [`get_statuscondition`](crate::publication::data_writer::DataWriter::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(
            self.status_condition_address.clone(),
            self.publisher.get_participant().executor_handle().clone(),
            self.publisher.get_participant().timer_handle().clone(),
        )
    }

    /// Async version of [`get_status_changes`](crate::publication::data_writer::DataWriter::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::publication::data_writer::DataWriter::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        let writer = self.writer_address();
        if !writer
            .send_actor_mail(data_writer_actor::IsEnabled)?
            .receive_reply()
            .await
        {
            let message_sender_actor = self
                .participant_address()
                .send_actor_mail(domain_participant_actor::GetMessageSender)?
                .receive_reply()
                .await;
            writer
                .send_actor_mail(data_writer_actor::Enable {
                    data_writer_address: writer.clone(),
                    message_sender_actor,
                    executor_handle: self.publisher.get_participant().executor_handle().clone(),
                    timer_handle: self.publisher.get_participant().timer_handle().clone(),
                })?
                .receive_reply()
                .await;

            self.announce_writer().await?;
        }
        Ok(())
    }

    /// Async version of [`get_instance_handle`](crate::publication::data_writer::DataWriter::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self
            .writer_address
            .send_actor_mail(data_writer_actor::GetInstanceHandle)?
            .receive_reply()
            .await)
    }
}
impl<'a, Foo> DataWriterAsync<Foo>
where
    Foo: 'a,
{
    /// Async version of [`set_listener`](crate::publication::data_writer::DataWriter::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: Option<Box<dyn DataWriterListenerAsync<'a, Foo = Foo> + Send + 'a>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.writer_address
            .send_actor_mail(data_writer_actor::SetListener {
                listener: a_listener
                    .map::<Box<dyn AnyDataWriterListener + Send>, _>(|b| Box::new(b)),
                status_kind: mask.to_vec(),
            })?
            .receive_reply()
            .await
    }
}
