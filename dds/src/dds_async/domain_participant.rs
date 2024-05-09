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
            domain_participant_actor::{DomainParticipantActor, FooTypeSupport},
            status_condition_actor::StatusConditionActor,
            subscriber_actor::SubscriberActor,
            topic_actor::TopicActor,
        },
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

    pub(crate) async fn announce_participant(&self) -> DdsResult<()> {
        if self.participant_address.upgrade()?.is_enabled().await {
            let builtin_publisher = self.get_builtin_publisher().await?;

            if let Some(spdp_participant_writer) = builtin_publisher
                .lookup_datawriter::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT)
                .await?
            {
                let data = self
                    .participant_address
                    .upgrade()?
                    .as_spdp_discovered_participant_data()
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
            let data = topic.as_discovered_topic_data().await;
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
            .upgrade()?
            .create_user_defined_publisher(
                qos,
                a_listener,
                mask.to_vec(),
                self.runtime_handle.clone(),
            )
            .await;
        let publisher = PublisherAsync::new(publisher_address, status_condition, self.clone());
        if self.participant_address.upgrade()?.is_enabled().await
            && self
                .participant_address
                .upgrade()?
                .get_qos()
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
        self.participant_address
            .upgrade()?
            .delete_user_defined_publisher(a_publisher.get_instance_handle().await?)
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
        let (subscriber_address, subscriber_status_condition) = self
            .participant_address
            .upgrade()?
            .create_user_defined_subscriber(
                qos,
                a_listener,
                mask.to_vec(),
                self.runtime_handle.clone(),
            )
            .await;

        let subscriber = SubscriberAsync::new(
            subscriber_address,
            subscriber_status_condition,
            self.clone(),
        );

        if self.participant_address.upgrade()?.is_enabled().await
            && self
                .participant_address
                .upgrade()?
                .get_qos()
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
        self.participant_address
            .upgrade()?
            .delete_user_defined_subscriber(a_subscriber.get_instance_handle().await?)
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
            .upgrade()?
            .create_user_defined_topic(
                topic_name.to_string(),
                type_name.to_string(),
                qos,
                a_listener,
                mask.to_vec(),
                dynamic_type_representation.into(),
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
        if self.participant_address.upgrade()?.is_enabled().await
            && self
                .participant_address
                .upgrade()?
                .get_qos()
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
        if a_topic.topic_address().upgrade().is_err() {
            Err(DdsError::AlreadyDeleted)
        } else {
            self.participant_address
                .upgrade()?
                .delete_user_defined_topic(a_topic.get_name())
                .await
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
        tokio::time::timeout(timeout.into(), async {
            loop {
                if let Some((topic_address, status_condition_address, type_name)) = self
                    .participant_address
                    .upgrade()?
                    .find_topic(
                        topic_name.to_owned(),
                        Arc::new(FooTypeSupport::new::<Foo>()),
                        self.runtime_handle.clone(),
                    )
                    .await?
                {
                    return Ok(TopicAsync::new(
                        topic_address,
                        status_condition_address,
                        type_name,
                        topic_name.to_owned(),
                        self.clone(),
                    ));
                }
            }
        })
        .await
        .map_err(|_| DdsError::Timeout)?
    }

    /// Async version of [`lookup_topicdescription`](crate::domain::domain_participant::DomainParticipant::lookup_topicdescription).
    #[tracing::instrument(skip(self))]
    pub async fn lookup_topicdescription(&self, topic_name: &str) -> DdsResult<Option<TopicAsync>> {
        if let Some((topic_address, status_condition_address, type_name)) = self
            .participant_address
            .upgrade()?
            .lookup_topicdescription(topic_name.to_owned())
            .await
        {
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
            .upgrade()?
            .ignore_participant(handle)
            .await
    }

    /// Async version of [`ignore_topic`](crate::domain::domain_participant::DomainParticipant::ignore_topic).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_topic(&self, handle: InstanceHandle) -> DdsResult<()> {
        self.participant_address
            .upgrade()?
            .ignore_topic(handle)
            .await
    }

    /// Async version of [`ignore_publication`](crate::domain::domain_participant::DomainParticipant::ignore_publication).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_publication(&self, handle: InstanceHandle) -> DdsResult<()> {
        self.participant_address
            .upgrade()?
            .ignore_publication(handle)
            .await
    }

    /// Async version of [`ignore_subscription`](crate::domain::domain_participant::DomainParticipant::ignore_subscription).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_subscription(&self, handle: InstanceHandle) -> DdsResult<()> {
        self.participant_address
            .upgrade()?
            .ignore_subscription(handle)
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
        let participant = self.participant_address.upgrade()?;

        for deleted_publisher in participant.drain_publisher_list().await {
            PublisherAsync::new(
                deleted_publisher.address(),
                deleted_publisher.get_statuscondition().await,
                self.clone(),
            )
            .delete_contained_entities()
            .await?;
        }

        for deleted_subscriber in participant.drain_subscriber_list().await {
            SubscriberAsync::new(
                deleted_subscriber.address(),
                deleted_subscriber.get_statuscondition().await,
                self.clone(),
            )
            .delete_contained_entities()
            .await?;
        }

        for deleted_topic in participant.drain_topic_list().await {
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
            .upgrade()?
            .set_default_publisher_qos(qos)
            .await
    }

    /// Async version of [`get_default_publisher_qos`](crate::domain::domain_participant::DomainParticipant::get_default_publisher_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_publisher_qos(&self) -> DdsResult<PublisherQos> {
        Ok(self
            .participant_address
            .upgrade()?
            .get_default_publisher_qos()
            .await)
    }

    /// Async version of [`set_default_subscriber_qos`](crate::domain::domain_participant::DomainParticipant::set_default_subscriber_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_subscriber_qos(&self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        self.participant_address
            .upgrade()?
            .set_default_subscriber_qos(qos)
            .await
    }

    /// Async version of [`get_default_subscriber_qos`](crate::domain::domain_participant::DomainParticipant::get_default_subscriber_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_subscriber_qos(&self) -> DdsResult<SubscriberQos> {
        Ok(self
            .participant_address
            .upgrade()?
            .get_default_subscriber_qos()
            .await)
    }

    /// Async version of [`set_default_topic_qos`](crate::domain::domain_participant::DomainParticipant::set_default_topic_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_topic_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        self.participant_address
            .upgrade()?
            .set_default_topic_qos(qos)
            .await
    }

    /// Async version of [`get_default_topic_qos`](crate::domain::domain_participant::DomainParticipant::get_default_topic_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_topic_qos(&self) -> DdsResult<TopicQos> {
        Ok(self
            .participant_address
            .upgrade()?
            .get_default_topic_qos()
            .await)
    }

    /// Async version of [`get_discovered_participants`](crate::domain::domain_participant::DomainParticipant::get_discovered_participants).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_participants(&self) -> DdsResult<Vec<InstanceHandle>> {
        Ok(self
            .participant_address
            .upgrade()?
            .get_discovered_participants()
            .await)
    }

    /// Async version of [`get_discovered_participant_data`](crate::domain::domain_participant::DomainParticipant::get_discovered_participant_data).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_participant_data(
        &self,
        participant_handle: InstanceHandle,
    ) -> DdsResult<ParticipantBuiltinTopicData> {
        self.participant_address
            .upgrade()?
            .get_discovered_participant_data(participant_handle)
            .await
    }

    /// Async version of [`get_discovered_topics`](crate::domain::domain_participant::DomainParticipant::get_discovered_topics).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_topics(&self) -> DdsResult<Vec<InstanceHandle>> {
        Ok(self
            .participant_address
            .upgrade()?
            .get_discovered_topics()
            .await)
    }

    /// Async version of [`get_discovered_topic_data`](crate::domain::domain_participant::DomainParticipant::get_discovered_topic_data).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_topic_data(
        &self,
        topic_handle: InstanceHandle,
    ) -> DdsResult<TopicBuiltinTopicData> {
        self.participant_address
            .upgrade()?
            .get_discovered_topic_data(topic_handle)
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
        Ok(self.participant_address.upgrade()?.get_current_time().await)
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

        self.participant_address.upgrade()?.set_qos(qos).await?;
        self.announce_participant().await
    }

    /// Async version of [`get_qos`](crate::domain::domain_participant::DomainParticipant::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<DomainParticipantQos> {
        Ok(self.participant_address.upgrade()?.get_qos().await)
    }

    /// Async version of [`set_listener`](crate::domain::domain_participant::DomainParticipant::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.participant_address
            .upgrade()?
            .set_listener(a_listener, mask.to_vec(), self.runtime_handle.clone())
            .await;
        Ok(())
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
        self.participant_address.upgrade()?.enable().await?;

        self.announce_participant().await?;

        Ok(())
    }

    /// Async version of [`get_instance_handle`](crate::domain::domain_participant::DomainParticipant::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self
            .participant_address
            .upgrade()?
            .get_instance_handle()
            .await)
    }
}

impl DomainParticipantAsync {
    pub(crate) async fn get_builtin_publisher(&self) -> DdsResult<PublisherAsync> {
        let publisher_address = self
            .participant_address
            .upgrade()?
            .get_builtin_publisher()
            .await;
        let publisher_status_condition = publisher_address.get_statuscondition().await;
        Ok(PublisherAsync::new(
            publisher_address.address(),
            publisher_status_condition,
            self.clone(),
        ))
    }
}
