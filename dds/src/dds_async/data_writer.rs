use std::{marker::PhantomData, time::Instant};

use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    implementation::{
        actors::{
            data_writer_actor::{self, DataWriterActor},
            domain_participant_actor::{self, DomainParticipantActor},
            publisher_actor::{self, PublisherActor},
            topic_actor::{self, TopicActor},
        },
        utils::actor::ActorAddress,
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
    publication::data_writer_listener::DataWriterListener,
    topic_definition::type_support::DdsSerialize,
};

use super::{condition::StatusConditionAsync, publisher::PublisherAsync, topic::TopicAsync};

/// Async version of ['DataWriter'](crate::publication::data_writer::DataWriter).
pub struct DataWriterAsync<Foo> {
    writer_address: ActorAddress<DataWriterActor>,
    publisher_address: ActorAddress<PublisherActor>,
    participant_address: ActorAddress<DomainParticipantActor>,
    runtime_handle: tokio::runtime::Handle,
    phantom: PhantomData<Foo>,
}

impl<Foo> Clone for DataWriterAsync<Foo> {
    fn clone(&self) -> Self {
        Self {
            writer_address: self.writer_address.clone(),
            publisher_address: self.publisher_address.clone(),
            participant_address: self.participant_address.clone(),
            runtime_handle: self.runtime_handle.clone(),
            phantom: self.phantom,
        }
    }
}

impl<Foo> DataWriterAsync<Foo> {
    pub(crate) fn new(
        writer_address: ActorAddress<DataWriterActor>,
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        runtime_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            writer_address,
            publisher_address,
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
                    .writer_address
                    .send_mail_and_await_reply(data_writer_actor::get_type_name::new())
                    .await
                && topic
                    .send_mail_and_await_reply(topic_actor::get_name::new())
                    .await
                    == self
                        .writer_address
                        .send_mail_and_await_reply(data_writer_actor::get_topic_name::new())
                        .await
            {
                return topic;
            }
        }
        panic!("Should always exist");
    }
}

impl<Foo> DataWriterAsync<Foo>
where
    Foo: DdsSerialize,
{
    /// Async version of ['register_instance'](crate::publication::data_writer::DataWriter::register_instance).
    #[tracing::instrument(skip(self, instance))]
    pub async fn register_instance(&self, instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        let timestamp = {
            self.participant_address
                .send_mail_and_await_reply(domain_participant_actor::get_current_time::new())
                .await?
        };
        self.register_instance_w_timestamp(instance, timestamp)
            .await
    }

    /// Async version of ['register_instance_w_timestamp'](crate::publication::data_writer::DataWriter::register_instance_w_timestamp).
    #[tracing::instrument(skip(self, _instance))]
    pub async fn register_instance_w_timestamp(
        &self,
        _instance: &Foo,
        _timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        todo!()
    }

    /// Async version of ['unregister_instance'](crate::publication::data_writer::DataWriter::unregister_instance).
    #[tracing::instrument(skip(self, instance))]
    pub async fn unregister_instance(
        &self,
        instance: &Foo,
        handle: Option<InstanceHandle>,
    ) -> DdsResult<()> {
        let timestamp = {
            self.participant_address
                .send_mail_and_await_reply(domain_participant_actor::get_current_time::new())
                .await?
        };
        self.unregister_instance_w_timestamp(instance, handle, timestamp)
            .await
    }

    /// Async version of ['unregister_instance_w_timestamp'](crate::publication::data_writer::DataWriter::unregister_instance_w_timestamp).
    #[tracing::instrument(skip(self, instance))]
    pub async fn unregister_instance_w_timestamp(
        &self,
        instance: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let type_name = self
            .writer_address
            .send_mail_and_await_reply(data_writer_actor::get_type_name::new())
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

            self.writer_address
                .send_mail_and_await_reply(data_writer_actor::unregister_instance_w_timestamp::new(
                    instance_serialized_key,
                    instance_handle,
                    timestamp,
                ))
                .await?
        } else {
            Err(DdsError::IllegalOperation)
        }
    }

    /// Async version of ['get_key_value'](crate::publication::data_writer::DataWriter::get_key_value).
    #[tracing::instrument(skip(self, _key_holder))]
    pub async fn get_key_value(
        &self,
        _key_holder: &mut Foo,
        _handle: InstanceHandle,
    ) -> DdsResult<()> {
        todo!()
    }

    /// Async version of ['lookup_instance'](crate::publication::data_writer::DataWriter::lookup_instance).
    #[tracing::instrument(skip(self, instance))]
    pub async fn lookup_instance(&self, instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        let type_name = self
            .writer_address
            .send_mail_and_await_reply(data_writer_actor::get_type_name::new())
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

        let mut serialized_foo = Vec::new();
        instance.serialize_data(&mut serialized_foo)?;
        let instance_handle = type_support.instance_handle_from_serialized_foo(&serialized_foo)?;

        self.writer_address
            .send_mail_and_await_reply(data_writer_actor::lookup_instance::new(instance_handle))
            .await?
    }

    /// Async version of ['write'](crate::publication::data_writer::DataWriter::write).
    #[tracing::instrument(skip(self, data))]
    pub async fn write(&self, data: &Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        let timestamp = {
            self.participant_address
                .send_mail_and_await_reply(domain_participant_actor::get_current_time::new())
                .await?
        };
        self.write_w_timestamp(data, handle, timestamp).await
    }

    /// Async version of ['write_w_timestamp'](crate::publication::data_writer::DataWriter::write_w_timestamp).
    #[tracing::instrument(skip(self, data))]
    pub async fn write_w_timestamp(
        &self,
        data: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let type_name = self
            .writer_address
            .send_mail_and_await_reply(data_writer_actor::get_type_name::new())
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

        let mut serialized_data = Vec::new();
        data.serialize_data(&mut serialized_data)?;
        let key = type_support.instance_handle_from_serialized_foo(&serialized_data)?;

        self.writer_address
            .send_mail_and_await_reply(data_writer_actor::write_w_timestamp::new(
                serialized_data,
                key,
                handle,
                timestamp,
            ))
            .await??;

        self.participant_address
            .send_mail(domain_participant_actor::send_message::new())
            .await?;

        Ok(())
    }

    /// Async version of ['dispose'](crate::publication::data_writer::DataWriter::dispose).
    #[tracing::instrument(skip(self, data))]
    pub async fn dispose(&self, data: &Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        let timestamp = {
            self.participant_address
                .send_mail_and_await_reply(domain_participant_actor::get_current_time::new())
                .await?
        };
        self.dispose_w_timestamp(data, handle, timestamp).await
    }

    /// Async version of ['dispose_w_timestamp'](crate::publication::data_writer::DataWriter::dispose_w_timestamp).
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

        let type_name = self
            .writer_address
            .send_mail_and_await_reply(data_writer_actor::get_type_name::new())
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

        let mut serialized_foo = Vec::new();
        data.serialize_data(&mut serialized_foo)?;
        let key = type_support.get_serialized_key_from_serialized_foo(&serialized_foo)?;

        self.writer_address
            .send_mail_and_await_reply(data_writer_actor::dispose_w_timestamp::new(
                key,
                instance_handle,
                timestamp,
            ))
            .await?
    }
}

impl<Foo> DataWriterAsync<Foo> {
    /// Async version of ['wait_for_acknowledgments'](crate::publication::data_writer::DataWriter::wait_for_acknowledgments).
    #[tracing::instrument(skip(self))]
    pub async fn wait_for_acknowledgments(&self, max_wait: Duration) -> DdsResult<()> {
        let start_time = Instant::now();
        while start_time.elapsed() < std::time::Duration::from(max_wait) {
            if self
                .writer_address
                .send_mail_and_await_reply(data_writer_actor::are_all_changes_acknowledge::new())
                .await?
            {
                return Ok(());
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
        }

        Err(DdsError::Timeout)
    }

    /// Async version of ['get_liveliness_lost_status'](crate::publication::data_writer::DataWriter::get_liveliness_lost_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_liveliness_lost_status(&self) -> DdsResult<LivelinessLostStatus> {
        todo!()
    }

    /// Async version of ['get_offered_deadline_missed_status'](crate::publication::data_writer::DataWriter::get_offered_deadline_missed_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_offered_deadline_missed_status(
        &self,
    ) -> DdsResult<OfferedDeadlineMissedStatus> {
        todo!()
    }

    /// Async version of ['get_offered_incompatible_qos_status'](crate::publication::data_writer::DataWriter::get_offered_incompatible_qos_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_offered_incompatible_qos_status(
        &self,
    ) -> DdsResult<OfferedIncompatibleQosStatus> {
        todo!()
    }

    /// Async version of ['get_publication_matched_status'](crate::publication::data_writer::DataWriter::get_publication_matched_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_publication_matched_status(&self) -> DdsResult<PublicationMatchedStatus> {
        self.writer_address
            .send_mail_and_await_reply(data_writer_actor::get_publication_matched_status::new())
            .await
    }

    /// Async version of ['get_topic'](crate::publication::data_writer::DataWriter::get_topic).
    #[tracing::instrument(skip(self))]
    pub async fn get_topic(&self) -> DdsResult<TopicAsync> {
        Ok(TopicAsync::new(
            self.topic_address().await,
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        ))
    }

    /// Async version of ['get_publisher'](crate::publication::data_writer::DataWriter::get_publisher).
    #[tracing::instrument(skip(self))]
    pub async fn get_publisher(&self) -> DdsResult<PublisherAsync> {
        Ok(PublisherAsync::new(
            self.publisher_address.clone(),
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        ))
    }

    /// Async version of ['assert_liveliness'](crate::publication::data_writer::DataWriter::assert_liveliness).
    #[tracing::instrument(skip(self))]
    pub async fn assert_liveliness(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of ['get_matched_subscription_data'](crate::publication::data_writer::DataWriter::get_matched_subscription_data).
    #[tracing::instrument(skip(self))]
    pub async fn get_matched_subscription_data(
        &self,
        subscription_handle: InstanceHandle,
    ) -> DdsResult<SubscriptionBuiltinTopicData> {
        self.writer_address
            .send_mail_and_await_reply(data_writer_actor::get_matched_subscription_data::new(
                subscription_handle,
            ))
            .await?
            .ok_or(DdsError::BadParameter)
    }

    /// Async version of ['get_matched_subscriptions'](crate::publication::data_writer::DataWriter::get_matched_subscriptions).
    #[tracing::instrument(skip(self))]
    pub async fn get_matched_subscriptions(&self) -> DdsResult<Vec<InstanceHandle>> {
        self.writer_address
            .send_mail_and_await_reply(data_writer_actor::get_matched_subscriptions::new())
            .await
    }
}

impl<Foo> DataWriterAsync<Foo> {
    /// Async version of ['set_qos'](crate::publication::data_writer::DataWriter::set_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        let q = match qos {
            QosKind::Default => {
                self.publisher_address
                    .send_mail_and_await_reply(publisher_actor::get_default_datawriter_qos::new())
                    .await?
            }
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };
        self.writer_address
            .send_mail_and_await_reply(data_writer_actor::set_qos::new(q))
            .await?;

        if self
            .writer_address
            .send_mail_and_await_reply(data_writer_actor::is_enabled::new())
            .await?
        {
            let type_name = self
                .writer_address
                .send_mail_and_await_reply(data_writer_actor::get_type_name::new())
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
            let discovered_writer_data = self
                .writer_address
                .send_mail_and_await_reply(data_writer_actor::as_discovered_writer_data::new(
                    TopicQos::default(),
                    self.publisher_address
                        .send_mail_and_await_reply(publisher_actor::get_qos::new())
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
                .await?;
            self.participant_address
                .send_mail(
                    domain_participant_actor::announce_created_or_modified_data_writer::new(
                        discovered_writer_data,
                    ),
                )
                .await?;
        }

        Ok(())
    }

    /// Async version of ['get_qos'](crate::publication::data_writer::DataWriter::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<DataWriterQos> {
        self.writer_address
            .send_mail_and_await_reply(data_writer_actor::get_qos::new())
            .await
    }

    /// Async version of ['get_statuscondition'](crate::publication::data_writer::DataWriter::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub async fn get_statuscondition(&self) -> DdsResult<StatusConditionAsync> {
        self.writer_address
            .send_mail_and_await_reply(data_writer_actor::get_statuscondition::new())
            .await
            .map(|c| StatusConditionAsync::new(c, self.runtime_handle.clone()))
    }

    /// Async version of ['get_status_changes'](crate::publication::data_writer::DataWriter::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of ['enable'](crate::publication::data_writer::DataWriter::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        if !self
            .writer_address
            .send_mail_and_await_reply(data_writer_actor::is_enabled::new())
            .await?
        {
            let type_name = self
                .writer_address
                .send_mail_and_await_reply(data_writer_actor::get_type_name::new())
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
            self.writer_address
                .send_mail_and_await_reply(data_writer_actor::enable::new())
                .await?;
            let discovered_writer_data = self
                .writer_address
                .send_mail_and_await_reply(data_writer_actor::as_discovered_writer_data::new(
                    TopicQos::default(),
                    self.publisher_address
                        .send_mail_and_await_reply(publisher_actor::get_qos::new())
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
                .await?;
            self.participant_address
                .send_mail(
                    domain_participant_actor::announce_created_or_modified_data_writer::new(
                        discovered_writer_data,
                    ),
                )
                .await?;
        }
        Ok(())
    }

    /// Async version of ['get_instance_handle'](crate::publication::data_writer::DataWriter::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.writer_address
            .send_mail_and_await_reply(data_writer_actor::get_instance_handle::new())
            .await
    }

    /// Async version of ['set_listener'](crate::publication::data_writer::DataWriter::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: impl DataWriterListener<Foo = Foo> + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.writer_address
            .send_mail_and_await_reply(data_writer_actor::set_listener::new(
                Box::new(a_listener),
                mask.to_vec(),
                self.runtime_handle.clone(),
            ))
            .await
    }
}
