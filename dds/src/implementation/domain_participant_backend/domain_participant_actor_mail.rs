use core::{future::Future, pin::Pin};
use std::sync::Arc;

use super::domain_participant_actor::DomainParticipantActor;
use crate::{
    builtin_topics::{
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    dds_async::{
        domain_participant_listener::DomainParticipantListenerAsync,
        publisher_listener::PublisherListenerAsync, subscriber_listener::SubscriberListenerAsync,
        topic_listener::TopicListenerAsync,
    },
    implementation::{
        any_data_reader_listener::AnyDataReaderListener,
        any_data_writer_listener::AnyDataWriterListener,
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantQos, PublisherQos, QosKind,
            SubscriberQos, TopicQos,
        },
        status::{
            InconsistentTopicStatus, OfferedDeadlineMissedStatus, PublicationMatchedStatus,
            StatusKind, SubscriptionMatchedStatus,
        },
        time::{Duration, Time},
    },
    runtime::{
        actor::{ActorAddress, MailHandler},
        oneshot::OneshotSender,
    },
    subscription::sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
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
    DeleteUserDefinedPublisher {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    CreateUserDefinedSubscriber {
        qos: QosKind<SubscriberQos>,
        a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
        mask: Vec<StatusKind>,
        reply_sender:
            OneshotSender<DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>>,
    },
    DeleteUserDefinedSubscriber {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    CreateTopic {
        topic_name: String,
        type_name: String,
        qos: QosKind<TopicQos>,
        a_listener: Option<Box<dyn TopicListenerAsync + Send>>,
        mask: Vec<StatusKind>,
        type_support: Arc<dyn DynamicType + Send + Sync>,
        reply_sender:
            OneshotSender<DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>>,
    },
    DeleteUserDefinedTopic {
        participant_handle: InstanceHandle,
        topic_name: String,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    FindTopic {
        topic_name: String,
        type_support: Arc<dyn DynamicType + Send + Sync>,
        #[allow(clippy::type_complexity)]
        reply_sender: OneshotSender<
            DdsResult<Option<(InstanceHandle, ActorAddress<StatusConditionActor>, String)>>,
        >,
    },
    LookupTopicdescription {
        topic_name: String,
        #[allow(clippy::type_complexity)]
        reply_sender: OneshotSender<
            DdsResult<Option<(String, InstanceHandle, ActorAddress<StatusConditionActor>)>>,
        >,
    },
    IgnoreParticipant {
        handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    IgnoreSubscription {
        handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    IgnorePublication {
        handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    DeleteContainedEntities {
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    SetDefaultPublisherQos {
        qos: QosKind<PublisherQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetDefaultPublisherQos {
        reply_sender: OneshotSender<DdsResult<PublisherQos>>,
    },
    SetDefaultSubscriberQos {
        qos: QosKind<SubscriberQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetDefaultSubscriberQos {
        reply_sender: OneshotSender<DdsResult<SubscriberQos>>,
    },
    SetDefaultTopicQos {
        qos: QosKind<TopicQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetDefaultTopicQos {
        reply_sender: OneshotSender<DdsResult<TopicQos>>,
    },
    GetDiscoveredParticipants {
        reply_sender: OneshotSender<DdsResult<Vec<InstanceHandle>>>,
    },
    GetDiscoveredParticipantData {
        participant_handle: InstanceHandle,

        reply_sender: OneshotSender<DdsResult<ParticipantBuiltinTopicData>>,
    },
    GetDiscoveredTopics {
        reply_sender: OneshotSender<DdsResult<Vec<InstanceHandle>>>,
    },
    GetDiscoveredTopicData {
        topic_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<TopicBuiltinTopicData>>,
    },
    GetCurrentTime {
        reply_sender: OneshotSender<Time>,
    },
    SetQos {
        qos: QosKind<DomainParticipantQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetQos {
        reply_sender: OneshotSender<DdsResult<DomainParticipantQos>>,
    },
    SetListener {
        listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    Enable {
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    IsEmpty {
        reply_sender: OneshotSender<bool>,
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

pub enum SubscriberServiceMail {
    CreateDataReader {
        subscriber_handle: InstanceHandle,
        topic_name: String,
        qos: QosKind<DataReaderQos>,
        a_listener: Option<Box<dyn AnyDataReaderListener>>,
        mask: Vec<StatusKind>,
        domain_participant_address: ActorAddress<DomainParticipantActor>,
        reply_sender:
            OneshotSender<DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>>,
    },
    DeleteDataReader {
        subscriber_handle: InstanceHandle,
        datareader_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    LookupDataReader {
        subscriber_handle: InstanceHandle,
        topic_name: String,
        #[allow(clippy::type_complexity)]
        reply_sender:
            OneshotSender<DdsResult<Option<(InstanceHandle, ActorAddress<StatusConditionActor>)>>>,
    },
    SetDefaultDataReaderQos {
        subscriber_handle: InstanceHandle,
        qos: QosKind<DataReaderQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetDefaultDataReaderQos {
        subscriber_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<DataReaderQos>>,
    },
    SetQos {
        subscriber_handle: InstanceHandle,
        qos: QosKind<SubscriberQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetSubscriberQos {
        subscriber_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<SubscriberQos>>,
    },
    SetListener {
        subscriber_handle: InstanceHandle,
        a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
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

pub enum ReaderServiceMail {
    Enable {
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        participant_address: ActorAddress<DomainParticipantActor>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    Read {
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_samples: i32,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
        specific_instance_handle: Option<InstanceHandle>,
        #[allow(clippy::type_complexity)]
        reply_sender: OneshotSender<DdsResult<Vec<(Option<Arc<[u8]>>, SampleInfo)>>>,
    },
    Take {
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_samples: i32,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
        specific_instance_handle: Option<InstanceHandle>,
        #[allow(clippy::type_complexity)]
        reply_sender: OneshotSender<DdsResult<Vec<(Option<Arc<[u8]>>, SampleInfo)>>>,
    },
    ReadNextInstance {
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
        #[allow(clippy::type_complexity)]
        reply_sender: OneshotSender<DdsResult<Vec<(Option<Arc<[u8]>>, SampleInfo)>>>,
    },
    TakeNextInstance {
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
        #[allow(clippy::type_complexity)]
        reply_sender: OneshotSender<DdsResult<Vec<(Option<Arc<[u8]>>, SampleInfo)>>>,
    },
    GetSubscriptionMatchedStatus {
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<SubscriptionMatchedStatus>>,
    },
    WaitForHistoricalData {
        participant_address: ActorAddress<DomainParticipantActor>,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_wait: Duration,
        reply_sender: OneshotSender<Pin<Box<dyn Future<Output = DdsResult<()>> + Send>>>,
    },
    GetMatchedPublicationData {
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        publication_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<PublicationBuiltinTopicData>>,
    },
    GetMatchedPublications {
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<Vec<InstanceHandle>>>,
    },
    SetQos {
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        qos: QosKind<DataReaderQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetQos {
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<DataReaderQos>>,
    },
    SetListener {
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        listener: Option<Box<dyn AnyDataReaderListener>>,
        listener_mask: Vec<StatusKind>,
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
    IsHistoricalDataReceived {
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
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
    Participant(ParticipantServiceMail),
    Topic(TopicServiceMail),
    Publisher(PublisherServiceMail),
    Writer(WriterServiceMail),
    Subscriber(SubscriberServiceMail),
    Reader(ReaderServiceMail),
    Message(MessageServiceMail),
    Event(EventServiceMail),
}

impl MailHandler<DomainParticipantMail> for DomainParticipantActor {
    fn handle(&mut self, message: DomainParticipantMail) {
        match message {
            DomainParticipantMail::Participant(participant_service_mail) => {
                self.handle_participant_service(participant_service_mail)
            }
            DomainParticipantMail::Topic(topic_service_mail) => {
                self.handle_topic_service(topic_service_mail);
            }
            DomainParticipantMail::Publisher(publisher_service_mail) => {
                self.handle_publisher_service(publisher_service_mail)
            }
            DomainParticipantMail::Writer(writer_service_mail) => {
                self.handle_writer_service(writer_service_mail);
            }
            DomainParticipantMail::Subscriber(subscriber_service_mail) => {
                self.handle_subscriber_service(subscriber_service_mail)
            }
            DomainParticipantMail::Reader(reader_service_mail) => {
                self.handle_reader_service(reader_service_mail)
            }
            DomainParticipantMail::Message(message_service_mail) => {
                self.handle_message_service(message_service_mail)
            }
            DomainParticipantMail::Event(event_service_mail) => {
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
            ParticipantServiceMail::DeleteUserDefinedPublisher {
                participant_handle,
                publisher_handle,
                reply_sender,
            } => {
                reply_sender
                    .send(self.delete_user_defined_publisher(participant_handle, publisher_handle));
            }
            ParticipantServiceMail::CreateUserDefinedSubscriber {
                qos,
                a_listener,
                mask,
                reply_sender,
            } => reply_sender.send(self.create_user_defined_subscriber(qos, a_listener, mask)),
            ParticipantServiceMail::DeleteUserDefinedSubscriber {
                participant_handle,
                subscriber_handle,
                reply_sender,
            } => reply_sender
                .send(self.delete_user_defined_subscriber(participant_handle, subscriber_handle)),
            ParticipantServiceMail::CreateTopic {
                topic_name,
                type_name,
                qos,
                a_listener,
                mask,
                type_support,
                reply_sender,
            } => reply_sender.send(self.create_topic(
                topic_name,
                type_name,
                qos,
                a_listener,
                mask,
                type_support,
            )),
            ParticipantServiceMail::DeleteUserDefinedTopic {
                participant_handle,
                topic_name,
                reply_sender,
            } => reply_sender.send(self.delete_user_defined_topic(participant_handle, topic_name)),
            ParticipantServiceMail::FindTopic {
                topic_name,
                type_support,
                reply_sender,
            } => reply_sender.send(self.find_topic(topic_name, type_support)),
            ParticipantServiceMail::LookupTopicdescription {
                topic_name,
                reply_sender,
            } => reply_sender.send(self.lookup_topicdescription(topic_name)),
            ParticipantServiceMail::IgnoreParticipant {
                handle,
                reply_sender,
            } => reply_sender.send(self.ignore_participant(handle)),
            ParticipantServiceMail::IgnoreSubscription {
                handle,
                reply_sender,
            } => reply_sender.send(self.ignore_subscription(handle)),
            ParticipantServiceMail::IgnorePublication {
                handle,
                reply_sender,
            } => reply_sender.send(self.ignore_publication(handle)),
            ParticipantServiceMail::DeleteContainedEntities { reply_sender } => {
                reply_sender.send(self.delete_participant_contained_entities())
            }
            ParticipantServiceMail::SetDefaultPublisherQos { qos, reply_sender } => {
                reply_sender.send(self.set_default_publisher_qos(qos))
            }
            ParticipantServiceMail::GetDefaultPublisherQos { reply_sender } => {
                reply_sender.send(self.get_default_publisher_qos())
            }
            ParticipantServiceMail::SetDefaultSubscriberQos { qos, reply_sender } => {
                reply_sender.send(self.set_default_subscriber_qos(qos))
            }
            ParticipantServiceMail::GetDefaultSubscriberQos { reply_sender } => {
                reply_sender.send(self.get_default_subscriber_qos())
            }
            ParticipantServiceMail::SetDefaultTopicQos { qos, reply_sender } => {
                reply_sender.send(self.set_default_topic_qos(qos))
            }
            ParticipantServiceMail::GetDefaultTopicQos { reply_sender } => {
                reply_sender.send(self.get_default_topic_qos())
            }
            ParticipantServiceMail::GetCurrentTime { reply_sender } => {
                reply_sender.send(self.get_current_time())
            }
            ParticipantServiceMail::GetDiscoveredParticipants { reply_sender } => {
                reply_sender.send(self.get_discovered_participants())
            }
            ParticipantServiceMail::GetDiscoveredParticipantData {
                participant_handle,
                reply_sender,
            } => reply_sender.send(self.get_discovered_participant_data(participant_handle)),
            ParticipantServiceMail::GetDiscoveredTopics { reply_sender } => {
                reply_sender.send(self.get_discovered_topics())
            }
            ParticipantServiceMail::GetDiscoveredTopicData {
                topic_handle,
                reply_sender,
            } => reply_sender.send(self.get_discovered_topic_data(topic_handle)),
            ParticipantServiceMail::SetQos { qos, reply_sender } => {
                reply_sender.send(self.set_domain_participant_qos(qos))
            }
            ParticipantServiceMail::GetQos { reply_sender } => {
                reply_sender.send(self.get_domain_participant_qos())
            }
            ParticipantServiceMail::SetListener {
                listener,
                status_kind,
                reply_sender,
            } => reply_sender.send(self.set_domain_participant_listener(listener, status_kind)),
            ParticipantServiceMail::Enable { reply_sender } => {
                reply_sender.send(self.enable_domain_participant())
            }
            ParticipantServiceMail::IsEmpty { reply_sender } => {
                reply_sender.send(self.is_participant_empty())
            }
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

    fn handle_subscriber_service(&mut self, subscriber_service_mail: SubscriberServiceMail) {
        match subscriber_service_mail {
            SubscriberServiceMail::CreateDataReader {
                subscriber_handle,
                topic_name,
                qos,
                a_listener,
                mask,
                domain_participant_address,
                reply_sender,
            } => reply_sender.send(self.create_data_reader(
                subscriber_handle,
                topic_name,
                qos,
                a_listener,
                mask,
                domain_participant_address,
            )),
            SubscriberServiceMail::DeleteDataReader {
                subscriber_handle,
                datareader_handle,
                reply_sender,
            } => reply_sender.send(self.delete_data_reader(subscriber_handle, datareader_handle)),
            SubscriberServiceMail::LookupDataReader {
                subscriber_handle,
                topic_name,
                reply_sender,
            } => reply_sender.send(self.lookup_data_reader(subscriber_handle, topic_name)),
            SubscriberServiceMail::SetDefaultDataReaderQos {
                subscriber_handle,
                qos,
                reply_sender,
            } => reply_sender.send(self.set_default_data_reader_qos(subscriber_handle, qos)),
            SubscriberServiceMail::GetDefaultDataReaderQos {
                subscriber_handle,
                reply_sender,
            } => reply_sender.send(self.get_default_data_reader_qos(subscriber_handle)),
            SubscriberServiceMail::SetQos {
                subscriber_handle,
                qos,
                reply_sender,
            } => reply_sender.send(self.set_subscriber_qos(subscriber_handle, qos)),
            SubscriberServiceMail::GetSubscriberQos {
                subscriber_handle,
                reply_sender,
            } => reply_sender.send(self.get_subscriber_qos(subscriber_handle)),
            SubscriberServiceMail::SetListener {
                subscriber_handle,
                a_listener,
                mask,
                reply_sender,
            } => {
                reply_sender.send(self.set_subscriber_listener(subscriber_handle, a_listener, mask))
            }
        }
    }

    fn handle_reader_service(&mut self, reader_service_mail: ReaderServiceMail) {
        match reader_service_mail {
            ReaderServiceMail::Read {
                subscriber_handle,
                data_reader_handle,
                max_samples,
                sample_states,
                view_states,
                instance_states,
                specific_instance_handle,
                reply_sender,
            } => reply_sender.send(self.read(
                subscriber_handle,
                data_reader_handle,
                max_samples,
                sample_states,
                view_states,
                instance_states,
                specific_instance_handle,
            )),
            ReaderServiceMail::Take {
                subscriber_handle,
                data_reader_handle,
                max_samples,
                sample_states,
                view_states,
                instance_states,
                specific_instance_handle,
                reply_sender,
            } => reply_sender.send(self.take(
                subscriber_handle,
                data_reader_handle,
                max_samples,
                sample_states,
                view_states,
                instance_states,
                specific_instance_handle,
            )),
            ReaderServiceMail::ReadNextInstance {
                subscriber_handle,
                data_reader_handle,
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
                reply_sender,
            } => reply_sender.send(self.read_next_instance(
                subscriber_handle,
                data_reader_handle,
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
            )),
            ReaderServiceMail::TakeNextInstance {
                subscriber_handle,
                data_reader_handle,
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
                reply_sender,
            } => reply_sender.send(self.take_next_instance(
                subscriber_handle,
                data_reader_handle,
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
            )),
            ReaderServiceMail::Enable {
                subscriber_handle,
                data_reader_handle,
                participant_address,
                reply_sender,
            } => reply_sender.send(self.enable_data_reader(
                subscriber_handle,
                data_reader_handle,
                participant_address,
            )),
            ReaderServiceMail::GetSubscriptionMatchedStatus {
                subscriber_handle,
                data_reader_handle,
                reply_sender,
            } => reply_sender
                .send(self.get_subscription_matched_status(subscriber_handle, data_reader_handle)),
            ReaderServiceMail::WaitForHistoricalData {
                participant_address,
                subscriber_handle,
                data_reader_handle,
                max_wait,
                reply_sender,
            } => reply_sender.send(self.wait_for_historical_data(
                participant_address,
                subscriber_handle,
                data_reader_handle,
                max_wait,
            )),
            ReaderServiceMail::GetMatchedPublicationData {
                subscriber_handle,
                data_reader_handle,
                publication_handle,
                reply_sender,
            } => reply_sender.send(self.get_matched_publication_data(
                subscriber_handle,
                data_reader_handle,
                publication_handle,
            )),
            ReaderServiceMail::GetMatchedPublications {
                subscriber_handle,
                data_reader_handle,
                reply_sender,
            } => reply_sender
                .send(self.get_matched_publications(subscriber_handle, data_reader_handle)),
            ReaderServiceMail::GetQos {
                subscriber_handle,
                data_reader_handle,
                reply_sender,
            } => reply_sender.send(self.get_data_reader_qos(subscriber_handle, data_reader_handle)),
            ReaderServiceMail::SetQos {
                subscriber_handle,
                data_reader_handle,
                qos,
                reply_sender,
            } => reply_sender.send(self.set_data_reader_qos(
                subscriber_handle,
                data_reader_handle,
                qos,
            )),
            ReaderServiceMail::SetListener {
                subscriber_handle,
                data_reader_handle,
                listener,
                listener_mask,
                reply_sender,
            } => reply_sender.send(self.set_data_reader_listener(
                subscriber_handle,
                data_reader_handle,
                listener,
                listener_mask,
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
            MessageServiceMail::IsHistoricalDataReceived {
                subscriber_handle,
                data_reader_handle,
                reply_sender,
            } => reply_sender
                .send(self.is_historical_data_received(subscriber_handle, data_reader_handle)),
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
