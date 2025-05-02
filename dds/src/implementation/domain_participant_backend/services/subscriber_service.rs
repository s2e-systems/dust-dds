use crate::{
    dds_async::subscriber_listener::SubscriberListenerAsync,
    implementation::{
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            entities::data_reader::DataReaderEntity, services::discovery_service,
        },
        listeners::subscriber_listener::SubscriberListenerActor,
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos},
        status::StatusKind,
    },
    runtime::{
        actor::{Actor, ActorAddress, MailHandler},
        oneshot::OneshotSender,
    },
};

pub struct DeleteDataReader {
    pub subscriber_handle: InstanceHandle,
    pub datareader_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<DeleteDataReader> for DomainParticipantActor {
    fn handle(&mut self, message: DeleteDataReader) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_reader) = subscriber.remove_data_reader(message.datareader_handle) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        message
            .participant_address
            .send_actor_mail(discovery_service::AnnounceDeletedDataReader { data_reader })
            .ok();
        message.reply_sender.send(Ok(()));
    }
}

pub struct LookupDataReader {
    pub subscriber_handle: InstanceHandle,
    pub topic_name: String,
    #[allow(clippy::type_complexity)]
    pub reply_sender:
        OneshotSender<DdsResult<Option<(InstanceHandle, ActorAddress<StatusConditionActor>)>>>,
}
impl MailHandler<LookupDataReader> for DomainParticipantActor {
    fn handle(&mut self, message: LookupDataReader) {
        if self
            .domain_participant
            .get_topic(&message.topic_name)
            .is_none()
        {
            message.reply_sender.send(Err(DdsError::BadParameter));
            return;
        }

        // Built-in subscriber is identified by the handle of the participant itself
        if self.domain_participant.instance_handle() == message.subscriber_handle {
            message.reply_sender.send(Ok(self
                .domain_participant
                .builtin_subscriber_mut()
                .data_reader_list_mut()
                .find(|dr| dr.topic_name() == message.topic_name)
                .map(|x: &mut DataReaderEntity| {
                    (x.instance_handle(), x.status_condition().address())
                })))
        } else {
            let Some(s) = self
                .domain_participant
                .get_mut_subscriber(message.subscriber_handle)
            else {
                message.reply_sender.send(Err(DdsError::AlreadyDeleted));
                return;
            };
            message.reply_sender.send(Ok(s
                .data_reader_list_mut()
                .find(|dr| dr.topic_name() == message.topic_name)
                .map(|x| (x.instance_handle(), x.status_condition().address()))))
        }
    }
}

pub struct SetDefaultDataReaderQos {
    pub subscriber_handle: InstanceHandle,
    pub qos: QosKind<DataReaderQos>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetDefaultDataReaderQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetDefaultDataReaderQos) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let qos = match message.qos {
            QosKind::Default => DataReaderQos::default(),
            QosKind::Specific(q) => q,
        };
        message
            .reply_sender
            .send(subscriber.set_default_data_reader_qos(qos));
    }
}

pub struct GetDefaultDataReaderQos {
    pub subscriber_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<DataReaderQos>>,
}
impl MailHandler<GetDefaultDataReaderQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetDefaultDataReaderQos) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message
            .reply_sender
            .send(Ok(subscriber.default_data_reader_qos().clone()));
    }
}

pub struct SetQos {
    pub subscriber_handle: InstanceHandle,
    pub qos: QosKind<SubscriberQos>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetQos) {
        let qos = match message.qos {
            QosKind::Default => self.domain_participant.default_subscriber_qos().clone(),
            QosKind::Specific(q) => q,
        };

        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message.reply_sender.send(subscriber.set_qos(qos));
    }
}

pub struct GetSubscriberQos {
    pub subscriber_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<SubscriberQos>>,
}
impl MailHandler<GetSubscriberQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetSubscriberQos) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message.reply_sender.send(Ok(subscriber.qos().clone()));
    }
}

pub struct SetListener {
    pub subscriber_handle: InstanceHandle,
    pub a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetListener> for DomainParticipantActor {
    fn handle(&mut self, message: SetListener) {
        let listener = message.a_listener.map(|l| {
            Actor::spawn(
                SubscriberListenerActor::new(l),
                &self.listener_executor.handle(),
            )
        });
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        subscriber.set_listener(listener, message.mask);
        message.reply_sender.send(Ok(()))
    }
}
