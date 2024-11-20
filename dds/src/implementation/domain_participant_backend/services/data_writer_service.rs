use core::{future::Future, pin::Pin};

use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData},
    implementation::{
        any_data_writer_listener::AnyDataWriterListener,
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            services::message_service::{self, AreAllChangesAcknowledged},
        },
        xtypes_glue::key_and_instance_handle::get_instance_handle_from_serialized_foo,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, QosKind},
        status::{OfferedDeadlineMissedStatus, PublicationMatchedStatus, StatusKind},
        time::{Duration, Time},
    },
    runtime::actor::{ActorAddress, Mail, MailHandler},
};

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

        todo!();

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
            todo!(),
            // self.domain_participant
            //     .get_topic(data_writer.topic_name())
            //     .unwrap()
            //     .type_support()
            //     .as_ref(),
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
        todo!()
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
        todo!()
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
        let publisher_qos = publisher.qos().clone();
        let data_writer = publisher
            .get_mut_data_writer(message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        if data_writer.enabled() {
            data_writer.set_qos(qos);

            todo!()
            // let publication_builtin_topic_data = PublicationBuiltinTopicData {
            //     key: BuiltInTopicKey {
            //         value: data_writer.transport_writer().guid(),
            //     },
            //     participant_key: BuiltInTopicKey { value: [0; 16] },
            //     topic_name: data_writer.topic_name().to_owned(),
            //     type_name: data_writer.type_name().to_owned(),
            //     durability: data_writer.qos().durability.clone(),
            //     deadline: data_writer.qos().deadline.clone(),
            //     latency_budget: data_writer.qos().latency_budget.clone(),
            //     liveliness: data_writer.qos().liveliness.clone(),
            //     reliability: data_writer.qos().reliability.clone(),
            //     lifespan: data_writer.qos().lifespan.clone(),
            //     user_data: data_writer.qos().user_data.clone(),
            //     ownership: data_writer.qos().ownership.clone(),
            //     ownership_strength: data_writer.qos().ownership_strength.clone(),
            //     destination_order: data_writer.qos().destination_order.clone(),
            //     presentation: publisher_qos.presentation.clone(),
            //     partition: publisher_qos.partition.clone(),
            //     topic_data: self.topic_list[data_writer.topic_name()]
            //         .qos()
            //         .topic_data
            //         .clone(),
            //     group_data: publisher_qos.group_data.clone(),
            //     representation: data_writer.qos().representation.clone(),
            // };

            // self.announce_created_or_modified_datawriter(publication_builtin_topic_data)?;
        } else {
            data_writer.set_qos(qos);
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
        todo!()
        // let qos = match qos {
        //     QosKind::Default => {
        //         self.publisher_address()
        //             .send_actor_mail(publisher_actor::GetDefaultDatawriterQos)?
        //             .receive_reply()
        //             .await
        //     }
        //     QosKind::Specific(q) => q,
        // };

        // self.writer_address
        //     .send_actor_mail(data_writer_actor::SetQos { qos })?
        //     .receive_reply()
        //     .await?;
        // if self
        //     .writer_address
        //     .send_actor_mail(data_writer_actor::IsEnabled)?
        //     .receive_reply()
        //     .await
        // {
        //     self.announce_writer().await?;
        // }

        // Ok(())
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
            message
                .participant_address
                .send_actor_mail(message_service::AnnounceDataWriter {
                    publisher_handle: message.publisher_handle,
                    data_writer_handle: message.data_writer_handle,
                })?;
        }

        Ok(())
    }
}

pub struct GetDataWriterInstanceHandle {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}
impl Mail for GetDataWriterInstanceHandle {
    type Result = DdsResult<InstanceHandle>;
}
impl MailHandler<GetDataWriterInstanceHandle> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetDataWriterInstanceHandle,
    ) -> <GetDataWriterInstanceHandle as Mail>::Result {
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

pub struct SetDataWriterListener {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub a_listener: Option<Box<dyn AnyDataWriterListener + Send>>,
    pub status_kind: Vec<StatusKind>,
}
impl Mail for SetDataWriterListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDataWriterListener> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetDataWriterListener,
    ) -> <SetDataWriterListener as Mail>::Result {
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
