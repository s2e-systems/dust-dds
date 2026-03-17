use std::{pin::Pin, sync::Arc};

use crate::{
    builtin_topics::{
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    dcps::{
        actor::ActorAddress,
        channels::{mpsc::MpscSender, oneshot::OneshotSender},
        domain_participant_factory::DcpsParticipantFactory,
        domain_participant_mail::DcpsDomainParticipantMail,
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
    runtime::DdsRuntime,
    transport::{interface::TransportParticipantFactory, types::CacheChange},
    xtypes::dynamic_type::{DynamicData, DynamicType},
};
use alloc::vec::Vec;

pub enum DcpsMail<R: DdsRuntime> {
    ParticipantFactory(ParticipantFactoryMail<R>),
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

pub enum ParticipantFactoryMail<R: DdsRuntime> {
    CreateParticipant {
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        dcps_listener: Option<DcpsDomainParticipantListener>,
        status_kind: Vec<StatusKind>,
        clock_handle: R::ClockHandle,
        timer_handle: R::TimerHandle,
        spawner_handle: R::SpawnerHandle,
        #[allow(clippy::type_complexity)]
        reply_sender: OneshotSender<
            DdsResult<(
                MpscSender<DcpsDomainParticipantMail>,
                InstanceHandle,
                ActorAddress<DcpsStatusCondition>,
            )>,
        >,
    },
    DeleteParticipant {
        handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<MpscSender<DcpsDomainParticipantMail>>>,
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
}

impl<T: TransportParticipantFactory> DcpsParticipantFactory<T> {
    pub async fn handle<R: DdsRuntime>(&mut self, message: DcpsMail<R>) {
        match message {
            DcpsMail::ParticipantFactory(ParticipantFactoryMail::CreateParticipant {
                domain_id,
                qos,
                dcps_listener,
                status_kind,
                clock_handle,
                timer_handle,
                spawner_handle,
                reply_sender,
            }) => reply_sender.send(
                self.create_participant::<R>(
                    domain_id,
                    qos,
                    dcps_listener,
                    status_kind,
                    clock_handle,
                    timer_handle,
                    spawner_handle,
                )
                .await,
            ),
            DcpsMail::ParticipantFactory(ParticipantFactoryMail::DeleteParticipant {
                handle,
                reply_sender,
            }) => reply_sender.send(self.delete_participant(handle)),
            DcpsMail::ParticipantFactory(ParticipantFactoryMail::SetDefaultParticipantQos {
                qos,
                reply_sender,
            }) => reply_sender.send(self.set_default_participant_qos(qos)),
            DcpsMail::ParticipantFactory(ParticipantFactoryMail::GetDefaultParticipantQos {
                reply_sender,
            }) => reply_sender.send(self.get_default_participant_qos()),
            DcpsMail::ParticipantFactory(ParticipantFactoryMail::SetQos { qos, reply_sender }) => {
                reply_sender.send(self.set_qos(qos))
            }
            DcpsMail::ParticipantFactory(ParticipantFactoryMail::GetQos { reply_sender }) => {
                reply_sender.send(self.get_qos())
            }
            DcpsMail::ParticipantFactory(ParticipantFactoryMail::SetConfiguration {
                configuration,
            }) => self.set_configuration(configuration),
            DcpsMail::ParticipantFactory(ParticipantFactoryMail::GetConfiguration {
                reply_sender,
            }) => reply_sender.send(self.get_configuration()),
            _ => todo!(),
        }
    }
}
