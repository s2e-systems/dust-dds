use super::{
    condition::StatusConditionAsync, domain_participant_listener::DomainParticipantListenerAsync,
    publisher::PublisherAsync, publisher_listener::PublisherListenerAsync,
    subscriber::SubscriberAsync, subscriber_listener::SubscriberListenerAsync, topic::TopicAsync,
    topic_listener::TopicListenerAsync,
};
use crate::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    domain::domain_participant_factory::DomainId,
    implementation::{
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            domain_participant_actor_mail::{DomainParticipantMail, ParticipantServiceMail},
        },
        listeners::{
            domain_participant_listener::DomainParticipantListenerActor,
            publisher_listener::PublisherListenerActor,
            subscriber_listener::SubscriberListenerActor, topic_listener::TopicListenerActor,
        },
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantQos, PublisherQos, QosKind, SubscriberQos, TopicQos},
        status::StatusKind,
        time::{Duration, Time},
    },
    runtime::{
        actor::{Actor, ActorAddress},
        executor::ExecutorHandle,
        oneshot::oneshot,
        timer::TimerHandle,
    },
    topic_definition::type_support::TypeSupport,
    xtypes::dynamic_type::DynamicType,
};
use std::sync::Arc;

/// Async version of [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant).
#[derive(Clone)]
pub struct DomainParticipantAsync {
    participant_address: ActorAddress<DomainParticipantActor>,
    status_condition_address: ActorAddress<StatusConditionActor>,
    builtin_subscriber_status_condition_address: ActorAddress<StatusConditionActor>,
    domain_id: DomainId,
    handle: InstanceHandle,
    timer_handle: TimerHandle,
    executor_handle: ExecutorHandle,
}

impl DomainParticipantAsync {
    pub(crate) fn new(
        participant_address: ActorAddress<DomainParticipantActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        builtin_subscriber_status_condition_address: ActorAddress<StatusConditionActor>,
        domain_id: DomainId,
        handle: InstanceHandle,
        timer_handle: TimerHandle,
        executor_handle: ExecutorHandle,
    ) -> Self {
        Self {
            participant_address,
            status_condition_address,
            builtin_subscriber_status_condition_address,
            domain_id,
            handle,
            timer_handle,
            executor_handle,
        }
    }

    pub(crate) fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        &self.participant_address
    }

    pub(crate) fn executor_handle(&self) -> &ExecutorHandle {
        &self.executor_handle
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
        let (reply_sender, reply_receiver) = oneshot();
        let status_condition = Actor::spawn(StatusConditionActor::default(), &self.executor_handle);
        let publisher_status_condition_address = status_condition.address();
        let listener_sender =
            a_listener.map(|l| PublisherListenerActor::spawn(l, self.executor_handle()));
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::CreateUserDefinedPublisher {
                    qos,
                    status_condition,
                    listener_sender,
                    mask: mask.to_vec(),
                    reply_sender,
                },
            ))?;
        let guid = reply_receiver.await??;
        let publisher = PublisherAsync::new(guid, publisher_status_condition_address, self.clone());

        Ok(publisher)
    }

    /// Async version of [`delete_publisher`](crate::domain::domain_participant::DomainParticipant::delete_publisher).
    #[tracing::instrument(skip(self, a_publisher))]
    pub async fn delete_publisher(&self, a_publisher: &PublisherAsync) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::DeleteUserDefinedPublisher {
                    participant_handle: a_publisher.get_participant().handle,
                    publisher_handle: a_publisher.get_instance_handle().await,
                    reply_sender,
                },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`create_subscriber`](crate::domain::domain_participant::DomainParticipant::create_subscriber).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn create_subscriber(
        &self,
        qos: QosKind<SubscriberQos>,
        a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
        mask: &[StatusKind],
    ) -> DdsResult<SubscriberAsync> {
        let (reply_sender, reply_receiver) = oneshot();
        let status_condition = Actor::spawn(StatusConditionActor::default(), &self.executor_handle);
        let subscriber_status_condition_address = status_condition.address();
        let listener_sender =
            a_listener.map(|l| SubscriberListenerActor::spawn(l, self.executor_handle()));
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::CreateUserDefinedSubscriber {
                    qos,
                    status_condition,
                    listener_sender,
                    mask: mask.to_vec(),
                    reply_sender,
                },
            ))?;
        let guid = reply_receiver.await??;
        let subscriber =
            SubscriberAsync::new(guid, subscriber_status_condition_address, self.clone());

        Ok(subscriber)
    }

    /// Async version of [`delete_subscriber`](crate::domain::domain_participant::DomainParticipant::delete_subscriber).
    #[tracing::instrument(skip(self, a_subscriber))]
    pub async fn delete_subscriber(&self, a_subscriber: &SubscriberAsync) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::DeleteUserDefinedSubscriber {
                    participant_handle: a_subscriber.get_participant().handle,
                    subscriber_handle: a_subscriber.get_instance_handle().await,
                    reply_sender,
                },
            ))?;
        reply_receiver.await?
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
        Foo: TypeSupport,
    {
        let type_support = Arc::new(Foo::get_type());

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
        dynamic_type_representation: Arc<dyn DynamicType + Send + Sync>,
    ) -> DdsResult<TopicAsync> {
        let (reply_sender, reply_receiver) = oneshot();
        let status_condition = Actor::spawn(StatusConditionActor::default(), &self.executor_handle);
        let topic_status_condition_address = status_condition.address();
        let listener_sender =
            a_listener.map(|l| TopicListenerActor::spawn(l, self.executor_handle()));
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::CreateTopic {
                    topic_name: topic_name.to_string(),
                    type_name: type_name.to_string(),
                    qos,
                    status_condition,
                    listener_sender,
                    mask: mask.to_vec(),
                    type_support: dynamic_type_representation,
                    reply_sender,
                },
            ))?;
        let guid = reply_receiver.await??;

        Ok(TopicAsync::new(
            guid,
            topic_status_condition_address,
            type_name.to_string(),
            topic_name.to_string(),
            self.clone(),
        ))
    }

    /// Async version of [`delete_topic`](crate::domain::domain_participant::DomainParticipant::delete_topic).
    #[tracing::instrument(skip(self, a_topic))]
    pub async fn delete_topic(&self, a_topic: &TopicAsync) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::DeleteUserDefinedTopic {
                    participant_handle: a_topic.get_participant().handle,
                    topic_name: a_topic.get_name(),
                    reply_sender,
                },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`find_topic`](crate::domain::domain_participant::DomainParticipant::find_topic).
    #[tracing::instrument(skip(self))]
    pub async fn find_topic<Foo>(
        &self,
        topic_name: &str,
        timeout: Duration,
    ) -> DdsResult<TopicAsync>
    where
        Foo: TypeSupport,
    {
        let type_support = Arc::new(Foo::get_type());
        let topic_name = topic_name.to_owned();
        let participant_address = self.participant_address.clone();
        let participant_async = self.clone();
        let executor_handle = self.executor_handle.clone();
        self.timer_handle
            .timeout(
                timeout.into(),
                Box::pin(async move {
                    loop {
                        let (reply_sender, reply_receiver) = oneshot();
                        let status_condition =
                            Actor::spawn(StatusConditionActor::default(), &executor_handle);

                        participant_address.send_actor_mail(DomainParticipantMail::Participant(
                            ParticipantServiceMail::FindTopic {
                                topic_name: topic_name.clone(),
                                type_support: type_support.clone(),
                                status_condition,
                                reply_sender,
                            },
                        ))?;
                        if let Some((guid, topic_status_condition_address, type_name)) =
                            reply_receiver.await??
                        {
                            return Ok(TopicAsync::new(
                                guid,
                                topic_status_condition_address,
                                type_name.to_string(),
                                topic_name.to_string(),
                                participant_async,
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
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::LookupTopicdescription {
                    topic_name: topic_name.to_owned(),
                    reply_sender,
                },
            ))?;
        if let Some((type_name, topic_handle, topic_status_condition_address)) =
            reply_receiver.await??
        {
            Ok(Some(TopicAsync::new(
                topic_handle,
                topic_status_condition_address,
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
            self.handle,
            self.builtin_subscriber_status_condition_address.clone(),
            self.clone(),
        )
    }

    /// Async version of [`ignore_participant`](crate::domain::domain_participant::DomainParticipant::ignore_participant).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_participant(&self, handle: InstanceHandle) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::IgnoreParticipant {
                    handle,
                    reply_sender,
                },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`ignore_topic`](crate::domain::domain_participant::DomainParticipant::ignore_topic).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_topic(&self, handle: InstanceHandle) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`ignore_publication`](crate::domain::domain_participant::DomainParticipant::ignore_publication).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_publication(&self, handle: InstanceHandle) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::IgnorePublication {
                    handle,
                    reply_sender,
                },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`ignore_subscription`](crate::domain::domain_participant::DomainParticipant::ignore_subscription).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_subscription(&self, handle: InstanceHandle) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::IgnoreSubscription {
                    handle,
                    reply_sender,
                },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`get_domain_id`](crate::domain::domain_participant::DomainParticipant::get_domain_id).
    #[tracing::instrument(skip(self))]
    pub fn get_domain_id(&self) -> DomainId {
        self.domain_id
    }

    /// Async version of [`delete_contained_entities`](crate::domain::domain_participant::DomainParticipant::delete_contained_entities).
    #[tracing::instrument(skip(self))]
    pub async fn delete_contained_entities(&self) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::DeleteContainedEntities { reply_sender },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`assert_liveliness`](crate::domain::domain_participant::DomainParticipant::assert_liveliness).
    #[tracing::instrument(skip(self))]
    pub async fn assert_liveliness(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`set_default_publisher_qos`](crate::domain::domain_participant::DomainParticipant::set_default_publisher_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_publisher_qos(&self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::SetDefaultPublisherQos { qos, reply_sender },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`get_default_publisher_qos`](crate::domain::domain_participant::DomainParticipant::get_default_publisher_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_publisher_qos(&self) -> DdsResult<PublisherQos> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::GetDefaultPublisherQos { reply_sender },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`set_default_subscriber_qos`](crate::domain::domain_participant::DomainParticipant::set_default_subscriber_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_subscriber_qos(&self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::SetDefaultSubscriberQos { qos, reply_sender },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`get_default_subscriber_qos`](crate::domain::domain_participant::DomainParticipant::get_default_subscriber_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_subscriber_qos(&self) -> DdsResult<SubscriberQos> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::GetDefaultSubscriberQos { reply_sender },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`set_default_topic_qos`](crate::domain::domain_participant::DomainParticipant::set_default_topic_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_topic_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::SetDefaultTopicQos { qos, reply_sender },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`get_default_topic_qos`](crate::domain::domain_participant::DomainParticipant::get_default_topic_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_topic_qos(&self) -> DdsResult<TopicQos> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::GetDefaultTopicQos { reply_sender },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`get_discovered_participants`](crate::domain::domain_participant::DomainParticipant::get_discovered_participants).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_participants(&self) -> DdsResult<Vec<InstanceHandle>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::GetDiscoveredParticipants { reply_sender },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`get_discovered_participant_data`](crate::domain::domain_participant::DomainParticipant::get_discovered_participant_data).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_participant_data(
        &self,
        participant_handle: InstanceHandle,
    ) -> DdsResult<ParticipantBuiltinTopicData> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::GetDiscoveredParticipantData {
                    participant_handle,
                    reply_sender,
                },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`get_discovered_topics`](crate::domain::domain_participant::DomainParticipant::get_discovered_topics).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_topics(&self) -> DdsResult<Vec<InstanceHandle>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::GetDiscoveredTopics { reply_sender },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`get_discovered_topic_data`](crate::domain::domain_participant::DomainParticipant::get_discovered_topic_data).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_topic_data(
        &self,
        topic_handle: InstanceHandle,
    ) -> DdsResult<TopicBuiltinTopicData> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::GetDiscoveredTopicData {
                    topic_handle,
                    reply_sender,
                },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`contains_entity`](crate::domain::domain_participant::DomainParticipant::contains_entity).
    #[tracing::instrument(skip(self))]
    pub async fn contains_entity(&self, _a_handle: InstanceHandle) -> DdsResult<bool> {
        todo!()
    }

    /// Async version of [`get_current_time`](crate::domain::domain_participant::DomainParticipant::get_current_time).
    #[tracing::instrument(skip(self))]
    pub async fn get_current_time(&self) -> DdsResult<Time> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::GetCurrentTime { reply_sender },
            ))?;
        Ok(reply_receiver.await?)
    }
}

impl DomainParticipantAsync {
    /// Async version of [`set_qos`](crate::domain::domain_participant::DomainParticipant::set_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::SetQos { qos, reply_sender },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`get_qos`](crate::domain::domain_participant::DomainParticipant::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<DomainParticipantQos> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::GetQos { reply_sender },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`set_listener`](crate::domain::domain_participant::DomainParticipant::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        let listener_sender =
            a_listener.map(|l| DomainParticipantListenerActor::spawn(l, self.executor_handle()));
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::SetListener {
                    listener_sender,
                    status_kind: mask.to_vec(),
                    reply_sender,
                },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`get_statuscondition`](crate::domain::domain_participant::DomainParticipant::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(self.status_condition_address.clone())
    }

    /// Async version of [`get_status_changes`](crate::domain::domain_participant::DomainParticipant::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::domain::domain_participant::DomainParticipant::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::Enable { reply_sender },
            ))?;
        reply_receiver.await?
    }

    /// Async version of [`get_instance_handle`](crate::domain::domain_participant::DomainParticipant::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> InstanceHandle {
        self.handle
    }
}
