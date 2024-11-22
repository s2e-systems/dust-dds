use core::{future::Future, pin::Pin};

use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    implementation::{
        any_data_writer_listener::AnyDataWriterListener,
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            services::message_service::AreAllChangesAcknowledged,
        },
        xtypes_glue::key_and_instance_handle::{
            get_instance_handle_from_serialized_foo, get_serialized_key_from_serialized_foo,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, QosKind},
        qos_policy::ReliabilityQosPolicyKind,
        status::{OfferedDeadlineMissedStatus, PublicationMatchedStatus, StatusKind},
        time::{Duration, DurationKind, Time},
    },
    runtime::actor::{ActorAddress, Mail, MailHandler},
};

use super::{discovery_service, event_service, message_service};

pub struct RegisterInstance {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub serialized_data: Vec<u8>,
    pub timestamp: Time,
}
impl Mail for RegisterInstance {
    type Result = DdsResult<Option<InstanceHandle>>;
}
impl MailHandler<RegisterInstance> for DomainParticipantActor {
    fn handle(&mut self, message: RegisterInstance) -> <RegisterInstance as Mail>::Result {
        todo!()
        // if !self
        //     .writer_address
        //     .send_actor_mail(data_writer_actor::IsEnabled)?
        //     .receive_reply()
        //     .await
        // {
        //     return Err(DdsError::NotEnabled);
        // }

        // let type_support = self
        //     .participant_address()
        //     .send_actor_mail(domain_participant_actor::GetTopicTypeSupport {
        //         topic_name: self.topic.get_name(),
        //     })?
        //     .receive_reply()
        //     .await?;

        // let serialized_data = instance.serialize_data()?;
        // let instance_handle =
        //     get_instance_handle_from_serialized_foo(&serialized_data, type_support.as_ref())?;

        // self.writer_address
        //     .send_actor_mail(data_writer_actor::RegisterInstanceWTimestamp { instance_handle })?
        //     .receive_reply()
        //     .await
    }
}

pub struct UnregisterInstance {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub serialized_data: Vec<u8>,
    pub timestamp: Time,
}
impl Mail for UnregisterInstance {
    type Result = DdsResult<()>;
}
impl MailHandler<UnregisterInstance> for DomainParticipantActor {
    fn handle(&mut self, message: UnregisterInstance) -> <UnregisterInstance as Mail>::Result {
        let publisher = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_writer = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let serialized_key = get_serialized_key_from_serialized_foo(
            &message.serialized_data,
            data_writer.type_support(),
        )?;
        data_writer.unregister_w_timestamp(serialized_key, message.timestamp)?;

        Ok(())
    }
}

pub struct LookupInstance {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub serialized_data: Vec<u8>,
}
impl Mail for LookupInstance {
    type Result = DdsResult<Option<InstanceHandle>>;
}
impl MailHandler<LookupInstance> for DomainParticipantActor {
    fn handle(&mut self, message: LookupInstance) -> <LookupInstance as Mail>::Result {
        let data_writer = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        if !data_writer.enabled() {
            return Err(DdsError::NotEnabled);
        }

        let instance_handle = get_instance_handle_from_serialized_foo(
            &message.serialized_data,
            data_writer.type_support(),
        )?;

        Ok(data_writer
            .contains_instance(&instance_handle)
            .then_some(instance_handle))
    }
}

pub struct WriteWTimestamp {
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub serialized_data: Vec<u8>,
    pub timestamp: Time,
}
impl Mail for WriteWTimestamp {
    type Result = DdsResult<()>;
}
impl MailHandler<WriteWTimestamp> for DomainParticipantActor {
    fn handle(&mut self, message: WriteWTimestamp) -> <WriteWTimestamp as Mail>::Result {
        let now = self.domain_participant.get_current_time();
        let publisher = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_writer = publisher
            .get_mut_data_writer(message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let instance_handle = get_instance_handle_from_serialized_foo(
            &message.serialized_data,
            data_writer.type_support(),
        )?;

        // if data_writer.qos().reliability.kind == ReliabilityQosPolicyKind::Reliable {
        // let start = std::time::Instant::now();
        // todo!();
        // data_writer.transport_writer().are_all_changes_acknowledged()
        // let timer_handle = self.publisher.get_participant().timer_handle().clone();
        // loop {
        //     if !self
        //         .writer_address
        //         .send_actor_mail(data_writer_actor::IsDataLostAfterAddingChange {
        //             instance_handle: change.instance_handle().into(),
        //         })?
        //         .receive_reply()
        //         .await
        //     {
        //         break;
        //     }
        //     timer_handle
        //         .sleep(std::time::Duration::from_millis(20))
        //         .await;
        //     if let DurationKind::Finite(timeout) = writer_qos.reliability.max_blocking_time {
        //         if std::time::Instant::now().duration_since(start) > timeout.into() {
        //             return Err(DdsError::Timeout);
        //         }
        //     }
        // }
        // }

        match data_writer.qos().lifespan.duration {
            DurationKind::Finite(lifespan_duration) => {
                let timer_handle = self.timer_driver.handle();
                let sleep_duration = message.timestamp - now + lifespan_duration;
                if sleep_duration > Duration::new(0, 0) {
                    let sequence_number = data_writer
                        .write_w_timestamp(message.serialized_data, message.timestamp)?;
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
                data_writer.write_w_timestamp(message.serialized_data, message.timestamp)?;
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

        Ok(())
    }
}

pub struct DisposeWTimestamp {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub serialized_data: Vec<u8>,
    pub timestamp: Time,
}
impl Mail for DisposeWTimestamp {
    type Result = DdsResult<()>;
}
impl MailHandler<DisposeWTimestamp> for DomainParticipantActor {
    fn handle(&mut self, message: DisposeWTimestamp) -> <DisposeWTimestamp as Mail>::Result {
        let publisher = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_writer = publisher
            .get_mut_data_writer(message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let serialized_key = get_serialized_key_from_serialized_foo(
            &message.serialized_data,
            data_writer.type_support(),
        )?;
        data_writer.dispose_w_timestamp(serialized_key, message.timestamp)
    }
}

pub struct WaitForAcknowledgments {
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub timeout: Duration,
}
impl Mail for WaitForAcknowledgments {
    type Result = Pin<Box<dyn Future<Output = DdsResult<()>> + Send>>;
}
impl MailHandler<WaitForAcknowledgments> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: WaitForAcknowledgments,
    ) -> <WaitForAcknowledgments as Mail>::Result {
        let timer_handle = self.timer_driver.handle();
        Box::pin(async move {
            timer_handle
                .timeout(
                    message.timeout.into(),
                    Box::pin(async move {
                        loop {
                            let all_changes_ack = message
                                .participant_address
                                .send_actor_mail(AreAllChangesAcknowledged {
                                    publisher_handle: message.publisher_handle,
                                    data_writer_handle: message.data_writer_handle,
                                })?
                                .receive_reply()
                                .await?;
                            if all_changes_ack {
                                return Ok(());
                            }
                        }
                    }),
                )
                .await
                .map_err(|_| DdsError::Timeout)?
        })
    }
}

pub struct GetOfferedDeadlineMissedStatus {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}
impl Mail for GetOfferedDeadlineMissedStatus {
    type Result = DdsResult<OfferedDeadlineMissedStatus>;
}
impl MailHandler<GetOfferedDeadlineMissedStatus> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetOfferedDeadlineMissedStatus,
    ) -> <GetOfferedDeadlineMissedStatus as Mail>::Result {
        let publisher = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_writer = publisher
            .get_mut_data_writer(message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        Ok(data_writer.get_offered_deadline_missed_status())
    }
}

pub struct GetPublicationMatchedStatus {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}
impl Mail for GetPublicationMatchedStatus {
    type Result = DdsResult<PublicationMatchedStatus>;
}
impl MailHandler<GetPublicationMatchedStatus> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetPublicationMatchedStatus,
    ) -> <GetPublicationMatchedStatus as Mail>::Result {
        let publisher = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_writer = publisher
            .get_mut_data_writer(message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        Ok(data_writer.get_publication_matched_status())
    }
}

pub struct GetMatchedSubscriptionData {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub subscription_handle: InstanceHandle,
}
impl Mail for GetMatchedSubscriptionData {
    type Result = DdsResult<SubscriptionBuiltinTopicData>;
}
impl MailHandler<GetMatchedSubscriptionData> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetMatchedSubscriptionData,
    ) -> <GetMatchedSubscriptionData as Mail>::Result {
        self.domain_participant
            .get_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_matched_subscription_data(&message.subscription_handle)
            .ok_or(DdsError::BadParameter)
            .cloned()
    }
}

pub struct GetMatchedSubscriptions {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}
impl Mail for GetMatchedSubscriptions {
    type Result = DdsResult<Vec<InstanceHandle>>;
}
impl MailHandler<GetMatchedSubscriptions> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetMatchedSubscriptions,
    ) -> <GetMatchedSubscriptions as Mail>::Result {
        Ok(self
            .domain_participant
            .get_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_matched_subscriptions())
    }
}

pub struct SetDataWriterQos {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub qos: QosKind<DataWriterQos>,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl Mail for SetDataWriterQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDataWriterQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetDataWriterQos) -> <SetDataWriterQos as Mail>::Result {
        let publisher = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let qos = match message.qos {
            QosKind::Default => publisher.default_datawriter_qos().clone(),
            QosKind::Specific(q) => q,
        };
        let data_writer = publisher
            .get_mut_data_writer(message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        data_writer.set_qos(qos)?;
        if data_writer.enabled() {
            message
                .participant_address
                .send_actor_mail(discovery_service::AnnounceDataWriter {
                    publisher_handle: message.publisher_handle,
                    data_writer_handle: message.data_writer_handle,
                })
                .ok();
        }

        Ok(())
    }
}

pub struct GetDataWriterQos {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}
impl Mail for GetDataWriterQos {
    type Result = DdsResult<DataWriterQos>;
}
impl MailHandler<GetDataWriterQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetDataWriterQos) -> <GetDataWriterQos as Mail>::Result {
        Ok(self
            .domain_participant
            .get_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .qos()
            .clone())
    }
}

pub struct Enable {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl Mail for Enable {
    type Result = DdsResult<()>;
}
impl MailHandler<Enable> for DomainParticipantActor {
    fn handle(&mut self, message: Enable) -> <Enable as Mail>::Result {
        let publisher = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_writer = publisher
            .get_mut_data_writer(message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        if !data_writer.enabled() {
            data_writer.enable();

            for subscription_builtin_topic_data in self
                .domain_participant
                .subscription_builtin_topic_data_list()
                .cloned()
            {
                message
                    .participant_address
                    .send_actor_mail(discovery_service::AddDiscoveredReader {
                        subscription_builtin_topic_data,
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
                })?;
        }

        Ok(())
    }
}

pub struct SetListener {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub a_listener: Option<Box<dyn AnyDataWriterListener + Send>>,
    pub status_kind: Vec<StatusKind>,
}
impl Mail for SetListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetListener> for DomainParticipantActor {
    fn handle(&mut self, message: SetListener) -> <SetListener as Mail>::Result {
        todo!()
        // let writer = self.writer_address();
        // if !writer
        //     .send_actor_mail(data_writer_actor::IsEnabled)?
        //     .receive_reply()
        //     .await
        // {
        //     let message_sender_actor = self
        //         .participant_address()
        //         .send_actor_mail(domain_participant_actor::GetMessageSender)?
        //         .receive_reply()
        //         .await;
        //     writer
        //         .send_actor_mail(data_writer_actor::Enable {
        //             data_writer_address: writer.clone(),
        //             message_sender_actor,
        //             executor_handle: self.publisher.get_participant().executor_handle().clone(),
        //             timer_handle: self.publisher.get_participant().timer_handle().clone(),
        //         })?
        //         .receive_reply()
        //         .await;

        //     self.announce_writer().await?;

        //     self.process_sedp_subscriptions_discovery().await?;
        // }
        // Ok(())
    }
}
