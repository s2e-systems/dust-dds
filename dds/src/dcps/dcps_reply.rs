use alloc::{string::String, vec::Vec};

use crate::{
    builtin_topics::{
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    dcps::{channels::notification::NotificationReceiver, status_mask::StatusMask},
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantFactoryQos, DomainParticipantQos,
            PublisherQos, SubscriberQos, TopicQos,
        },
        sample_info::SampleInfo,
        status::{
            InconsistentTopicStatus, OfferedDeadlineMissedStatus, PublicationMatchedStatus,
            SubscriptionMatchedStatus,
        },
        time::Time,
    },
    xtypes::dynamic_type::{DynamicData, DynamicType},
};

#[allow(clippy::large_enum_variant)]
pub enum DcpsReply {
    None,
    Unit(DdsResult<()>),
    WriteResult(DdsResult<Option<NotificationReceiver>>),
    InstanceHandle(DdsResult<InstanceHandle>),
    InstanceHandleAndString(DdsResult<(InstanceHandle, String)>),
    InstanceHandleOpt(Option<InstanceHandle>),
    InstanceHandleDdsOpt(DdsResult<Option<InstanceHandle>>),
    StringOpt(DdsResult<Option<String>>),
    DomainParticipantFactoryQos(DomainParticipantFactoryQos),
    DomainParticipantQos(DomainParticipantQos),
    DomainParticipantQosResult(DdsResult<DomainParticipantQos>),
    PublisherQos(DdsResult<PublisherQos>),
    SubscriberQos(DdsResult<SubscriberQos>),
    TopicQos(DdsResult<TopicQos>),
    DataWriterQos(DdsResult<DataWriterQos>),
    DataReaderQos(DdsResult<DataReaderQos>),
    StatusMask(DdsResult<StatusMask>),
    Bool(DdsResult<bool>),
    Time(DdsResult<Time>),
    DynamicType(DdsResult<DynamicType<'static>>),
    InconsistentTopicStatus(DdsResult<InconsistentTopicStatus>),
    OfferedDeadlineMissedStatus(DdsResult<OfferedDeadlineMissedStatus>),
    PublicationMatchedStatus(DdsResult<PublicationMatchedStatus>),
    SubscriptionMatchedStatus(DdsResult<SubscriptionMatchedStatus>),
    ParticipantBuiltinTopicData(DdsResult<ParticipantBuiltinTopicData>),
    TopicBuiltinTopicData(DdsResult<TopicBuiltinTopicData>),
    PublicationBuiltinTopicData(DdsResult<PublicationBuiltinTopicData>),
    SubscriptionBuiltinTopicData(DdsResult<SubscriptionBuiltinTopicData>),
    InstanceHandleList(DdsResult<Vec<InstanceHandle>>),
    DynamicDataSampleList(DdsResult<Vec<(Option<DynamicData<'static>>, SampleInfo)>>),
}

impl DcpsReply {
    pub fn into_unit(self) -> DdsResult<()> {
        match self {
            DcpsReply::Unit(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_write_result(self) -> DdsResult<Option<NotificationReceiver>> {
        match self {
            DcpsReply::WriteResult(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_instance_handle(self) -> DdsResult<InstanceHandle> {
        match self {
            DcpsReply::InstanceHandle(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_instance_handle_and_string(self) -> DdsResult<(InstanceHandle, String)> {
        match self {
            DcpsReply::InstanceHandleAndString(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_instance_handle_opt(self) -> Option<InstanceHandle> {
        match self {
            DcpsReply::InstanceHandleOpt(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_instance_handle_dds_opt(self) -> DdsResult<Option<InstanceHandle>> {
        match self {
            DcpsReply::InstanceHandleDdsOpt(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_string_opt(self) -> DdsResult<Option<String>> {
        match self {
            DcpsReply::StringOpt(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_domain_participant_factory_qos(self) -> DomainParticipantFactoryQos {
        match self {
            DcpsReply::DomainParticipantFactoryQos(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_domain_participant_qos(self) -> DomainParticipantQos {
        match self {
            DcpsReply::DomainParticipantQos(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_domain_participant_qos_result(self) -> DdsResult<DomainParticipantQos> {
        match self {
            DcpsReply::DomainParticipantQosResult(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_publisher_qos(self) -> DdsResult<PublisherQos> {
        match self {
            DcpsReply::PublisherQos(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_subscriber_qos(self) -> DdsResult<SubscriberQos> {
        match self {
            DcpsReply::SubscriberQos(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_topic_qos(self) -> DdsResult<TopicQos> {
        match self {
            DcpsReply::TopicQos(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_data_writer_qos(self) -> DdsResult<DataWriterQos> {
        match self {
            DcpsReply::DataWriterQos(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_data_reader_qos(self) -> DdsResult<DataReaderQos> {
        match self {
            DcpsReply::DataReaderQos(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_status_mask(self) -> DdsResult<StatusMask> {
        match self {
            DcpsReply::StatusMask(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_bool(self) -> DdsResult<bool> {
        match self {
            DcpsReply::Bool(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_time(self) -> DdsResult<Time> {
        match self {
            DcpsReply::Time(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_dynamic_type(self) -> DdsResult<DynamicType<'static>> {
        match self {
            DcpsReply::DynamicType(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_inconsistent_topic_status(self) -> DdsResult<InconsistentTopicStatus> {
        match self {
            DcpsReply::InconsistentTopicStatus(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_offered_deadline_missed_status(self) -> DdsResult<OfferedDeadlineMissedStatus> {
        match self {
            DcpsReply::OfferedDeadlineMissedStatus(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_publication_matched_status(self) -> DdsResult<PublicationMatchedStatus> {
        match self {
            DcpsReply::PublicationMatchedStatus(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_subscription_matched_status(self) -> DdsResult<SubscriptionMatchedStatus> {
        match self {
            DcpsReply::SubscriptionMatchedStatus(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_participant_builtin_topic_data(self) -> DdsResult<ParticipantBuiltinTopicData> {
        match self {
            DcpsReply::ParticipantBuiltinTopicData(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_topic_builtin_topic_data(self) -> DdsResult<TopicBuiltinTopicData> {
        match self {
            DcpsReply::TopicBuiltinTopicData(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_publication_builtin_topic_data(self) -> DdsResult<PublicationBuiltinTopicData> {
        match self {
            DcpsReply::PublicationBuiltinTopicData(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_subscription_builtin_topic_data(self) -> DdsResult<SubscriptionBuiltinTopicData> {
        match self {
            DcpsReply::SubscriptionBuiltinTopicData(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_instance_handle_list(self) -> DdsResult<Vec<InstanceHandle>> {
        match self {
            DcpsReply::InstanceHandleList(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }

    pub fn into_dynamic_data_sample_list(
        self,
    ) -> DdsResult<Vec<(Option<DynamicData<'static>>, SampleInfo)>> {
        match self {
            DcpsReply::DynamicDataSampleList(res) => res,
            _ => panic!("unexpected reply variant"),
        }
    }
}
