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
        xtypes_glue::key_and_instance_handle::{
            get_instance_handle_from_serialized_foo, get_serialized_key_from_serialized_foo,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, QosKind},
        status::{OfferedDeadlineMissedStatus, PublicationMatchedStatus, StatusKind},
        time::{Duration, DurationKind, Time},
    },
    runtime::{
        actor::{Actor, ActorAddress, MailHandler},
        oneshot::{oneshot, OneshotSender},
    },
};

use super::{discovery_service, event_service, message_service};

pub struct UnregisterInstance {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub serialized_data: Vec<u8>,
    pub timestamp: Time,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<UnregisterInstance> for DomainParticipantActor {
    fn handle(&mut self, message: UnregisterInstance) {
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
            .send(data_writer.unregister_w_timestamp(serialized_key, message.timestamp));
    }
}

pub struct LookupInstance {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub serialized_data: Vec<u8>,
    pub reply_sender: OneshotSender<DdsResult<Option<InstanceHandle>>>,
}
impl MailHandler<LookupInstance> for DomainParticipantActor {
    fn handle(&mut self, message: LookupInstance) {
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

        if !data_writer.enabled() {
            message.reply_sender.send(Err(DdsError::NotEnabled));
            return;
        }

        let instance_handle = match get_instance_handle_from_serialized_foo(
            &message.serialized_data,
            data_writer.type_support(),
        ) {
            Ok(k) => k,
            Err(e) => {
                message.reply_sender.send(Err(e.into()));
                return;
            }
        };

        message.reply_sender.send(Ok(data_writer
            .contains_instance(&instance_handle)
            .then_some(instance_handle)));
    }
}

pub struct WriteWTimestamp {
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub serialized_data: Vec<u8>,
    pub timestamp: Time,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<WriteWTimestamp> for DomainParticipantActor {
    fn handle(&mut self, message: WriteWTimestamp) {
        let now = self.domain_participant.get_current_time();
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
        let instance_handle = match get_instance_handle_from_serialized_foo(
            &message.serialized_data,
            data_writer.type_support(),
        ) {
            Ok(k) => k,
            Err(e) => {
                message.reply_sender.send(Err(e.into()));
                return;
            }
        };

        match data_writer.qos().lifespan.duration {
            DurationKind::Finite(lifespan_duration) => {
                let timer_handle = self.timer_driver.handle();
                let sleep_duration = message.timestamp - now + lifespan_duration;
                if sleep_duration > Duration::new(0, 0) {
                    let sequence_number = match data_writer
                        .write_w_timestamp(message.serialized_data, message.timestamp)
                    {
                        Ok(s) => s,
                        Err(e) => {
                            message.reply_sender.send(Err(e));
                            return;
                        }
                    };

                    let participant_address = message.participant_address.clone();
                    self.backend_executor.handle().spawn(async move {
                        timer_handle.sleep(sleep_duration.into()).await;
                        participant_address
                            .send_actor_mail(message_service::RemoveWriterChange {
                                publisher_handle: message.publisher_handle,
                                data_writer_handle: message.data_writer_handle,
                                sequence_number,
                            })
                            .ok();
                    });
                }
            }
            DurationKind::Infinite => {
                match data_writer.write_w_timestamp(message.serialized_data, message.timestamp) {
                    Ok(_) => (),
                    Err(e) => {
                        message.reply_sender.send(Err(e));
                        return;
                    }
                };
            }
        }

        if let DurationKind::Finite(deadline_missed_period) = data_writer.qos().deadline.period {
            let timer_handle = self.timer_driver.handle();
            let offered_deadline_missed_task = self.backend_executor.handle().spawn(async move {
                loop {
                    timer_handle.sleep(deadline_missed_period.into()).await;
                    message
                        .participant_address
                        .send_actor_mail(event_service::OfferedDeadlineMissed {
                            publisher_handle: message.publisher_handle,
                            data_writer_handle: message.data_writer_handle,
                            change_instance_handle: instance_handle,
                            participant_address: message.participant_address.clone(),
                        })
                        .ok();
                }
            });
            data_writer.insert_instance_deadline_missed_task(
                instance_handle,
                offered_deadline_missed_task,
            );
        }

        message.reply_sender.send(Ok(()));
    }
}

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

pub struct SetDataWriterQos {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub qos: QosKind<DataWriterQos>,
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetDataWriterQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetDataWriterQos) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let qos = match message.qos {
            QosKind::Default => publisher.default_datawriter_qos().clone(),
            QosKind::Specific(q) => q,
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == message.data_writer_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        match data_writer.set_qos(qos) {
            Ok(_) => (),
            Err(e) => {
                message.reply_sender.send(Err(e));
                return;
            }
        }
        if data_writer.enabled() {
            message
                .participant_address
                .send_actor_mail(discovery_service::AnnounceDataWriter {
                    publisher_handle: message.publisher_handle,
                    data_writer_handle: message.data_writer_handle,
                })
                .ok();
        }

        message.reply_sender.send(Ok(()));
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

pub struct Enable {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl MailHandler<Enable> for DomainParticipantActor {
    fn handle(&mut self, message: Enable) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            return;
        };
        let Some(data_writer) = publisher.get_mut_data_writer(message.data_writer_handle) else {
            return;
        };
        if !data_writer.enabled() {
            data_writer.enable();

            for discovered_reader_data in self
                .domain_participant
                .discovered_reader_data_list()
                .cloned()
            {
                message
                    .participant_address
                    .send_actor_mail(discovery_service::AddDiscoveredReader {
                        discovered_reader_data,
                        publisher_handle: message.publisher_handle,
                        data_writer_handle: message.data_writer_handle,
                        participant_address: message.participant_address.clone(),
                    })
                    .ok();
            }

            message
                .participant_address
                .send_actor_mail(discovery_service::AnnounceDataWriter {
                    publisher_handle: message.publisher_handle,
                    data_writer_handle: message.data_writer_handle,
                })
                .ok();
        }
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
