use crate::{
    dds_async::publisher_listener::PublisherListenerAsync,
    implementation::{
        domain_participant_backend::domain_participant_actor::DomainParticipantActor,
        listeners::publisher_listener::PublisherListenerActor,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        status::StatusKind,
    },
    runtime::{
        actor::{Actor, ActorAddress, MailHandler},
        oneshot::OneshotSender,
    },
};

use super::discovery_service;

pub struct DeleteDataWriter {
    pub publisher_handle: InstanceHandle,
    pub datawriter_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<DeleteDataWriter> for DomainParticipantActor {
    fn handle(&mut self, message: DeleteDataWriter) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        let Some(data_writer) = publisher.remove_data_writer(message.datawriter_handle) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        message
            .participant_address
            .send_actor_mail(discovery_service::AnnounceDeletedDataWriter { data_writer })
            .ok();
        message.reply_sender.send(Ok(()))
    }
}

pub struct SetDefaultDataWriterQos {
    pub publisher_handle: InstanceHandle,
    pub qos: QosKind<DataWriterQos>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetDefaultDataWriterQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetDefaultDataWriterQos) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        let qos = match message.qos {
            QosKind::Default => DataWriterQos::default(),
            QosKind::Specific(q) => q,
        };
        message
            .reply_sender
            .send(publisher.set_default_datawriter_qos(qos));
    }
}

pub struct GetDefaultDataWriterQos {
    pub publisher_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<DataWriterQos>>,
}

impl MailHandler<GetDefaultDataWriterQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetDefaultDataWriterQos) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        message
            .reply_sender
            .send(Ok(publisher.default_datawriter_qos().clone()));
    }
}

pub struct SetQos {
    pub publisher_handle: InstanceHandle,
    pub qos: QosKind<PublisherQos>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetQos) {
        let qos = match message.qos {
            QosKind::Default => self.domain_participant.default_publisher_qos().clone(),
            QosKind::Specific(q) => q,
        };
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message.reply_sender.send(publisher.set_qos(qos));
    }
}

pub struct GetQos {
    pub publisher_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<PublisherQos>>,
}
impl MailHandler<GetQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetQos) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message.reply_sender.send(Ok(publisher.qos().clone()));
    }
}

pub struct SetListener {
    pub publisher_handle: InstanceHandle,
    pub a_listener: Option<Box<dyn PublisherListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetListener> for DomainParticipantActor {
    fn handle(&mut self, message: SetListener) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        publisher.set_listener(
            message.a_listener.map(|l| {
                Actor::spawn(
                    PublisherListenerActor::new(l),
                    &self.listener_executor.handle(),
                )
            }),
            message.mask,
        );

        message.reply_sender.send(Ok(()));
    }
}
