use std::sync::Arc;

use crate::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    data_representation_builtin_endpoints::{
        discovered_topic_data::DCPS_TOPIC,
        spdp_discovered_participant_data::{SpdpDiscoveredParticipantData, DCPS_PARTICIPANT},
    },
    domain::domain_participant_factory::DomainId,
    implementation::{
        actor::{Actor, ActorAddress},
        actors::{
            data_reader_actor, data_writer_actor,
            domain_participant_actor::{
                self, DomainParticipantActor, FooTypeSupport, BUILT_IN_TOPIC_NAME_LIST,
            },
            publisher_actor,
            status_condition_actor::StatusConditionActor,
            subscriber_actor::{self, SubscriberActor},
            topic_actor::{self, TopicActor},
        },
        runtime::{executor::ExecutorHandle, timer::TimerHandle},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantQos, PublisherQos, QosKind, SubscriberQos, TopicQos},
        status::StatusKind,
        time::{Duration, Time},
    },
    topic_definition::type_support::{DdsHasKey, DdsKey, DdsTypeXml, DynamicTypeInterface},
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
    executor_handle: ExecutorHandle,
    timer_handle: TimerHandle,
}

impl DomainParticipantAsync {
    pub(crate) fn new(
        participant_address: ActorAddress<DomainParticipantActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        builtin_subscriber_address: ActorAddress<SubscriberActor>,
        builtin_subscriber_status_condition_address: ActorAddress<StatusConditionActor>,
        domain_id: DomainId,
        executor_handle: ExecutorHandle,
        timer_handle: TimerHandle,
    ) -> Self {
        Self {
            participant_address,
            status_condition_address,
            builtin_subscriber_address,
            builtin_subscriber_status_condition_address,
            domain_id,
            executor_handle,
            timer_handle,
        }
    }

    pub(crate) fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        &self.participant_address
    }

    pub(crate) fn executor_handle(&self) -> &ExecutorHandle {
        &self.executor_handle
    }

    pub(crate) fn timer_handle(&self) -> &TimerHandle {
        &self.timer_handle
    }

    pub(crate) async fn announce_participant(&self) -> DdsResult<()> {
        if self
            .participant_address
            .send_actor_mail(domain_participant_actor::IsEnabled)?
            .receive_reply()
            .await
        {
            let builtin_publisher = self.get_builtin_publisher().await?;

            if let Some(spdp_participant_writer) = builtin_publisher
                .lookup_datawriter::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT)
                .await?
            {
                let data = self
                    .participant_address
                    .send_actor_mail(domain_participant_actor::AsSpdpDiscoveredParticipantData)?
                    .receive_reply()
                    .await;
                spdp_participant_writer.write(&data, None).await?;
            }
        }
        Ok(())
    }

    async fn announce_deleted_topic(&self, topic: Actor<TopicActor>) -> DdsResult<()> {
        let builtin_publisher = self.get_builtin_publisher().await?;

        if let Some(sedp_topics_announcer) = builtin_publisher.lookup_datawriter(DCPS_TOPIC).await?
        {
            let data = topic
                .send_actor_mail(topic_actor::AsDiscoveredTopicData)
                .receive_reply()
                .await;
            sedp_topics_announcer.dispose(&data, None).await?;
        }

        Ok(())
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
            .send_actor_mail(domain_participant_actor::CreateUserDefinedPublisher {
                qos,
                a_listener,
                mask: mask.to_vec(),
                executor_handle: self.executor_handle.clone(),
            })?
            .receive_reply()
            .await;
        let publisher = PublisherAsync::new(publisher_address, status_condition, self.clone());
        if self
            .participant_address
            .send_actor_mail(domain_participant_actor::IsEnabled)?
            .receive_reply()
            .await
            && self
                .participant_address
                .send_actor_mail(domain_participant_actor::GetQos)?
                .receive_reply()
                .await
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
        if a_publisher
            .publisher_address()
            .send_actor_mail(publisher_actor::IsEmpty)?
            .receive_reply()
            .await
        {
            if let Some(deleted_publisher) = self
                .participant_address
                .send_actor_mail(domain_participant_actor::DeleteUserDefinedPublisher {
                    handle: a_publisher.get_instance_handle().await?,
                })?
                .receive_reply()
                .await
            {
                deleted_publisher.stop().await;
                Ok(())
            } else {
                Err(DdsError::PreconditionNotMet(
                    "Publisher can only be deleted from its parent participant".to_string(),
                ))
            }
        } else {
            Err(DdsError::PreconditionNotMet(
                "Publisher still contains data writers".to_string(),
            ))
        }
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
            .send_actor_mail(domain_participant_actor::CreateUserDefinedSubscriber {
                qos,
                a_listener,
                mask: mask.to_vec(),
                executor_handle: self.executor_handle.clone(),
            })?
            .receive_reply()
            .await;

        let subscriber = SubscriberAsync::new(
            subscriber_address,
            subscriber_status_condition,
            self.clone(),
        );

        if self
            .participant_address
            .send_actor_mail(domain_participant_actor::IsEnabled)?
            .receive_reply()
            .await
            && self
                .participant_address
                .send_actor_mail(domain_participant_actor::GetQos)?
                .receive_reply()
                .await
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
        if a_subscriber
            .subscriber_address()
            .send_actor_mail(subscriber_actor::IsEmpty)?
            .receive_reply()
            .await
        {
            if let Some(deleted_subscriber) = self
                .participant_address
                .send_actor_mail(domain_participant_actor::DeleteUserDefinedSubscriber {
                    handle: a_subscriber.get_instance_handle().await?,
                })?
                .receive_reply()
                .await
            {
                deleted_subscriber.stop().await;
                Ok(())
            } else {
                Err(DdsError::PreconditionNotMet(
                    "Subscriber can only be deleted from its parent participant".to_string(),
                ))
            }
        } else {
            Err(DdsError::PreconditionNotMet(
                "Subscriber still contains data readers".to_string(),
            ))
        }
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
            .send_actor_mail(domain_participant_actor::CreateUserDefinedTopic {
                topic_name: topic_name.to_string(),
                type_name: type_name.to_string(),
                qos,
                a_listener,
                mask: mask.to_vec(),
                type_support: dynamic_type_representation.into(),
                executor_handle: self.executor_handle.clone(),
            })?
            .receive_reply()
            .await?;
        let topic = TopicAsync::new(
            topic_address,
            topic_status_condition,
            type_name.to_string(),
            topic_name.to_string(),
            self.clone(),
        );
        if self
            .participant_address
            .send_actor_mail(domain_participant_actor::IsEnabled)?
            .receive_reply()
            .await
            && self
                .participant_address
                .send_actor_mail(domain_participant_actor::GetQos)?
                .receive_reply()
                .await
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
        if a_topic.topic_address().is_closed() {
            Err(DdsError::AlreadyDeleted)
        } else {
            let topic_name = a_topic.get_name();
            if BUILT_IN_TOPIC_NAME_LIST.contains(&topic_name.as_str()) {
                return Ok(());
            }
            let publisher_list = self
                .participant_address
                .send_actor_mail(domain_participant_actor::GetPublisherList)?
                .receive_reply()
                .await;
            for publisher in publisher_list {
                let data_writer_list = publisher
                    .send_actor_mail(publisher_actor::GetDataWriterList)?
                    .receive_reply()
                    .await;
                for dw in data_writer_list {
                    if dw
                        .send_actor_mail(data_writer_actor::GetTopicName)?
                        .receive_reply()
                        .await?
                        == topic_name
                    {
                        return Err(DdsError::PreconditionNotMet(
                            "Topic still attached to some data writer".to_string(),
                        ));
                    }
                }
            }

            let subscriber_list = self
                .participant_address
                .send_actor_mail(domain_participant_actor::GetSubscriberList)?
                .receive_reply()
                .await;
            for subscriber in subscriber_list {
                let data_reader_list = subscriber
                    .send_actor_mail(subscriber_actor::GetDataReaderList)?
                    .receive_reply()
                    .await;
                for dr in data_reader_list {
                    if dr
                        .send_actor_mail(data_reader_actor::GetTopicName)?
                        .receive_reply()
                        .await?
                        == topic_name
                    {
                        return Err(DdsError::PreconditionNotMet(
                            "Topic still attached to some data reader".to_string(),
                        ));
                    }
                }
            }
            if let Some(deleted_topic) = self
                .participant_address
                .send_actor_mail(domain_participant_actor::DeleteUserDefinedTopic {
                    topic_name: a_topic.get_name(),
                })?
                .receive_reply()
                .await
            {
                deleted_topic.stop().await;
                Ok(())
            } else {
                Err(DdsError::PreconditionNotMet(
                    "Topic can only be deleted from its parent participant".to_string(),
                ))
            }
        }
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
        let participant_address = self.participant_address.clone();
        let runtime_handle = self.executor_handle.clone();
        let type_support = Arc::new(FooTypeSupport::new::<Foo>());
        let topic_name = topic_name.to_owned();
        let participant = self.clone();
        self.timer_handle
            .timeout(
                timeout.into(),
                Box::pin(async move {
                    loop {
                        if let Some((topic_address, status_condition_address)) = participant_address
                            .send_actor_mail(domain_participant_actor::FindTopic {
                                topic_name: topic_name.clone(),
                                type_support: type_support.clone(),
                                executor_handle: runtime_handle.clone(),
                            })?
                            .receive_reply()
                            .await?
                        {
                            let type_name = topic_address
                                .send_actor_mail(topic_actor::GetTypeName)?
                                .receive_reply()
                                .await;
                            return Ok(TopicAsync::new(
                                topic_address,
                                status_condition_address,
                                type_name,
                                topic_name.to_owned(),
                                participant,
                            ));
                        }
                    }
                }),
            )
            .await
            .map_err(|_| DdsError::Timeout)?
    }

    /// Async version of [`lookup_topicdescription`](crate::domain::domain_participant::DomainParticipant::lookup_topicdescription).
    #[tracing::instrument(skip(self))]
    pub async fn lookup_topicdescription(&self, topic_name: &str) -> DdsResult<Option<TopicAsync>> {
        if let Some((topic_address, status_condition_address)) = self
            .participant_address
            .send_actor_mail(domain_participant_actor::LookupTopicdescription {
                topic_name: topic_name.to_owned(),
            })?
            .receive_reply()
            .await?
        {
            let type_name = topic_address
                .send_actor_mail(topic_actor::GetTypeName)?
                .receive_reply()
                .await;
            Ok(Some(TopicAsync::new(
                topic_address,
                status_condition_address,
                type_name,
                topic_name.to_owned(),
                self.clone(),
            )))
        } else {
            Ok(None)
        }
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
        self.participant_address
            .send_actor_mail(domain_participant_actor::IgnoreParticipant { handle })?
            .receive_reply()
            .await
    }

    /// Async version of [`ignore_topic`](crate::domain::domain_participant::DomainParticipant::ignore_topic).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_topic(&self, handle: InstanceHandle) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`ignore_publication`](crate::domain::domain_participant::DomainParticipant::ignore_publication).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_publication(&self, handle: InstanceHandle) -> DdsResult<()> {
        self.participant_address
            .send_actor_mail(domain_participant_actor::IgnorePublication { handle })?
            .receive_reply()
            .await
    }

    /// Async version of [`ignore_subscription`](crate::domain::domain_participant::DomainParticipant::ignore_subscription).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_subscription(&self, handle: InstanceHandle) -> DdsResult<()> {
        self.participant_address
            .send_actor_mail(domain_participant_actor::IgnoreSubscription { handle })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_domain_id`](crate::domain::domain_participant::DomainParticipant::get_domain_id).
    #[tracing::instrument(skip(self))]
    pub fn get_domain_id(&self) -> DomainId {
        self.domain_id
    }

    /// Async version of [`delete_contained_entities`](crate::domain::domain_participant::DomainParticipant::delete_contained_entities).
    #[tracing::instrument(skip(self))]
    pub async fn delete_contained_entities(&self) -> DdsResult<()> {
        for deleted_publisher in self
            .participant_address
            .send_actor_mail(domain_participant_actor::DrainPublisherList)?
            .receive_reply()
            .await
        {
            PublisherAsync::new(
                deleted_publisher.address(),
                deleted_publisher
                    .send_actor_mail(publisher_actor::GetStatuscondition)
                    .receive_reply()
                    .await,
                self.clone(),
            )
            .delete_contained_entities()
            .await?;
        }

        for deleted_subscriber in self
            .participant_address
            .send_actor_mail(domain_participant_actor::DrainSubscriberList)?
            .receive_reply()
            .await
        {
            SubscriberAsync::new(
                deleted_subscriber.address(),
                deleted_subscriber
                    .send_actor_mail(subscriber_actor::GetStatuscondition)
                    .receive_reply()
                    .await,
                self.clone(),
            )
            .delete_contained_entities()
            .await?;
        }

        for deleted_topic in self
            .participant_address
            .send_actor_mail(domain_participant_actor::DrainTopicList)?
            .receive_reply()
            .await
        {
            self.announce_deleted_topic(deleted_topic).await?;
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
            .send_actor_mail(domain_participant_actor::SetDefaultPublisherQos { qos })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_default_publisher_qos`](crate::domain::domain_participant::DomainParticipant::get_default_publisher_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_publisher_qos(&self) -> DdsResult<PublisherQos> {
        Ok(self
            .participant_address
            .send_actor_mail(domain_participant_actor::GetDefaultPublisherQos)?
            .receive_reply()
            .await)
    }

    /// Async version of [`set_default_subscriber_qos`](crate::domain::domain_participant::DomainParticipant::set_default_subscriber_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_subscriber_qos(&self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        self.participant_address
            .send_actor_mail(domain_participant_actor::SetDefaultSubscriberQos { qos })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_default_subscriber_qos`](crate::domain::domain_participant::DomainParticipant::get_default_subscriber_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_subscriber_qos(&self) -> DdsResult<SubscriberQos> {
        Ok(self
            .participant_address
            .send_actor_mail(domain_participant_actor::GetDefaultSubscriberQos)?
            .receive_reply()
            .await)
    }

    /// Async version of [`set_default_topic_qos`](crate::domain::domain_participant::DomainParticipant::set_default_topic_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_topic_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        self.participant_address
            .send_actor_mail(domain_participant_actor::SetDefaultTopicQos { qos })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_default_topic_qos`](crate::domain::domain_participant::DomainParticipant::get_default_topic_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_topic_qos(&self) -> DdsResult<TopicQos> {
        Ok(self
            .participant_address
            .send_actor_mail(domain_participant_actor::GetDefaultTopicQos)?
            .receive_reply()
            .await)
    }

    /// Async version of [`get_discovered_participants`](crate::domain::domain_participant::DomainParticipant::get_discovered_participants).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_participants(&self) -> DdsResult<Vec<InstanceHandle>> {
        Ok(self
            .participant_address
            .send_actor_mail(domain_participant_actor::GetDiscoveredParticipants)?
            .receive_reply()
            .await)
    }

    /// Async version of [`get_discovered_participant_data`](crate::domain::domain_participant::DomainParticipant::get_discovered_participant_data).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_participant_data(
        &self,
        participant_handle: InstanceHandle,
    ) -> DdsResult<ParticipantBuiltinTopicData> {
        self.participant_address
            .send_actor_mail(domain_participant_actor::GetDiscoveredParticipantData {
                participant_handle,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_discovered_topics`](crate::domain::domain_participant::DomainParticipant::get_discovered_topics).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_topics(&self) -> DdsResult<Vec<InstanceHandle>> {
        Ok(self
            .participant_address
            .send_actor_mail(domain_participant_actor::GetDiscoveredTopics)?
            .receive_reply()
            .await)
    }

    /// Async version of [`get_discovered_topic_data`](crate::domain::domain_participant::DomainParticipant::get_discovered_topic_data).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_topic_data(
        &self,
        topic_handle: InstanceHandle,
    ) -> DdsResult<TopicBuiltinTopicData> {
        self.participant_address
            .send_actor_mail(domain_participant_actor::GetDiscoveredTopicData { topic_handle })?
            .receive_reply()
            .await
    }

    /// Async version of [`contains_entity`](crate::domain::domain_participant::DomainParticipant::contains_entity).
    #[tracing::instrument(skip(self))]
    pub async fn contains_entity(&self, _a_handle: InstanceHandle) -> DdsResult<bool> {
        todo!()
    }

    /// Async version of [`get_current_time`](crate::domain::domain_participant::DomainParticipant::get_current_time).
    #[tracing::instrument(skip(self))]
    pub async fn get_current_time(&self) -> DdsResult<Time> {
        Ok(self
            .participant_address
            .send_actor_mail(domain_participant_actor::GetCurrentTime)?
            .receive_reply()
            .await)
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

        self.participant_address
            .send_actor_mail(domain_participant_actor::SetQos { qos })?
            .receive_reply()
            .await?;
        self.announce_participant().await
    }

    /// Async version of [`get_qos`](crate::domain::domain_participant::DomainParticipant::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<DomainParticipantQos> {
        Ok(self
            .participant_address
            .send_actor_mail(domain_participant_actor::GetQos)?
            .receive_reply()
            .await)
    }

    /// Async version of [`set_listener`](crate::domain::domain_participant::DomainParticipant::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.participant_address
            .send_actor_mail(domain_participant_actor::SetListener {
                listener: a_listener,
                status_kind: mask.to_vec(),
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_statuscondition`](crate::domain::domain_participant::DomainParticipant::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(
            self.status_condition_address.clone(),
            self.executor_handle.clone(),
            self.timer_handle.clone(),
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
        if !self
            .participant_address
            .send_actor_mail(domain_participant_actor::IsEnabled)?
            .receive_reply()
            .await
        {
            let builtin_publisher = self
                .participant_address
                .send_actor_mail(domain_participant_actor::GetBuiltinPublisher)?
                .receive_reply()
                .await;
            builtin_publisher
                .send_actor_mail(publisher_actor::Enable)?
                .receive_reply()
                .await;

            let builtin_subscriber = self
                .participant_address
                .send_actor_mail(domain_participant_actor::GetBuiltInSubscriber)?
                .receive_reply()
                .await;
            builtin_subscriber
                .send_actor_mail(subscriber_actor::Enable)?
                .receive_reply()
                .await;

            self.participant_address
                .send_actor_mail(domain_participant_actor::Enable)?
                .receive_reply()
                .await?;

            for builtin_reader in builtin_subscriber
                .send_actor_mail(subscriber_actor::GetDataReaderList)?
                .receive_reply()
                .await
            {
                builtin_reader
                    .send_actor_mail(data_reader_actor::Enable)?
                    .receive_reply()
                    .await;
            }
            for builtin_writer in builtin_publisher
                .send_actor_mail(publisher_actor::GetDataWriterList)?
                .receive_reply()
                .await
            {
                let message_sender_actor = self
                    .participant_address
                    .send_actor_mail(domain_participant_actor::GetMessageSender)?
                    .receive_reply()
                    .await;
                builtin_writer
                    .send_actor_mail(data_writer_actor::Enable {
                        data_writer_address: builtin_writer.clone(),
                        message_sender_actor,
                        executor_handle: self.executor_handle.clone(),
                        timer_handle: self.timer_handle.clone(),
                    })?
                    .receive_reply()
                    .await;
            }

            self.announce_participant().await?;
        }
        Ok(())
    }

    /// Async version of [`get_instance_handle`](crate::domain::domain_participant::DomainParticipant::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self
            .participant_address
            .send_actor_mail(domain_participant_actor::GetInstanceHandle)?
            .receive_reply()
            .await)
    }
}

impl DomainParticipantAsync {
    pub(crate) async fn get_builtin_publisher(&self) -> DdsResult<PublisherAsync> {
        let publisher_address = self
            .participant_address
            .send_actor_mail(domain_participant_actor::GetBuiltinPublisher)?
            .receive_reply()
            .await;
        let publisher_status_condition = publisher_address
            .send_actor_mail(publisher_actor::GetStatuscondition)?
            .receive_reply()
            .await;
        Ok(PublisherAsync::new(
            publisher_address,
            publisher_status_condition,
            self.clone(),
        ))
    }
}
