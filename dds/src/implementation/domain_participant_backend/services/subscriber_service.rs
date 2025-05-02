use crate::{
    dds_async::subscriber_listener::SubscriberListenerAsync,
    implementation::{
        domain_participant_backend::domain_participant_actor::DomainParticipantActor,
        listeners::subscriber_listener::SubscriberListenerActor,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{QosKind, SubscriberQos},
        status::StatusKind,
    },
    runtime::{
        actor::{Actor, MailHandler},
        oneshot::OneshotSender,
    },
};

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
