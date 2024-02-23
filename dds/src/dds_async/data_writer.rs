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
        condition::StatusCondition,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, QosKind, TopicQos},
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, StatusKind,
        },
        time::{Duration, Time},
    },
    publication::{data_writer_listener::DataWriterListener, publisher::Publisher},
    topic_definition::{topic::Topic, type_support::DdsSerialize},
};

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

    #[tracing::instrument(skip(self, _instance))]
    pub async fn register_instance_w_timestamp(
        &self,
        _instance: &Foo,
        _timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        todo!()
    }

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

    #[tracing::instrument(skip(self, _key_holder))]
    pub async fn get_key_value(
        &self,
        _key_holder: &mut Foo,
        _handle: InstanceHandle,
    ) -> DdsResult<()> {
        todo!()
    }

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

    #[tracing::instrument(skip(self, data))]
    pub async fn write(&self, data: &Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        let timestamp = {
            self.participant_address
                .send_mail_and_await_reply(domain_participant_actor::get_current_time::new())
                .await?
        };
        self.write_w_timestamp(data, handle, timestamp).await
    }

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

    #[tracing::instrument(skip(self, data))]
    pub async fn dispose(&self, data: &Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        let timestamp = {
            self.participant_address
                .send_mail_and_await_reply(domain_participant_actor::get_current_time::new())
                .await?
        };
        self.dispose_w_timestamp(data, handle, timestamp).await
    }

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
            std::thread::sleep(std::time::Duration::from_millis(25));
        }

        Err(DdsError::Timeout)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_liveliness_lost_status(&self) -> DdsResult<LivelinessLostStatus> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_offered_deadline_missed_status(
        &self,
    ) -> DdsResult<OfferedDeadlineMissedStatus> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_offered_incompatible_qos_status(
        &self,
    ) -> DdsResult<OfferedIncompatibleQosStatus> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_publication_matched_status(&self) -> DdsResult<PublicationMatchedStatus> {
        self.writer_address
            .send_mail_and_await_reply(data_writer_actor::get_publication_matched_status::new())
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_topic(&self) -> DdsResult<Topic> {
        Ok(Topic::new(
            self.topic_address().await,
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        ))
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_publisher(&self) -> DdsResult<Publisher> {
        Ok(Publisher::new(
            self.publisher_address.clone(),
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        ))
    }

    #[tracing::instrument(skip(self))]
    pub async fn assert_liveliness(&self) -> DdsResult<()> {
        todo!()
    }

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

    #[tracing::instrument(skip(self))]
    pub async fn get_matched_subscriptions(&self) -> DdsResult<Vec<InstanceHandle>> {
        self.writer_address
            .send_mail_and_await_reply(data_writer_actor::get_matched_subscriptions::new())
            .await
    }
}

impl<Foo> DataWriterAsync<Foo> {
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

    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<DataWriterQos> {
        self.writer_address
            .send_mail_and_await_reply(data_writer_actor::get_qos::new())
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        self.writer_address
            .send_mail_and_await_reply(data_writer_actor::get_statuscondition::new())
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

    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.writer_address
            .send_mail_and_await_reply(data_writer_actor::get_instance_handle::new())
            .await
    }

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
