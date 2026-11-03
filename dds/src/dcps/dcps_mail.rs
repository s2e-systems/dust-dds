use alloc::{boxed::Box, string::String, vec::Vec};

use crate::{
    builtin_topics::{
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    dcps::{
        channels::{
            notification::NotificationSender,
            oneshot::{OneshotReceiver, OneshotSender},
            rpc::{Operation, RpcClient},
        },
        dcps_domain_participant::data_reader_entity::SampleList,
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
        error::DdsResult,
        instance::InstanceHandle,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantFactoryQos, DomainParticipantQos,
            PublisherQos, QosKind, SubscriberQos, TopicQos,
        },
        sample_info::{InstanceStateMask, SampleStateMask, ViewStateMask},
        status::{
            InconsistentTopicStatus, OfferedDeadlineMissedStatus, PublicationMatchedStatus,
            SubscriptionMatchedStatus,
        },
        time::{Duration, Time},
    },
    xtypes::dynamic_type::{DynamicData, DynamicType},
};

pub enum DomainParticipantFactoryMail {
    CreateParticipant(Box<CreateParticipantMail>),
    DeleteParticipant {
        participant_handle: InstanceHandle,
    },
    SetDefaultParticipantQos {
        qos: Box<QosKind<DomainParticipantQos>>,
    },
    GetDefaultParticipantQos,
    SetQos {
        qos: Box<QosKind<DomainParticipantFactoryQos>>,
    },
    GetQos,
    LookupParticipant {
        domain_id: DomainId,
    },
}

pub struct CreateParticipantMail {
    pub domain_id: DomainId,
    pub qos: QosKind<DomainParticipantQos>,
    pub dcps_listener: Option<DcpsDomainParticipantListener>,
    pub listener_mask: StatusMask,
}

// =========================================================================
// Standalone Operation Structs
// =========================================================================

// --- Participant Operations ---

pub struct CreateUserDefinedPublisherMail {
    pub participant_handle: InstanceHandle,
    pub qos: QosKind<PublisherQos>,
    pub dcps_listener: Option<DcpsPublisherListener>,
    pub listener_mask: StatusMask,
}

pub struct DeleteUserDefinedPublisherMail {
    pub participant_handle: InstanceHandle,
    pub parent_participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
}

pub struct CreateUserDefinedSubscriberMail {
    pub participant_handle: InstanceHandle,
    pub qos: QosKind<SubscriberQos>,
    pub dcps_listener: Option<DcpsSubscriberListener>,
    pub listener_mask: StatusMask,
}

pub struct DeleteUserDefinedSubscriberMail {
    pub participant_handle: InstanceHandle,
    pub parent_participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
}

pub struct CreateTopicMail {
    pub participant_handle: InstanceHandle,
    pub topic_name: String,
    pub type_name: String,
    pub qos: QosKind<TopicQos>,
    pub dcps_listener: Option<DcpsTopicListener>,
    pub listener_mask: StatusMask,
    pub type_support: DynamicType<'static>,
}

pub struct DeleteUserDefinedTopicMail {
    pub participant_handle: InstanceHandle,
    pub parent_participant_handle: InstanceHandle,
    pub topic_name: String,
}

pub struct CreateContentFilteredTopicMail {
    pub participant_handle: InstanceHandle,
    pub name: String,
    pub related_topic_name: String,
    pub filter_expression: String,
    pub expression_parameters: Vec<String>,
}

pub struct DeleteContentFilteredTopicMail {
    pub participant_handle: InstanceHandle,
    pub name: String,
}

pub struct FindTopicMail {
    pub participant_handle: InstanceHandle,
    pub topic_name: String,
    pub type_support: DynamicType<'static>,
    pub timeout: Duration,
    pub reply_sender: OneshotSender<DdsResult<(InstanceHandle, String)>>,
}

pub struct LookupTopicdescriptionMail {
    pub participant_handle: InstanceHandle,
    pub topic_name: String,
}

pub struct IgnoreParticipantMail {
    pub participant_handle: InstanceHandle,
    pub handle: InstanceHandle,
}

pub struct IgnoreSubscriptionMail {
    pub participant_handle: InstanceHandle,
    pub handle: InstanceHandle,
}

pub struct IgnorePublicationMail {
    pub participant_handle: InstanceHandle,
    pub handle: InstanceHandle,
}

pub struct DeleteContainedEntitiesMail {
    pub participant_handle: InstanceHandle,
}

pub struct SetDefaultPublisherQosMail {
    pub participant_handle: InstanceHandle,
    pub qos: Box<QosKind<PublisherQos>>,
}

pub struct GetDefaultPublisherQosMail {
    pub participant_handle: InstanceHandle,
}

pub struct SetDefaultSubscriberQosMail {
    pub participant_handle: InstanceHandle,
    pub qos: Box<QosKind<SubscriberQos>>,
}

pub struct GetDefaultSubscriberQosMail {
    pub participant_handle: InstanceHandle,
}

pub struct SetDefaultTopicQosMail {
    pub participant_handle: InstanceHandle,
    pub qos: Box<QosKind<TopicQos>>,
}

pub struct GetDefaultTopicQosMail {
    pub participant_handle: InstanceHandle,
}

pub struct GetDiscoveredParticipantsMail {
    pub participant_handle: InstanceHandle,
}

pub struct GetDiscoveredParticipantDataMail {
    pub participant_handle: InstanceHandle,
    pub discovered_participant_handle: InstanceHandle,
}

pub struct GetDiscoveredTopicsMail {
    pub participant_handle: InstanceHandle,
}

pub struct GetDiscoveredTopicDataMail {
    pub participant_handle: InstanceHandle,
    pub topic_handle: InstanceHandle,
}

pub struct GetCurrentTimeMail {
    pub participant_handle: InstanceHandle,
}

pub struct SetParticipantQosMail {
    pub participant_handle: InstanceHandle,
    pub qos: Box<QosKind<DomainParticipantQos>>,
}

pub struct GetParticipantQosMail {
    pub participant_handle: InstanceHandle,
}

pub struct SetParticipantListenerMail {
    pub participant_handle: InstanceHandle,
    pub dcps_listener: Option<DcpsDomainParticipantListener>,
    pub listener_mask: StatusMask,
}

pub struct EnableParticipantMail {
    pub participant_handle: InstanceHandle,
}

pub struct DeleteParticipantMail;

// --- Topic Operations ---

pub struct GetInconsistentTopicStatusMail {
    pub participant_handle: InstanceHandle,
    pub topic_name: String,
}

pub struct SetTopicQosMail {
    pub participant_handle: InstanceHandle,
    pub topic_name: String,
    pub topic_qos: Box<QosKind<TopicQos>>,
}

pub struct GetTopicQosMail {
    pub participant_handle: InstanceHandle,
    pub topic_name: String,
}

pub struct EnableTopicMail {
    pub participant_handle: InstanceHandle,
    pub topic_name: String,
}

pub struct GetTopicTypeSupportMail {
    pub participant_handle: InstanceHandle,
    pub topic_name: String,
}

// --- Publisher Operations ---

pub struct CreateDataWriterMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub topic_name: String,
    pub qos: QosKind<DataWriterQos>,
    pub dcps_listener: Option<DcpsDataWriterListener>,
    pub listener_mask: StatusMask,
}

pub struct DeleteDataWriterMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub datawriter_handle: InstanceHandle,
}

pub struct GetDefaultDataWriterQosMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
}

pub struct SetDefaultDataWriterQosMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub qos: Box<QosKind<DataWriterQos>>,
}

pub struct GetPublisherQosMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
}

pub struct SetPublisherQosMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub qos: Box<QosKind<PublisherQos>>,
}

pub struct SetPublisherListenerMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub dcps_listener: Option<DcpsPublisherListener>,
    pub listener_mask: StatusMask,
}

// --- Subscriber Operations ---

pub struct CreateDataReaderMail {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
    pub topic_name: String,
    pub qos: QosKind<DataReaderQos>,
    pub dcps_listener: Option<DcpsDataReaderListener>,
    pub listener_mask: StatusMask,
}

pub struct DeleteDataReaderMail {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
    pub datareader_handle: InstanceHandle,
}

pub struct LookupDataReaderMail {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
    pub topic_name: String,
}

pub struct SetDefaultDataReaderQosMail {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
    pub qos: Box<QosKind<DataReaderQos>>,
}

pub struct GetDefaultDataReaderQosMail {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
}

pub struct SetSubscriberQosMail {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
    pub qos: Box<QosKind<SubscriberQos>>,
}

pub struct GetSubscriberQosMail {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
}

pub struct SetSubscriberListenerMail {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
    pub dcps_listener: Option<DcpsSubscriberListener>,
    pub listener_mask: StatusMask,
}

// --- Writer Operations ---

/// Result of a write operation, which may either complete immediately or return a pending write receiver.
#[derive(Debug)]
pub enum WriteReply {
    /// The write completed immediately.
    Ok,
    /// The write is pending acknowledgment or queue availability and must be awaited.
    Pending(OneshotReceiver<DdsResult<()>>),
}

pub struct WriteWTimestampMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub dynamic_data: DynamicData<'static>,
    pub timestamp: Option<Time>,
}

pub struct DisposeWTimestampMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub dynamic_data: DynamicData<'static>,
    pub timestamp: Option<Time>,
}

pub struct RegisterInstanceMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub dynamic_data: DynamicData<'static>,
    pub timestamp: Option<Time>,
}

pub struct UnregisterInstanceMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub dynamic_data: DynamicData<'static>,
    pub timestamp: Option<Time>,
}

pub struct LookupInstanceMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub dynamic_data: DynamicData<'static>,
}

pub struct GetDataWriterQosMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}

pub struct SetDataWriterQosMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub qos: Box<QosKind<DataWriterQos>>,
}

pub struct GetMatchedSubscriptionsMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}

pub struct GetMatchedSubscriptionDataMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub subscription_handle: InstanceHandle,
}

pub struct GetPublicationMatchedStatusMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}

pub struct GetOfferedDeadlineMissedStatusMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}

pub struct EnableDataWriterMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}

pub struct SetDataWriterListenerMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub dcps_listener: Option<DcpsDataWriterListener>,
    pub listener_mask: StatusMask,
}

// --- Reader Operations ---

pub struct ReadMail {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub max_samples: i32,
    pub sample_state_mask: SampleStateMask,
    pub view_state_mask: ViewStateMask,
    pub instance_state_mask: InstanceStateMask,
    pub specific_instance_handle: Option<InstanceHandle>,
}

pub struct TakeMail(pub ReadMail);

pub struct ReadNextInstanceMail {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub max_samples: i32,
    pub previous_handle: Option<InstanceHandle>,
    pub sample_state_mask: SampleStateMask,
    pub view_state_mask: ViewStateMask,
    pub instance_state_mask: InstanceStateMask,
}

pub struct TakeNextInstanceMail(pub ReadNextInstanceMail);

pub struct EnableDataReaderMail {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}

pub struct GetSubscriptionMatchedStatusMail {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}

pub struct GetMatchedPublicationDataMail {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub publication_handle: InstanceHandle,
}

pub struct GetMatchedPublicationsMail {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}

pub struct SetDataReaderQosMail {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub qos: Box<QosKind<DataReaderQos>>,
}

pub struct GetDataReaderQosMail {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}

pub struct SetDataReaderListenerMail {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub dcps_listener: Option<DcpsDataReaderListener>,
    pub listener_mask: StatusMask,
}

// --- StatusCondition Operations ---

pub struct GetStatusConditionEnabledStatusesMail {
    pub entity: StatusConditionEntity,
}

pub struct SetStatusConditionEnabledStatusesMail {
    pub entity: StatusConditionEntity,
    pub status_mask: StatusMask,
}

pub struct GetStatusConditionTriggerValueMail {
    pub entity: StatusConditionEntity,
}

pub struct RegisterStatusConditionNotificationMail {
    pub entity: StatusConditionEntity,
    pub notification_sender: NotificationSender,
}

// --- Message Operations ---

pub struct NotifyAcknowledgmentsMail {
    pub participant_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}

pub struct NotifyHistoricalDataMail {
    pub participant_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}

// =========================================================================
// Flat DomainParticipantMail (Single Envelope Enum)
// =========================================================================

pub enum DomainParticipantMail {
    // Participant
    CreateUserDefinedPublisher(CreateUserDefinedPublisherMail),
    DeleteUserDefinedPublisher(DeleteUserDefinedPublisherMail),
    CreateUserDefinedSubscriber(CreateUserDefinedSubscriberMail),
    DeleteUserDefinedSubscriber(DeleteUserDefinedSubscriberMail),
    CreateTopic(Box<CreateTopicMail>),
    DeleteUserDefinedTopic(DeleteUserDefinedTopicMail),
    CreateContentFilteredTopic(Box<CreateContentFilteredTopicMail>),
    DeleteContentFilteredTopic(DeleteContentFilteredTopicMail),
    FindTopic(FindTopicMail),
    LookupTopicdescription(LookupTopicdescriptionMail),
    IgnoreParticipant(IgnoreParticipantMail),
    IgnoreSubscription(IgnoreSubscriptionMail),
    IgnorePublication(IgnorePublicationMail),
    DeleteContainedEntities(DeleteContainedEntitiesMail),
    SetDefaultPublisherQos(SetDefaultPublisherQosMail),
    GetDefaultPublisherQos(GetDefaultPublisherQosMail),
    SetDefaultSubscriberQos(SetDefaultSubscriberQosMail),
    GetDefaultSubscriberQos(GetDefaultSubscriberQosMail),
    SetDefaultTopicQos(SetDefaultTopicQosMail),
    GetDefaultTopicQos(GetDefaultTopicQosMail),
    GetDiscoveredParticipants(GetDiscoveredParticipantsMail),
    GetDiscoveredParticipantData(GetDiscoveredParticipantDataMail),
    GetDiscoveredTopics(GetDiscoveredTopicsMail),
    GetDiscoveredTopicData(GetDiscoveredTopicDataMail),
    GetCurrentTime(GetCurrentTimeMail),
    SetParticipantQos(SetParticipantQosMail),
    GetParticipantQos(GetParticipantQosMail),
    SetParticipantListener(SetParticipantListenerMail),
    EnableParticipant(EnableParticipantMail),
    DeleteParticipant,

    // Topic
    GetInconsistentTopicStatus(GetInconsistentTopicStatusMail),
    SetTopicQos(SetTopicQosMail),
    GetTopicQos(GetTopicQosMail),
    EnableTopic(EnableTopicMail),
    GetTopicTypeSupport(GetTopicTypeSupportMail),

    // Publisher
    CreateDataWriter(Box<CreateDataWriterMail>),
    DeleteDataWriter(DeleteDataWriterMail),
    GetDefaultDataWriterQos(GetDefaultDataWriterQosMail),
    SetDefaultDataWriterQos(SetDefaultDataWriterQosMail),
    GetPublisherQos(GetPublisherQosMail),
    SetPublisherQos(SetPublisherQosMail),
    SetPublisherListener(SetPublisherListenerMail),

    // Subscriber
    CreateDataReader(Box<CreateDataReaderMail>),
    DeleteDataReader(DeleteDataReaderMail),
    LookupDataReader(LookupDataReaderMail),
    SetDefaultDataReaderQos(SetDefaultDataReaderQosMail),
    GetDefaultDataReaderQos(GetDefaultDataReaderQosMail),
    SetSubscriberQos(SetSubscriberQosMail),
    GetSubscriberQos(GetSubscriberQosMail),
    SetSubscriberListener(SetSubscriberListenerMail),

    // Writer
    WriteWTimestamp(WriteWTimestampMail),
    DisposeWTimestamp(DisposeWTimestampMail),
    RegisterInstance(RegisterInstanceMail),
    UnregisterInstance(UnregisterInstanceMail),
    LookupInstance(LookupInstanceMail),
    GetDataWriterQos(GetDataWriterQosMail),
    SetDataWriterQos(SetDataWriterQosMail),
    GetMatchedSubscriptions(GetMatchedSubscriptionsMail),
    GetMatchedSubscriptionData(GetMatchedSubscriptionDataMail),
    GetPublicationMatchedStatus(GetPublicationMatchedStatusMail),
    GetOfferedDeadlineMissedStatus(GetOfferedDeadlineMissedStatusMail),
    EnableDataWriter(EnableDataWriterMail),
    SetDataWriterListener(SetDataWriterListenerMail),

    // Reader
    Read(ReadMail),
    Take(ReadMail),
    ReadNextInstance(ReadNextInstanceMail),
    TakeNextInstance(ReadNextInstanceMail),
    EnableDataReader(EnableDataReaderMail),
    GetSubscriptionMatchedStatus(GetSubscriptionMatchedStatusMail),
    GetMatchedPublicationData(GetMatchedPublicationDataMail),
    GetMatchedPublications(GetMatchedPublicationsMail),
    SetDataReaderQos(SetDataReaderQosMail),
    GetDataReaderQos(GetDataReaderQosMail),
    SetDataReaderListener(SetDataReaderListenerMail),

    // StatusCondition
    GetStatusConditionEnabledStatuses(GetStatusConditionEnabledStatusesMail),
    SetStatusConditionEnabledStatuses(SetStatusConditionEnabledStatusesMail),
    GetStatusConditionTriggerValue(GetStatusConditionTriggerValueMail),
    RegisterStatusConditionNotification(RegisterStatusConditionNotificationMail),

    // Message
    NotifyAcknowledgments(NotifyAcknowledgmentsMail),
    NotifyHistoricalData(NotifyHistoricalDataMail),
}

// =========================================================================
// Internal DcpsReply (Pre-Allocated Storage for In-Place Mailbox)
// =========================================================================

#[derive(Debug)]
pub enum DcpsReply {
    Ok(DdsResult<()>),
    PendingWrite(OneshotReceiver<DdsResult<()>>),
    InstanceHandle(DdsResult<InstanceHandle>),
    OptionInstanceHandle(DdsResult<Option<InstanceHandle>>),
    ParticipantQos(DdsResult<DomainParticipantQos>),
    PublisherQos(DdsResult<PublisherQos>),
    SubscriberQos(DdsResult<SubscriberQos>),
    TopicQos(DdsResult<Box<TopicQos>>),
    DataWriterQos(DdsResult<Box<DataWriterQos>>),
    DataReaderQos(DdsResult<Box<DataReaderQos>>),
    InstanceHandleList(DdsResult<Vec<InstanceHandle>>),
    ParticipantBuiltinTopicData(DdsResult<Box<ParticipantBuiltinTopicData>>),
    TopicBuiltinTopicData(DdsResult<Box<TopicBuiltinTopicData>>),
    PublicationBuiltinTopicData(DdsResult<Box<PublicationBuiltinTopicData>>),
    SubscriptionBuiltinTopicData(DdsResult<Box<SubscriptionBuiltinTopicData>>),
    InconsistentTopicStatus(DdsResult<InconsistentTopicStatus>),
    PublicationMatchedStatus(DdsResult<PublicationMatchedStatus>),
    SubscriptionMatchedStatus(DdsResult<SubscriptionMatchedStatus>),
    OfferedDeadlineMissedStatus(DdsResult<OfferedDeadlineMissedStatus>),
    Time(DdsResult<Time>),
    DynamicType(DdsResult<DynamicType<'static>>),
    TopicDescription(DdsResult<Option<String>>),
    StatusMask(DdsResult<StatusMask>),
    TriggerValue(DdsResult<bool>),
    Samples(DdsResult<(DynamicType<'static>, SampleList)>),
    CreatedParticipant(DdsResult<(InstanceHandle, RpcClient<DomainParticipantMail>)>),
    FactoryQos(DdsResult<DomainParticipantFactoryQos>),
    OptionCreatedParticipant(DdsResult<Option<(InstanceHandle, RpcClient<DomainParticipantMail>)>>),
}

impl DcpsReply {
    pub(crate) fn expect_ok(self) -> DdsResult<()> {
        match self {
            DcpsReply::Ok(r) => r,
            _ => panic!("expected Ok reply"),
        }
    }

    pub(crate) fn expect_created_participant(
        self,
    ) -> DdsResult<(InstanceHandle, RpcClient<DomainParticipantMail>)> {
        match self {
            DcpsReply::CreatedParticipant(r) => r,
            _ => panic!("expected CreatedParticipant reply"),
        }
    }

    pub(crate) fn expect_option_created_participant(
        self,
    ) -> DdsResult<Option<(InstanceHandle, RpcClient<DomainParticipantMail>)>> {
        match self {
            DcpsReply::OptionCreatedParticipant(r) => r,
            _ => panic!("expected OptionCreatedParticipant reply"),
        }
    }

    pub(crate) fn expect_participant_qos(self) -> DdsResult<DomainParticipantQos> {
        match self {
            DcpsReply::ParticipantQos(r) => r,
            _ => panic!("expected ParticipantQos reply"),
        }
    }

    pub(crate) fn expect_factory_qos(self) -> DdsResult<DomainParticipantFactoryQos> {
        match self {
            DcpsReply::FactoryQos(r) => r,
            _ => panic!("expected FactoryQos reply"),
        }
    }
}

// =========================================================================
// Operation Trait Implementations
// =========================================================================

// --- Participant Operations ---

impl Operation for CreateUserDefinedPublisherMail {
    type Output = InstanceHandle;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::CreateUserDefinedPublisher(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::InstanceHandle(res) => res,
            other => panic!("expected InstanceHandle reply, got {:?}", other),
        }
    }
}

impl Operation for DeleteUserDefinedPublisherMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::DeleteUserDefinedPublisher(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for CreateUserDefinedSubscriberMail {
    type Output = InstanceHandle;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::CreateUserDefinedSubscriber(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::InstanceHandle(res) => res,
            other => panic!("expected InstanceHandle reply, got {:?}", other),
        }
    }
}

impl Operation for DeleteUserDefinedSubscriberMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::DeleteUserDefinedSubscriber(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for CreateTopicMail {
    type Output = InstanceHandle;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::CreateTopic(Box::new(self))
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::InstanceHandle(res) => res,
            other => panic!("expected InstanceHandle reply, got {:?}", other),
        }
    }
}

impl Operation for DeleteUserDefinedTopicMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::DeleteUserDefinedTopic(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for CreateContentFilteredTopicMail {
    type Output = InstanceHandle;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::CreateContentFilteredTopic(Box::new(self))
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::InstanceHandle(res) => res,
            other => panic!("expected InstanceHandle reply, got {:?}", other),
        }
    }
}

impl Operation for DeleteContentFilteredTopicMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::DeleteContentFilteredTopic(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for FindTopicMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::FindTopic(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for LookupTopicdescriptionMail {
    type Output = Option<String>;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::LookupTopicdescription(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::TopicDescription(res) => res,
            other => panic!("expected TopicDescription reply, got {:?}", other),
        }
    }
}

impl Operation for IgnoreParticipantMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::IgnoreParticipant(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for IgnoreSubscriptionMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::IgnoreSubscription(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for IgnorePublicationMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::IgnorePublication(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for DeleteContainedEntitiesMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::DeleteContainedEntities(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for SetDefaultPublisherQosMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::SetDefaultPublisherQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for GetDefaultPublisherQosMail {
    type Output = PublisherQos;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetDefaultPublisherQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::PublisherQos(res) => res,
            other => panic!("expected PublisherQos reply, got {:?}", other),
        }
    }
}

impl Operation for SetDefaultSubscriberQosMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::SetDefaultSubscriberQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for GetDefaultSubscriberQosMail {
    type Output = SubscriberQos;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetDefaultSubscriberQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::SubscriberQos(res) => res,
            other => panic!("expected SubscriberQos reply, got {:?}", other),
        }
    }
}

impl Operation for SetDefaultTopicQosMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::SetDefaultTopicQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for GetDefaultTopicQosMail {
    type Output = TopicQos;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetDefaultTopicQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::TopicQos(res) => res.map(|b| *b),
            other => panic!("expected TopicQos reply, got {:?}", other),
        }
    }
}

impl Operation for GetDiscoveredParticipantsMail {
    type Output = Vec<InstanceHandle>;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetDiscoveredParticipants(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::InstanceHandleList(res) => res,
            other => panic!("expected InstanceHandleList reply, got {:?}", other),
        }
    }
}

impl Operation for GetDiscoveredParticipantDataMail {
    type Output = ParticipantBuiltinTopicData;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetDiscoveredParticipantData(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::ParticipantBuiltinTopicData(res) => res.map(|b| *b),
            other => panic!(
                "expected ParticipantBuiltinTopicData reply, got {:?}",
                other
            ),
        }
    }
}

impl Operation for GetDiscoveredTopicsMail {
    type Output = Vec<InstanceHandle>;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetDiscoveredTopics(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::InstanceHandleList(res) => res,
            other => panic!("expected InstanceHandleList reply, got {:?}", other),
        }
    }
}

impl Operation for GetDiscoveredTopicDataMail {
    type Output = TopicBuiltinTopicData;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetDiscoveredTopicData(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::TopicBuiltinTopicData(res) => res.map(|b| *b),
            other => panic!("expected TopicBuiltinTopicData reply, got {:?}", other),
        }
    }
}

impl Operation for GetCurrentTimeMail {
    type Output = Time;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetCurrentTime(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Time(res) => res,
            other => panic!("expected Time reply, got {:?}", other),
        }
    }
}

impl Operation for SetParticipantQosMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::SetParticipantQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for GetParticipantQosMail {
    type Output = DomainParticipantQos;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetParticipantQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::ParticipantQos(res) => res,
            other => panic!("expected ParticipantQos reply, got {:?}", other),
        }
    }
}

impl Operation for SetParticipantListenerMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::SetParticipantListener(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for EnableParticipantMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::EnableParticipant(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for DeleteParticipantMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::DeleteParticipant
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

// --- Topic Operations ---

impl Operation for GetInconsistentTopicStatusMail {
    type Output = InconsistentTopicStatus;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetInconsistentTopicStatus(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::InconsistentTopicStatus(res) => res,
            other => panic!("expected InconsistentTopicStatus reply, got {:?}", other),
        }
    }
}

impl Operation for SetTopicQosMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::SetTopicQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for GetTopicQosMail {
    type Output = TopicQos;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetTopicQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::TopicQos(res) => res.map(|b| *b),
            other => panic!("expected TopicQos reply, got {:?}", other),
        }
    }
}

impl Operation for EnableTopicMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::EnableTopic(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for GetTopicTypeSupportMail {
    type Output = DynamicType<'static>;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetTopicTypeSupport(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::DynamicType(res) => res,
            other => panic!("expected DynamicType reply, got {:?}", other),
        }
    }
}

// --- Publisher Operations ---

impl Operation for CreateDataWriterMail {
    type Output = InstanceHandle;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::CreateDataWriter(Box::new(self))
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::InstanceHandle(res) => res,
            other => panic!("expected InstanceHandle reply, got {:?}", other),
        }
    }
}

impl Operation for DeleteDataWriterMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::DeleteDataWriter(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for GetDefaultDataWriterQosMail {
    type Output = DataWriterQos;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetDefaultDataWriterQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::DataWriterQos(res) => res.map(|b| *b),
            other => panic!("expected DataWriterQos reply, got {:?}", other),
        }
    }
}

impl Operation for SetDefaultDataWriterQosMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::SetDefaultDataWriterQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for GetPublisherQosMail {
    type Output = PublisherQos;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetPublisherQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::PublisherQos(res) => res,
            other => panic!("expected PublisherQos reply, got {:?}", other),
        }
    }
}

impl Operation for SetPublisherQosMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::SetPublisherQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for SetPublisherListenerMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::SetPublisherListener(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

// --- Subscriber Operations ---

impl Operation for CreateDataReaderMail {
    type Output = InstanceHandle;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::CreateDataReader(Box::new(self))
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::InstanceHandle(res) => res,
            other => panic!("expected InstanceHandle reply, got {:?}", other),
        }
    }
}

impl Operation for DeleteDataReaderMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::DeleteDataReader(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for LookupDataReaderMail {
    type Output = Option<InstanceHandle>;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::LookupDataReader(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::OptionInstanceHandle(res) => res,
            other => panic!("expected OptionInstanceHandle reply, got {:?}", other),
        }
    }
}

impl Operation for SetDefaultDataReaderQosMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::SetDefaultDataReaderQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for GetDefaultDataReaderQosMail {
    type Output = DataReaderQos;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetDefaultDataReaderQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::DataReaderQos(res) => res.map(|b| *b),
            other => panic!("expected DataReaderQos reply, got {:?}", other),
        }
    }
}

impl Operation for SetSubscriberQosMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::SetSubscriberQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for GetSubscriberQosMail {
    type Output = SubscriberQos;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetSubscriberQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::SubscriberQos(res) => res,
            other => panic!("expected SubscriberQos reply, got {:?}", other),
        }
    }
}

impl Operation for SetSubscriberListenerMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::SetSubscriberListener(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

// --- Writer Operations ---

impl Operation for WriteWTimestampMail {
    type Output = WriteReply;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::WriteWTimestamp(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res.map(|_| WriteReply::Ok),
            DcpsReply::PendingWrite(receiver) => Ok(WriteReply::Pending(receiver)),
            other => panic!("expected Ok or PendingWrite reply, got {:?}", other),
        }
    }
}

impl Operation for DisposeWTimestampMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::DisposeWTimestamp(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for RegisterInstanceMail {
    type Output = Option<InstanceHandle>;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::RegisterInstance(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::OptionInstanceHandle(res) => res,
            other => panic!("expected OptionInstanceHandle reply, got {:?}", other),
        }
    }
}

impl Operation for UnregisterInstanceMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::UnregisterInstance(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for LookupInstanceMail {
    type Output = Option<InstanceHandle>;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::LookupInstance(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::OptionInstanceHandle(res) => res,
            other => panic!("expected OptionInstanceHandle reply, got {:?}", other),
        }
    }
}

impl Operation for GetDataWriterQosMail {
    type Output = DataWriterQos;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetDataWriterQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::DataWriterQos(res) => res.map(|b| *b),
            other => panic!("expected DataWriterQos reply, got {:?}", other),
        }
    }
}

impl Operation for SetDataWriterQosMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::SetDataWriterQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for GetMatchedSubscriptionsMail {
    type Output = Vec<InstanceHandle>;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetMatchedSubscriptions(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::InstanceHandleList(res) => res,
            other => panic!("expected InstanceHandleList reply, got {:?}", other),
        }
    }
}

impl Operation for GetMatchedSubscriptionDataMail {
    type Output = SubscriptionBuiltinTopicData;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetMatchedSubscriptionData(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::SubscriptionBuiltinTopicData(res) => res.map(|b| *b),
            other => panic!(
                "expected SubscriptionBuiltinTopicData reply, got {:?}",
                other
            ),
        }
    }
}

impl Operation for GetPublicationMatchedStatusMail {
    type Output = PublicationMatchedStatus;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetPublicationMatchedStatus(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::PublicationMatchedStatus(res) => res,
            other => panic!("expected PublicationMatchedStatus reply, got {:?}", other),
        }
    }
}

impl Operation for GetOfferedDeadlineMissedStatusMail {
    type Output = OfferedDeadlineMissedStatus;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetOfferedDeadlineMissedStatus(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::OfferedDeadlineMissedStatus(res) => res,
            other => panic!(
                "expected OfferedDeadlineMissedStatus reply, got {:?}",
                other
            ),
        }
    }
}

impl Operation for EnableDataWriterMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::EnableDataWriter(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for SetDataWriterListenerMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::SetDataWriterListener(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

// --- Reader Operations ---

impl Operation for ReadMail {
    type Output = (DynamicType<'static>, SampleList);

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::Read(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Samples(res) => res,
            other => panic!("expected Samples reply, got {:?}", other),
        }
    }
}

impl Operation for TakeMail {
    type Output = (DynamicType<'static>, SampleList);

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::Take(self.0)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Samples(res) => res,
            other => panic!("expected Samples reply, got {:?}", other),
        }
    }
}

impl Operation for ReadNextInstanceMail {
    type Output = (DynamicType<'static>, SampleList);

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::ReadNextInstance(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Samples(res) => res,
            other => panic!("expected Samples reply, got {:?}", other),
        }
    }
}

impl Operation for TakeNextInstanceMail {
    type Output = (DynamicType<'static>, SampleList);

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::TakeNextInstance(self.0)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Samples(res) => res,
            other => panic!("expected Samples reply, got {:?}", other),
        }
    }
}

impl Operation for EnableDataReaderMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::EnableDataReader(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for GetSubscriptionMatchedStatusMail {
    type Output = SubscriptionMatchedStatus;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetSubscriptionMatchedStatus(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::SubscriptionMatchedStatus(res) => res,
            other => panic!("expected SubscriptionMatchedStatus reply, got {:?}", other),
        }
    }
}

impl Operation for GetMatchedPublicationDataMail {
    type Output = PublicationBuiltinTopicData;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetMatchedPublicationData(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::PublicationBuiltinTopicData(res) => res.map(|b| *b),
            other => panic!(
                "expected PublicationBuiltinTopicData reply, got {:?}",
                other
            ),
        }
    }
}

impl Operation for GetMatchedPublicationsMail {
    type Output = Vec<InstanceHandle>;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetMatchedPublications(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::InstanceHandleList(res) => res,
            other => panic!("expected InstanceHandleList reply, got {:?}", other),
        }
    }
}

impl Operation for SetDataReaderQosMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::SetDataReaderQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for GetDataReaderQosMail {
    type Output = DataReaderQos;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetDataReaderQos(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::DataReaderQos(res) => res.map(|b| *b),
            other => panic!("expected DataReaderQos reply, got {:?}", other),
        }
    }
}

impl Operation for SetDataReaderListenerMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::SetDataReaderListener(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

// --- StatusCondition Operations ---

impl Operation for GetStatusConditionEnabledStatusesMail {
    type Output = StatusMask;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetStatusConditionEnabledStatuses(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::StatusMask(res) => res,
            other => panic!("expected StatusMask reply, got {:?}", other),
        }
    }
}

impl Operation for SetStatusConditionEnabledStatusesMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::SetStatusConditionEnabledStatuses(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for GetStatusConditionTriggerValueMail {
    type Output = bool;

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::GetStatusConditionTriggerValue(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::TriggerValue(res) => res,
            other => panic!("expected TriggerValue reply, got {:?}", other),
        }
    }
}

impl Operation for RegisterStatusConditionNotificationMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::RegisterStatusConditionNotification(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

// --- Message Operations ---

impl Operation for NotifyAcknowledgmentsMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::NotifyAcknowledgments(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}

impl Operation for NotifyHistoricalDataMail {
    type Output = ();

    fn into_mail(self) -> DomainParticipantMail {
        DomainParticipantMail::NotifyHistoricalData(self)
    }

    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output> {
        match reply {
            DcpsReply::Ok(res) => res,
            other => panic!("expected Ok reply, got {:?}", other),
        }
    }
}
