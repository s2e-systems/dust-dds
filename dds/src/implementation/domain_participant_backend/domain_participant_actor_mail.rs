use core::{future::Future, pin::Pin};
use std::sync::Arc;

use super::domain_participant_actor::DomainParticipantActor;
use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    dds_async::publisher_listener::PublisherListenerAsync,
    implementation::{
        any_data_writer_listener::AnyDataWriterListener,
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind, TopicQos},
        status::{
            InconsistentTopicStatus, OfferedDeadlineMissedStatus, PublicationMatchedStatus,
            StatusKind,
        },
        time::{Duration, Time},
    },
    runtime::{
        actor::{ActorAddress, MailHandler},
        oneshot::OneshotSender,
    },
    xtypes::dynamic_type::DynamicType,
};

pub enum ParticipantServiceMail {
    CreateUserDefinedPublisher {
        qos: QosKind<PublisherQos>,
        a_listener: Option<Box<dyn PublisherListenerAsync + Send>>,
        mask: Vec<StatusKind>,
        reply_sender:
            OneshotSender<DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>>,
    },
}

pub enum TopicServiceMail {
    GetInconsistentTopicStatus {
        topic_name: String,
        reply_sender: OneshotSender<DdsResult<InconsistentTopicStatus>>,
    },
    SetQos {
        topic_name: String,
        topic_qos: QosKind<TopicQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetQos {
        topic_name: String,
        reply_sender: OneshotSender<DdsResult<TopicQos>>,
    },
    Enable {
        topic_name: String,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetTypeSupport {
        topic_name: String,
        reply_sender: OneshotSender<DdsResult<Arc<dyn DynamicType + Send + Sync>>>,
    },
}

pub enum PublisherServiceMail {
    CreateDataWriter {
        publisher_handle: InstanceHandle,
        topic_name: String,
        qos: QosKind<DataWriterQos>,
        a_listener: Option<Box<dyn AnyDataWriterListener + Send>>,
        mask: Vec<StatusKind>,
        participant_address: ActorAddress<DomainParticipantActor>,
        reply_sender:
            OneshotSender<DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>>,
    },
    DeleteDataWriter {
        publisher_handle: InstanceHandle,
        datawriter_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetDefaultDataWriterQos {
        publisher_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<DataWriterQos>>,
    },
    SetDefaultDataWriterQos {
        publisher_handle: InstanceHandle,
        qos: QosKind<DataWriterQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetPublisherQos {
        publisher_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<PublisherQos>>,
    },
    SetPublisherQos {
        publisher_handle: InstanceHandle,
        qos: QosKind<PublisherQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    SetPublisherListener {
        publisher_handle: InstanceHandle,
        a_listener: Option<Box<dyn PublisherListenerAsync + Send>>,
        mask: Vec<StatusKind>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
}

pub enum WriterServiceMail {
    SetListener {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        listener: Option<Box<dyn AnyDataWriterListener + Send>>,
        listener_mask: Vec<StatusKind>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetDataWriterQos {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<DataWriterQos>>,
    },
    GetMatchedSubscriptions {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<Vec<InstanceHandle>>>,
    },
    GetMatchedSubscriptionData {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        subscription_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<SubscriptionBuiltinTopicData>>,
    },
    GetPublicationMatchedStatus {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<PublicationMatchedStatus>>,
    },
    UnregisterInstance {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        serialized_data: Vec<u8>,
        timestamp: Time,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    LookupInstance {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        serialized_data: Vec<u8>,
        reply_sender: OneshotSender<DdsResult<Option<InstanceHandle>>>,
    },
    WriteWTimestamp {
        participant_address: ActorAddress<DomainParticipantActor>,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        serialized_data: Vec<u8>,
        timestamp: Time,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    DisposeWTimestamp {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        serialized_data: Vec<u8>,
        timestamp: Time,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    WaitForAcknowledgments {
        participant_address: ActorAddress<DomainParticipantActor>,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        timeout: Duration,
        reply_sender: OneshotSender<Pin<Box<dyn Future<Output = DdsResult<()>> + Send>>>,
    },
    GetOfferedDeadlineMissedStatus {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<OfferedDeadlineMissedStatus>>,
    },
    EnableDataWriter {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        participant_address: ActorAddress<DomainParticipantActor>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    SetDataWriterQos {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        qos: QosKind<DataWriterQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
}

pub enum MessageServiceMail {
    RemoveWriterChange {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        sequence_number: i64,
    },
    AreAllChangesAcknowledged {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<bool>>,
    },
}

pub enum EventServiceMail {
    OfferedDeadlineMissed {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        change_instance_handle: InstanceHandle,
        participant_address: ActorAddress<DomainParticipantActor>,
    },
}

pub enum DomainParticipantMail {
    ParticipantService(ParticipantServiceMail),
    TopicService(TopicServiceMail),
    PublisherService(PublisherServiceMail),
    WriterService(WriterServiceMail),
    MessageService(MessageServiceMail),
    EventService(EventServiceMail),
}

impl MailHandler<DomainParticipantMail> for DomainParticipantActor {
    fn handle(&mut self, message: DomainParticipantMail) {
        match message {
            DomainParticipantMail::ParticipantService(participant_service_mail) => {
                self.handle_participant_service(participant_service_mail)
            }
            DomainParticipantMail::TopicService(topic_service_mail) => {
                self.handle_topic_service(topic_service_mail);
            }
            DomainParticipantMail::PublisherService(publisher_service_mail) => {
                self.handle_publisher_service(publisher_service_mail)
            }
            DomainParticipantMail::WriterService(writer_service_mail) => {
                self.handle_writer_service(writer_service_mail);
            }
            DomainParticipantMail::MessageService(message_service_mail) => {
                self.handle_message_service(message_service_mail)
            }
            DomainParticipantMail::EventService(event_service_mail) => {
                self.handle_event_service(event_service_mail)
            }
        };
    }
}

impl DomainParticipantActor {
    fn handle_participant_service(&mut self, participant_service_mail: ParticipantServiceMail) {
        match participant_service_mail {
            ParticipantServiceMail::CreateUserDefinedPublisher {
                qos,
                a_listener,
                mask,
                reply_sender,
            } => reply_sender.send(self.create_user_defined_publisher(qos, a_listener, mask)),
        }
    }

    fn handle_topic_service(&mut self, topic_service_mail: TopicServiceMail) {
        match topic_service_mail {
            TopicServiceMail::GetInconsistentTopicStatus {
                topic_name,
                reply_sender,
            } => reply_sender.send(self.get_inconsistent_topic_status(topic_name)),
            TopicServiceMail::SetQos {
                topic_name,
                topic_qos,
                reply_sender,
            } => reply_sender.send(self.set_topic_qos(topic_name, topic_qos)),
            TopicServiceMail::GetQos {
                topic_name,
                reply_sender,
            } => reply_sender.send(self.get_topic_qos(topic_name)),
            TopicServiceMail::Enable {
                topic_name,
                reply_sender,
            } => reply_sender.send(self.enable_topic(topic_name)),
            TopicServiceMail::GetTypeSupport {
                topic_name,
                reply_sender,
            } => reply_sender.send(self.get_type_support(topic_name)),
        }
    }

    fn handle_publisher_service(&mut self, publisher_service_mail: PublisherServiceMail) {
        match publisher_service_mail {
            PublisherServiceMail::CreateDataWriter {
                publisher_handle,
                topic_name,
                qos,
                a_listener,
                mask,
                participant_address,
                reply_sender,
            } => {
                reply_sender.send(self.create_data_writer(
                    publisher_handle,
                    topic_name,
                    qos,
                    a_listener,
                    mask,
                    participant_address,
                ));
            }
            PublisherServiceMail::DeleteDataWriter {
                publisher_handle,
                datawriter_handle,
                reply_sender,
            } => reply_sender.send(self.delete_data_writer(publisher_handle, datawriter_handle)),
            PublisherServiceMail::GetDefaultDataWriterQos {
                publisher_handle,
                reply_sender,
            } => reply_sender.send(self.get_default_datawriter_qos(publisher_handle)),
            PublisherServiceMail::SetDefaultDataWriterQos {
                publisher_handle,
                qos,
                reply_sender,
            } => reply_sender.send(self.set_default_datawriter_qos(publisher_handle, qos)),
            PublisherServiceMail::GetPublisherQos {
                publisher_handle,
                reply_sender,
            } => reply_sender.send(self.get_publisher_qos(publisher_handle)),
            PublisherServiceMail::SetPublisherQos {
                publisher_handle,
                qos,
                reply_sender,
            } => reply_sender.send(self.set_publisher_qos(publisher_handle, qos)),
            PublisherServiceMail::SetPublisherListener {
                publisher_handle,
                a_listener,
                mask,
                reply_sender,
            } => reply_sender.send(self.set_publisher_listener(publisher_handle, a_listener, mask)),
        }
    }

    fn handle_writer_service(&mut self, writer_service_mail: WriterServiceMail) {
        match writer_service_mail {
            WriterServiceMail::SetListener {
                publisher_handle,
                data_writer_handle,
                listener,
                listener_mask,
                reply_sender,
            } => reply_sender.send(self.set_listener_data_writer(
                publisher_handle,
                data_writer_handle,
                listener,
                listener_mask,
            )),
            WriterServiceMail::GetDataWriterQos {
                publisher_handle,
                data_writer_handle,
                reply_sender,
            } => reply_sender.send(self.get_data_writer_qos(publisher_handle, data_writer_handle)),
            WriterServiceMail::GetMatchedSubscriptions {
                publisher_handle,
                data_writer_handle,
                reply_sender,
            } => reply_sender
                .send(self.get_matched_subscriptions(publisher_handle, data_writer_handle)),
            WriterServiceMail::GetMatchedSubscriptionData {
                publisher_handle,
                data_writer_handle,
                subscription_handle,
                reply_sender,
            } => reply_sender.send(self.get_matched_subscription_data(
                publisher_handle,
                data_writer_handle,
                subscription_handle,
            )),
            WriterServiceMail::GetPublicationMatchedStatus {
                publisher_handle,
                data_writer_handle,
                reply_sender,
            } => reply_sender
                .send(self.get_publication_matched_status(publisher_handle, data_writer_handle)),
            WriterServiceMail::UnregisterInstance {
                publisher_handle,
                data_writer_handle,
                serialized_data,
                timestamp,
                reply_sender,
            } => reply_sender.send(self.unregister_instance(
                publisher_handle,
                data_writer_handle,
                serialized_data,
                timestamp,
            )),
            WriterServiceMail::LookupInstance {
                publisher_handle,
                data_writer_handle,
                serialized_data,
                reply_sender,
            } => reply_sender.send(self.lookup_instance(
                publisher_handle,
                data_writer_handle,
                serialized_data,
            )),
            WriterServiceMail::WriteWTimestamp {
                participant_address,
                publisher_handle,
                data_writer_handle,
                serialized_data,
                timestamp,
                reply_sender,
            } => reply_sender.send(self.write_w_timestamp(
                participant_address,
                publisher_handle,
                data_writer_handle,
                serialized_data,
                timestamp,
            )),
            WriterServiceMail::DisposeWTimestamp {
                publisher_handle,
                data_writer_handle,
                serialized_data,
                timestamp,
                reply_sender,
            } => reply_sender.send(self.dispose_w_timestamp(
                publisher_handle,
                data_writer_handle,
                serialized_data,
                timestamp,
            )),
            WriterServiceMail::WaitForAcknowledgments {
                participant_address,
                publisher_handle,
                data_writer_handle,
                timeout,
                reply_sender,
            } => reply_sender.send(self.wait_for_acknowledgments(
                participant_address,
                publisher_handle,
                data_writer_handle,
                timeout,
            )),
            WriterServiceMail::GetOfferedDeadlineMissedStatus {
                publisher_handle,
                data_writer_handle,
                reply_sender,
            } => reply_sender.send(
                self.get_offered_deadline_missed_status(publisher_handle, data_writer_handle),
            ),
            WriterServiceMail::EnableDataWriter {
                publisher_handle,
                data_writer_handle,
                participant_address,
                reply_sender,
            } => reply_sender.send(self.enable_data_writer(
                publisher_handle,
                data_writer_handle,
                participant_address,
            )),
            WriterServiceMail::SetDataWriterQos {
                publisher_handle,
                data_writer_handle,
                qos,
                reply_sender,
            } => reply_sender.send(self.set_data_writer_qos(
                publisher_handle,
                data_writer_handle,
                qos,
            )),
        }
    }

    fn handle_message_service(&mut self, message_service_mail: MessageServiceMail) {
        match message_service_mail {
            MessageServiceMail::RemoveWriterChange {
                publisher_handle,
                data_writer_handle,
                sequence_number,
            } => self.remove_writer_change(publisher_handle, data_writer_handle, sequence_number),
            MessageServiceMail::AreAllChangesAcknowledged {
                publisher_handle,
                data_writer_handle,
                reply_sender,
            } => reply_sender
                .send(self.are_all_changes_acknowledged(publisher_handle, data_writer_handle)),
        }
    }

    fn handle_event_service(&mut self, event_service_mail: EventServiceMail) {
        match event_service_mail {
            EventServiceMail::OfferedDeadlineMissed {
                publisher_handle,
                data_writer_handle,
                change_instance_handle,
                participant_address,
            } => self.offered_deadline_missed(
                publisher_handle,
                data_writer_handle,
                change_instance_handle,
                participant_address,
            ),
        }
    }
}
