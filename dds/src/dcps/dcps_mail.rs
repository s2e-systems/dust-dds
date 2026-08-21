use alloc::{string::String, vec::Vec};

use crate::{
    dcps::{
        channels::notification::NotificationSender,
        listeners::{
            data_reader_listener::DcpsDataReaderListener,
            data_writer_listener::DcpsDataWriterListener,
            domain_participant_listener::DcpsDomainParticipantListener,
            publisher_listener::DcpsPublisherListener, subscriber_listener::DcpsSubscriberListener,
            topic_listener::DcpsTopicListener,
        },
        status_condition::StatusConditionEntity,
        status_mask::StatusMask,
    },
    infrastructure::{
        domain::DomainId,
        instance::InstanceHandle,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantFactoryQos, DomainParticipantQos,
            PublisherQos, QosKind, SubscriberQos, TopicQos,
        },
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
        time::{Duration, Time},
    },
    transport::{interface::RtpsTransportParticipant, types::GuidPrefix},
    xtypes::dynamic_type::{DynamicData, DynamicType},
};

pub enum DcpsMail {
    ParticipantFactory(ParticipantFactoryMail),
    Participant(ParticipantServiceMail),
    Topic(TopicServiceMail),
    Publisher(PublisherServiceMail),
    Writer(WriterServiceMail),
    Subscriber(SubscriberServiceMail),
    Reader(ReaderServiceMail),
    StatusCondition(StatusConditionMail),
    Message(MessageServiceMail),
}

#[allow(clippy::large_enum_variant)]
pub enum ParticipantFactoryMail {
    CreateParticipant {
        guid_prefix: GuidPrefix,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        dcps_listener: Option<DcpsDomainParticipantListener>,
        listener_mask: StatusMask,
        transport_participant: RtpsTransportParticipant,
        domain_tag: String,
        participant_announcement_interval: core::time::Duration,
    },
    DeleteParticipant {
        participant_handle: InstanceHandle,
    },
    SetDefaultParticipantQos {
        qos: QosKind<DomainParticipantQos>,
    },
    GetDefaultParticipantQos {},
    SetQos {
        qos: QosKind<DomainParticipantFactoryQos>,
    },
    GetQos {},
    LookupParticipant {
        domain_id: DomainId,
    },
}

pub enum ParticipantServiceMail {
    CreateUserDefinedPublisher {
        participant_handle: InstanceHandle,
        qos: QosKind<PublisherQos>,
        dcps_listener: Option<DcpsPublisherListener>,
        listener_mask: StatusMask,
    },
    DeleteUserDefinedPublisher {
        participant_handle: InstanceHandle,
        parent_participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
    },
    CreateUserDefinedSubscriber {
        participant_handle: InstanceHandle,
        qos: QosKind<SubscriberQos>,
        dcps_listener: Option<DcpsSubscriberListener>,
        listener_mask: StatusMask,
    },
    DeleteUserDefinedSubscriber {
        participant_handle: InstanceHandle,
        parent_participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
    },
    CreateTopic {
        participant_handle: InstanceHandle,
        topic_name: String,
        type_name: String,
        qos: QosKind<TopicQos>,
        dcps_listener: Option<DcpsTopicListener>,
        listener_mask: StatusMask,
        type_support: DynamicType<'static>,
    },
    DeleteUserDefinedTopic {
        participant_handle: InstanceHandle,
        parent_participant_handle: InstanceHandle,
        topic_name: String,
    },
    CreateContentFilteredTopic {
        participant_handle: InstanceHandle,
        name: String,
        related_topic_name: String,
        filter_expression: String,
        expression_parameters: Vec<String>,
    },
    DeleteContentFilteredTopic {
        participant_handle: InstanceHandle,
        name: String,
    },
    FindTopic {
        participant_handle: InstanceHandle,
        topic_name: String,
        type_support: DynamicType<'static>,
        timeout: Duration,
    },
    LookupTopicdescription {
        participant_handle: InstanceHandle,
        topic_name: String,
    },
    IgnoreParticipant {
        participant_handle: InstanceHandle,
        handle: InstanceHandle,
    },
    IgnoreSubscription {
        participant_handle: InstanceHandle,
        handle: InstanceHandle,
    },
    IgnorePublication {
        participant_handle: InstanceHandle,
        handle: InstanceHandle,
    },
    DeleteContainedEntities {
        participant_handle: InstanceHandle,
    },
    SetDefaultPublisherQos {
        participant_handle: InstanceHandle,
        qos: QosKind<PublisherQos>,
    },
    GetDefaultPublisherQos {
        participant_handle: InstanceHandle,
    },
    SetDefaultSubscriberQos {
        participant_handle: InstanceHandle,
        qos: QosKind<SubscriberQos>,
    },
    GetDefaultSubscriberQos {
        participant_handle: InstanceHandle,
    },
    SetDefaultTopicQos {
        participant_handle: InstanceHandle,
        qos: QosKind<TopicQos>,
    },
    GetDefaultTopicQos {
        participant_handle: InstanceHandle,
    },
    GetDiscoveredParticipants {
        participant_handle: InstanceHandle,
    },
    GetDiscoveredParticipantData {
        participant_handle: InstanceHandle,
        discovered_participant_handle: InstanceHandle,
    },
    GetDiscoveredTopics {
        participant_handle: InstanceHandle,
    },
    GetDiscoveredTopicData {
        participant_handle: InstanceHandle,
        topic_handle: InstanceHandle,
    },
    GetCurrentTime {
        participant_handle: InstanceHandle,
    },
    SetQos {
        participant_handle: InstanceHandle,
        qos: QosKind<DomainParticipantQos>,
    },
    GetQos {
        participant_handle: InstanceHandle,
    },
    SetListener {
        participant_handle: InstanceHandle,
        dcps_listener: Option<DcpsDomainParticipantListener>,
        listener_mask: StatusMask,
    },
    Enable {
        participant_handle: InstanceHandle,
    },
}

pub enum TopicServiceMail {
    GetInconsistentTopicStatus {
        participant_handle: InstanceHandle,
        topic_name: String,
    },
    SetQos {
        participant_handle: InstanceHandle,
        topic_name: String,
        topic_qos: QosKind<TopicQos>,
    },
    GetQos {
        participant_handle: InstanceHandle,
        topic_name: String,
    },
    Enable {
        participant_handle: InstanceHandle,
        topic_name: String,
    },
    GetTypeSupport {
        participant_handle: InstanceHandle,
        topic_name: String,
    },
}

pub enum PublisherServiceMail {
    CreateDataWriter {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        topic_name: String,
        qos: QosKind<DataWriterQos>,
        dcps_listener: Option<DcpsDataWriterListener>,
        listener_mask: StatusMask,
    },
    DeleteDataWriter {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        datawriter_handle: InstanceHandle,
    },
    GetDefaultDataWriterQos {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
    },
    SetDefaultDataWriterQos {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        qos: QosKind<DataWriterQos>,
    },
    GetPublisherQos {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
    },
    SetPublisherQos {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        qos: QosKind<PublisherQos>,
    },
    SetPublisherListener {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        dcps_listener: Option<DcpsPublisherListener>,
        listener_mask: StatusMask,
    },
}

pub enum SubscriberServiceMail {
    CreateDataReader {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        topic_name: String,
        qos: QosKind<DataReaderQos>,
        dcps_listener: Option<DcpsDataReaderListener>,
        listener_mask: StatusMask,
    },
    DeleteDataReader {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        datareader_handle: InstanceHandle,
    },
    LookupDataReader {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        topic_name: String,
    },
    SetDefaultDataReaderQos {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        qos: QosKind<DataReaderQos>,
    },
    GetDefaultDataReaderQos {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
    },
    SetQos {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        qos: QosKind<SubscriberQos>,
    },
    GetSubscriberQos {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
    },
    SetListener {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        dcps_listener: Option<DcpsSubscriberListener>,
        listener_mask: StatusMask,
    },
}

pub enum WriterServiceMail {
    SetListener {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dcps_listener: Option<DcpsDataWriterListener>,
        listener_mask: StatusMask,
    },
    GetDataWriterQos {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    },
    GetMatchedSubscriptions {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    },
    GetMatchedSubscriptionData {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        subscription_handle: InstanceHandle,
    },
    GetPublicationMatchedStatus {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    },
    RegisterInstance {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData<'static>,
        timestamp: Option<Time>,
    },
    UnregisterInstance {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData<'static>,
        timestamp: Option<Time>,
    },
    LookupInstance {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData<'static>,
    },
    WriteWTimestamp {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData<'static>,
        timestamp: Option<Time>,
    },
    DisposeWTimestamp {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData<'static>,
        timestamp: Option<Time>,
    },
    GetOfferedDeadlineMissedStatus {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    },
    EnableDataWriter {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    },
    SetDataWriterQos {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        qos: QosKind<DataWriterQos>,
    },
}

pub enum ReaderServiceMail {
    Enable {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
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
    },
    GetSubscriptionMatchedStatus {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    },
    GetMatchedPublicationData {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        publication_handle: InstanceHandle,
    },
    GetMatchedPublications {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    },
    SetQos {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        qos: QosKind<DataReaderQos>,
    },
    GetQos {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    },
    SetListener {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        dcps_listener: Option<DcpsDataReaderListener>,
        listener_mask: StatusMask,
    },
}

pub enum StatusConditionMail {
    GetStatusConditionEnabledStatuses {
        entity: StatusConditionEntity,
    },
    SetStatusConditionEnabledStatuses {
        entity: StatusConditionEntity,
        status_mask: StatusMask,
    },
    GetStatusConditionTriggerValue {
        entity: StatusConditionEntity,
    },
    RegisterNotification {
        entity: StatusConditionEntity,
        notification_sender: NotificationSender,
    },
}

pub enum MessageServiceMail {
    NotifyAcknowledgments {
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        notification_sender: NotificationSender,
    },
    NotifyHistoricalData {
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        notification_sender: NotificationSender,
    },
    HandleData {
        participant_handle: InstanceHandle,
        data_message: Vec<u8>,
    },
}
