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
        actor::ActorAddress,
        actors::domain_participant_actor::{self, DomainParticipantActor},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DomainParticipantQos, PublisherQos, QosKind, SubscriberQos, TopicQos},
        status::StatusKind,
        time::{Duration, Time},
    },
    topic_definition::type_support::TypeSupport,
    xtypes::dynamic_type::DynamicType,
};
use std::sync::Arc;

/// Async version of [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant).
#[derive(Clone)]
pub struct DomainParticipantAsync {
    participant_address: ActorAddress<DomainParticipantActor>,
    domain_id: DomainId,
}

impl DomainParticipantAsync {
    pub(crate) fn new(
        participant_address: ActorAddress<DomainParticipantActor>,
        domain_id: DomainId,
    ) -> Self {
        Self {
            participant_address,
            domain_id,
        }
    }

    pub(crate) fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        &self.participant_address
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
        let guid = self
            .participant_address
            .send_actor_mail(domain_participant_actor::CreateUserDefinedPublisher {
                qos,
                a_listener,
                mask: mask.to_vec(),
            })?
            .receive_reply()
            .await?;
        let publisher = PublisherAsync::new(guid, self.clone());

        Ok(publisher)
    }

    /// Async version of [`delete_publisher`](crate::domain::domain_participant::DomainParticipant::delete_publisher).
    #[tracing::instrument(skip(self, a_publisher))]
    pub async fn delete_publisher(&self, a_publisher: &PublisherAsync) -> DdsResult<()> {
        self.participant_address
            .send_actor_mail(domain_participant_actor::DeleteUserDefinedPublisher {
                guid: a_publisher.guid(),
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`create_subscriber`](crate::domain::domain_participant::DomainParticipant::create_subscriber).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn create_subscriber(
        &self,
        qos: QosKind<SubscriberQos>,
        a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
        mask: &[StatusKind],
    ) -> DdsResult<SubscriberAsync> {
        let guid = self
            .participant_address
            .send_actor_mail(domain_participant_actor::CreateUserDefinedSubscriber {
                qos,
                a_listener,
                mask: mask.to_vec(),
            })?
            .receive_reply()
            .await?;
        let subscriber = SubscriberAsync::new(guid, self.clone());

        Ok(subscriber)
    }

    /// Async version of [`delete_subscriber`](crate::domain::domain_participant::DomainParticipant::delete_subscriber).
    #[tracing::instrument(skip(self, a_subscriber))]
    pub async fn delete_subscriber(&self, a_subscriber: &SubscriberAsync) -> DdsResult<()> {
        self.participant_address
            .send_actor_mail(domain_participant_actor::DeleteUserDefinedSubscriber {
                guid: a_subscriber.guid(),
            })?
            .receive_reply()
            .await
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
        self.participant_address
            .send_actor_mail(domain_participant_actor::CreateUserDefinedTopic {
                topic_name: topic_name.to_string(),
                type_name: type_name.to_string(),
                qos,
                a_listener,
                mask: mask.to_vec(),
                type_support: dynamic_type_representation,
            })?
            .receive_reply()
            .await?;

        Ok(TopicAsync::new(
            type_name.to_string(),
            topic_name.to_string(),
            self.clone(),
        ))
    }

    /// Async version of [`delete_topic`](crate::domain::domain_participant::DomainParticipant::delete_topic).
    #[tracing::instrument(skip(self, a_topic))]
    pub async fn delete_topic(&self, a_topic: &TopicAsync) -> DdsResult<()> {
        self.participant_address
            .send_actor_mail(domain_participant_actor::DeleteUserDefinedTopic {
                topic_name: a_topic.get_name(),
            })?
            .receive_reply()
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
        Foo: TypeSupport,
    {
        let type_support = Arc::new(Foo::get_type());
        self.participant_address
            .send_actor_mail(domain_participant_actor::FindTopic {
                topic_name: topic_name.to_string(),
                type_support,
            })?
            .receive_reply()
            .await?;
        todo!()
        // let participant_address = self.participant_address.clone();
        // let runtime_handle = self.executor_handle.clone();
        // let type_support = Arc::new(Foo::get_type());
        // let topic_name = topic_name.to_owned();
        // let type_name = self
        //     .participant_address
        //     .send_actor_mail(domain_participant_actor::GetTopicTypeName {
        //         topic_name: topic_name.to_string(),
        //     })?
        //     .receive_reply()
        //     .await?;
        // let participant = self.clone();
        // self.timer_handle
        //     .timeout(
        //         timeout.into(),
        //         Box::pin(async move {
        //             loop {
        //                 if let Some(_) = participant_address
        //                     .send_actor_mail(domain_participant_actor::FindTopic {
        //                         topic_name: topic_name.clone(),
        //                         type_support: type_support.clone(),
        //                         executor_handle: runtime_handle.clone(),
        //                     })?
        //                     .receive_reply()
        //                     .await?
        //                 {
        //                     return Ok(TopicAsync::new(
        //                         type_name,
        //                         topic_name.to_owned(),
        //                         participant,
        //                     ));
        //                 }
        //             }
        //         }),
        //     )
        //     .await
        //     .map_err(|_| DdsError::Timeout)?
    }

    /// Async version of [`lookup_topicdescription`](crate::domain::domain_participant::DomainParticipant::lookup_topicdescription).
    #[tracing::instrument(skip(self))]
    pub async fn lookup_topicdescription(&self, topic_name: &str) -> DdsResult<Option<TopicAsync>> {
        if let Some(type_name) = self
            .participant_address
            .send_actor_mail(domain_participant_actor::LookupTopicdescription {
                topic_name: topic_name.to_owned(),
            })?
            .receive_reply()
            .await?
        {
            Ok(Some(TopicAsync::new(
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
        SubscriberAsync::new(todo!(), self.clone())
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
        self.participant_address
            .send_actor_mail(domain_participant_actor::DeleteParticipantContainedEntities)?
            .receive_reply()
            .await
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
        self.participant_address
            .send_actor_mail(domain_participant_actor::SetDomainParticipantQos { qos })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_qos`](crate::domain::domain_participant::DomainParticipant::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<DomainParticipantQos> {
        self.participant_address
            .send_actor_mail(domain_participant_actor::GetDomainParticipantQos)?
            .receive_reply()
            .await
    }

    /// Async version of [`set_listener`](crate::domain::domain_participant::DomainParticipant::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.participant_address
            .send_actor_mail(domain_participant_actor::SetDomainParticipantListener {
                listener: a_listener,
                status_kind: mask.to_vec(),
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_statuscondition`](crate::domain::domain_participant::DomainParticipant::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(self.participant_address.clone(), todo!())
    }

    /// Async version of [`get_status_changes`](crate::domain::domain_participant::DomainParticipant::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::domain::domain_participant::DomainParticipant::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        self.participant_address
            .send_actor_mail(domain_participant_actor::EnableDomainParticipant)?
            .receive_reply()
            .await
    }

    /// Async version of [`get_instance_handle`](crate::domain::domain_participant::DomainParticipant::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.participant_address
            .send_actor_mail(domain_participant_actor::GetDomainParticipantInstanceHandle)?
            .receive_reply()
            .await
    }
}
