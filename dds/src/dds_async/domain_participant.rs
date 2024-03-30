use std::time::Instant;

use crate::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    domain::domain_participant_factory::DomainId,
    implementation::{
        actors::{
            domain_participant_actor::{DomainParticipantActor, FooTypeSupport},
            status_condition_actor::StatusConditionActor,
            subscriber_actor::SubscriberActor,
        },
        utils::{actor::ActorAddress, instance_handle_from_key::get_instance_handle_from_key},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantQos, PublisherQos, QosKind, SubscriberQos, TopicQos},
        status::{StatusKind, NO_STATUS},
        time::{Duration, Time},
    },
    topic_definition::type_support::{
        DdsHasKey, DdsKey, DdsSerialize, DdsTypeXml, DynamicTypeInterface,
    },
};

use super::{
    condition::StatusConditionAsync, domain_participant_listener::DomainParticipantListenerAsync,
    publisher::PublisherAsync, publisher_listener::PublisherListenerAsync,
    subscriber::SubscriberAsync, subscriber_listener::SubscriberListenerAsync, topic::TopicAsync,
    topic_listener::TopicListenerAsync,
};

/// Async version of [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant).
#[derive(Clone)]
pub struct DomainParticipantAsync {
    participant_address: ActorAddress<DomainParticipantActor>,
    status_condition_address: ActorAddress<StatusConditionActor>,
    builtin_subscriber_address: ActorAddress<SubscriberActor>,
    builtin_subscriber_status_condition_address: ActorAddress<StatusConditionActor>,
    domain_id: DomainId,
    runtime_handle: tokio::runtime::Handle,
}

impl DomainParticipantAsync {
    pub(crate) fn new(
        participant_address: ActorAddress<DomainParticipantActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        builtin_subscriber_address: ActorAddress<SubscriberActor>,
        builtin_subscriber_status_condition_address: ActorAddress<StatusConditionActor>,
        domain_id: DomainId,
        runtime_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            participant_address,
            status_condition_address,
            builtin_subscriber_address,
            builtin_subscriber_status_condition_address,
            domain_id,
            runtime_handle,
        }
    }

    pub(crate) fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        &self.participant_address
    }

    pub(crate) fn runtime_handle(&self) -> &tokio::runtime::Handle {
        &self.runtime_handle
    }
}

impl DomainParticipantAsync {
    /// Async version of [`create_publisher`](crate::domain::domain_participant::DomainParticipant::create_publisher).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn create_publisher(
        &self,
        qos: QosKind<PublisherQos>,
        a_listener: Option<Box<dyn PublisherListenerAsync + Send>>,
        mask: &[StatusKind],
    ) -> DdsResult<PublisherAsync> {
        let (publisher_address, status_condition) = self
            .participant_address
            .create_user_defined_publisher(
                qos,
                a_listener,
                mask.to_vec(),
                self.runtime_handle.clone(),
            )
            .await?;
        let publisher = PublisherAsync::new(publisher_address, status_condition, self.clone());
        if self.participant_address.is_enabled().await?
            && self
                .participant_address
                .get_qos()
                .await?
                .entity_factory
                .autoenable_created_entities
        {
            publisher.enable().await?;
        }

        Ok(publisher)
    }

    /// Async version of [`delete_publisher`](crate::domain::domain_participant::DomainParticipant::delete_publisher).
    #[tracing::instrument(skip(self, a_publisher))]
    pub async fn delete_publisher(&self, a_publisher: &PublisherAsync) -> DdsResult<()> {
        self.participant_address
            .delete_user_defined_publisher(a_publisher.get_instance_handle().await?)
            .await?
    }

    /// Async version of [`create_subscriber`](crate::domain::domain_participant::DomainParticipant::create_subscriber).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn create_subscriber(
        &self,
        qos: QosKind<SubscriberQos>,
        a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
        mask: &[StatusKind],
    ) -> DdsResult<SubscriberAsync> {
        let (subscriber_address, subscriber_status_condition) = self
            .participant_address
            .create_user_defined_subscriber(
                qos,
                a_listener,
                mask.to_vec(),
                self.runtime_handle.clone(),
            )
            .await?;

        let subscriber = SubscriberAsync::new(
            subscriber_address,
            subscriber_status_condition,
            self.clone(),
        );

        if self.participant_address.is_enabled().await?
            && self
                .participant_address
                .get_qos()
                .await?
                .entity_factory
                .autoenable_created_entities
        {
            subscriber.enable().await?;
        }

        Ok(subscriber)
    }

    /// Async version of [`delete_subscriber`](crate::domain::domain_participant::DomainParticipant::delete_subscriber).
    #[tracing::instrument(skip(self, a_subscriber))]
    pub async fn delete_subscriber(&self, a_subscriber: &SubscriberAsync) -> DdsResult<()> {
        self.participant_address
            .delete_user_defined_subscriber(a_subscriber.get_instance_handle().await?)
            .await?
    }

    /// Async version of [`create_topic`](crate::domain::domain_participant::DomainParticipant::create_topic).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn create_topic<Foo>(
        &self,
        topic_name: &str,
        type_name: &str,
        qos: QosKind<TopicQos>,
        a_listener: Option<Box<dyn TopicListenerAsync + Send>>,
        mask: &[StatusKind],
    ) -> DdsResult<TopicAsync>
    where
        Foo: DdsKey + DdsHasKey + DdsTypeXml,
    {
        let type_support = Box::new(FooTypeSupport::new::<Foo>());

        self.create_dynamic_topic(topic_name, type_name, qos, a_listener, mask, type_support)
            .await
    }

    #[doc(hidden)]
    #[tracing::instrument(skip(self, a_listener, dynamic_type_representation))]
    pub async fn create_dynamic_topic(
        &self,
        topic_name: &str,
        type_name: &str,
        qos: QosKind<TopicQos>,
        a_listener: Option<Box<dyn TopicListenerAsync + Send>>,
        mask: &[StatusKind],
        dynamic_type_representation: Box<dyn DynamicTypeInterface + Send + Sync>,
    ) -> DdsResult<TopicAsync> {
        let (topic_address, topic_status_condition) = self
            .participant_address
            .create_user_defined_topic(
                topic_name.to_string(),
                type_name.to_string(),
                qos,
                a_listener,
                mask.to_vec(),
                dynamic_type_representation,
                self.runtime_handle.clone(),
            )
            .await?;
        let topic = TopicAsync::new(
            topic_address,
            topic_status_condition,
            type_name.to_string(),
            topic_name.to_string(),
            self.clone(),
        );
        if self.participant_address.is_enabled().await?
            && self
                .participant_address
                .get_qos()
                .await?
                .entity_factory
                .autoenable_created_entities
        {
            topic.enable().await?;
        }

        Ok(topic)
    }

    /// Async version of [`delete_topic`](crate::domain::domain_participant::DomainParticipant::delete_topic).
    #[tracing::instrument(skip(self, a_topic))]
    pub async fn delete_topic(&self, a_topic: &TopicAsync) -> DdsResult<()> {
        self.participant_address
            .delete_user_defined_topic(a_topic.get_instance_handle().await?)
            .await?
    }

    /// Async version of [`find_topic`](crate::domain::domain_participant::DomainParticipant::find_topic).
    #[tracing::instrument(skip(self))]
    pub async fn find_topic<Foo>(
        &self,
        topic_name: &str,
        timeout: Duration,
    ) -> DdsResult<TopicAsync>
    where
        Foo: DdsKey + DdsHasKey + DdsTypeXml,
    {
        let start_time = Instant::now();

        while start_time.elapsed() < std::time::Duration::from(timeout) {
            for topic in self
                .participant_address
                .get_user_defined_topic_list()
                .await?
            {
                if topic.get_name().await? == topic_name {
                    let type_name = topic.get_type_name().await?;
                    let topic_status_condition = topic.get_statuscondition().await?;
                    return Ok(TopicAsync::new(
                        topic,
                        topic_status_condition,
                        type_name,
                        topic_name.to_string(),
                        self.clone(),
                    ));
                }
            }

            for discovered_topic_handle in self.participant_address.discovered_topic_list().await? {
                if let Ok(discovered_topic_data) = self
                    .participant_address
                    .discovered_topic_data(discovered_topic_handle)
                    .await?
                {
                    if discovered_topic_data.name() == topic_name {
                        let qos = TopicQos {
                            topic_data: discovered_topic_data.topic_data().clone(),
                            durability: discovered_topic_data.durability().clone(),
                            deadline: discovered_topic_data.deadline().clone(),
                            latency_budget: discovered_topic_data.latency_budget().clone(),
                            liveliness: discovered_topic_data.liveliness().clone(),
                            reliability: discovered_topic_data.reliability().clone(),
                            destination_order: discovered_topic_data.destination_order().clone(),
                            history: discovered_topic_data.history().clone(),
                            resource_limits: discovered_topic_data.resource_limits().clone(),
                            transport_priority: discovered_topic_data.transport_priority().clone(),
                            lifespan: discovered_topic_data.lifespan().clone(),
                            ownership: discovered_topic_data.ownership().clone(),
                        };
                        let topic = self
                            .create_topic::<Foo>(
                                topic_name,
                                discovered_topic_data.get_type_name(),
                                QosKind::Specific(qos),
                                None,
                                NO_STATUS,
                            )
                            .await?;
                        return Ok(topic);
                    }
                }
            }
        }

        Err(DdsError::Timeout)
    }

    /// Async version of [`lookup_topicdescription`](crate::domain::domain_participant::DomainParticipant::lookup_topicdescription).
    #[tracing::instrument(skip(self))]
    pub async fn lookup_topicdescription(
        &self,
        _topic_name: &str,
    ) -> DdsResult<Option<TopicAsync>> {
        todo!()
        // self.call_participant_method(|dp| {
        //     Ok(
        //         crate::implementation::behavior::domain_participant::lookup_topicdescription(
        //             dp,
        //             topic_name,
        //             Foo,
        //         )?
        //         .map(|x| Topic::new(TopicNodeKind::UserDefined(x))),
        //     )
        // })
    }

    /// Async version of [`get_builtin_subscriber`](crate::domain::domain_participant::DomainParticipant::get_builtin_subscriber).
    #[tracing::instrument(skip(self))]
    pub fn get_builtin_subscriber(&self) -> SubscriberAsync {
        SubscriberAsync::new(
            self.builtin_subscriber_address.clone(),
            self.builtin_subscriber_status_condition_address.clone(),
            self.clone(),
        )
    }

    /// Async version of [`ignore_participant`](crate::domain::domain_participant::DomainParticipant::ignore_participant).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_participant(&self, handle: InstanceHandle) -> DdsResult<()> {
        self.participant_address.ignore_participant(handle).await?
    }

    /// Async version of [`ignore_topic`](crate::domain::domain_participant::DomainParticipant::ignore_topic).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_topic(&self, handle: InstanceHandle) -> DdsResult<()> {
        self.participant_address.ignore_topic(handle).await?
    }

    /// Async version of [`ignore_publication`](crate::domain::domain_participant::DomainParticipant::ignore_publication).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_publication(&self, handle: InstanceHandle) -> DdsResult<()> {
        self.participant_address.ignore_publication(handle).await?
    }

    /// Async version of [`ignore_subscription`](crate::domain::domain_participant::DomainParticipant::ignore_subscription).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_subscription(&self, handle: InstanceHandle) -> DdsResult<()> {
        self.participant_address.ignore_subscription(handle).await?
    }

    /// Async version of [`get_domain_id`](crate::domain::domain_participant::DomainParticipant::get_domain_id).
    #[tracing::instrument(skip(self))]
    pub fn get_domain_id(&self) -> DomainId {
        self.domain_id
    }

    /// Async version of [`delete_contained_entities`](crate::domain::domain_participant::DomainParticipant::delete_contained_entities).
    #[tracing::instrument(skip(self))]
    pub async fn delete_contained_entities(&self) -> DdsResult<()> {
        for publisher in self
            .participant_address
            .get_user_defined_publisher_list()
            .await?
        {
            for data_writer in publisher.data_writer_list().await? {
                publisher
                    .datawriter_delete(data_writer.get_instance_handle().await?)
                    .await?;
            }
            self.participant_address
                .delete_user_defined_publisher(publisher.get_instance_handle().await?)
                .await??;
        }
        for subscriber in self
            .participant_address
            .get_user_defined_subscriber_list()
            .await?
        {
            for data_reader in subscriber.data_reader_list().await? {
                subscriber
                    .data_reader_delete(data_reader.get_instance_handle().await?)
                    .await?;
            }
            self.participant_address
                .delete_user_defined_subscriber(subscriber.get_instance_handle().await?)
                .await??;
        }
        for topic in self
            .participant_address
            .get_user_defined_topic_list()
            .await?
        {
            self.participant_address
                .delete_user_defined_topic(topic.get_instance_handle().await?)
                .await??;
        }
        Ok(())
    }

    /// Async version of [`assert_liveliness`](crate::domain::domain_participant::DomainParticipant::assert_liveliness).
    #[tracing::instrument(skip(self))]
    pub async fn assert_liveliness(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`set_default_publisher_qos`](crate::domain::domain_participant::DomainParticipant::set_default_publisher_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_publisher_qos(&self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        self.participant_address
            .set_default_publisher_qos(qos)
            .await?
    }

    /// Async version of [`get_default_publisher_qos`](crate::domain::domain_participant::DomainParticipant::get_default_publisher_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_publisher_qos(&self) -> DdsResult<PublisherQos> {
        self.participant_address.get_default_publisher_qos().await
    }

    /// Async version of [`set_default_subscriber_qos`](crate::domain::domain_participant::DomainParticipant::set_default_subscriber_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_subscriber_qos(&self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        self.participant_address
            .set_default_subscriber_qos(qos)
            .await?
    }

    /// Async version of [`get_default_subscriber_qos`](crate::domain::domain_participant::DomainParticipant::get_default_subscriber_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_subscriber_qos(&self) -> DdsResult<SubscriberQos> {
        self.participant_address.get_default_subscriber_qos().await
    }

    /// Async version of [`set_default_topic_qos`](crate::domain::domain_participant::DomainParticipant::set_default_topic_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_topic_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        self.participant_address.set_default_topic_qos(qos).await?
    }

    /// Async version of [`get_default_topic_qos`](crate::domain::domain_participant::DomainParticipant::get_default_topic_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_topic_qos(&self) -> DdsResult<TopicQos> {
        self.participant_address.get_default_topic_qos().await
    }

    /// Async version of [`get_discovered_participants`](crate::domain::domain_participant::DomainParticipant::get_discovered_participants).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_participants(&self) -> DdsResult<Vec<InstanceHandle>> {
        self.participant_address.get_discovered_participants().await
    }

    /// Async version of [`get_discovered_participant_data`](crate::domain::domain_participant::DomainParticipant::get_discovered_participant_data).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_participant_data(
        &self,
        participant_handle: InstanceHandle,
    ) -> DdsResult<ParticipantBuiltinTopicData> {
        self.participant_address
            .get_discovered_participant_data(participant_handle)
            .await?
    }

    /// Async version of [`get_discovered_topics`](crate::domain::domain_participant::DomainParticipant::get_discovered_topics).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_topics(&self) -> DdsResult<Vec<InstanceHandle>> {
        self.participant_address.discovered_topic_list().await
    }

    /// Async version of [`get_discovered_topic_data`](crate::domain::domain_participant::DomainParticipant::get_discovered_topic_data).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_topic_data(
        &self,
        topic_handle: InstanceHandle,
    ) -> DdsResult<TopicBuiltinTopicData> {
        self.participant_address
            .discovered_topic_data(topic_handle)
            .await?
    }

    /// Async version of [`contains_entity`](crate::domain::domain_participant::DomainParticipant::contains_entity).
    #[tracing::instrument(skip(self))]
    pub async fn contains_entity(&self, _a_handle: InstanceHandle) -> DdsResult<bool> {
        todo!()
        // self.call_participant_method(|dp| {
        //     crate::implementation::behavior::domain_participant::contains_entity(dp, a_handle)
        // })
    }

    /// Async version of [`get_current_time`](crate::domain::domain_participant::DomainParticipant::get_current_time).
    #[tracing::instrument(skip(self))]
    pub async fn get_current_time(&self) -> DdsResult<Time> {
        self.participant_address.get_current_time().await
    }
}

impl DomainParticipantAsync {
    /// Async version of [`set_qos`](crate::domain::domain_participant::DomainParticipant::set_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.participant_address.set_qos(qos).await?
    }

    /// Async version of [`get_qos`](crate::domain::domain_participant::DomainParticipant::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<DomainParticipantQos> {
        self.participant_address.get_qos().await
    }

    /// Async version of [`set_listener`](crate::domain::domain_participant::DomainParticipant::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.participant_address
            .set_listener(a_listener, mask.to_vec(), self.runtime_handle.clone())
            .await
    }

    /// Async version of [`get_statuscondition`](crate::domain::domain_participant::DomainParticipant::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(
            self.status_condition_address.clone(),
            self.runtime_handle.clone(),
        )
    }

    /// Async version of [`get_status_changes`](crate::domain::domain_participant::DomainParticipant::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::domain::domain_participant::DomainParticipant::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        if !self.participant_address.is_enabled().await? {
            self.participant_address
                .get_builtin_publisher()
                .await?
                .enable()
                .await?;
            self.participant_address
                .get_built_in_subscriber()
                .await?
                .enable()
                .await?;

            for builtin_reader in self
                .participant_address
                .get_built_in_subscriber()
                .await?
                .data_reader_list()
                .await?
            {
                builtin_reader.enable().await?;
            }

            for builtin_writer in self
                .participant_address
                .get_builtin_publisher()
                .await?
                .data_writer_list()
                .await?
            {
                builtin_writer.enable().await?;
            }

            self.participant_address.enable().await?;

            let domain_participant_address = self.participant_address.clone();

            // Spawn the task that regularly announces the domain participant
            self.runtime_handle.spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
                loop {
                    let r: DdsResult<()> = async {
                        let builtin_publisher =
                            domain_participant_address.get_builtin_publisher().await?;
                        let data_writer_list = builtin_publisher.data_writer_list().await?;
                        for data_writer in data_writer_list {
                            if data_writer.get_type_name().await
                                == Ok("SpdpDiscoveredParticipantData".to_string())
                            {
                                let spdp_discovered_participant_data = domain_participant_address
                                    .as_spdp_discovered_participant_data()
                                    .await?;
                                let mut serialized_data = Vec::new();
                                spdp_discovered_participant_data
                                    .serialize_data(&mut serialized_data)?;

                                let timestamp =
                                    domain_participant_address.get_current_time().await?;

                                data_writer
                                    .write_w_timestamp(
                                        serialized_data,
                                        get_instance_handle_from_key(
                                            &spdp_discovered_participant_data.get_key()?,
                                        )
                                        .unwrap(),
                                        None,
                                        timestamp,
                                    )
                                    .await??;

                                domain_participant_address.send_message().await?;
                            }
                        }

                        Ok(())
                    }
                    .await;

                    if r.is_err() {
                        break;
                    }

                    interval.tick().await;
                }
            });
        }

        Ok(())
    }

    /// Async version of [`get_instance_handle`](crate::domain::domain_participant::DomainParticipant::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.participant_address.get_instance_handle().await
    }
}
