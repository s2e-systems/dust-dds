use alloc::{boxed::Box, string::String, sync::Arc};
use core::pin::Pin;

use crate::{
    builtin_topics::{
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    dcps::{
        actor::ActorAddress,
        channels::oneshot::OneshotSender,
        listeners::{
            data_reader_listener::DcpsDataReaderListener,
            data_writer_listener::DcpsDataWriterListener,
            domain_participant_listener::DcpsDomainParticipantListener,
            publisher_listener::DcpsPublisherListener, subscriber_listener::DcpsSubscriberListener,
            topic_listener::DcpsTopicListener,
        },
        status_condition::DcpsStatusCondition,
    },
    dds_async::configuration::DustDdsConfiguration,
    infrastructure::{
        domain::DomainId,
        error::DdsResult,
        instance::InstanceHandle,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantFactoryQos, DomainParticipantQos,
            PublisherQos, QosKind, SubscriberQos, TopicQos,
        },
        sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
        status::{
            InconsistentTopicStatus, OfferedDeadlineMissedStatus, PublicationMatchedStatus,
            StatusKind, SubscriptionMatchedStatus,
        },
        time::{Duration, Time},
    },
    xtypes::dynamic_type::{DynamicData, DynamicType},
};
use alloc::vec::Vec;

pub enum DcpsMail {
    ParticipantFactory(ParticipantFactoryMail),
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

pub enum ParticipantFactoryMail {
    CreateParticipant {
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        dcps_listener: Option<DcpsDomainParticipantListener>,
        status_kind: Vec<StatusKind>,
        reply_sender: OneshotSender<DdsResult<(InstanceHandle, ActorAddress<DcpsStatusCondition>)>>,
    },
    DeleteParticipant {
        participant_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    SetDefaultParticipantQos {
        qos: QosKind<DomainParticipantQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetDefaultParticipantQos {
        reply_sender: OneshotSender<DomainParticipantQos>,
    },
    SetQos {
        qos: QosKind<DomainParticipantFactoryQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetQos {
        reply_sender: OneshotSender<DomainParticipantFactoryQos>,
    },
    SetConfiguration {
        configuration: DustDdsConfiguration,
    },
    GetConfiguration {
        reply_sender: OneshotSender<DustDdsConfiguration>,
    },
}

pub enum ParticipantServiceMail {
    CreateUserDefinedPublisher {
        participant_handle: InstanceHandle,
        qos: QosKind<PublisherQos>,
        dcps_listener: Option<DcpsPublisherListener>,
        mask: Vec<StatusKind>,
        reply_sender: OneshotSender<DdsResult<InstanceHandle>>,
    },
    DeleteUserDefinedPublisher {
        participant_handle: InstanceHandle,
        parent_participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    CreateUserDefinedSubscriber {
        participant_handle: InstanceHandle,
        qos: QosKind<SubscriberQos>,
        dcps_listener: Option<DcpsSubscriberListener>,
        mask: Vec<StatusKind>,
        reply_sender: OneshotSender<DdsResult<(InstanceHandle, ActorAddress<DcpsStatusCondition>)>>,
    },
    DeleteUserDefinedSubscriber {
        participant_handle: InstanceHandle,
        parent_participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    CreateTopic {
        participant_handle: InstanceHandle,
        topic_name: String,
        type_name: String,
        qos: QosKind<TopicQos>,
        dcps_listener: Option<DcpsTopicListener>,
        mask: Vec<StatusKind>,
        type_support: Arc<DynamicType>,
        reply_sender: OneshotSender<DdsResult<(InstanceHandle, ActorAddress<DcpsStatusCondition>)>>,
    },
    DeleteUserDefinedTopic {
        participant_handle: InstanceHandle,
        parent_participant_handle: InstanceHandle,
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
        participant_handle: InstanceHandle,
        topic_name: String,
        type_support: Arc<DynamicType>,
        #[allow(clippy::type_complexity)]
        reply_sender: OneshotSender<
            DdsResult<Option<(InstanceHandle, ActorAddress<DcpsStatusCondition>, String)>>,
        >,
    },
    LookupTopicdescription {
        participant_handle: InstanceHandle,
        topic_name: String,
        #[allow(clippy::type_complexity)]
        reply_sender: OneshotSender<
            DdsResult<Option<(String, InstanceHandle, ActorAddress<DcpsStatusCondition>)>>,
        >,
    },
    IgnoreParticipant {
        participant_handle: InstanceHandle,
        handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    IgnoreSubscription {
        participant_handle: InstanceHandle,
        handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    IgnorePublication {
        participant_handle: InstanceHandle,
        handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    DeleteContainedEntities {
        participant_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    SetDefaultPublisherQos {
        participant_handle: InstanceHandle,
        qos: QosKind<PublisherQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetDefaultPublisherQos {
        participant_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<PublisherQos>>,
    },
    SetDefaultSubscriberQos {
        participant_handle: InstanceHandle,
        qos: QosKind<SubscriberQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetDefaultSubscriberQos {
        participant_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<SubscriberQos>>,
    },
    SetDefaultTopicQos {
        participant_handle: InstanceHandle,
        qos: QosKind<TopicQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetDefaultTopicQos {
        participant_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<TopicQos>>,
    },
    GetDiscoveredParticipants {
        participant_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<Vec<InstanceHandle>>>,
    },
    GetDiscoveredParticipantData {
        participant_handle: InstanceHandle,
        discovered_participant_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<ParticipantBuiltinTopicData>>,
    },
    GetDiscoveredTopics {
        participant_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<Vec<InstanceHandle>>>,
    },
    GetDiscoveredTopicData {
        participant_handle: InstanceHandle,
        topic_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<TopicBuiltinTopicData>>,
    },
    GetCurrentTime {
        participant_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<Time>>,
    },
    SetQos {
        participant_handle: InstanceHandle,
        qos: QosKind<DomainParticipantQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetQos {
        participant_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<DomainParticipantQos>>,
    },
    SetListener {
        participant_handle: InstanceHandle,
        dcps_listener: Option<DcpsDomainParticipantListener>,
        status_kind: Vec<StatusKind>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    Enable {
        participant_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
}

pub enum TopicServiceMail {
    GetInconsistentTopicStatus {
        participant_handle: InstanceHandle,
        topic_name: String,
        reply_sender: OneshotSender<DdsResult<InconsistentTopicStatus>>,
    },
    SetQos {
        participant_handle: InstanceHandle,
        topic_name: String,
        topic_qos: QosKind<TopicQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetQos {
        participant_handle: InstanceHandle,
        topic_name: String,
        reply_sender: OneshotSender<DdsResult<TopicQos>>,
    },
    Enable {
        participant_handle: InstanceHandle,
        topic_name: String,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetTypeSupport {
        participant_handle: InstanceHandle,
        topic_name: String,
        reply_sender: OneshotSender<DdsResult<Arc<DynamicType>>>,
    },
}

pub enum PublisherServiceMail {
    CreateDataWriter {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        topic_name: String,
        qos: QosKind<DataWriterQos>,
        dcps_listener: Option<DcpsDataWriterListener>,
        mask: Vec<StatusKind>,
        reply_sender: OneshotSender<DdsResult<(InstanceHandle, ActorAddress<DcpsStatusCondition>)>>,
    },
    DeleteDataWriter {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        datawriter_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetDefaultDataWriterQos {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<DataWriterQos>>,
    },
    SetDefaultDataWriterQos {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        qos: QosKind<DataWriterQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetPublisherQos {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<PublisherQos>>,
    },
    SetPublisherQos {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        qos: QosKind<PublisherQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    SetPublisherListener {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        dcps_listener: Option<DcpsPublisherListener>,
        mask: Vec<StatusKind>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
}

pub enum SubscriberServiceMail {
    CreateDataReader {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        topic_name: String,
        qos: QosKind<DataReaderQos>,
        dcps_listener: Option<DcpsDataReaderListener>,
        mask: Vec<StatusKind>,
        reply_sender: OneshotSender<DdsResult<(InstanceHandle, ActorAddress<DcpsStatusCondition>)>>,
    },
    DeleteDataReader {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        datareader_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    LookupDataReader {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        topic_name: String,
        #[allow(clippy::type_complexity)]
        reply_sender:
            OneshotSender<DdsResult<Option<(InstanceHandle, ActorAddress<DcpsStatusCondition>)>>>,
    },
    SetDefaultDataReaderQos {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        qos: QosKind<DataReaderQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetDefaultDataReaderQos {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<DataReaderQos>>,
    },
    SetQos {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        qos: QosKind<SubscriberQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetSubscriberQos {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<SubscriberQos>>,
    },
    SetListener {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        dcps_listener: Option<DcpsSubscriberListener>,
        mask: Vec<StatusKind>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
}

pub enum WriterServiceMail {
    SetListener {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dcps_listener: Option<DcpsDataWriterListener>,
        listener_mask: Vec<StatusKind>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetDataWriterQos {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<DataWriterQos>>,
    },
    GetMatchedSubscriptions {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<Vec<InstanceHandle>>>,
    },
    GetMatchedSubscriptionData {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        subscription_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<SubscriptionBuiltinTopicData>>,
    },
    GetPublicationMatchedStatus {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<PublicationMatchedStatus>>,
    },
    UnregisterInstance {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData,
        timestamp: Time,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    LookupInstance {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData,
        reply_sender: OneshotSender<DdsResult<Option<InstanceHandle>>>,
    },
    WriteWTimestamp {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData,
        timestamp: Time,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    DisposeWTimestamp {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData,
        timestamp: Time,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetOfferedDeadlineMissedStatus {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<OfferedDeadlineMissedStatus>>,
    },
    EnableDataWriter {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    SetDataWriterQos {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        qos: QosKind<DataWriterQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
}

pub enum ReaderServiceMail {
    Enable {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    Read {
        participant_handle: InstanceHandle,
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
        participant_handle: InstanceHandle,
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
        participant_handle: InstanceHandle,
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
        participant_handle: InstanceHandle,
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
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<SubscriptionMatchedStatus>>,
    },
    WaitForHistoricalData {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_wait: Duration,
        #[allow(clippy::type_complexity)]
        reply_sender: OneshotSender<DdsResult<Pin<Box<dyn Future<Output = DdsResult<()>> + Send>>>>,
    },
    GetMatchedPublicationData {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        publication_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<PublicationBuiltinTopicData>>,
    },
    GetMatchedPublications {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<Vec<InstanceHandle>>>,
    },
    SetQos {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        qos: QosKind<DataReaderQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetQos {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<DataReaderQos>>,
    },
    SetListener {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        dcps_listener: Option<DcpsDataReaderListener>,
        listener_mask: Vec<StatusKind>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
}

pub enum MessageServiceMail {
    RemoveWriterChange {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        sequence_number: i64,
    },
    AreAllChangesAcknowledged {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<bool>>,
    },
    IsHistoricalDataReceived {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<bool>>,
    },
    HandleData {
        participant_handle: InstanceHandle,
        data_message: Arc<[u8]>,
    },
    Poke {
        participant_handle: InstanceHandle,
    },
}

pub enum EventServiceMail {
    OfferedDeadlineMissed {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        change_instance_handle: InstanceHandle,
    },
    RequestedDeadlineMissed {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        change_instance_handle: InstanceHandle,
    },
}

pub enum DiscoveryServiceMail {
    AnnounceParticipant { participant_handle: InstanceHandle },
}
