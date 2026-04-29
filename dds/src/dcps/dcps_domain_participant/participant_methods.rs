use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    dcps::{
        dcps_domain_participant::{
            BUILT_IN_TOPIC_NAME_LIST, ContentFilteredTopicEntity, DcpsDomainParticipant,
            PublisherEntity, SubscriberEntity, TopicDescriptionKind, TopicEntity,
        },
        listeners::{
            domain_participant_listener::DcpsDomainParticipantListener,
            publisher_listener::DcpsPublisherListener, subscriber_listener::DcpsSubscriberListener,
            topic_listener::DcpsTopicListener,
        },
        status_condition::DcpsStatusCondition,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantQos, PublisherQos, QosKind, SubscriberQos, TopicQos},
        status::StatusKind,
    },
    runtime::DdsRuntime,
    transport::types::{USER_DEFINED_READER_GROUP, USER_DEFINED_TOPIC, USER_DEFINED_WRITER_GROUP},
    xtypes::dynamic_type::DynamicType,
};

impl DcpsDomainParticipant {
    #[tracing::instrument(skip(self, dcps_listener, runtime))]
    pub fn create_user_defined_publisher(
        &mut self,
        qos: QosKind<PublisherQos>,
        dcps_listener: Option<DcpsPublisherListener>,
        mask: Vec<StatusKind>,
        runtime: &impl DdsRuntime,
    ) -> DdsResult<InstanceHandle> {
        let publisher_qos = match qos {
            QosKind::Default => self.domain_participant.default_publisher_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let publisher_handle = InstanceHandle::new([
            self.domain_participant.instance_handle[0],
            self.domain_participant.instance_handle[1],
            self.domain_participant.instance_handle[2],
            self.domain_participant.instance_handle[3],
            self.domain_participant.instance_handle[4],
            self.domain_participant.instance_handle[5],
            self.domain_participant.instance_handle[6],
            self.domain_participant.instance_handle[7],
            self.domain_participant.instance_handle[8],
            self.domain_participant.instance_handle[9],
            self.domain_participant.instance_handle[10],
            self.domain_participant.instance_handle[11],
            self.publisher_counter,
            0,
            0,
            USER_DEFINED_WRITER_GROUP,
        ]);
        self.publisher_counter += 1;
        let data_writer_list = Default::default();
        let listener_sender = dcps_listener.map(|l| l.spawn(&runtime.spawner()));
        let mut publisher = PublisherEntity::new(
            publisher_qos,
            publisher_handle,
            data_writer_list,
            listener_sender,
            mask,
        );

        if self.domain_participant.enabled
            && self
                .domain_participant
                .qos
                .entity_factory
                .autoenable_created_entities
        {
            publisher.enabled = true;
        }

        self.domain_participant
            .user_defined_publisher_list
            .push(publisher);

        Ok(publisher_handle)
    }

    #[tracing::instrument(skip(self))]
    pub fn delete_user_defined_publisher(
        &mut self,
        participant_handle: &InstanceHandle,
        publisher_handle: &InstanceHandle,
    ) -> DdsResult<()> {
        if participant_handle != &self.domain_participant.instance_handle {
            return Err(DdsError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant".to_string(),
            ));
        }
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if !publisher.data_writer_list.is_empty() {
            return Err(DdsError::PreconditionNotMet(
                "Publisher still contains data writers".to_string(),
            ));
        }
        let Some(_) = self.domain_participant.remove_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(())
    }

    #[tracing::instrument(skip(self, dcps_listener, runtime))]
    pub fn create_user_defined_subscriber(
        &mut self,
        qos: QosKind<SubscriberQos>,
        dcps_listener: Option<DcpsSubscriberListener>,
        mask: Vec<StatusKind>,
        runtime: &impl DdsRuntime,
    ) -> DdsResult<InstanceHandle> {
        let subscriber_qos = match qos {
            QosKind::Default => self.domain_participant.default_subscriber_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let subscriber_handle = InstanceHandle::new([
            self.domain_participant.instance_handle[0],
            self.domain_participant.instance_handle[1],
            self.domain_participant.instance_handle[2],
            self.domain_participant.instance_handle[3],
            self.domain_participant.instance_handle[4],
            self.domain_participant.instance_handle[5],
            self.domain_participant.instance_handle[6],
            self.domain_participant.instance_handle[7],
            self.domain_participant.instance_handle[8],
            self.domain_participant.instance_handle[9],
            self.domain_participant.instance_handle[10],
            self.domain_participant.instance_handle[11],
            self.subscriber_counter,
            0,
            0,
            USER_DEFINED_READER_GROUP,
        ]);
        self.subscriber_counter += 1;

        let listener_mask = mask.to_vec();
        let data_reader_list = Default::default();

        let listener_sender = dcps_listener.map(|l| l.spawn(&runtime.spawner()));
        let mut subscriber = SubscriberEntity::new(
            subscriber_handle,
            subscriber_qos,
            data_reader_list,
            listener_sender,
            listener_mask,
        );

        if self.domain_participant.enabled
            && self
                .domain_participant
                .qos
                .entity_factory
                .autoenable_created_entities
        {
            subscriber.enabled = true;
        }

        self.domain_participant
            .user_defined_subscriber_list
            .push(subscriber);

        Ok(subscriber_handle)
    }

    #[tracing::instrument(skip(self))]
    pub fn delete_user_defined_subscriber(
        &mut self,
        participant_handle: &InstanceHandle,
        subscriber_handle: &InstanceHandle,
    ) -> DdsResult<()> {
        if &self.domain_participant.instance_handle != participant_handle {
            return Err(DdsError::PreconditionNotMet(
                "Subscriber can only be deleted from its parent participant".to_string(),
            ));
        }

        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter()
            .find(|x| &x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if !subscriber.data_reader_list.is_empty() {
            return Err(DdsError::PreconditionNotMet(
                "Subscriber still contains data readers".to_string(),
            ));
        }
        let Some(_) = self.domain_participant.remove_subscriber(subscriber_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(self, dcps_listener, type_support, runtime))]
    pub fn create_topic(
        &mut self,
        topic_name: String,
        type_name: String,
        qos: QosKind<TopicQos>,
        dcps_listener: Option<DcpsTopicListener>,
        mask: Vec<StatusKind>,
        type_support: &'static dyn DynamicType,
        runtime: &impl DdsRuntime,
    ) -> DdsResult<InstanceHandle> {
        if self
            .domain_participant
            .topic_description_list
            .iter()
            .any(|x| x.topic_name() == topic_name)
        {
            return Err(DdsError::PreconditionNotMet(format!(
                "Topic with name {topic_name} already exists.
         To access this topic call the lookup_topicdescription method.",
            )));
        }

        let status_condition = DcpsStatusCondition::default();
        let qos = match qos {
            QosKind::Default => self.domain_participant.default_topic_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let topic_handle = InstanceHandle::new([
            self.domain_participant.instance_handle[0],
            self.domain_participant.instance_handle[1],
            self.domain_participant.instance_handle[2],
            self.domain_participant.instance_handle[3],
            self.domain_participant.instance_handle[4],
            self.domain_participant.instance_handle[5],
            self.domain_participant.instance_handle[6],
            self.domain_participant.instance_handle[7],
            self.domain_participant.instance_handle[8],
            self.domain_participant.instance_handle[9],
            self.domain_participant.instance_handle[10],
            self.domain_participant.instance_handle[11],
            0,
            self.topic_counter.to_ne_bytes()[0],
            self.topic_counter.to_ne_bytes()[1],
            USER_DEFINED_TOPIC,
        ]);
        self.topic_counter += 1;
        let listener_sender = dcps_listener.map(|l| l.spawn(&runtime.spawner()));
        let topic = TopicEntity::new(
            qos,
            type_name,
            topic_name.clone(),
            topic_handle,
            status_condition,
            listener_sender,
            mask,
            type_support,
        );

        self.domain_participant
            .topic_description_list
            .push(TopicDescriptionKind::Topic(topic));

        if self.domain_participant.enabled
            && self
                .domain_participant
                .qos
                .entity_factory
                .autoenable_created_entities
        {
            self.enable_topic(topic_name, runtime)?;
        }

        Ok(topic_handle)
    }

    #[tracing::instrument(skip(self))]
    pub fn delete_user_defined_topic(
        &mut self,
        participant_handle: &InstanceHandle,
        topic_name: String,
    ) -> DdsResult<()> {
        if &self.domain_participant.instance_handle != participant_handle {
            return Err(DdsError::PreconditionNotMet(
                "Topic can only be deleted from its parent participant".to_string(),
            ));
        }

        if BUILT_IN_TOPIC_NAME_LIST.contains(&topic_name.as_str()) {
            return Ok(());
        }

        let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter()
            .find(|x| x.topic_name() == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        for publisher in self.domain_participant.user_defined_publisher_list.iter() {
            for writer in publisher.data_writer_list.iter() {
                if core::ptr::addr_eq(writer.type_support, topic.type_support) {
                    return Err(DdsError::PreconditionNotMet(
                        "Topic still attached to some data writer or data reader".to_string(),
                    ));
                }
            }
        }

        for subscriber in self.domain_participant.user_defined_subscriber_list.iter() {
            for reader in subscriber.data_reader_list.iter() {
                if core::ptr::addr_eq(reader.type_support, topic.type_support) {
                    return Err(DdsError::PreconditionNotMet(
                        "Topic still attached to some data writer or data reader".to_string(),
                    ));
                }
            }
        }

        self.domain_participant
            .topic_description_list
            .retain(|x| x.topic_name() != topic_name);

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn create_content_filtered_topic(
        &mut self,
        participant_handle: &InstanceHandle,
        name: String,
        related_topic_name: String,
        filter_expression: String,
        expression_parameters: Vec<String>,
    ) -> DdsResult<InstanceHandle> {
        if !self
            .domain_participant
            .topic_description_list
            .iter()
            .any(|x| x.topic_name() == related_topic_name)
        {
            return Err(DdsError::PreconditionNotMet(format!(
                "Related topic with name {related_topic_name} does not exist."
            )));
        }

        let topic_handle = InstanceHandle::new([
            self.domain_participant.instance_handle[0],
            self.domain_participant.instance_handle[1],
            self.domain_participant.instance_handle[2],
            self.domain_participant.instance_handle[3],
            self.domain_participant.instance_handle[4],
            self.domain_participant.instance_handle[5],
            self.domain_participant.instance_handle[6],
            self.domain_participant.instance_handle[7],
            self.domain_participant.instance_handle[8],
            self.domain_participant.instance_handle[9],
            self.domain_participant.instance_handle[10],
            self.domain_participant.instance_handle[11],
            0,
            self.topic_counter.to_ne_bytes()[0],
            self.topic_counter.to_ne_bytes()[1],
            USER_DEFINED_TOPIC,
        ]);
        self.topic_counter += 1;

        let topic = ContentFilteredTopicEntity::new(
            name,
            related_topic_name,
            filter_expression,
            expression_parameters,
        );
        self.domain_participant
            .topic_description_list
            .push(TopicDescriptionKind::ContentFilteredTopic(topic));

        Ok(topic_handle)
    }

    #[tracing::instrument(skip(self))]
    pub fn delete_content_filtered_topic(
        &mut self,
        participant_handle: &InstanceHandle,
        name: String,
    ) -> DdsResult<()> {
        Ok(())
    }

    #[tracing::instrument(skip(self, type_support))]
    pub fn find_topic(
        &mut self,
        topic_name: String,
        type_support: &'static dyn DynamicType,
    ) -> DdsResult<Option<(InstanceHandle, String)>> {
        if let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter()
            .find(|x| x.topic_name() == topic_name)
        {
            Ok(Some((topic.instance_handle, topic.type_name.clone())))
        } else {
            if let Some(discovered_topic_data) = self.domain_participant.find_topic(&topic_name) {
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
                    representation: discovered_topic_data.representation().clone(),
                };
                let type_name = discovered_topic_data.type_name.clone();
                let topic_handle = InstanceHandle::new([
                    self.domain_participant.instance_handle[0],
                    self.domain_participant.instance_handle[1],
                    self.domain_participant.instance_handle[2],
                    self.domain_participant.instance_handle[3],
                    self.domain_participant.instance_handle[4],
                    self.domain_participant.instance_handle[5],
                    self.domain_participant.instance_handle[6],
                    self.domain_participant.instance_handle[7],
                    self.domain_participant.instance_handle[8],
                    self.domain_participant.instance_handle[9],
                    self.domain_participant.instance_handle[10],
                    self.domain_participant.instance_handle[11],
                    0,
                    self.topic_counter.to_ne_bytes()[0],
                    self.topic_counter.to_ne_bytes()[1],
                    USER_DEFINED_TOPIC,
                ]);
                self.topic_counter += 1;
                let status_condition = DcpsStatusCondition::default();
                let mut topic = TopicEntity::new(
                    qos,
                    type_name.clone(),
                    topic_name.clone(),
                    topic_handle,
                    status_condition,
                    None,
                    vec![],
                    type_support,
                );
                topic.enabled = true;

                match self
                    .domain_participant
                    .topic_description_list
                    .iter_mut()
                    .find(|x| x.topic_name() == topic.topic_name)
                {
                    Some(TopicDescriptionKind::Topic(x)) => *x = topic,
                    Some(TopicDescriptionKind::ContentFilteredTopic(_)) => {
                        return Err(DdsError::IllegalOperation);
                    }
                    None => self
                        .domain_participant
                        .topic_description_list
                        .push(TopicDescriptionKind::Topic(topic)),
                }

                return Ok(Some((topic_handle, type_name)));
            }
            Ok(None)
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn lookup_topicdescription(
        &mut self,
        topic_name: String,
    ) -> DdsResult<Option<(String, InstanceHandle)>> {
        if let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter()
            .find(|x| x.topic_name() == topic_name)
        {
            Ok(Some((topic.type_name.clone(), topic.instance_handle)))
        } else {
            Ok(None)
        }
    }

    /// Ignore participant with the specified [`handle`](InstanceHandle).
    #[tracing::instrument(skip(self))]
    pub fn ignore_participant(&mut self, handle: &InstanceHandle) -> DdsResult<()> {
        // Check enabled
        if !self.domain_participant.enabled {
            return Err(DdsError::NotEnabled);
        }

        // Add to ignored participants
        if !self.domain_participant.ignored_participants.insert(*handle) {
            // Already ignored
            return Ok(());
        }

        // Remove participant
        self.remove_discovered_participant(handle);

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn ignore_publication(&mut self, handle: &InstanceHandle) -> DdsResult<()> {
        if !self.domain_participant.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.domain_participant.ignored_publications.insert(*handle);
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn ignore_subscription(&mut self, handle: &InstanceHandle) -> DdsResult<()> {
        if !self.domain_participant.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.domain_participant
            .ignored_subscriptions
            .insert(*handle);
        Ok(())
    }

    #[tracing::instrument(skip(self, runtime))]
    pub fn delete_participant_contained_entities(
        &mut self,
        runtime: &impl DdsRuntime,
    ) -> DdsResult<()> {
        let deleted_publisher_list: Vec<PublisherEntity> = self
            .domain_participant
            .user_defined_publisher_list
            .drain(..)
            .collect();
        for mut publisher in deleted_publisher_list {
            for data_writer in publisher.data_writer_list.drain(..) {
                self.announce_deleted_data_writer(data_writer, runtime);
            }
        }

        let deleted_subscriber_list: Vec<SubscriberEntity> = self
            .domain_participant
            .user_defined_subscriber_list
            .drain(..)
            .collect();
        for mut subscriber in deleted_subscriber_list {
            for data_reader in subscriber.data_reader_list.drain(..) {
                self.announce_deleted_data_reader(data_reader, runtime);
            }
        }

        self.domain_participant
            .topic_description_list
            .retain(|x| BUILT_IN_TOPIC_NAME_LIST.contains(&x.topic_name()));

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn set_default_publisher_qos(&mut self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => PublisherQos::default(),
            QosKind::Specific(q) => q,
        };

        self.domain_participant.default_publisher_qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_default_publisher_qos(&mut self) -> DdsResult<PublisherQos> {
        Ok(self.domain_participant.default_publisher_qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn set_default_subscriber_qos(&mut self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => SubscriberQos::default(),
            QosKind::Specific(q) => q,
        };

        self.domain_participant.default_subscriber_qos = qos;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_default_subscriber_qos(&mut self) -> DdsResult<SubscriberQos> {
        Ok(self.domain_participant.default_subscriber_qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn set_default_topic_qos(&mut self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => TopicQos::default(),
            QosKind::Specific(q) => {
                if q.is_consistent().is_ok() {
                    q
                } else {
                    return Err(DdsError::InconsistentPolicy);
                }
            }
        };

        qos.is_consistent()?;
        self.domain_participant.default_topic_qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_default_topic_qos(&self) -> DdsResult<TopicQos> {
        Ok(self.domain_participant.default_topic_qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_discovered_participants(&mut self) -> DdsResult<Vec<InstanceHandle>> {
        Ok(self
            .domain_participant
            .discovered_participant_list
            .iter()
            .map(|p| InstanceHandle::new(p.dds_participant_data.key().value))
            .collect())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_discovered_participant_data(
        &mut self,
        participant_handle: &InstanceHandle,
    ) -> DdsResult<ParticipantBuiltinTopicData> {
        let Some(handle) = self
            .domain_participant
            .discovered_participant_list
            .iter()
            .find(|p| &p.dds_participant_data.key().value == participant_handle)
        else {
            return Err(DdsError::BadParameter);
        };
        Ok(handle.dds_participant_data.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_discovered_topics(&mut self) -> DdsResult<Vec<InstanceHandle>> {
        Ok(self
            .domain_participant
            .discovered_topic_list
            .iter()
            .map(|x| InstanceHandle::new(x.key().value))
            .collect())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_discovered_topic_data(
        &mut self,
        topic_handle: &InstanceHandle,
    ) -> DdsResult<TopicBuiltinTopicData> {
        let Some(handle) = self
            .domain_participant
            .get_discovered_topic_data(topic_handle)
        else {
            return Err(DdsError::PreconditionNotMet(String::from(
                "Topic with this handle not discovered",
            )));
        };

        Ok(handle.clone())
    }

    #[tracing::instrument(skip(self, runtime))]
    pub fn set_domain_participant_qos(
        &mut self,
        qos: QosKind<DomainParticipantQos>,
        runtime: &impl DdsRuntime,
    ) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.domain_participant.qos = qos;
        if self.domain_participant.enabled {
            self.announce_participant(runtime);
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_domain_participant_qos(&mut self) -> DdsResult<DomainParticipantQos> {
        Ok(self.domain_participant.qos.clone())
    }

    #[tracing::instrument(skip(self, dcps_listener, runtime))]
    pub fn set_domain_participant_listener(
        &mut self,
        dcps_listener: Option<DcpsDomainParticipantListener>,
        status_kind: Vec<StatusKind>,
        runtime: &impl DdsRuntime,
    ) -> DdsResult<()> {
        let listener_sender = dcps_listener.map(|l| l.spawn(&runtime.spawner()));
        self.domain_participant.listener_sender = listener_sender;
        self.domain_participant.listener_mask = status_kind;

        Ok(())
    }

    #[tracing::instrument(skip(self, runtime))]
    pub fn enable_domain_participant(&mut self, runtime: &impl DdsRuntime) -> DdsResult<()> {
        if !self.domain_participant.enabled {
            for t in &mut self.domain_participant.topic_description_list {
                if let TopicDescriptionKind::Topic(t) = t {
                    t.enabled = true;
                }
            }
            for dw in &mut self.domain_participant.builtin_publisher.data_writer_list {
                dw.enabled = true;
            }
            self.domain_participant.builtin_publisher.enabled = true;

            for dr in &mut self.domain_participant.builtin_subscriber.data_reader_list {
                dr.enabled = true;
            }
            self.domain_participant.builtin_subscriber.enabled = true;
            self.domain_participant.enabled = true;

            self.announce_participant(runtime);
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn is_participant_empty(&mut self) -> bool {
        self.domain_participant.is_empty()
    }
}
