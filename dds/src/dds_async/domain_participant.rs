use std::time::Instant;

use crate::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    domain::{
        domain_participant_factory::{DomainId, DomainParticipantFactory},
        domain_participant_listener::DomainParticipantListener,
    },
    implementation::{
        actors::{
            data_reader_actor, data_writer_actor,
            domain_participant_actor::{self, DomainParticipantActor, FooTypeSupport},
            publisher_actor, subscriber_actor, topic_actor,
        },
        utils::{actor::ActorAddress, instance_handle_from_key::get_instance_handle_from_key},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        listeners::NoOpListener,
        qos::{DomainParticipantQos, PublisherQos, QosKind, SubscriberQos, TopicQos},
        status::{StatusKind, NO_STATUS},
        time::{Duration, Time},
    },
    publication::publisher_listener::PublisherListener,
    subscription::subscriber_listener::SubscriberListener,
    topic_definition::{
        topic_listener::TopicListener,
        type_support::{DdsHasKey, DdsKey, DdsSerialize, DdsTypeXml, DynamicTypeInterface},
    },
};

use super::{
    condition::StatusConditionAsync, publisher::PublisherAsync, subscriber::SubscriberAsync,
    topic::TopicAsync,
};

/// Async version of [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant).
pub struct DomainParticipantAsync {
    participant_address: ActorAddress<DomainParticipantActor>,
    runtime_handle: tokio::runtime::Handle,
}

impl DomainParticipantAsync {
    pub(crate) fn new(
        participant_address: ActorAddress<DomainParticipantActor>,
        runtime_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            participant_address,
            runtime_handle,
        }
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
        a_listener: impl PublisherListener + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<PublisherAsync> {
        let publisher_address = self
            .participant_address
            .send_mail_and_await_reply(domain_participant_actor::create_publisher::new(
                qos,
                Box::new(a_listener),
                mask.to_vec(),
                self.runtime_handle.clone(),
            ))
            .await?;

        let publisher = PublisherAsync::new(
            publisher_address,
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        );
        if self
            .participant_address
            .send_mail_and_await_reply(domain_participant_actor::is_enabled::new())
            .await?
            && self
                .participant_address
                .send_mail_and_await_reply(domain_participant_actor::get_qos::new())
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
        if self
            .participant_address
            .send_mail_and_await_reply(domain_participant_actor::get_instance_handle::new())
            .await?
            != a_publisher
                .get_participant()
                .await?
                .get_instance_handle()
                .await?
        {
            return Err(DdsError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant".to_string(),
            ));
        }

        self.participant_address
            .send_mail_and_await_reply(
                domain_participant_actor::delete_user_defined_publisher::new(
                    a_publisher.get_instance_handle().await?,
                ),
            )
            .await?
    }

    /// Async version of [`create_subscriber`](crate::domain::domain_participant::DomainParticipant::create_subscriber).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn create_subscriber(
        &self,
        qos: QosKind<SubscriberQos>,
        a_listener: impl SubscriberListener + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<SubscriberAsync> {
        let subscriber_address = self
            .participant_address
            .send_mail_and_await_reply(domain_participant_actor::create_subscriber::new(
                qos,
                Box::new(a_listener),
                mask.to_vec(),
                self.runtime_handle.clone(),
            ))
            .await?;

        let subscriber = SubscriberAsync::new(
            subscriber_address,
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        );

        if self
            .participant_address
            .send_mail_and_await_reply(domain_participant_actor::is_enabled::new())
            .await?
            && self
                .participant_address
                .send_mail_and_await_reply(domain_participant_actor::get_qos::new())
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
        if self.get_instance_handle().await?
            != a_subscriber
                .get_participant()
                .await?
                .get_instance_handle()
                .await?
        {
            return Err(DdsError::PreconditionNotMet(
                "Subscriber can only be deleted from its parent participant".to_string(),
            ));
        }

        self.participant_address
            .send_mail_and_await_reply(
                domain_participant_actor::delete_user_defined_subscriber::new(
                    a_subscriber.get_instance_handle().await?,
                ),
            )
            .await?
    }

    /// Async version of [`create_topic`](crate::domain::domain_participant::DomainParticipant::create_topic).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn create_topic<Foo>(
        &self,
        topic_name: &str,
        type_name: &str,
        qos: QosKind<TopicQos>,
        a_listener: impl TopicListener + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<TopicAsync>
    where
        Foo: DdsKey + DdsHasKey + DdsTypeXml,
    {
        let type_support = FooTypeSupport::new::<Foo>();

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
        a_listener: impl TopicListener + Send + 'static,
        mask: &[StatusKind],
        dynamic_type_representation: impl DynamicTypeInterface + Send + Sync + 'static,
    ) -> DdsResult<TopicAsync> {
        self.participant_address
            .send_mail_and_await_reply(domain_participant_actor::register_type::new(
                type_name.to_string(),
                Box::new(dynamic_type_representation),
            ))
            .await?;

        let topic_address = self
            .participant_address
            .send_mail_and_await_reply(domain_participant_actor::create_topic::new(
                topic_name.to_string(),
                type_name.to_string(),
                qos,
                Box::new(a_listener),
                mask.to_vec(),
                self.runtime_handle.clone(),
            ))
            .await?;

        let topic = TopicAsync::new(
            topic_address,
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        );
        if self
            .participant_address
            .send_mail_and_await_reply(domain_participant_actor::is_enabled::new())
            .await?
            && self
                .participant_address
                .send_mail_and_await_reply(domain_participant_actor::get_qos::new())
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
        if self.get_instance_handle().await?
            != a_topic
                .get_participant()
                .await?
                .get_instance_handle()
                .await?
        {
            return Err(DdsError::PreconditionNotMet(
                "Topic can only be deleted from its parent participant".to_string(),
            ));
        }

        for publisher in self
            .participant_address
            .send_mail_and_await_reply(
                domain_participant_actor::get_user_defined_publisher_list::new(),
            )
            .await?
        {
            let data_writer_list = publisher
                .send_mail_and_await_reply(publisher_actor::data_writer_list::new())
                .await?;
            for data_writer in data_writer_list {
                if data_writer
                    .send_mail_and_await_reply(data_writer_actor::get_type_name::new())
                    .await?
                    == a_topic.get_type_name().await?
                    && data_writer
                        .send_mail_and_await_reply(data_writer_actor::get_topic_name::new())
                        .await?
                        == a_topic.get_name().await?
                {
                    return Err(DdsError::PreconditionNotMet(
                        "Topic still attached to some data writer".to_string(),
                    ));
                }
            }
        }

        for subscriber in self
            .participant_address
            .send_mail_and_await_reply(
                domain_participant_actor::get_user_defined_subscriber_list::new(),
            )
            .await?
        {
            let data_reader_list = subscriber
                .send_mail_and_await_reply(subscriber_actor::data_reader_list::new())
                .await?;
            for data_reader in data_reader_list {
                if data_reader
                    .send_mail_and_await_reply(data_reader_actor::get_type_name::new())
                    .await?
                    == a_topic.get_type_name().await?
                    && data_reader
                        .send_mail_and_await_reply(data_reader_actor::get_topic_name::new())
                        .await?
                        == a_topic.get_name().await?
                {
                    return Err(DdsError::PreconditionNotMet(
                        "Topic still attached to some data reader".to_string(),
                    ));
                }
            }
        }

        self.participant_address
            .send_mail_and_await_reply(domain_participant_actor::delete_topic::new(
                a_topic.get_instance_handle().await?,
            ))
            .await
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
                .send_mail_and_await_reply(
                    domain_participant_actor::get_user_defined_topic_list::new(),
                )
                .await?
            {
                if topic
                    .send_mail_and_await_reply(topic_actor::get_name::new())
                    .await?
                    == topic_name
                {
                    return Ok(TopicAsync::new(
                        topic,
                        self.participant_address.clone(),
                        self.runtime_handle.clone(),
                    ));
                }
            }

            for discovered_topic_handle in self
                .participant_address
                .send_mail_and_await_reply(domain_participant_actor::discovered_topic_list::new())
                .await?
            {
                if let Ok(discovered_topic_data) = self
                    .participant_address
                    .send_mail_and_await_reply(
                        domain_participant_actor::discovered_topic_data::new(
                            discovered_topic_handle,
                        ),
                    )
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
                                NoOpListener::new(),
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
    pub async fn get_builtin_subscriber(&self) -> DdsResult<SubscriberAsync> {
        Ok(SubscriberAsync::new(
            self.participant_address
                .send_mail_and_await_reply(domain_participant_actor::get_built_in_subscriber::new())
                .await?,
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        ))
    }

    /// Async version of [`ignore_participant`](crate::domain::domain_participant::DomainParticipant::ignore_participant).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_participant(&self, handle: InstanceHandle) -> DdsResult<()> {
        if self
            .participant_address
            .send_mail_and_await_reply(domain_participant_actor::is_enabled::new())
            .await?
        {
            self.participant_address
                .send_mail_and_await_reply(domain_participant_actor::ignore_participant::new(
                    handle,
                ))
                .await
        } else {
            Err(DdsError::NotEnabled)
        }
    }

    /// Async version of [`ignore_topic`](crate::domain::domain_participant::DomainParticipant::ignore_topic).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_topic(&self, handle: InstanceHandle) -> DdsResult<()> {
        if self
            .participant_address
            .send_mail_and_await_reply(domain_participant_actor::is_enabled::new())
            .await?
        {
            self.participant_address
                .send_mail_and_await_reply(domain_participant_actor::ignore_topic::new(handle))
                .await
        } else {
            Err(DdsError::NotEnabled)
        }
    }

    /// Async version of [`ignore_publication`](crate::domain::domain_participant::DomainParticipant::ignore_publication).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_publication(&self, handle: InstanceHandle) -> DdsResult<()> {
        if self
            .participant_address
            .send_mail_and_await_reply(domain_participant_actor::is_enabled::new())
            .await?
        {
            self.participant_address
                .send_mail_and_await_reply(domain_participant_actor::ignore_publication::new(
                    handle,
                ))
                .await
        } else {
            Err(DdsError::NotEnabled)
        }
    }

    /// Async version of [`ignore_subscription`](crate::domain::domain_participant::DomainParticipant::ignore_subscription).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_subscription(&self, handle: InstanceHandle) -> DdsResult<()> {
        if self
            .participant_address
            .send_mail_and_await_reply(domain_participant_actor::is_enabled::new())
            .await?
        {
            self.participant_address
                .send_mail_and_await_reply(domain_participant_actor::ignore_subscription::new(
                    handle,
                ))
                .await
        } else {
            Err(DdsError::NotEnabled)
        }
    }

    /// Async version of [`get_domain_id`](crate::domain::domain_participant::DomainParticipant::get_domain_id).
    #[tracing::instrument(skip(self))]
    pub async fn get_domain_id(&self) -> DdsResult<DomainId> {
        self.participant_address
            .send_mail_and_await_reply(domain_participant_actor::get_domain_id::new())
            .await
    }

    /// Async version of [`delete_contained_entities`](crate::domain::domain_participant::DomainParticipant::delete_contained_entities).
    #[tracing::instrument(skip(self))]
    pub async fn delete_contained_entities(&self) -> DdsResult<()> {
        for publisher in self
            .participant_address
            .send_mail_and_await_reply(
                domain_participant_actor::get_user_defined_publisher_list::new(),
            )
            .await?
        {
            for data_writer in publisher
                .send_mail_and_await_reply(publisher_actor::data_writer_list::new())
                .await?
            {
                publisher
                    .send_mail_and_await_reply(
                        publisher_actor::datawriter_delete::new(
                            data_writer
                                .send_mail_and_await_reply(
                                    data_writer_actor::get_instance_handle::new(),
                                )
                                .await?,
                        ),
                    )
                    .await?;
            }
            self.participant_address
                .send_mail_and_await_reply(
                    domain_participant_actor::delete_user_defined_publisher::new(
                        publisher
                            .send_mail_and_await_reply(publisher_actor::get_instance_handle::new())
                            .await?,
                    ),
                )
                .await??;
        }
        for subscriber in self
            .participant_address
            .send_mail_and_await_reply(
                domain_participant_actor::get_user_defined_subscriber_list::new(),
            )
            .await?
        {
            for data_reader in subscriber
                .send_mail_and_await_reply(subscriber_actor::data_reader_list::new())
                .await?
            {
                subscriber
                    .send_mail_and_await_reply(
                        subscriber_actor::data_reader_delete::new(
                            data_reader
                                .send_mail_and_await_reply(
                                    data_reader_actor::get_instance_handle::new(),
                                )
                                .await?,
                        ),
                    )
                    .await?;
            }
            self.participant_address
                .send_mail_and_await_reply(
                    domain_participant_actor::delete_user_defined_subscriber::new(
                        subscriber
                            .send_mail_and_await_reply(subscriber_actor::get_instance_handle::new())
                            .await?,
                    ),
                )
                .await??;
        }
        for topic in self
            .participant_address
            .send_mail_and_await_reply(domain_participant_actor::get_user_defined_topic_list::new())
            .await?
        {
            self.participant_address
                .send_mail_and_await_reply(domain_participant_actor::delete_topic::new(
                    topic
                        .send_mail_and_await_reply(topic_actor::get_instance_handle::new())
                        .await?,
                ))
                .await?;
        }
        Ok(())
    }

    /// Async version of [`assert_liveliness`](crate::domain::domain_participant::DomainParticipant::assert_liveliness).
    #[tracing::instrument(skip(self))]
    pub async fn assert_liveliness(&self) -> DdsResult<()> {
        todo!()
        // self.call_participant_method(|dp| {
        //     crate::implementation::behavior::domain_participant::assert_liveliness(dp)
        // })
    }

    /// Async version of [`set_default_publisher_qos`](crate::domain::domain_participant::DomainParticipant::set_default_publisher_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_publisher_qos(&self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => PublisherQos::default(),
            QosKind::Specific(q) => q,
        };
        self.participant_address
            .send_mail_and_await_reply(domain_participant_actor::set_default_publisher_qos::new(
                qos,
            ))
            .await
    }

    /// Async version of [`get_default_publisher_qos`](crate::domain::domain_participant::DomainParticipant::get_default_publisher_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_publisher_qos(&self) -> DdsResult<PublisherQos> {
        self.participant_address
            .send_mail_and_await_reply(domain_participant_actor::default_publisher_qos::new())
            .await
    }

    /// Async version of [`set_default_subscriber_qos`](crate::domain::domain_participant::DomainParticipant::set_default_subscriber_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_subscriber_qos(&self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => SubscriberQos::default(),
            QosKind::Specific(q) => q,
        };

        self.participant_address
            .send_mail_and_await_reply(domain_participant_actor::set_default_subscriber_qos::new(
                qos,
            ))
            .await
    }

    /// Async version of [`get_default_subscriber_qos`](crate::domain::domain_participant::DomainParticipant::get_default_subscriber_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_subscriber_qos(&self) -> DdsResult<SubscriberQos> {
        self.participant_address
            .send_mail_and_await_reply(domain_participant_actor::default_subscriber_qos::new())
            .await
    }

    /// Async version of [`set_default_topic_qos`](crate::domain::domain_participant::DomainParticipant::set_default_topic_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_topic_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => TopicQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };
        self.participant_address
            .send_mail_and_await_reply(domain_participant_actor::set_default_topic_qos::new(qos))
            .await
    }

    /// Async version of [`get_default_topic_qos`](crate::domain::domain_participant::DomainParticipant::get_default_topic_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_topic_qos(&self) -> DdsResult<TopicQos> {
        self.participant_address
            .send_mail_and_await_reply(domain_participant_actor::default_topic_qos::new())
            .await
    }

    /// Async version of [`get_discovered_participants`](crate::domain::domain_participant::DomainParticipant::get_discovered_participants).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_participants(&self) -> DdsResult<Vec<InstanceHandle>> {
        self.participant_address
            .send_mail_and_await_reply(domain_participant_actor::get_discovered_participants::new())
            .await
    }

    /// Async version of [`get_discovered_participant_data`](crate::domain::domain_participant::DomainParticipant::get_discovered_participant_data).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_participant_data(
        &self,
        _participant_handle: InstanceHandle,
    ) -> DdsResult<ParticipantBuiltinTopicData> {
        todo!()
    }

    /// Async version of [`get_discovered_topics`](crate::domain::domain_participant::DomainParticipant::get_discovered_topics).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_topics(&self) -> DdsResult<Vec<InstanceHandle>> {
        self.participant_address
            .send_mail_and_await_reply(domain_participant_actor::discovered_topic_list::new())
            .await
    }

    /// Async version of [`get_discovered_topic_data`](crate::domain::domain_participant::DomainParticipant::get_discovered_topic_data).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_topic_data(
        &self,
        topic_handle: InstanceHandle,
    ) -> DdsResult<TopicBuiltinTopicData> {
        self.participant_address
            .send_mail_and_await_reply(domain_participant_actor::discovered_topic_data::new(
                topic_handle,
            ))
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
        self.participant_address
            .send_mail_and_await_reply(domain_participant_actor::get_current_time::new())
            .await
    }
}

impl DomainParticipantAsync {
    /// Async version of [`set_qos`](crate::domain::domain_participant::DomainParticipant::set_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => {
                DomainParticipantFactory::get_instance().get_default_participant_qos()?
            }
            QosKind::Specific(q) => q,
        };

        self.participant_address
            .send_mail_and_await_reply(domain_participant_actor::set_qos::new(qos))
            .await
    }

    /// Async version of [`get_qos`](crate::domain::domain_participant::DomainParticipant::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<DomainParticipantQos> {
        self.participant_address
            .send_mail_and_await_reply(domain_participant_actor::get_qos::new())
            .await
    }

    /// Async version of [`set_listener`](crate::domain::domain_participant::DomainParticipant::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: impl DomainParticipantListener + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.participant_address
            .send_mail_and_await_reply(domain_participant_actor::set_listener::new(
                Box::new(a_listener),
                mask.to_vec(),
                self.runtime_handle.clone(),
            ))
            .await
    }

    /// Async version of [`get_statuscondition`](crate::domain::domain_participant::DomainParticipant::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub async fn get_statuscondition(&self) -> DdsResult<StatusConditionAsync> {
        todo!()
    }

    /// Async version of [`get_status_changes`](crate::domain::domain_participant::DomainParticipant::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::domain::domain_participant::DomainParticipant::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        if !self
            .participant_address
            .send_mail_and_await_reply(domain_participant_actor::is_enabled::new())
            .await?
        {
            self.participant_address
                .send_mail_and_await_reply(domain_participant_actor::get_builtin_publisher::new())
                .await?
                .send_mail_and_await_reply(publisher_actor::enable::new())
                .await?;
            self.participant_address
                .send_mail_and_await_reply(domain_participant_actor::get_built_in_subscriber::new())
                .await?
                .send_mail_and_await_reply(subscriber_actor::enable::new())
                .await?;

            for builtin_reader in self
                .participant_address
                .send_mail_and_await_reply(domain_participant_actor::get_built_in_subscriber::new())
                .await?
                .send_mail_and_await_reply(subscriber_actor::data_reader_list::new())
                .await?
            {
                builtin_reader
                    .send_mail_and_await_reply(data_reader_actor::enable::new())
                    .await?;
            }

            for builtin_writer in self
                .participant_address
                .send_mail_and_await_reply(domain_participant_actor::get_builtin_publisher::new())
                .await?
                .send_mail_and_await_reply(publisher_actor::data_writer_list::new())
                .await?
            {
                builtin_writer
                    .send_mail_and_await_reply(data_writer_actor::enable::new())
                    .await?;
            }

            self.participant_address
                .send_mail_and_await_reply(domain_participant_actor::enable::new())
                .await?;

            let domain_participant_address = self.participant_address.clone();

            // Spawn the task that regularly announces the domain participant
            self.runtime_handle.spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
                loop {
                    let r: DdsResult<()> = async {
                        let builtin_publisher = domain_participant_address
                            .send_mail_and_await_reply(domain_participant_actor::get_builtin_publisher::new())
                            .await?;
                        let data_writer_list = builtin_publisher
                            .send_mail_and_await_reply(publisher_actor::data_writer_list::new())
                            .await?;
                        for data_writer in data_writer_list {
                            if data_writer
                                .send_mail_and_await_reply(data_writer_actor::get_type_name::new())
                                .await
                                == Ok("SpdpDiscoveredParticipantData".to_string())
                            {
                                let spdp_discovered_participant_data = domain_participant_address
                                    .send_mail_and_await_reply(
                                        domain_participant_actor::as_spdp_discovered_participant_data::new(),
                                    )
                                    .await?;
                                let mut serialized_data = Vec::new();
                                spdp_discovered_participant_data.serialize_data(&mut serialized_data)?;

                                let timestamp = domain_participant_address
                                    .send_mail_and_await_reply(
                                        domain_participant_actor::get_current_time::new(),
                                    )
                                    .await?;

                                data_writer
                                    .send_mail_and_await_reply(
                                        data_writer_actor::write_w_timestamp::new(
                                            serialized_data,
                                            get_instance_handle_from_key(&spdp_discovered_participant_data.get_key()?)
                                                .unwrap(),
                                            None,
                                            timestamp,
                                        ),
                                    )
                                    .await??;


                                domain_participant_address.send_mail(domain_participant_actor::send_message::new()).await?;
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
        self.participant_address
            .send_mail_and_await_reply(domain_participant_actor::get_instance_handle::new())
            .await
    }
}
