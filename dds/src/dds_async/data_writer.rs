use std::marker::PhantomData;

use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    implementation::{
        actor::ActorAddress,
        actors::{
            any_data_writer_listener::AnyDataWriterListener, data_writer_actor::DataWriterActor,
            domain_participant_actor::DomainParticipantActor, publisher_actor::PublisherActor,
            status_condition_actor::StatusConditionActor,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, QosKind, TopicQos},
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

    pub(crate) fn runtime_handle(&self) -> &tokio::runtime::Handle {
        self.publisher.runtime_handle()
    }

    pub(crate) fn writer_address(&self) -> &ActorAddress<DataWriterActor> {
        &self.writer_address
    }

    async fn announce_writer(&self) -> DdsResult<()> {
        let type_name = self.writer_address.upgrade()?.get_type_name().await;
        let type_support = self
            .participant_address()
            .upgrade()?
            .get_type_support(type_name.clone())
            .await
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(format!(
                    "Type with name {} not registered with parent domain participant",
                    type_name
                ))
            })?;
        let discovered_writer_data = self
            .writer_address
            .upgrade()?
            .as_discovered_writer_data(
                TopicQos::default(),
                self.publisher_address().upgrade()?.get_qos().await,
                self.participant_address()
                    .upgrade()?
                    .get_default_unicast_locator_list()
                    .await,
                self.participant_address()
                    .upgrade()?
                    .get_default_multicast_locator_list()
                    .await,
                type_support.xml_type(),
            )
            .await;
        self.participant_address()
            .upgrade()?
            .announce_created_or_modified_data_writer(discovered_writer_data)
            .await;

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
            .upgrade()?
            .get_current_time()
            .await;
        self.register_instance_w_timestamp(instance, timestamp)
            .await
    }

    /// Async version of [`register_instance_w_timestamp`](crate::publication::data_writer::DataWriter::register_instance_w_timestamp).
    #[tracing::instrument(skip(self, _instance))]
    pub async fn register_instance_w_timestamp(
        &self,
        _instance: &Foo,
        _timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        todo!()
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
            .upgrade()?
            .get_current_time()
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
        let type_name = self.writer_address.upgrade()?.get_type_name().await;
        let type_support = self
            .participant_address()
            .upgrade()?
            .get_type_support(type_name.clone())
            .await
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(format!(
                    "Type with name {} not registered with parent domain participant",
                    type_name
                ))
            })?;
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

            let mut serialized_foo = Vec::new();
            instance.serialize_data(&mut serialized_foo)?;
            let instance_serialized_key =
                type_support.get_serialized_key_from_serialized_foo(&serialized_foo)?;
            let message_sender_actor = self
                .participant_address()
                .upgrade()?
                .get_message_sender()
                .await;
            let header = self
                .participant_address()
                .upgrade()?
                .get_rtps_message_header()
                .await;
            let now = self
                .participant_address()
                .upgrade()?
                .get_current_time()
                .await;
            let data_writer = self.writer_address.upgrade()?;

            data_writer
                .unregister_instance_w_timestamp(
                    instance_serialized_key,
                    instance_handle,
                    timestamp,
                    message_sender_actor,
                    header,
                    now,
                    data_writer.clone(),
                )
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
        let type_name = self.writer_address.upgrade()?.get_type_name().await;
        let type_support = self
            .participant_address()
            .upgrade()?
            .get_type_support(type_name.clone())
            .await
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(format!(
                    "Type with name {} not registered with parent domain participant",
                    type_name
                ))
            })?;

        let mut serialized_foo = Vec::new();
        instance.serialize_data(&mut serialized_foo)?;
        let instance_handle = type_support.instance_handle_from_serialized_foo(&serialized_foo)?;

        self.writer_address
            .upgrade()?
            .lookup_instance(instance_handle)
            .await
    }

    /// Async version of [`write`](crate::publication::data_writer::DataWriter::write).
    #[tracing::instrument(skip(self, data))]
    pub async fn write(&self, data: &Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        let timestamp = self
            .participant_address()
            .upgrade()?
            .get_current_time()
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
        let type_name = self.writer_address.upgrade()?.get_type_name().await;
        let type_support = self
            .participant_address()
            .upgrade()?
            .get_type_support(type_name.clone())
            .await
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(format!(
                    "Type with name {} not registered with parent domain participant",
                    type_name
                ))
            })?;

        let mut serialized_data = Vec::new();
        data.serialize_data(&mut serialized_data)?;
        let key = type_support.instance_handle_from_serialized_foo(&serialized_data)?;

        let message_sender_actor = self
            .participant_address()
            .upgrade()?
            .get_message_sender()
            .await;
        let header = self
            .participant_address()
            .upgrade()?
            .get_rtps_message_header()
            .await;
        let now = self
            .participant_address()
            .upgrade()?
            .get_current_time()
            .await;
        let data_writer = self.writer_address.upgrade()?;
        data_writer
            .write_w_timestamp(
                serialized_data,
                key,
                handle,
                timestamp,
                message_sender_actor,
                header,
                now,
                data_writer.clone(),
            )
            .await?;

        Ok(())
    }

    /// Async version of [`dispose`](crate::publication::data_writer::DataWriter::dispose).
    #[tracing::instrument(skip(self, data))]
    pub async fn dispose(&self, data: &Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        let timestamp = self
            .participant_address()
            .upgrade()?
            .get_current_time()
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

        let type_name = self.writer_address.upgrade()?.get_type_name().await;
        let type_support = self
            .participant_address()
            .upgrade()?
            .get_type_support(type_name.clone())
            .await
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(format!(
                    "Type with name {} not registered with parent domain participant",
                    type_name
                ))
            })?;

        let mut serialized_foo = Vec::new();
        data.serialize_data(&mut serialized_foo)?;
        let key = type_support.get_serialized_key_from_serialized_foo(&serialized_foo)?;
        let message_sender_actor = self
            .participant_address()
            .upgrade()?
            .get_message_sender()
            .await;
        let header = self
            .participant_address()
            .upgrade()?
            .get_rtps_message_header()
            .await;
        let now = self
            .participant_address()
            .upgrade()?
            .get_current_time()
            .await;
        let data_writer = self.writer_address.upgrade()?;
        data_writer
            .dispose_w_timestamp(
                key,
                instance_handle,
                timestamp,
                message_sender_actor,
                header,
                now,
                data_writer.clone(),
            )
            .await
    }
}

impl<Foo> DataWriterAsync<Foo> {
    /// Async version of [`wait_for_acknowledgments`](crate::publication::data_writer::DataWriter::wait_for_acknowledgments).
    #[tracing::instrument(skip(self))]
    pub async fn wait_for_acknowledgments(&self, max_wait: Duration) -> DdsResult<()> {
        tokio::time::timeout(max_wait.into(), async {
            loop {
                if self
                    .writer_address
                    .upgrade()?
                    .are_all_changes_acknowledge()
                    .await
                {
                    return Ok(());
                }
            }
        })
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
            .upgrade()?
            .get_publication_matched_status()
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
            .upgrade()?
            .get_matched_subscription_data(subscription_handle)
            .await
            .ok_or(DdsError::BadParameter)
    }

    /// Async version of [`get_matched_subscriptions`](crate::publication::data_writer::DataWriter::get_matched_subscriptions).
    #[tracing::instrument(skip(self))]
    pub async fn get_matched_subscriptions(&self) -> DdsResult<Vec<InstanceHandle>> {
        Ok(self
            .writer_address
            .upgrade()?
            .get_matched_subscriptions()
            .await)
    }
}

impl<Foo> DataWriterAsync<Foo> {
    /// Async version of [`set_qos`](crate::publication::data_writer::DataWriter::set_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        let q = match qos {
            QosKind::Default => {
                self.publisher_address()
                    .upgrade()?
                    .get_default_datawriter_qos()
                    .await
            }
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        if self.writer_address.upgrade()?.is_enabled().await {
            let current_qos = self.get_qos().await?;
            q.check_immutability(&current_qos)?;

            self.writer_address.upgrade()?.set_qos(q).await;

            self.announce_writer().await?;
        } else {
            self.writer_address.upgrade()?.set_qos(q).await;
        }

        Ok(())
    }

    /// Async version of [`get_qos`](crate::publication::data_writer::DataWriter::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<DataWriterQos> {
        Ok(self.writer_address.upgrade()?.get_qos().await)
    }

    /// Async version of [`get_statuscondition`](crate::publication::data_writer::DataWriter::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(
            self.status_condition_address.clone(),
            self.runtime_handle().clone(),
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
        if !self.writer_address.upgrade()?.is_enabled().await {
            self.writer_address.upgrade()?.enable().await;

            self.announce_writer().await?;
        }
        Ok(())
    }

    /// Async version of [`get_instance_handle`](crate::publication::data_writer::DataWriter::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.writer_address.upgrade()?.get_instance_handle().await)
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
        a_listener: Option<Box<dyn DataWriterListenerAsync<Foo = Foo> + Send + 'a>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.writer_address
            .upgrade()?
            .set_listener(
                a_listener.map::<Box<dyn AnyDataWriterListener + Send>, _>(|b| Box::new(b)),
                mask.to_vec(),
                self.runtime_handle().clone(),
            )
            .await;
        Ok(())
    }
}
