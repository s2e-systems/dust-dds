use std::sync::Arc;

use crate::{
    builtin_topics::{
        BuiltInTopicKey, ParticipantBuiltinTopicData, TopicBuiltinTopicData, DCPS_PARTICIPANT,
        DCPS_PUBLICATION, DCPS_SUBSCRIPTION, DCPS_TOPIC,
    },
    dds_async::{
        domain_participant_listener::DomainParticipantListenerAsync,
        publisher_listener::PublisherListenerAsync, subscriber_listener::SubscriberListenerAsync,
        topic_listener::TopicListenerAsync,
    },
    implementation::{
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            entities::{
                publisher::PublisherEntity, subscriber::SubscriberEntity, topic::TopicEntity,
            },
        },
        listeners::{
            publisher_listener::PublisherListenerThread,
            subscriber_listener::SubscriberListenerThread,
        },
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantQos, PublisherQos, QosKind, SubscriberQos, TopicQos},
        status::StatusKind,
        time::Time,
    },
    runtime::actor::{Actor, ActorAddress, Mail, MailHandler},
    xtypes::dynamic_type::DynamicType,
};

pub const BUILT_IN_TOPIC_NAME_LIST: [&str; 4] = [
    DCPS_PARTICIPANT,
    DCPS_TOPIC,
    DCPS_PUBLICATION,
    DCPS_SUBSCRIPTION,
];

pub struct CreateUserDefinedPublisher {
    pub qos: QosKind<PublisherQos>,
    pub a_listener: Option<Box<dyn PublisherListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
}
impl Mail for CreateUserDefinedPublisher {
    type Result = DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>;
}
impl MailHandler<CreateUserDefinedPublisher> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: CreateUserDefinedPublisher,
    ) -> <CreateUserDefinedPublisher as Mail>::Result {
        let publisher_qos = match message.qos {
            QosKind::Default => self.domain_participant.default_publisher_qos().clone(),
            QosKind::Specific(q) => q,
        };

        let publisher_handle = self.instance_handle_counter.generate_new_instance_handle();
        let status_condition =
            Actor::spawn(StatusConditionActor::default(), &self.executor.handle());
        let publisher_status_condition_address = status_condition.address();
        let mut publisher = PublisherEntity::new(
            publisher_qos,
            publisher_handle,
            message.a_listener.map(PublisherListenerThread::new),
            message.mask,
            status_condition,
        );

        if self.domain_participant.enabled()
            && self
                .domain_participant
                .qos()
                .entity_factory
                .autoenable_created_entities
        {
            publisher.enable();
        }

        self.domain_participant.insert_publisher(publisher);

        Ok((publisher_handle, publisher_status_condition_address))
    }
}

pub struct DeleteUserDefinedPublisher {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
}
impl Mail for DeleteUserDefinedPublisher {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteUserDefinedPublisher> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteUserDefinedPublisher,
    ) -> <DeleteUserDefinedPublisher as Mail>::Result {
        if message.participant_handle != self.domain_participant.instance_handle() {
            return Err(DdsError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant".to_string(),
            ));
        }

        if self
            .domain_participant
            .get_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .data_writer_list()
            .count()
            > 0
        {
            return Err(DdsError::PreconditionNotMet(
                "Publisher still contains data writers".to_string(),
            ));
        }
        self.domain_participant
            .remove_publisher(&message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        Ok(())
    }
}

pub struct CreateUserDefinedSubscriber {
    pub qos: QosKind<SubscriberQos>,
    pub a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
}
impl Mail for CreateUserDefinedSubscriber {
    type Result = DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>;
}
impl MailHandler<CreateUserDefinedSubscriber> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: CreateUserDefinedSubscriber,
    ) -> <CreateUserDefinedSubscriber as Mail>::Result {
        let subscriber_qos = match message.qos {
            QosKind::Default => self.domain_participant.default_subscriber_qos().clone(),
            QosKind::Specific(q) => q,
        };
        let subscriber_handle = self.instance_handle_counter.generate_new_instance_handle();
        let status_kind = message.mask.to_vec();

        let mut subscriber = SubscriberEntity::new(
            subscriber_handle,
            subscriber_qos,
            Actor::spawn(StatusConditionActor::default(), &self.executor.handle()),
            message.a_listener.map(SubscriberListenerThread::new),
            status_kind,
        );

        let subscriber_status_condition_address = subscriber.status_condition().address();

        if self.domain_participant.enabled()
            && self
                .domain_participant
                .qos()
                .entity_factory
                .autoenable_created_entities
        {
            subscriber.enable();
        }

        self.domain_participant.insert_subscriber(subscriber);

        Ok((
            subscriber_handle.into(),
            subscriber_status_condition_address,
        ))
    }
}

pub struct DeleteUserDefinedSubscriber {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
}
impl Mail for DeleteUserDefinedSubscriber {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteUserDefinedSubscriber> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteUserDefinedSubscriber,
    ) -> <DeleteUserDefinedSubscriber as Mail>::Result {
        if self.domain_participant.instance_handle() != message.participant_handle {
            return Err(DdsError::PreconditionNotMet(
                "Subscriber can only be deleted from its parent participant".to_string(),
            ));
        }

        if self
            .domain_participant
            .get_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .data_reader_list()
            .count()
            > 0
        {
            return Err(DdsError::PreconditionNotMet(
                "Subscriber still contains data readers".to_string(),
            ));
        }
        self.domain_participant
            .remove_subscriber(&message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        Ok(())
    }
}

pub struct CreateUserDefinedTopic {
    pub topic_name: String,
    pub type_name: String,
    pub qos: QosKind<TopicQos>,
    pub a_listener: Option<Box<dyn TopicListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
    pub type_support: Arc<dyn DynamicType + Send + Sync>,
}
impl Mail for CreateUserDefinedTopic {
    type Result = DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>;
}
impl MailHandler<CreateUserDefinedTopic> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: CreateUserDefinedTopic,
    ) -> <CreateUserDefinedTopic as Mail>::Result {
        if self
            .domain_participant
            .get_topic(&message.topic_name)
            .is_some()
        {
            return Err(DdsError::PreconditionNotMet(format!(
                "Topic with name {} already exists.
             To access this topic call the lookup_topicdescription method.",
                message.topic_name
            )));
        }

        let qos = match message.qos {
            QosKind::Default => self.domain_participant.get_default_topic_qos().clone(),
            QosKind::Specific(q) => q,
        };

        let topic_handle = self.instance_handle_counter.generate_new_instance_handle();
        let status_condition =
            Actor::spawn(StatusConditionActor::default(), &self.executor.handle());
        let topic_status_condition_address = status_condition.address();

        let topic = TopicEntity::new(
            qos,
            message.type_name,
            message.topic_name.clone(),
            topic_handle,
            status_condition,
            None,
            vec![],
            message.type_support,
        );

        if self.domain_participant.enabled()
            && self
                .domain_participant
                .qos()
                .entity_factory
                .autoenable_created_entities
        {
            let topic_qos = topic.qos();
            let topic_builtin_topic_data = TopicBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: topic.instance_handle().into(),
                },
                name: topic.topic_name().to_owned(),
                type_name: topic.type_name().to_owned(),
                durability: topic_qos.durability.clone(),
                deadline: topic_qos.deadline.clone(),
                latency_budget: topic_qos.latency_budget.clone(),
                liveliness: topic_qos.liveliness.clone(),
                reliability: topic_qos.reliability.clone(),
                transport_priority: topic_qos.transport_priority.clone(),
                lifespan: topic_qos.lifespan.clone(),
                destination_order: topic_qos.destination_order.clone(),
                history: topic_qos.history.clone(),
                resource_limits: topic_qos.resource_limits.clone(),
                ownership: topic_qos.ownership.clone(),
                topic_data: topic_qos.topic_data.clone(),
                representation: topic_qos.representation.clone(),
            };
            self.domain_participant
                .announce_topic(topic_builtin_topic_data)?;
        }

        self.domain_participant.insert_topic(topic);

        Ok((topic_handle, topic_status_condition_address))
    }
}

pub struct DeleteUserDefinedTopic {
    pub participant_handle: InstanceHandle,
    pub topic_name: String,
}
impl Mail for DeleteUserDefinedTopic {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteUserDefinedTopic> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteUserDefinedTopic,
    ) -> <DeleteUserDefinedTopic as Mail>::Result {
        if self.domain_participant.instance_handle() != message.participant_handle {
            return Err(DdsError::PreconditionNotMet(
                "Topic can only be deleted from its parent participant".to_string(),
            ));
        }

        if BUILT_IN_TOPIC_NAME_LIST.contains(&message.topic_name.as_str()) {
            return Ok(());
        }

        if Arc::strong_count(
            self.domain_participant
                .get_topic(&message.topic_name)
                .ok_or(DdsError::AlreadyDeleted)?
                .type_support(),
        ) > 1
        {
            return Err(DdsError::PreconditionNotMet(
                "Topic still attached to some data writer or data reader".to_string(),
            ));
        }

        self.domain_participant
            .remove_topic(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?;

        Ok(())
    }
}

pub struct FindTopic {
    pub topic_name: String,
    pub type_support: Arc<dyn DynamicType + Send + Sync>,
}
impl Mail for FindTopic {
    type Result = DdsResult<Option<(InstanceHandle, ActorAddress<StatusConditionActor>, String)>>;
}
impl MailHandler<FindTopic> for DomainParticipantActor {
    fn handle(&mut self, message: FindTopic) -> <FindTopic as Mail>::Result {
        if let Some(topic) = self.domain_participant.get_topic(&message.topic_name) {
            Ok(Some((
                topic.instance_handle().into(),
                topic.status_condition().address(),
                topic.type_name().to_owned(),
            )))
        } else {
            if let Some(discovered_topic_data) =
                self.domain_participant.find_topic(&message.topic_name)
            {
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
                let topic_handle = self.instance_handle_counter.generate_new_instance_handle();
                let mut topic = TopicEntity::new(
                    qos,
                    type_name.clone(),
                    message.topic_name.clone(),
                    topic_handle,
                    Actor::spawn(StatusConditionActor::default(), &self.executor.handle()),
                    None,
                    vec![],
                    message.type_support,
                );
                topic.enable();
                let topic_status_condition_address = topic.status_condition().address();

                self.domain_participant.insert_topic(topic);
                return Ok(Some((
                    topic_handle.into(),
                    topic_status_condition_address,
                    type_name,
                )));
            }
            Ok(None)
        }
    }
}

pub struct LookupTopicdescription {
    pub topic_name: String,
}
impl Mail for LookupTopicdescription {
    type Result = DdsResult<Option<(String, InstanceHandle, ActorAddress<StatusConditionActor>)>>;
}
impl MailHandler<LookupTopicdescription> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: LookupTopicdescription,
    ) -> <LookupTopicdescription as Mail>::Result {
        if let Some(topic) = self.domain_participant.get_topic(&message.topic_name) {
            Ok(Some((
                topic.type_name().to_owned(),
                topic.instance_handle().into(),
                topic.status_condition().address(),
            )))
        } else {
            Ok(None)
        }
    }
}

pub struct IgnoreParticipant {
    pub handle: InstanceHandle,
}
impl Mail for IgnoreParticipant {
    type Result = DdsResult<()>;
}
impl MailHandler<IgnoreParticipant> for DomainParticipantActor {
    fn handle(&mut self, message: IgnoreParticipant) -> <IgnoreParticipant as Mail>::Result {
        if self.domain_participant.enabled() {
            self.domain_participant.ignore_participant(message.handle);
            Ok(())
        } else {
            Err(DdsError::NotEnabled)
        }
    }
}

pub struct IgnoreSubscription {
    pub handle: InstanceHandle,
}
impl Mail for IgnoreSubscription {
    type Result = DdsResult<()>;
}
impl MailHandler<IgnoreSubscription> for DomainParticipantActor {
    fn handle(&mut self, message: IgnoreSubscription) -> <IgnoreSubscription as Mail>::Result {
        if self.domain_participant.enabled() {
            self.domain_participant.ignore_subscription(message.handle);
            Ok(())
        } else {
            Err(DdsError::NotEnabled)
        }
    }
}

pub struct IgnorePublication {
    pub handle: InstanceHandle,
}
impl Mail for IgnorePublication {
    type Result = DdsResult<()>;
}
impl MailHandler<IgnorePublication> for DomainParticipantActor {
    fn handle(&mut self, message: IgnorePublication) -> <IgnorePublication as Mail>::Result {
        if self.domain_participant.enabled() {
            self.domain_participant.ignore_publication(message.handle);
            Ok(())
        } else {
            Err(DdsError::NotEnabled)
        }
    }
}

pub struct DeleteParticipantContainedEntities;
impl Mail for DeleteParticipantContainedEntities {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteParticipantContainedEntities> for DomainParticipantActor {
    fn handle(
        &mut self,
        _: DeleteParticipantContainedEntities,
    ) -> <DeleteParticipantContainedEntities as Mail>::Result {
        let deleted_publisher_list: Vec<PublisherEntity> =
            self.domain_participant.drain_publisher_list().collect();
        for mut publisher in deleted_publisher_list {
            let publisher_qos = publisher.qos().clone();
            todo!();
            // for data_writer in publisher.drain_data_writer_list() {
            //     let publication_builtin_topic_data = PublicationBuiltinTopicData {
            //         key: BuiltInTopicKey {
            //             value: data_writer.transport_writer().guid(),
            //         },
            //         participant_key: BuiltInTopicKey { value: [0; 16] },
            //         topic_name: data_writer.topic_name().to_owned(),
            //         type_name: data_writer.type_name().to_owned(),
            //         durability: data_writer.qos().durability.clone(),
            //         deadline: data_writer.qos().deadline.clone(),
            //         latency_budget: data_writer.qos().latency_budget.clone(),
            //         liveliness: data_writer.qos().liveliness.clone(),
            //         reliability: data_writer.qos().reliability.clone(),
            //         lifespan: data_writer.qos().lifespan.clone(),
            //         user_data: data_writer.qos().user_data.clone(),
            //         ownership: data_writer.qos().ownership.clone(),
            //         ownership_strength: data_writer.qos().ownership_strength.clone(),
            //         destination_order: data_writer.qos().destination_order.clone(),
            //         presentation: publisher_qos.presentation.clone(),
            //         partition: publisher_qos.partition.clone(),
            //         topic_data: self.topic_list[data_writer.topic_name()]
            //             .qos()
            //             .topic_data
            //             .clone(),
            //         group_data: publisher_qos.group_data.clone(),
            //         representation: data_writer.qos().representation.clone(),
            //     };
            //     self.announce_deleted_data_writer(publication_builtin_topic_data)?;
            // }
        }

        let deleted_subscriber_list: Vec<SubscriberEntity> =
            self.domain_participant.drain_subscriber_list().collect();
        for mut subscriber in deleted_subscriber_list {
            let subscriber_qos = subscriber.qos().clone();
            for data_reader in subscriber.drain_data_reader_list() {
                todo!()
                // let subscription_builtin_topic_data = SubscriptionBuiltinTopicData {
                //     key: BuiltInTopicKey {
                //         value: data_reader.transport_reader().guid(),
                //     },
                //     participant_key: BuiltInTopicKey { value: [0; 16] },
                //     topic_name: data_reader.topic_name().to_owned(),
                //     type_name: data_reader.type_name().to_owned(),
                //     durability: data_reader.qos().durability.clone(),
                //     deadline: data_reader.qos().deadline.clone(),
                //     latency_budget: data_reader.qos().latency_budget.clone(),
                //     liveliness: data_reader.qos().liveliness.clone(),
                //     reliability: data_reader.qos().reliability.clone(),
                //     ownership: data_reader.qos().ownership.clone(),
                //     destination_order: data_reader.qos().destination_order.clone(),
                //     user_data: data_reader.qos().user_data.clone(),
                //     time_based_filter: data_reader.qos().time_based_filter.clone(),
                //     presentation: subscriber_qos.presentation.clone(),
                //     partition: subscriber_qos.partition.clone(),
                //     topic_data: self.topic_list[data_reader.topic_name()]
                //         .qos()
                //         .topic_data
                //         .clone(),
                //     group_data: subscriber_qos.group_data.clone(),
                //     representation: data_reader.qos().representation.clone(),
                // };

                // self.announce_deleted_data_reader(subscription_builtin_topic_data)?;
            }
        }

        self.domain_participant.delete_all_topics();

        Ok(())
    }
}

pub struct SetDefaultPublisherQos {
    pub qos: QosKind<PublisherQos>,
}
impl Mail for SetDefaultPublisherQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDefaultPublisherQos> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetDefaultPublisherQos,
    ) -> <SetDefaultPublisherQos as Mail>::Result {
        let qos = match message.qos {
            QosKind::Default => PublisherQos::default(),
            QosKind::Specific(q) => q,
        };

        self.domain_participant.set_default_publisher_qos(qos);
        Ok(())
    }
}

pub struct GetDefaultPublisherQos;
impl Mail for GetDefaultPublisherQos {
    type Result = DdsResult<PublisherQos>;
}
impl MailHandler<GetDefaultPublisherQos> for DomainParticipantActor {
    fn handle(&mut self, _: GetDefaultPublisherQos) -> <GetDefaultPublisherQos as Mail>::Result {
        Ok(self.domain_participant.default_publisher_qos().clone())
    }
}

pub struct SetDefaultSubscriberQos {
    pub qos: QosKind<SubscriberQos>,
}
impl Mail for SetDefaultSubscriberQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDefaultSubscriberQos> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetDefaultSubscriberQos,
    ) -> <SetDefaultSubscriberQos as Mail>::Result {
        let qos = match message.qos {
            QosKind::Default => SubscriberQos::default(),
            QosKind::Specific(q) => q,
        };

        self.domain_participant.set_default_subscriber_qos(qos);

        Ok(())
    }
}

pub struct GetDefaultSubscriberQos;
impl Mail for GetDefaultSubscriberQos {
    type Result = DdsResult<SubscriberQos>;
}
impl MailHandler<GetDefaultSubscriberQos> for DomainParticipantActor {
    fn handle(&mut self, _: GetDefaultSubscriberQos) -> <GetDefaultSubscriberQos as Mail>::Result {
        Ok(self.domain_participant.default_subscriber_qos().clone())
    }
}

pub struct SetDefaultTopicQos {
    pub qos: QosKind<TopicQos>,
}
impl Mail for SetDefaultTopicQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDefaultTopicQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetDefaultTopicQos) -> <SetDefaultTopicQos as Mail>::Result {
        let qos = match message.qos {
            QosKind::Default => TopicQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        self.domain_participant.set_default_topic_qos(qos);

        Ok(())
    }
}

pub struct GetDefaultTopicQos;
impl Mail for GetDefaultTopicQos {
    type Result = DdsResult<TopicQos>;
}
impl MailHandler<GetDefaultTopicQos> for DomainParticipantActor {
    fn handle(&mut self, _: GetDefaultTopicQos) -> <GetDefaultTopicQos as Mail>::Result {
        Ok(self.domain_participant.get_default_topic_qos().clone())
    }
}

pub struct GetDiscoveredParticipants;
impl Mail for GetDiscoveredParticipants {
    type Result = DdsResult<Vec<InstanceHandle>>;
}
impl MailHandler<GetDiscoveredParticipants> for DomainParticipantActor {
    fn handle(
        &mut self,
        _: GetDiscoveredParticipants,
    ) -> <GetDiscoveredParticipants as Mail>::Result {
        Ok(self.domain_participant.get_discovered_participants())
    }
}

pub struct GetDiscoveredParticipantData {
    pub participant_handle: InstanceHandle,
}
impl Mail for GetDiscoveredParticipantData {
    type Result = DdsResult<ParticipantBuiltinTopicData>;
}
impl MailHandler<GetDiscoveredParticipantData> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetDiscoveredParticipantData,
    ) -> <GetDiscoveredParticipantData as Mail>::Result {
        Ok(self
            .domain_participant
            .get_discovered_participant_data(&message.participant_handle)
            .ok_or(DdsError::BadParameter)?
            .dds_participant_data
            .clone())
    }
}

pub struct GetDiscoveredTopics;
impl Mail for GetDiscoveredTopics {
    type Result = DdsResult<Vec<InstanceHandle>>;
}
impl MailHandler<GetDiscoveredTopics> for DomainParticipantActor {
    fn handle(&mut self, _: GetDiscoveredTopics) -> <GetDiscoveredTopics as Mail>::Result {
        Ok(self.domain_participant.get_discovered_topics())
    }
}

pub struct GetDiscoveredTopicData {
    pub topic_handle: InstanceHandle,
}
impl Mail for GetDiscoveredTopicData {
    type Result = DdsResult<TopicBuiltinTopicData>;
}
impl MailHandler<GetDiscoveredTopicData> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetDiscoveredTopicData,
    ) -> <GetDiscoveredTopicData as Mail>::Result {
        self.domain_participant
            .get_discovered_topic_data(&message.topic_handle)
            .cloned()
            .ok_or(DdsError::PreconditionNotMet(
                "Topic with this handle not discovered".to_owned(),
            ))
    }
}

pub struct GetCurrentTime;
impl Mail for GetCurrentTime {
    type Result = Time;
}
impl MailHandler<GetCurrentTime> for DomainParticipantActor {
    fn handle(&mut self, _: GetCurrentTime) -> <GetCurrentTime as Mail>::Result {
        self.domain_participant.get_current_time()
    }
}

pub struct SetDomainParticipantQos {
    pub qos: QosKind<DomainParticipantQos>,
}
impl Mail for SetDomainParticipantQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDomainParticipantQos> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetDomainParticipantQos,
    ) -> <SetDomainParticipantQos as Mail>::Result {
        let qos = match message.qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.domain_participant.set_qos(qos);
        self.domain_participant.announce_participant()
    }
}

pub struct GetDomainParticipantQos;
impl Mail for GetDomainParticipantQos {
    type Result = DdsResult<DomainParticipantQos>;
}
impl MailHandler<GetDomainParticipantQos> for DomainParticipantActor {
    fn handle(&mut self, _: GetDomainParticipantQos) -> <GetDomainParticipantQos as Mail>::Result {
        Ok(self.domain_participant.qos().clone())
    }
}

pub struct SetDomainParticipantListener {
    pub listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
    pub status_kind: Vec<StatusKind>,
}
impl Mail for SetDomainParticipantListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDomainParticipantListener> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetDomainParticipantListener,
    ) -> <SetDomainParticipantListener as Mail>::Result {
        todo!()
        // if let Some(l) = self.participant_listener_thread.take() {
        //     l.join()?;
        // }
        // self.participant_listener_thread = message.listener.map(ParticipantListenerThread::new);
        // self.status_kind = message.status_kind;
        // Ok(())
    }
}

pub struct EnableDomainParticipant;
impl Mail for EnableDomainParticipant {
    type Result = DdsResult<()>;
}
impl MailHandler<EnableDomainParticipant> for DomainParticipantActor {
    fn handle(&mut self, _: EnableDomainParticipant) -> <EnableDomainParticipant as Mail>::Result {
        if !self.domain_participant.enabled() {
            self.domain_participant.enable();
            self.domain_participant.announce_participant()?;
        }
        Ok(())
    }
}

pub struct IsEmpty;
impl Mail for IsEmpty {
    type Result = bool;
}
impl MailHandler<IsEmpty> for DomainParticipantActor {
    fn handle(&mut self, _: IsEmpty) -> <IsEmpty as Mail>::Result {
        self.domain_participant.is_empty()
    }
}
