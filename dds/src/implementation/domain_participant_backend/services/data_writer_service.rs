use core::{future::Future, pin::Pin};

use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    implementation::{
        any_data_writer_listener::AnyDataWriterListener,
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            services::message_service::AreAllChangesAcknowledged,
        },
        listeners::data_writer_listener::DataWriterListenerActor,
        status_condition::status_condition_actor,
        xtypes_glue::key_and_instance_handle::get_serialized_key_from_serialized_foo,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::DataWriterQos,
        status::{OfferedDeadlineMissedStatus, PublicationMatchedStatus, StatusKind},
        time::{Duration, Time},
    },
    runtime::{
        actor::{Actor, ActorAddress, MailHandler},
        oneshot::{oneshot, OneshotSender},
    },
};

pub struct DisposeWTimestamp {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub serialized_data: Vec<u8>,
    pub timestamp: Time,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<DisposeWTimestamp> for DomainParticipantActor {
    fn handle(&mut self, message: DisposeWTimestamp) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == message.data_writer_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let serialized_key = match get_serialized_key_from_serialized_foo(
            &message.serialized_data,
            data_writer.type_support(),
        ) {
            Ok(k) => k,
            Err(e) => {
                message.reply_sender.send(Err(e.into()));
                return;
            }
        };
        message
            .reply_sender
            .send(data_writer.dispose_w_timestamp(serialized_key, message.timestamp));
    }
}

pub struct WaitForAcknowledgments {
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub timeout: Duration,
    pub reply_sender: OneshotSender<Pin<Box<dyn Future<Output = DdsResult<()>> + Send>>>,
}
impl MailHandler<WaitForAcknowledgments> for DomainParticipantActor {
    fn handle(&mut self, message: WaitForAcknowledgments) {
        let timer_handle = self.timer_driver.handle();
        message.reply_sender.send(Box::pin(async move {
            timer_handle
                .timeout(
                    message.timeout.into(),
                    Box::pin(async move {
                        loop {
                            let (reply_sender, reply_receiver) = oneshot();
                            message
                                .participant_address
                                .send_actor_mail(AreAllChangesAcknowledged {
                                    publisher_handle: message.publisher_handle,
                                    data_writer_handle: message.data_writer_handle,
                                    reply_sender,
                                })
                                .ok();
                            let reply = reply_receiver.await;
                            match reply {
                                Ok(are_changes_acknowledged) => match are_changes_acknowledged {
                                    Ok(true) => return Ok(()),
                                    Ok(false) => (),
                                    Err(e) => return Err(e),
                                },
                                Err(e) => {
                                    return Err(DdsError::Error(format!("Channel error: {:?}", e)))
                                }
                            }
                        }
                    }),
                )
                .await
                .map_err(|_| DdsError::Timeout)?
        }))
    }
}

pub struct GetOfferedDeadlineMissedStatus {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<OfferedDeadlineMissedStatus>>,
}
impl MailHandler<GetOfferedDeadlineMissedStatus> for DomainParticipantActor {
    fn handle(&mut self, message: GetOfferedDeadlineMissedStatus) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == message.data_writer_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message
            .reply_sender
            .send(Ok(data_writer.get_offered_deadline_missed_status()))
    }
}

pub struct GetPublicationMatchedStatus {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<PublicationMatchedStatus>>,
}
impl MailHandler<GetPublicationMatchedStatus> for DomainParticipantActor {
    fn handle(&mut self, message: GetPublicationMatchedStatus) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == message.data_writer_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        let status = data_writer.get_publication_matched_status();

        data_writer.status_condition().send_actor_mail(
            status_condition_actor::RemoveCommunicationState {
                state: StatusKind::PublicationMatched,
            },
        );
        message.reply_sender.send(Ok(status));
    }
}

pub struct GetMatchedSubscriptionData {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub subscription_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<SubscriptionBuiltinTopicData>>,
}
impl MailHandler<GetMatchedSubscriptionData> for DomainParticipantActor {
    fn handle(&mut self, message: GetMatchedSubscriptionData) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == message.data_writer_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message.reply_sender.send(
            data_writer
                .get_matched_subscription_data(&message.subscription_handle)
                .ok_or(DdsError::BadParameter)
                .cloned(),
        );
    }
}

pub struct GetMatchedSubscriptions {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<Vec<InstanceHandle>>>,
}
impl MailHandler<GetMatchedSubscriptions> for DomainParticipantActor {
    fn handle(&mut self, message: GetMatchedSubscriptions) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == message.data_writer_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message
            .reply_sender
            .send(Ok(data_writer.get_matched_subscriptions()))
    }
}

pub struct GetDataWriterQos {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<DataWriterQos>>,
}
impl MailHandler<GetDataWriterQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetDataWriterQos) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == message.data_writer_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message.reply_sender.send(Ok(data_writer.qos().clone()));
    }
}

pub struct SetListener {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub listener: Option<Box<dyn AnyDataWriterListener + Send>>,
    pub listener_mask: Vec<StatusKind>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetListener> for DomainParticipantActor {
    fn handle(&mut self, message: SetListener) {
        let listener = message.listener.map(|l| {
            Actor::spawn(
                DataWriterListenerActor::new(l),
                &self.listener_executor.handle(),
            )
        });
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            return;
        };
        let Some(data_writer) = publisher.get_mut_data_writer(message.data_writer_handle) else {
            return;
        };

        data_writer.set_listener(listener, message.listener_mask);

        message.reply_sender.send(Ok(()));
    }
}
