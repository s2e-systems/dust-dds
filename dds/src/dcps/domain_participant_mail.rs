use super::domain_participant::DcpsDomainParticipant;
use crate::{
    builtin_topics::{
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    dcps::{
        actor::{ActorAddress, MailHandler},
        channels::{mpsc::MpscSender, oneshot::OneshotSender},
        listeners::{
            data_reader_listener::DcpsDataReaderListener,
            data_writer_listener::DcpsDataWriterListener,
            domain_participant_listener::DcpsDomainParticipantListener,
            publisher_listener::DcpsPublisherListener, subscriber_listener::DcpsSubscriberListener,
            topic_listener::DcpsTopicListener,
        },
        status_condition::DcpsStatusCondition,
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantQos, PublisherQos, QosKind,
            SubscriberQos, TopicQos,
        },
        sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
        status::{
            InconsistentTopicStatus, OfferedDeadlineMissedStatus, PublicationMatchedStatus,
            StatusKind, SubscriptionMatchedStatus,
        },
        time::{Duration, Time},
    },
    runtime::DdsRuntime,
    transport::types::CacheChange,
    xtypes::dynamic_type::{DynamicData, DynamicType},
};
use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec};
use core::{future::Future, pin::Pin};

pub enum ParticipantServiceMail {
    CreateUserDefinedPublisher {
        qos: QosKind<PublisherQos>,
        dcps_listener: Option<DcpsPublisherListener>,
        mask: Vec<StatusKind>,
        reply_sender: OneshotSender<DdsResult<InstanceHandle>>,
    },
    DeleteUserDefinedPublisher {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    CreateUserDefinedSubscriber {
        qos: QosKind<SubscriberQos>,
        dcps_listener: Option<DcpsSubscriberListener>,
        mask: Vec<StatusKind>,
        reply_sender: OneshotSender<DdsResult<(InstanceHandle, ActorAddress<DcpsStatusCondition>)>>,
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
        dcps_listener: Option<DcpsTopicListener>,
        mask: Vec<StatusKind>,
        type_support: Arc<DynamicType>,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
        reply_sender: OneshotSender<DdsResult<(InstanceHandle, ActorAddress<DcpsStatusCondition>)>>,
    },
    DeleteUserDefinedTopic {
        participant_handle: InstanceHandle,
        topic_name: String,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    CreateContentFilteredTopic {
        participant_handle: InstanceHandle,
        name: String,
        related_topic_name: String,
        filter_expression: String,
        expression_parameters: Vec<String>,
        reply_sender: OneshotSender<DdsResult<InstanceHandle>>,
    },
    DeleteContentFilteredTopic {
        participant_handle: InstanceHandle,
        name: String,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    FindTopic {
        topic_name: String,
        type_support: Arc<DynamicType>,
        #[allow(clippy::type_complexity)]
        reply_sender: OneshotSender<
            DdsResult<Option<(InstanceHandle, ActorAddress<DcpsStatusCondition>, String)>>,
        >,
    },
    LookupTopicdescription {
        topic_name: String,
        #[allow(clippy::type_complexity)]
        reply_sender: OneshotSender<
            DdsResult<Option<(String, InstanceHandle, ActorAddress<DcpsStatusCondition>)>>,
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
        participant_address: MpscSender<DcpsDomainParticipantMail>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetQos {
        reply_sender: OneshotSender<DdsResult<DomainParticipantQos>>,
    },
    SetListener {
        dcps_listener: Option<DcpsDomainParticipantListener>,
        status_kind: Vec<StatusKind>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    Enable {
        participant_address: MpscSender<DcpsDomainParticipantMail>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    IsEmpty {
        reply_sender: OneshotSender<bool>,
    },
    /// Send a TypeLookup request to discover remote types (XTypes 1.3)
    #[cfg(feature = "type_lookup")]
    RequestTypeLookup {
        /// Target participant's GUID prefix
        target_participant_prefix: [u8; 12],
        /// TypeIdentifiers to look up (raw XCDR-encoded bytes)
        type_ids: Vec<Vec<u8>>,
        /// Sender to deliver the reply
        #[allow(clippy::type_complexity)]
        reply_sender: OneshotSender<
            DdsResult<crate::dcps::data_representation_builtin_endpoints::type_lookup::TypeLookupReply>,
        >,
    },
    /// Get discovered writer info for a topic to enable type discovery
    #[cfg(feature = "type_lookup")]
    GetDiscoveredWriterForTopic {
        /// Topic name to search for
        topic_name: String,
        /// Returns: (participant_prefix, type_information bytes) if found
        #[allow(clippy::type_complexity)]
        reply_sender: OneshotSender<DdsResult<Option<([u8; 12], Option<Vec<u8>>)>>>,
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
        participant_address: MpscSender<DcpsDomainParticipantMail>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetTypeSupport {
        topic_name: String,
        reply_sender: OneshotSender<DdsResult<Arc<DynamicType>>>,
    },
}

pub enum PublisherServiceMail {
    CreateDataWriter {
        publisher_handle: InstanceHandle,
        topic_name: String,
        qos: QosKind<DataWriterQos>,
        dcps_listener: Option<DcpsDataWriterListener>,
        mask: Vec<StatusKind>,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
        reply_sender: OneshotSender<DdsResult<(InstanceHandle, ActorAddress<DcpsStatusCondition>)>>,
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
        dcps_listener: Option<DcpsPublisherListener>,
        mask: Vec<StatusKind>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
}

pub enum SubscriberServiceMail {
    CreateDataReader {
        subscriber_handle: InstanceHandle,
        topic_name: String,
        qos: QosKind<DataReaderQos>,
        dcps_listener: Option<DcpsDataReaderListener>,
        mask: Vec<StatusKind>,
        domain_participant_address: MpscSender<DcpsDomainParticipantMail>,
        reply_sender: OneshotSender<DdsResult<(InstanceHandle, ActorAddress<DcpsStatusCondition>)>>,
    },
    CreateDynamicDataReader {
        subscriber_handle: InstanceHandle,
        topic_name: String,
        dynamic_type: DynamicType,
        qos: QosKind<DataReaderQos>,
        domain_participant_address: MpscSender<DcpsDomainParticipantMail>,
        reply_sender: OneshotSender<DdsResult<(InstanceHandle, ActorAddress<DcpsStatusCondition>)>>,
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
            OneshotSender<DdsResult<Option<(InstanceHandle, ActorAddress<DcpsStatusCondition>)>>>,
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
        dcps_listener: Option<DcpsSubscriberListener>,
        mask: Vec<StatusKind>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
}

pub enum WriterServiceMail {
    SetListener {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dcps_listener: Option<DcpsDataWriterListener>,
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
        dynamic_data: DynamicData,
        timestamp: Time,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    LookupInstance {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData,
        reply_sender: OneshotSender<DdsResult<Option<InstanceHandle>>>,
    },
    WriteWTimestamp {
        participant_address: MpscSender<DcpsDomainParticipantMail>,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData,
        timestamp: Time,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    DisposeWTimestamp {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData,
        timestamp: Time,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetOfferedDeadlineMissedStatus {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<OfferedDeadlineMissedStatus>>,
    },
    EnableDataWriter {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    SetDataWriterQos {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        qos: QosKind<DataWriterQos>,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
}

pub enum ReaderServiceMail {
    Enable {
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
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
        reply_sender: OneshotSender<DdsResult<Vec<(Option<DynamicData>, SampleInfo)>>>,
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
        reply_sender: OneshotSender<DdsResult<Vec<(Option<DynamicData>, SampleInfo)>>>,
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
        reply_sender: OneshotSender<DdsResult<Vec<(Option<DynamicData>, SampleInfo)>>>,
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
        reply_sender: OneshotSender<DdsResult<Vec<(Option<DynamicData>, SampleInfo)>>>,
    },
    GetSubscriptionMatchedStatus {
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<SubscriptionMatchedStatus>>,
    },
    WaitForHistoricalData {
        participant_address: MpscSender<DcpsDomainParticipantMail>,
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
        participant_address: MpscSender<DcpsDomainParticipantMail>,
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
        dcps_listener: Option<DcpsDataReaderListener>,
        listener_mask: Vec<StatusKind>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
}

pub enum MessageServiceMail {
    AddCacheChange {
        participant_address: MpscSender<DcpsDomainParticipantMail>,
        cache_change: CacheChange,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    },
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
    AddBuiltinParticipantsDetectorCacheChange {
        cache_change: CacheChange,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    },
    AddBuiltinPublicationsDetectorCacheChange {
        cache_change: CacheChange,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    },
    AddBuiltinSubscriptionsDetectorCacheChange {
        cache_change: CacheChange,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    },
    AddBuiltinTopicsDetectorCacheChange {
        cache_change: CacheChange,
    },
    #[cfg(feature = "type_lookup")]
    AddTypeLookupRequestCacheChange {
        cache_change: CacheChange,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    },
    #[cfg(feature = "type_lookup")]
    AddTypeLookupReplyCacheChange {
        cache_change: CacheChange,
    },
    Poke,
}

pub enum EventServiceMail {
    OfferedDeadlineMissed {
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        change_instance_handle: InstanceHandle,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    },
    RequestedDeadlineMissed {
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        change_instance_handle: InstanceHandle,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    },
}

pub enum DiscoveryServiceMail {
    AnnounceParticipant {
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    },
    AnnounceDeletedParticipant,
    /// Periodic check to remove participants whose lease has expired
    CheckParticipantLiveness,
}

pub enum DcpsDomainParticipantMail {
    Participant(ParticipantServiceMail),
    Topic(TopicServiceMail),
    Publisher(PublisherServiceMail),
    Writer(WriterServiceMail),
    Subscriber(SubscriberServiceMail),
    Reader(ReaderServiceMail),
    Message(MessageServiceMail),
    Event(EventServiceMail),
    Discovery(DiscoveryServiceMail),
}

impl<R: DdsRuntime> MailHandler for DcpsDomainParticipant<R> {
    type Mail = DcpsDomainParticipantMail;
    async fn handle(&mut self, message: DcpsDomainParticipantMail) {
        match message {
            DcpsDomainParticipantMail::Participant(participant_service_mail) => {
                self.handle_participant_service(participant_service_mail)
                    .await
            }
            DcpsDomainParticipantMail::Topic(topic_service_mail) => {
                self.handle_topic_service(topic_service_mail).await
            }
            DcpsDomainParticipantMail::Publisher(publisher_service_mail) => {
                self.handle_publisher_service(publisher_service_mail).await
            }
            DcpsDomainParticipantMail::Writer(writer_service_mail) => {
                self.handle_writer_service(writer_service_mail).await
            }
            DcpsDomainParticipantMail::Subscriber(subscriber_service_mail) => {
                self.handle_subscriber_service(subscriber_service_mail)
                    .await
            }
            DcpsDomainParticipantMail::Reader(reader_service_mail) => {
                self.handle_reader_service(reader_service_mail).await
            }
            DcpsDomainParticipantMail::Message(message_service_mail) => {
                self.handle_message_service(message_service_mail).await
            }
            DcpsDomainParticipantMail::Event(event_service_mail) => {
                self.handle_event_service(event_service_mail).await
            }
            DcpsDomainParticipantMail::Discovery(discovery_service_mail) => {
                self.handle_discovery_service(discovery_service_mail).await
            }
        };
    }
}

impl<R: DdsRuntime> DcpsDomainParticipant<R> {
    async fn handle_participant_service(
        &mut self,
        participant_service_mail: ParticipantServiceMail,
    ) {
        match participant_service_mail {
            ParticipantServiceMail::CreateUserDefinedPublisher {
                qos,
                dcps_listener,
                mask,
                reply_sender,
            } => reply_sender.send(self.create_user_defined_publisher(qos, dcps_listener, mask)),
            ParticipantServiceMail::DeleteUserDefinedPublisher {
                participant_handle,
                publisher_handle,
                reply_sender,
            } => reply_sender
                .send(self.delete_user_defined_publisher(participant_handle, publisher_handle)),
            ParticipantServiceMail::CreateUserDefinedSubscriber {
                qos,
                dcps_listener,
                mask,
                reply_sender,
            } => reply_sender.send(self.create_user_defined_subscriber(qos, dcps_listener, mask)),
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
                dcps_listener,
                mask,
                type_support,
                participant_address,
                reply_sender,
            } => reply_sender.send(
                self.create_topic(
                    topic_name,
                    type_name,
                    qos,
                    dcps_listener,
                    mask,
                    type_support,
                    participant_address,
                )
                .await,
            ),
            ParticipantServiceMail::DeleteUserDefinedTopic {
                participant_handle,
                topic_name,
                reply_sender,
            } => reply_sender.send(self.delete_user_defined_topic(participant_handle, topic_name)),
            ParticipantServiceMail::CreateContentFilteredTopic {
                participant_handle,
                name,
                related_topic_name,
                filter_expression,
                expression_parameters,
                reply_sender,
            } => reply_sender.send(
                self.create_content_filtered_topic(
                    participant_handle,
                    name,
                    related_topic_name,
                    filter_expression,
                    expression_parameters,
                )
                .await,
            ),
            ParticipantServiceMail::DeleteContentFilteredTopic {
                participant_handle,
                name,
                reply_sender,
            } => reply_sender.send(self.delete_content_filtered_topic(participant_handle, name)),
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
            } => reply_sender.send(self.ignore_participant(handle).await),
            ParticipantServiceMail::IgnoreSubscription {
                handle,
                reply_sender,
            } => reply_sender.send(self.ignore_subscription(handle)),
            ParticipantServiceMail::IgnorePublication {
                handle,
                reply_sender,
            } => reply_sender.send(self.ignore_publication(handle)),
            ParticipantServiceMail::DeleteContainedEntities { reply_sender } => {
                reply_sender.send(self.delete_participant_contained_entities().await)
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
            ParticipantServiceMail::SetQos {
                qos,
                participant_address,
                reply_sender,
            } => reply_sender.send(
                self.set_domain_participant_qos(qos, participant_address)
                    .await,
            ),
            ParticipantServiceMail::GetQos { reply_sender } => {
                reply_sender.send(self.get_domain_participant_qos())
            }
            ParticipantServiceMail::SetListener {
                dcps_listener,
                status_kind,
                reply_sender,
            } => {
                reply_sender.send(self.set_domain_participant_listener(dcps_listener, status_kind))
            }
            ParticipantServiceMail::Enable {
                participant_address,
                reply_sender,
            } => reply_sender.send(self.enable_domain_participant(participant_address).await),
            ParticipantServiceMail::IsEmpty { reply_sender } => {
                reply_sender.send(self.is_participant_empty())
            }
            #[cfg(feature = "type_lookup")]
            ParticipantServiceMail::RequestTypeLookup {
                target_participant_prefix,
                type_ids,
                reply_sender,
            } => {
                self.send_type_lookup_request(target_participant_prefix, type_ids, reply_sender)
                    .await
            }
            #[cfg(feature = "type_lookup")]
            ParticipantServiceMail::GetDiscoveredWriterForTopic {
                topic_name,
                reply_sender,
            } => {
                let result = self.get_discovered_writer_for_topic(&topic_name);
                reply_sender.send(Ok(result))
            }
        }
    }

    async fn handle_topic_service(&mut self, topic_service_mail: TopicServiceMail) {
        match topic_service_mail {
            TopicServiceMail::GetInconsistentTopicStatus {
                topic_name,
                reply_sender,
            } => reply_sender.send(self.get_inconsistent_topic_status(topic_name).await),
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
                participant_address,
                reply_sender,
            } => reply_sender.send(self.enable_topic(topic_name, participant_address).await),
            TopicServiceMail::GetTypeSupport {
                topic_name,
                reply_sender,
            } => reply_sender.send(self.get_type_support(topic_name)),
        }
    }

    async fn handle_publisher_service(&mut self, publisher_service_mail: PublisherServiceMail) {
        match publisher_service_mail {
            PublisherServiceMail::CreateDataWriter {
                publisher_handle,
                topic_name,
                qos,
                dcps_listener,
                mask,
                participant_address,
                reply_sender,
            } => reply_sender.send(
                self.create_data_writer(
                    publisher_handle,
                    topic_name,
                    qos,
                    dcps_listener,
                    mask,
                    participant_address,
                )
                .await,
            ),
            PublisherServiceMail::DeleteDataWriter {
                publisher_handle,
                datawriter_handle,
                reply_sender,
            } => reply_sender.send(
                self.delete_data_writer(publisher_handle, datawriter_handle)
                    .await,
            ),
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
                dcps_listener,
                mask,
                reply_sender,
            } => reply_sender.send(self.set_publisher_listener(
                publisher_handle,
                dcps_listener,
                mask,
            )),
        }
    }

    async fn handle_writer_service(&mut self, writer_service_mail: WriterServiceMail) {
        match writer_service_mail {
            WriterServiceMail::SetListener {
                publisher_handle,
                data_writer_handle,
                dcps_listener,
                listener_mask,
                reply_sender,
            } => reply_sender.send(self.set_listener_data_writer(
                publisher_handle,
                data_writer_handle,
                dcps_listener,
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
            } => reply_sender.send(
                self.get_publication_matched_status(publisher_handle, data_writer_handle)
                    .await,
            ),
            WriterServiceMail::UnregisterInstance {
                publisher_handle,
                data_writer_handle,
                dynamic_data,
                timestamp,
                reply_sender,
            } => reply_sender.send(
                self.unregister_instance(
                    publisher_handle,
                    data_writer_handle,
                    dynamic_data,
                    timestamp,
                )
                .await,
            ),
            WriterServiceMail::LookupInstance {
                publisher_handle,
                data_writer_handle,
                dynamic_data,
                reply_sender,
            } => reply_sender.send(self.lookup_instance(
                publisher_handle,
                data_writer_handle,
                dynamic_data,
            )),
            WriterServiceMail::WriteWTimestamp {
                participant_address,
                publisher_handle,
                data_writer_handle,
                dynamic_data,
                timestamp,
                reply_sender,
            } => {
                self.write_w_timestamp(
                    participant_address,
                    publisher_handle,
                    data_writer_handle,
                    dynamic_data,
                    timestamp,
                    reply_sender,
                )
                .await
            }

            WriterServiceMail::DisposeWTimestamp {
                publisher_handle,
                data_writer_handle,
                dynamic_data,
                timestamp,
                reply_sender,
            } => reply_sender.send(
                self.dispose_w_timestamp(
                    publisher_handle,
                    data_writer_handle,
                    dynamic_data,
                    timestamp,
                )
                .await,
            ),
            WriterServiceMail::GetOfferedDeadlineMissedStatus {
                publisher_handle,
                data_writer_handle,
                reply_sender,
            } => reply_sender.send(
                self.get_offered_deadline_missed_status(publisher_handle, data_writer_handle)
                    .await,
            ),
            WriterServiceMail::EnableDataWriter {
                publisher_handle,
                data_writer_handle,
                participant_address,
                reply_sender,
            } => reply_sender.send(
                self.enable_data_writer(publisher_handle, data_writer_handle, participant_address)
                    .await,
            ),
            WriterServiceMail::SetDataWriterQos {
                publisher_handle,
                data_writer_handle,
                qos,
                participant_address,
                reply_sender,
            } => reply_sender.send(
                self.set_data_writer_qos(
                    publisher_handle,
                    data_writer_handle,
                    qos,
                    participant_address,
                )
                .await,
            ),
        }
    }

    async fn handle_subscriber_service(&mut self, subscriber_service_mail: SubscriberServiceMail) {
        match subscriber_service_mail {
            SubscriberServiceMail::CreateDataReader {
                subscriber_handle,
                topic_name,
                qos,
                dcps_listener,
                mask,
                domain_participant_address,
                reply_sender,
            } => reply_sender.send(
                self.create_data_reader(
                    subscriber_handle,
                    topic_name,
                    qos,
                    dcps_listener,
                    mask,
                    domain_participant_address,
                )
                .await,
            ),
            SubscriberServiceMail::CreateDynamicDataReader {
                subscriber_handle,
                topic_name,
                dynamic_type,
                qos,
                domain_participant_address,
                reply_sender,
            } => reply_sender.send(
                self.create_dynamic_data_reader(
                    subscriber_handle,
                    topic_name,
                    dynamic_type,
                    qos,
                    domain_participant_address,
                )
                .await,
            ),
            SubscriberServiceMail::DeleteDataReader {
                subscriber_handle,
                datareader_handle,
                reply_sender,
            } => reply_sender.send(
                self.delete_data_reader(subscriber_handle, datareader_handle)
                    .await,
            ),
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
                dcps_listener,
                mask,
                reply_sender,
            } => reply_sender.send(self.set_subscriber_listener(
                subscriber_handle,
                dcps_listener,
                mask,
            )),
        }
    }

    async fn handle_reader_service(&mut self, reader_service_mail: ReaderServiceMail) {
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
            } => reply_sender.send(
                self.read(
                    subscriber_handle,
                    data_reader_handle,
                    max_samples,
                    sample_states,
                    view_states,
                    instance_states,
                    specific_instance_handle,
                )
                .await,
            ),
            ReaderServiceMail::Take {
                subscriber_handle,
                data_reader_handle,
                max_samples,
                sample_states,
                view_states,
                instance_states,
                specific_instance_handle,
                reply_sender,
            } => reply_sender.send(
                self.take(
                    subscriber_handle,
                    data_reader_handle,
                    max_samples,
                    sample_states,
                    view_states,
                    instance_states,
                    specific_instance_handle,
                )
                .await,
            ),
            ReaderServiceMail::ReadNextInstance {
                subscriber_handle,
                data_reader_handle,
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
                reply_sender,
            } => reply_sender.send(
                self.read_next_instance(
                    subscriber_handle,
                    data_reader_handle,
                    max_samples,
                    previous_handle,
                    sample_states,
                    view_states,
                    instance_states,
                )
                .await,
            ),
            ReaderServiceMail::TakeNextInstance {
                subscriber_handle,
                data_reader_handle,
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
                reply_sender,
            } => reply_sender.send(
                self.take_next_instance(
                    subscriber_handle,
                    data_reader_handle,
                    max_samples,
                    previous_handle,
                    sample_states,
                    view_states,
                    instance_states,
                )
                .await,
            ),
            ReaderServiceMail::Enable {
                subscriber_handle,
                data_reader_handle,
                participant_address,
                reply_sender,
            } => reply_sender.send(
                self.enable_data_reader(subscriber_handle, data_reader_handle, participant_address)
                    .await,
            ),
            ReaderServiceMail::GetSubscriptionMatchedStatus {
                subscriber_handle,
                data_reader_handle,
                reply_sender,
            } => reply_sender.send(
                self.get_subscription_matched_status(subscriber_handle, data_reader_handle)
                    .await,
            ),
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
                participant_address,
                reply_sender,
            } => reply_sender.send(
                self.set_data_reader_qos(
                    subscriber_handle,
                    data_reader_handle,
                    qos,
                    participant_address,
                )
                .await,
            ),
            ReaderServiceMail::SetListener {
                subscriber_handle,
                data_reader_handle,
                dcps_listener,
                listener_mask,
                reply_sender,
            } => reply_sender.send(self.set_data_reader_listener(
                subscriber_handle,
                data_reader_handle,
                dcps_listener,
                listener_mask,
            )),
        }
    }

    async fn handle_message_service(&mut self, message_service_mail: MessageServiceMail) {
        match message_service_mail {
            MessageServiceMail::AddCacheChange {
                participant_address,
                cache_change,
                subscriber_handle,
                data_reader_handle,
            } => {
                self.add_cache_change(
                    participant_address,
                    cache_change,
                    subscriber_handle,
                    data_reader_handle,
                )
                .await
            }
            MessageServiceMail::RemoveWriterChange {
                publisher_handle,
                data_writer_handle,
                sequence_number,
            } => {
                self.remove_writer_change(publisher_handle, data_writer_handle, sequence_number)
                    .await
            }
            MessageServiceMail::AreAllChangesAcknowledged {
                publisher_handle,
                data_writer_handle,
                reply_sender,
            } => reply_sender.send(
                self.are_all_changes_acknowledged(publisher_handle, data_writer_handle)
                    .await,
            ),
            MessageServiceMail::IsHistoricalDataReceived {
                subscriber_handle,
                data_reader_handle,
                reply_sender,
            } => reply_sender.send(
                self.is_historical_data_received(subscriber_handle, data_reader_handle)
                    .await,
            ),
            MessageServiceMail::AddBuiltinParticipantsDetectorCacheChange {
                cache_change,
                participant_address,
            } => {
                self.add_builtin_participants_detector_cache_change(
                    cache_change,
                    participant_address,
                )
                .await
            }
            MessageServiceMail::AddBuiltinPublicationsDetectorCacheChange {
                cache_change,
                participant_address,
            } => {
                self.add_builtin_publications_detector_cache_change(
                    cache_change,
                    participant_address,
                )
                .await;
            }
            MessageServiceMail::AddBuiltinSubscriptionsDetectorCacheChange {
                cache_change,
                participant_address,
            } => {
                self.add_builtin_subscriptions_detector_cache_change(
                    cache_change,
                    participant_address,
                )
                .await
            }
            MessageServiceMail::AddBuiltinTopicsDetectorCacheChange { cache_change } => {
                self.add_builtin_topics_detector_cache_change(cache_change)
                    .await
            }
            #[cfg(feature = "type_lookup")]
            MessageServiceMail::AddTypeLookupRequestCacheChange {
                cache_change,
                participant_address,
            } => {
                self.handle_type_lookup_request(cache_change, participant_address)
                    .await
            }
            #[cfg(feature = "type_lookup")]
            MessageServiceMail::AddTypeLookupReplyCacheChange { cache_change } => {
                self.handle_type_lookup_reply(cache_change).await
            }
            MessageServiceMail::Poke => self.poke().await,
        }
    }

    async fn handle_event_service(&mut self, event_service_mail: EventServiceMail) {
        match event_service_mail {
            EventServiceMail::OfferedDeadlineMissed {
                publisher_handle,
                data_writer_handle,
                change_instance_handle,
                participant_address,
            } => {
                self.offered_deadline_missed(
                    publisher_handle,
                    data_writer_handle,
                    change_instance_handle,
                    participant_address,
                )
                .await
            }
            EventServiceMail::RequestedDeadlineMissed {
                subscriber_handle,
                data_reader_handle,
                change_instance_handle,
                participant_address,
            } => {
                self.requested_deadline_missed(
                    subscriber_handle,
                    data_reader_handle,
                    change_instance_handle,
                    participant_address,
                )
                .await
            }
        }
    }

    async fn handle_discovery_service(&mut self, discovery_service_mail: DiscoveryServiceMail) {
        match discovery_service_mail {
            DiscoveryServiceMail::AnnounceParticipant {
                participant_address,
            } => {
                self.announce_participant(participant_address).await;
            }
            DiscoveryServiceMail::AnnounceDeletedParticipant => {
                self.announce_deleted_participant().await;
            }
            DiscoveryServiceMail::CheckParticipantLiveness => {
                self.check_participant_liveness().await;
            }
        }
    }
}
