use super::{
    publisher::PublisherAsync, publisher_listener::PublisherListener, subscriber::SubscriberAsync,
    subscriber_listener::SubscriberListener, topic::TopicAsync, topic_listener::TopicListener,
};
use crate::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    dcps::{
        actor::ActorAddress,
        channels::{mpsc::MpscSender, oneshot::oneshot},
        domain_participant_mail::{DcpsDomainParticipantMail, ParticipantServiceMail},
        listeners::{
            domain_participant_listener::DcpsDomainParticipantListener,
            publisher_listener::DcpsPublisherListener, subscriber_listener::DcpsSubscriberListener,
            topic_listener::DcpsTopicListener,
        },
        status_condition::DcpsStatusCondition,
    },
    dds_async::{
        content_filtered_topic::ContentFilteredTopicAsync,
        domain_participant_listener::DomainParticipantListener,
        topic_description::TopicDescriptionAsync,
    },
    infrastructure::{
        domain::DomainId,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantQos, PublisherQos, QosKind, SubscriberQos, TopicQos},
        status::StatusKind,
        time::Time,
        type_support::TypeSupport,
    },
    xtypes::dynamic_type::DynamicType,
};
use alloc::{
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};

/// Async version of [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant).
pub struct DomainParticipantAsync {
    participant_address: MpscSender<DcpsDomainParticipantMail>,
    builtin_subscriber_status_condition_address: ActorAddress<DcpsStatusCondition>,
    domain_id: DomainId,
    handle: InstanceHandle,
}

impl Clone for DomainParticipantAsync {
    fn clone(&self) -> Self {
        Self {
            participant_address: self.participant_address.clone(),
            builtin_subscriber_status_condition_address: self
                .builtin_subscriber_status_condition_address
                .clone(),
            domain_id: self.domain_id,
            handle: self.handle,
        }
    }
}

impl DomainParticipantAsync {
    pub(crate) fn new(
        participant_address: MpscSender<DcpsDomainParticipantMail>,
        builtin_subscriber_status_condition_address: ActorAddress<DcpsStatusCondition>,
        domain_id: DomainId,
        handle: InstanceHandle,
    ) -> Self {
        Self {
            participant_address,
            builtin_subscriber_status_condition_address,
            domain_id,
            handle,
        }
    }

    pub(crate) fn participant_address(&self) -> &MpscSender<DcpsDomainParticipantMail> {
        &self.participant_address
    }
}

impl DomainParticipantAsync {
    /// Async version of [`create_publisher`](crate::domain::domain_participant::DomainParticipant::create_publisher).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn create_publisher(
        &self,
        qos: QosKind<PublisherQos>,
        a_listener: Option<impl PublisherListener + Send + 'static>,
        mask: &[StatusKind],
    ) -> DdsResult<PublisherAsync> {
        let (reply_sender, reply_receiver) = oneshot();
        let dcps_listener = a_listener.map(DcpsPublisherListener::new);
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::CreateUserDefinedPublisher {
                    qos,
                    dcps_listener,
                    mask: mask.to_vec(),
                    reply_sender,
                },
            ))
            .await?;
        let guid = reply_receiver.await??;
        let publisher = PublisherAsync::new(guid, self.clone());

        Ok(publisher)
    }

    /// Async version of [`delete_publisher`](crate::domain::domain_participant::DomainParticipant::delete_publisher).
    #[tracing::instrument(skip(self, a_publisher))]
    pub async fn delete_publisher(&self, a_publisher: &PublisherAsync) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::DeleteUserDefinedPublisher {
                    participant_handle: a_publisher.get_participant().handle,
                    publisher_handle: a_publisher.get_instance_handle().await,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`create_subscriber`](crate::domain::domain_participant::DomainParticipant::create_subscriber).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn create_subscriber(
        &self,
        qos: QosKind<SubscriberQos>,
        a_listener: Option<impl SubscriberListener + Send + 'static>,
        mask: &[StatusKind],
    ) -> DdsResult<SubscriberAsync> {
        let (reply_sender, reply_receiver) = oneshot();
        let dcps_listener = a_listener.map(DcpsSubscriberListener::new);
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::CreateUserDefinedSubscriber {
                    qos,
                    dcps_listener,
                    mask: mask.to_vec(),
                    reply_sender,
                },
            ))
            .await?;
        let (guid, subscriber_status_condition_address) = reply_receiver.await??;
        let subscriber =
            SubscriberAsync::new(guid, subscriber_status_condition_address, self.clone());

        Ok(subscriber)
    }

    /// Async version of [`delete_subscriber`](crate::domain::domain_participant::DomainParticipant::delete_subscriber).
    #[tracing::instrument(skip(self, a_subscriber))]
    pub async fn delete_subscriber(&self, a_subscriber: &SubscriberAsync) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::DeleteUserDefinedSubscriber {
                    participant_handle: a_subscriber.get_participant().handle,
                    subscriber_handle: a_subscriber.get_instance_handle().await,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`create_topic`](crate::domain::domain_participant::DomainParticipant::create_topic).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn create_topic<Foo>(
        &self,
        topic_name: &str,
        type_name: &str,
        qos: QosKind<TopicQos>,
        a_listener: Option<impl TopicListener + Send + 'static>,
        mask: &[StatusKind],
    ) -> DdsResult<TopicDescriptionAsync>
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
        a_listener: Option<impl TopicListener + Send + 'static>,
        mask: &[StatusKind],
        dynamic_type_representation: Arc<DynamicType>,
    ) -> DdsResult<TopicDescriptionAsync> {
        let (reply_sender, reply_receiver) = oneshot();
        let dcps_listener = a_listener.map(DcpsTopicListener::new);
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::CreateTopic {
                    topic_name: String::from(topic_name),
                    type_name: String::from(type_name),
                    qos,
                    dcps_listener,
                    mask: mask.to_vec(),
                    type_support: dynamic_type_representation,
                    participant_address: self.participant_address.clone(),
                    reply_sender,
                },
            ))
            .await?;
        let (guid, topic_status_condition_address) = reply_receiver.await??;

        Ok(TopicDescriptionAsync::Topic(TopicAsync::new(
            guid,
            topic_status_condition_address,
            String::from(type_name),
            String::from(topic_name),
            self.clone(),
        )))
    }

    /// Async version of [`delete_topic`](crate::domain::domain_participant::DomainParticipant::delete_topic).
    #[tracing::instrument(skip(self, a_topic))]
    pub async fn delete_topic(&self, a_topic: &TopicDescriptionAsync) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::DeleteUserDefinedTopic {
                    participant_handle: a_topic.get_participant().handle,
                    topic_name: a_topic.get_name(),
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`create_contentfilteredtopic`](crate::domain::domain_participant::DomainParticipant::create_contentfilteredtopic).
    #[tracing::instrument(skip(self, related_topic))]
    pub async fn create_contentfilteredtopic(
        &self,
        name: &str,
        related_topic: &TopicDescriptionAsync,
        filter_expression: String,
        expression_parameters: Vec<String>,
    ) -> DdsResult<TopicDescriptionAsync> {
        let topic = match related_topic {
            TopicDescriptionAsync::Topic(t) => t.clone(),
            TopicDescriptionAsync::ContentFilteredTopic(_) => return Err(DdsError::BadParameter),
        };
        let name = name.to_string();
        let related_topic_name = related_topic.get_name();
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::CreateContentFilteredTopic {
                    participant_handle: topic.get_participant().handle,
                    name: name.clone(),
                    related_topic_name,
                    filter_expression,
                    expression_parameters,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await??;
        Ok(TopicDescriptionAsync::ContentFilteredTopic(
            ContentFilteredTopicAsync::new(name.clone(), topic),
        ))
    }

    /// Async version of [`delete_contentfilteredtopic`](crate::domain::domain_participant::DomainParticipant::delete_contentfilteredtopic).
    #[tracing::instrument(skip(self, _a_contentfilteredtopic))]
    pub async fn delete_contentfilteredtopic(
        &self,
        _a_contentfilteredtopic: &TopicDescriptionAsync,
    ) -> DdsResult<()> {
        todo!()
        // let topic = match a_contentfilteredtopic {
        //     TopicDescriptionAsync::Topic(_) => return Err(DdsError::BadParameter),
        //     TopicDescriptionAsync::ContentFilteredTopic(t) => t.clone(),
        // };
        // let (reply_sender, reply_receiver) = oneshot();
        // self.participant_address
        //     .send(DomainParticipantMail::Participant(
        //         ParticipantServiceMail::DeleteContentFilteredTopic {
        //             participant_handle: topic.get_participant().handle,
        //             name: topic.get_name(),
        //             reply_sender,
        //         },
        //     ))
        //     .await?;
        // reply_receiver.await?
    }

    /// Async version of [`find_topic`](crate::domain::domain_participant::DomainParticipant::find_topic).
    #[tracing::instrument(skip(self))]
    pub async fn find_topic<Foo>(&self, topic_name: &str) -> DdsResult<TopicDescriptionAsync>
    where
        Foo: TypeSupport,
    {
        let type_support = Arc::new(Foo::get_type());
        let topic_name = String::from(topic_name);
        let participant_address = self.participant_address.clone();
        let participant_async = self.clone();
        loop {
            let (reply_sender, reply_receiver) = oneshot();

            participant_address
                .send(DcpsDomainParticipantMail::Participant(
                    ParticipantServiceMail::FindTopic {
                        topic_name: topic_name.clone(),
                        type_support: type_support.clone(),
                        reply_sender,
                    },
                ))
                .await?;
            if let Some((guid, topic_status_condition_address, type_name)) =
                reply_receiver.await??
            {
                return Ok(TopicDescriptionAsync::Topic(TopicAsync::new(
                    guid,
                    topic_status_condition_address,
                    type_name,
                    topic_name,
                    participant_async,
                )));
            }
        }
    }

    /// Async version of [`lookup_topicdescription`](crate::domain::domain_participant::DomainParticipant::lookup_topicdescription).
    #[tracing::instrument(skip(self))]
    pub async fn lookup_topicdescription(
        &self,
        topic_name: &str,
    ) -> DdsResult<Option<TopicDescriptionAsync>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::LookupTopicdescription {
                    topic_name: String::from(topic_name),
                    reply_sender,
                },
            ))
            .await?;
        if let Some((type_name, topic_handle, topic_status_condition_address)) =
            reply_receiver.await??
        {
            Ok(Some(TopicDescriptionAsync::Topic(TopicAsync::new(
                topic_handle,
                topic_status_condition_address,
                type_name,
                String::from(topic_name),
                self.clone(),
            ))))
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
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::IgnoreParticipant {
                    handle,
                    reply_sender,
                },
            ))
            .await?;
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
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::IgnorePublication {
                    handle,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`ignore_subscription`](crate::domain::domain_participant::DomainParticipant::ignore_subscription).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_subscription(&self, handle: InstanceHandle) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::IgnoreSubscription {
                    handle,
                    reply_sender,
                },
            ))
            .await?;
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
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::DeleteContainedEntities { reply_sender },
            ))
            .await?;
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
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::SetDefaultPublisherQos { qos, reply_sender },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`get_default_publisher_qos`](crate::domain::domain_participant::DomainParticipant::get_default_publisher_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_publisher_qos(&self) -> DdsResult<PublisherQos> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::GetDefaultPublisherQos { reply_sender },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`set_default_subscriber_qos`](crate::domain::domain_participant::DomainParticipant::set_default_subscriber_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_subscriber_qos(&self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::SetDefaultSubscriberQos { qos, reply_sender },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`get_default_subscriber_qos`](crate::domain::domain_participant::DomainParticipant::get_default_subscriber_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_subscriber_qos(&self) -> DdsResult<SubscriberQos> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::GetDefaultSubscriberQos { reply_sender },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`set_default_topic_qos`](crate::domain::domain_participant::DomainParticipant::set_default_topic_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_topic_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::SetDefaultTopicQos { qos, reply_sender },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`get_default_topic_qos`](crate::domain::domain_participant::DomainParticipant::get_default_topic_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_topic_qos(&self) -> DdsResult<TopicQos> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::GetDefaultTopicQos { reply_sender },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`get_discovered_participants`](crate::domain::domain_participant::DomainParticipant::get_discovered_participants).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_participants(&self) -> DdsResult<Vec<InstanceHandle>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::GetDiscoveredParticipants { reply_sender },
            ))
            .await?;
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
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::GetDiscoveredParticipantData {
                    participant_handle,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`get_discovered_topics`](crate::domain::domain_participant::DomainParticipant::get_discovered_topics).
    #[tracing::instrument(skip(self))]
    pub async fn get_discovered_topics(&self) -> DdsResult<Vec<InstanceHandle>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::GetDiscoveredTopics { reply_sender },
            ))
            .await?;
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
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::GetDiscoveredTopicData {
                    topic_handle,
                    reply_sender,
                },
            ))
            .await?;
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
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::GetCurrentTime { reply_sender },
            ))
            .await?;
        reply_receiver.await
    }

    /// Request type information from a remote participant using TypeLookup service (XTypes 1.3).
    ///
    /// This method sends a TypeLookup request to the specified remote participant and waits
    /// for the reply containing the requested TypeObjects.
    ///
    /// # Arguments
    /// * `target_participant_prefix` - The GUID prefix of the target participant (first 12 bytes of GUID)
    /// * `type_ids` - List of TypeIdentifiers to look up (raw XCDR-encoded bytes)
    ///
    /// # Returns
    /// The TypeLookup reply containing the requested types, or an error if the request fails.
    #[cfg(feature = "type_lookup")]
    #[tracing::instrument(skip(self))]
    pub async fn request_type_lookup(
        &self,
        target_participant_prefix: [u8; 12],
        type_ids: Vec<Vec<u8>>,
    ) -> DdsResult<crate::dcps::data_representation_builtin_endpoints::type_lookup::TypeLookupReply>
    {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::RequestTypeLookup {
                    target_participant_prefix,
                    type_ids,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// This operation discovers the [`DynamicType`] for a topic by querying remote participants using the TypeLookup service.
    /// It finds a discovered writer for the given topic name, extracts the type information from the writer's discovery data,
    /// sends a TypeLookup request to the writer's participant, and converts the received TypeObject to a [`DynamicType`].
    /// The returned [`DynamicType`] can be used with [`SubscriberAsync::create_dynamic_datareader`] to create a reader
    /// without compile-time type knowledge.
    #[cfg(feature = "type_lookup")]
    #[tracing::instrument(skip(self))]
    pub async fn discover_type(&self, topic_name: &str) -> DdsResult<DynamicType> {
        use crate::dcps::data_representation_builtin_endpoints::type_lookup::TypeLookupReturn;
        use crate::infrastructure::error::DdsError;

        // Step 1: Find a discovered writer for this topic
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::GetDiscoveredWriterForTopic {
                    topic_name: topic_name.to_string(),
                    reply_sender,
                },
            ))
            .await?;

        let Some((participant_prefix, type_info)) = reply_receiver.await?? else {
            return Err(DdsError::PreconditionNotMet(
                alloc::format!("No discovered writer found for topic '{}'", topic_name),
            ));
        };

        // Step 2: Extract TypeIdentifier from type_information
        let type_info_bytes = type_info.ok_or_else(|| {
            DdsError::PreconditionNotMet(alloc::format!(
                "Discovered writer for topic '{}' has no type information",
                topic_name
            ))
        })?;

        if type_info_bytes.is_empty() {
            return Err(DdsError::PreconditionNotMet(alloc::format!(
                "Discovered writer for topic '{}' has empty type information",
                topic_name
            )));
        }

        // Step 3: Send TypeLookup request
        // Parse TypeInformation to extract the TypeIdentifier
        use crate::xtypes::type_object::TypeInformation;

        let type_id_bytes = if let Some((type_info, _)) =
            TypeInformation::deserialize_from_bytes(&type_info_bytes)
        {
            // Extract the complete TypeIdentifier bytes
            type_info.get_complete_type_identifier_bytes()
        } else {
            // If parsing fails, try using the raw bytes as TypeIdentifier
            // (for compatibility with implementations that send TypeIdentifier directly)
            tracing::debug!("Failed to parse TypeInformation, using raw bytes");
            type_info_bytes.clone()
        };

        let reply = self
            .request_type_lookup(participant_prefix, alloc::vec![type_id_bytes])
            .await?;

        // Step 4: Extract TypeObject from reply and convert to DynamicType
        match reply.data {
            TypeLookupReturn::GetTypes(types_out) => {
                if types_out.types.is_empty() {
                    return Err(DdsError::PreconditionNotMet(alloc::format!(
                        "TypeLookup returned no types for topic '{}'",
                        topic_name
                    )));
                }

                let type_pair = &types_out.types[0];
                if type_pair.type_object.is_empty() {
                    return Err(DdsError::PreconditionNotMet(alloc::format!(
                        "TypeLookup returned empty TypeObject for topic '{}'. \
                         The remote participant may not support TypeObject serialization.",
                        topic_name
                    )));
                }

                // Deserialize TypeObject from XCDR2 bytes
                use crate::xtypes::type_object::TypeObject;
                use crate::xtypes::dynamic_type::DynamicTypeBuilderFactory;

                let (type_object, _bytes_read) =
                    TypeObject::deserialize_from_bytes(&type_pair.type_object).ok_or_else(
                        || {
                            DdsError::PreconditionNotMet(alloc::format!(
                                "Failed to deserialize TypeObject ({} bytes) for topic '{}'",
                                type_pair.type_object.len(),
                                topic_name
                            ))
                        },
                    )?;

                // Convert TypeObject to DynamicType
                DynamicTypeBuilderFactory::create_type_w_type_object(type_object).map_err(|e| {
                    DdsError::PreconditionNotMet(alloc::format!(
                        "Failed to convert TypeObject to DynamicType for topic '{}': {:?}",
                        topic_name,
                        e
                    ))
                })
            }
            TypeLookupReturn::GetTypeDependencies(_) => Err(DdsError::PreconditionNotMet(
                "Unexpected GetTypeDependencies response".to_string(),
            )),
            TypeLookupReturn::Unknown(hash, _) => Err(DdsError::PreconditionNotMet(
                alloc::format!("Unknown TypeLookup response: 0x{:08x}", hash),
            )),
        }
    }
}

impl DomainParticipantAsync {
    /// Async version of [`set_qos`](crate::domain::domain_participant::DomainParticipant::set_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::SetQos {
                    qos,
                    participant_address: self.participant_address.clone(),
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`get_qos`](crate::domain::domain_participant::DomainParticipant::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<DomainParticipantQos> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::GetQos { reply_sender },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`set_listener`](crate::domain::domain_participant::DomainParticipant::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: Option<impl DomainParticipantListener + Send + 'static>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        let dcps_listener = a_listener.map(DcpsDomainParticipantListener::new);
        self.participant_address
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::SetListener {
                    dcps_listener,
                    status_kind: mask.to_vec(),
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
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
            .send(DcpsDomainParticipantMail::Participant(
                ParticipantServiceMail::Enable {
                    participant_address: self.participant_address.clone(),
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`get_instance_handle`](crate::domain::domain_participant::DomainParticipant::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> InstanceHandle {
        self.handle
    }
}
