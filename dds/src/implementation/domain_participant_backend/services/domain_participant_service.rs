use crate::{
    builtin_topics::{
        ParticipantBuiltinTopicData, TopicBuiltinTopicData, DCPS_PARTICIPANT, DCPS_PUBLICATION,
        DCPS_SUBSCRIPTION, DCPS_TOPIC,
    },
    dds_async::{
        domain_participant_listener::DomainParticipantListenerAsync,
        subscriber_listener::SubscriberListenerAsync,
    },
    implementation::{
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            entities::subscriber::SubscriberEntity,
        },
        listeners::{
            domain_participant_listener::DomainParticipantListenerActor,
            subscriber_listener::SubscriberListenerActor,
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
};

use super::discovery_service;

pub const BUILT_IN_TOPIC_NAME_LIST: [&str; 4] = [
    DCPS_PARTICIPANT,
    DCPS_TOPIC,
    DCPS_PUBLICATION,
    DCPS_SUBSCRIPTION,
];

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
