use std::sync::Arc;

use crate::{
    builtin_topics::{
        ParticipantBuiltinTopicData, TopicBuiltinTopicData, DCPS_PARTICIPANT, DCPS_PUBLICATION,
        DCPS_SUBSCRIPTION, DCPS_TOPIC,
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
        time::Time,
    },
    runtime::{
        actor::{Actor, ActorAddress, MailHandler},
        oneshot::OneshotSender,
    },
    xtypes::dynamic_type::DynamicType,
};

use super::{discovery_service, topic_service};

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
    pub reply_sender:
        OneshotSender<DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>>,
}
impl MailHandler<CreateUserDefinedPublisher> for DomainParticipantActor {
    fn handle(&mut self, message: CreateUserDefinedPublisher) {
        let publisher_qos = match message.qos {
            QosKind::Default => self.domain_participant.default_publisher_qos().clone(),
            QosKind::Specific(q) => q,
        };

        let publisher_handle = self.instance_handle_counter.generate_new_instance_handle();
        let status_condition = Actor::spawn(
            StatusConditionActor::default(),
            &self.listener_executor.handle(),
        );
        let publisher_status_condition_address = status_condition.address();
        let listener = message.a_listener.map(|l| {
            Actor::spawn(
                PublisherListenerActor::new(l),
                &self.listener_executor.handle(),
            )
        });
        let mut publisher = PublisherEntity::new(
            publisher_qos,
            publisher_handle,
            listener,
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

        message
            .reply_sender
            .send(Ok((publisher_handle, publisher_status_condition_address)));
    }
}

pub struct DeleteUserDefinedPublisher {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<DeleteUserDefinedPublisher> for DomainParticipantActor {
    fn handle(&mut self, message: DeleteUserDefinedPublisher) {
        if message.participant_handle != self.domain_participant.instance_handle() {
            message.reply_sender.send(Err(DdsError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant".to_string(),
            )));
            return;
        }
        let Some(publisher) = self
            .domain_participant
            .get_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        if publisher.data_writer_list().count() > 0 {
            message.reply_sender.send(Err(DdsError::PreconditionNotMet(
                "Publisher still contains data writers".to_string(),
            )));
            return;
        }
        let Some(_) = self
            .domain_participant
            .remove_publisher(&message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message.reply_sender.send(Ok(()));
    }
}

pub struct CreateUserDefinedSubscriber {
    pub qos: QosKind<SubscriberQos>,
    pub a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
    pub reply_sender:
        OneshotSender<DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>>,
}
impl MailHandler<CreateUserDefinedSubscriber> for DomainParticipantActor {
    fn handle(&mut self, message: CreateUserDefinedSubscriber) {
        let subscriber_qos = match message.qos {
            QosKind::Default => self.domain_participant.default_subscriber_qos().clone(),
            QosKind::Specific(q) => q,
        };
        let subscriber_handle = self.instance_handle_counter.generate_new_instance_handle();
        let listener = message.a_listener.map(|l| {
            Actor::spawn(
                SubscriberListenerActor::new(l),
                &self.listener_executor.handle(),
            )
        });
        let listener_mask = message.mask.to_vec();

        let mut subscriber = SubscriberEntity::new(
            subscriber_handle,
            subscriber_qos,
            Actor::spawn(
                StatusConditionActor::default(),
                &self.listener_executor.handle(),
            ),
            listener,
            listener_mask,
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

        message
            .reply_sender
            .send(Ok((subscriber_handle, subscriber_status_condition_address)))
    }
}

pub struct DeleteUserDefinedSubscriber {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<DeleteUserDefinedSubscriber> for DomainParticipantActor {
    fn handle(&mut self, message: DeleteUserDefinedSubscriber) {
        if self.domain_participant.instance_handle() != message.participant_handle {
            message.reply_sender.send(Err(DdsError::PreconditionNotMet(
                "Subscriber can only be deleted from its parent participant".to_string(),
            )));
            return;
        }

        let Some(subscriber) = self
            .domain_participant
            .get_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        if subscriber.data_reader_list().count() > 0 {
            message.reply_sender.send(Err(DdsError::PreconditionNotMet(
                "Subscriber still contains data readers".to_string(),
            )));
            return;
        }
        let Some(_) = self
            .domain_participant
            .remove_subscriber(&message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message.reply_sender.send(Ok(()))
    }
}

pub struct CreateTopic {
    pub topic_name: String,
    pub type_name: String,
    pub qos: QosKind<TopicQos>,
    pub a_listener: Option<Box<dyn TopicListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
    pub type_support: Arc<dyn DynamicType + Send + Sync>,
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub reply_sender:
        OneshotSender<DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>>,
}
impl MailHandler<CreateTopic> for DomainParticipantActor {
    fn handle(&mut self, message: CreateTopic) {
        if self
            .domain_participant
            .get_topic(&message.topic_name)
            .is_some()
        {
            message
                .reply_sender
                .send(Err(DdsError::PreconditionNotMet(format!(
                    "Topic with name {} already exists.
             To access this topic call the lookup_topicdescription method.",
                    message.topic_name
                ))));
            return;
        }

        let qos = match message.qos {
            QosKind::Default => self.domain_participant.get_default_topic_qos().clone(),
            QosKind::Specific(q) => q,
        };

        let topic_handle = self.instance_handle_counter.generate_new_instance_handle();
        let status_condition = Actor::spawn(
            StatusConditionActor::default(),
            &self.listener_executor.handle(),
        );
        let topic_status_condition_address = status_condition.address();
        let topic_listener = message
            .a_listener
            .map(|l| Actor::spawn(TopicListenerActor::new(l), &self.listener_executor.handle()));
        let topic = TopicEntity::new(
            qos,
            message.type_name,
            message.topic_name.clone(),
            topic_handle,
            status_condition,
            topic_listener,
            message.mask,
            message.type_support,
        );

        self.domain_participant.insert_topic(topic);

        if self.domain_participant.enabled()
            && self
                .domain_participant
                .qos()
                .entity_factory
                .autoenable_created_entities
        {
            message
                .participant_address
                .send_actor_mail(topic_service::Enable {
                    topic_name: message.topic_name.clone(),
                    participant_address: message.participant_address.clone(),
                })
                .ok();
        }

        message
            .reply_sender
            .send(Ok((topic_handle, topic_status_condition_address)))
    }
}

pub struct DeleteUserDefinedTopic {
    pub participant_handle: InstanceHandle,
    pub topic_name: String,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<DeleteUserDefinedTopic> for DomainParticipantActor {
    fn handle(&mut self, message: DeleteUserDefinedTopic) {
        if self.domain_participant.instance_handle() != message.participant_handle {
            message.reply_sender.send(Err(DdsError::PreconditionNotMet(
                "Topic can only be deleted from its parent participant".to_string(),
            )));
            return;
        }

        if BUILT_IN_TOPIC_NAME_LIST.contains(&message.topic_name.as_str()) {
            message.reply_sender.send(Ok(()));
            return;
        }

        let Some(topic) = self.domain_participant.get_topic(&message.topic_name) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        if Arc::strong_count(topic.type_support()) > 1 {
            message.reply_sender.send(Err(DdsError::PreconditionNotMet(
                "Topic still attached to some data writer or data reader".to_string(),
            )));
            return;
        }

        let Some(_) = self.domain_participant.remove_topic(&message.topic_name) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message.reply_sender.send(Ok(()));
    }
}

pub struct FindTopic {
    pub topic_name: String,
    pub type_support: Arc<dyn DynamicType + Send + Sync>,
    pub reply_sender: OneshotSender<
        DdsResult<Option<(InstanceHandle, ActorAddress<StatusConditionActor>, String)>>,
    >,
}
impl MailHandler<FindTopic> for DomainParticipantActor {
    fn handle(&mut self, message: FindTopic) {
        if let Some(topic) = self.domain_participant.get_topic(&message.topic_name) {
            message.reply_sender.send(Ok(Some((
                topic.instance_handle(),
                topic.status_condition().address(),
                topic.type_name().to_owned(),
            ))));
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
                    Actor::spawn(
                        StatusConditionActor::default(),
                        &self.listener_executor.handle(),
                    ),
                    None,
                    vec![],
                    message.type_support,
                );
                topic.enable();
                let topic_status_condition_address = topic.status_condition().address();

                self.domain_participant.insert_topic(topic);
                message.reply_sender.send(Ok(Some((
                    topic_handle,
                    topic_status_condition_address,
                    type_name,
                ))));
                return;
            }
            message.reply_sender.send(Ok(None));
        }
    }
}

pub struct LookupTopicdescription {
    pub topic_name: String,
    pub reply_sender: OneshotSender<
        DdsResult<Option<(String, InstanceHandle, ActorAddress<StatusConditionActor>)>>,
    >,
}
impl MailHandler<LookupTopicdescription> for DomainParticipantActor {
    fn handle(&mut self, message: LookupTopicdescription) {
        if let Some(topic) = self.domain_participant.get_topic(&message.topic_name) {
            message.reply_sender.send(Ok(Some((
                topic.type_name().to_owned(),
                topic.instance_handle(),
                topic.status_condition().address(),
            ))))
        } else {
            message.reply_sender.send(Ok(None))
        }
    }
}

pub struct IgnoreParticipant {
    pub handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<IgnoreParticipant> for DomainParticipantActor {
    fn handle(&mut self, message: IgnoreParticipant) {
        if self.domain_participant.enabled() {
            self.domain_participant.ignore_participant(message.handle);
            message.reply_sender.send(Ok(()))
        } else {
            message.reply_sender.send(Err(DdsError::NotEnabled))
        }
    }
}

pub struct IgnoreSubscription {
    pub handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<IgnoreSubscription> for DomainParticipantActor {
    fn handle(&mut self, message: IgnoreSubscription) {
        if self.domain_participant.enabled() {
            self.domain_participant.ignore_subscription(message.handle);
            message.reply_sender.send(Ok(()))
        } else {
            message.reply_sender.send(Err(DdsError::NotEnabled))
        }
    }
}

pub struct IgnorePublication {
    pub handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<IgnorePublication> for DomainParticipantActor {
    fn handle(&mut self, message: IgnorePublication) {
        if self.domain_participant.enabled() {
            self.domain_participant.ignore_publication(message.handle);
            message.reply_sender.send(Ok(()))
        } else {
            message.reply_sender.send(Err(DdsError::NotEnabled))
        }
    }
}

pub struct DeleteContainedEntities {
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<DeleteContainedEntities> for DomainParticipantActor {
    fn handle(&mut self, message: DeleteContainedEntities) {
        let deleted_publisher_list: Vec<PublisherEntity> =
            self.domain_participant.drain_publisher_list().collect();
        for mut publisher in deleted_publisher_list {
            for data_writer in publisher.drain_data_writer_list() {
                message
                    .participant_address
                    .send_actor_mail(discovery_service::AnnounceDeletedDataWriter { data_writer })
                    .ok();
            }
        }

        let deleted_subscriber_list: Vec<SubscriberEntity> =
            self.domain_participant.drain_subscriber_list().collect();
        for mut subscriber in deleted_subscriber_list {
            for data_reader in subscriber.drain_data_reader_list() {
                message
                    .participant_address
                    .send_actor_mail(discovery_service::AnnounceDeletedDataReader { data_reader })
                    .ok();
            }
        }

        self.domain_participant.delete_all_topics();

        message.reply_sender.send(Ok(()))
    }
}

pub struct SetDefaultPublisherQos {
    pub qos: QosKind<PublisherQos>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetDefaultPublisherQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetDefaultPublisherQos) {
        let qos = match message.qos {
            QosKind::Default => PublisherQos::default(),
            QosKind::Specific(q) => q,
        };

        self.domain_participant.set_default_publisher_qos(qos);
        message.reply_sender.send(Ok(()))
    }
}

pub struct GetDefaultPublisherQos {
    pub reply_sender: OneshotSender<DdsResult<PublisherQos>>,
}
impl MailHandler<GetDefaultPublisherQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetDefaultPublisherQos) {
        message
            .reply_sender
            .send(Ok(self.domain_participant.default_publisher_qos().clone()))
    }
}

pub struct SetDefaultSubscriberQos {
    pub qos: QosKind<SubscriberQos>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetDefaultSubscriberQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetDefaultSubscriberQos) {
        let qos = match message.qos {
            QosKind::Default => SubscriberQos::default(),
            QosKind::Specific(q) => q,
        };

        self.domain_participant.set_default_subscriber_qos(qos);

        message.reply_sender.send(Ok(()))
    }
}

pub struct GetDefaultSubscriberQos {
    pub reply_sender: OneshotSender<DdsResult<SubscriberQos>>,
}
impl MailHandler<GetDefaultSubscriberQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetDefaultSubscriberQos) {
        message
            .reply_sender
            .send(Ok(self.domain_participant.default_subscriber_qos().clone()))
    }
}

pub struct SetDefaultTopicQos {
    pub qos: QosKind<TopicQos>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetDefaultTopicQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetDefaultTopicQos) {
        let qos = match message.qos {
            QosKind::Default => TopicQos::default(),
            QosKind::Specific(q) => {
                if q.is_consistent().is_ok() {
                    q
                } else {
                    message.reply_sender.send(Err(DdsError::InconsistentPolicy));
                    return;
                }
            }
        };

        message
            .reply_sender
            .send(self.domain_participant.set_default_topic_qos(qos))
    }
}

pub struct GetDefaultTopicQos {
    pub reply_sender: OneshotSender<DdsResult<TopicQos>>,
}

impl MailHandler<GetDefaultTopicQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetDefaultTopicQos) {
        message
            .reply_sender
            .send(Ok(self.domain_participant.get_default_topic_qos().clone()))
    }
}

pub struct GetDiscoveredParticipants {
    pub reply_sender: OneshotSender<DdsResult<Vec<InstanceHandle>>>,
}
impl MailHandler<GetDiscoveredParticipants> for DomainParticipantActor {
    fn handle(&mut self, message: GetDiscoveredParticipants) {
        message
            .reply_sender
            .send(Ok(self.domain_participant.get_discovered_participants()))
    }
}

pub struct GetDiscoveredParticipantData {
    pub participant_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<ParticipantBuiltinTopicData>>,
}
impl MailHandler<GetDiscoveredParticipantData> for DomainParticipantActor {
    fn handle(&mut self, message: GetDiscoveredParticipantData) {
        let Some(handle) = self
            .domain_participant
            .get_discovered_participant_data(&message.participant_handle)
        else {
            message.reply_sender.send(Err(DdsError::BadParameter));
            return;
        };
        message
            .reply_sender
            .send(Ok(handle.dds_participant_data.clone()));
    }
}

pub struct GetDiscoveredTopics {
    pub reply_sender: OneshotSender<DdsResult<Vec<InstanceHandle>>>,
}
impl MailHandler<GetDiscoveredTopics> for DomainParticipantActor {
    fn handle(&mut self, message: GetDiscoveredTopics) {
        message
            .reply_sender
            .send(Ok(self.domain_participant.get_discovered_topics()))
    }
}

pub struct GetDiscoveredTopicData {
    pub topic_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<TopicBuiltinTopicData>>,
}
impl MailHandler<GetDiscoveredTopicData> for DomainParticipantActor {
    fn handle(&mut self, message: GetDiscoveredTopicData) {
        let Some(handle) = self
            .domain_participant
            .get_discovered_topic_data(&message.topic_handle)
        else {
            message.reply_sender.send(Err(DdsError::PreconditionNotMet(
                "Topic with this handle not discovered".to_owned(),
            )));
            return;
        };

        message.reply_sender.send(Ok(handle.clone()))
    }
}

pub struct GetCurrentTime {
    pub reply_sender: OneshotSender<Time>,
}
impl MailHandler<GetCurrentTime> for DomainParticipantActor {
    fn handle(&mut self, message: GetCurrentTime) {
        message
            .reply_sender
            .send(self.domain_participant.get_current_time())
    }
}

pub struct SetDomainParticipantQos {
    pub qos: QosKind<DomainParticipantQos>,
    pub domain_participant_address: ActorAddress<DomainParticipantActor>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}

impl MailHandler<SetDomainParticipantQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetDomainParticipantQos) {
        let qos = match message.qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.domain_participant.set_qos(qos);
        if self.domain_participant.enabled() {
            message
                .domain_participant_address
                .send_actor_mail(discovery_service::AnnounceParticipant)
                .ok();
        }
        message.reply_sender.send(Ok(()))
    }
}

pub struct GetDomainParticipantQos {
    pub reply_sender: OneshotSender<DdsResult<DomainParticipantQos>>,
}
impl MailHandler<GetDomainParticipantQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetDomainParticipantQos) {
        message
            .reply_sender
            .send(Ok(self.domain_participant.qos().clone()));
    }
}

pub struct SetListener {
    pub listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
    pub status_kind: Vec<StatusKind>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetListener> for DomainParticipantActor {
    fn handle(&mut self, message: SetListener) {
        let participant_listener = message.listener.map(|l| {
            Actor::spawn(
                DomainParticipantListenerActor::new(l),
                &self.listener_executor.handle(),
            )
        });
        self.domain_participant
            .set_listener(participant_listener, message.status_kind);
        message.reply_sender.send(Ok(()))
    }
}

pub struct Enable {
    pub domain_participant_address: ActorAddress<DomainParticipantActor>,
}
impl MailHandler<Enable> for DomainParticipantActor {
    fn handle(&mut self, message: Enable) {
        if !self.domain_participant.enabled() {
            self.domain_participant.enable();

            message
                .domain_participant_address
                .send_actor_mail(discovery_service::AnnounceParticipant)
                .ok();
        }
    }
}

pub struct IsEmpty {
    pub reply_sender: OneshotSender<bool>,
}
impl MailHandler<IsEmpty> for DomainParticipantActor {
    fn handle(&mut self, message: IsEmpty) {
        message
            .reply_sender
            .send(self.domain_participant.is_empty());
    }
}
