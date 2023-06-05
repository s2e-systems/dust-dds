use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    implementation::{
        data_representation_builtin_endpoints::discovered_writer_data::DiscoveredWriterData,
        dds::dds_data_writer::DdsDataWriter,
        rtps::stateful_writer::RtpsStatefulWriter,
        utils::actor::{MailHandler, Mail},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, TopicQos},
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus,
        },
        time::Time,
    },
    topic_definition::type_support::DdsSerializedKey,
};

pub struct Enable;

impl Mail for Enable {
    type Result = ();
}

impl<T> MailHandler<Enable> for DdsDataWriter<T> {
    fn handle(&mut self, _message: Enable) -> <Enable as Mail>::Result {
        self.enable();
    }
}

pub struct GetTypeName;

impl Mail for GetTypeName {
    type Result = &'static str;
}

impl<T> MailHandler<GetTypeName> for DdsDataWriter<T> {
    fn handle(&mut self, _message: GetTypeName) -> <GetTypeName as Mail>::Result {
        self.get_type_name()
    }
}

pub struct GetTopicName;

impl Mail for GetTopicName {
    type Result = String;
}

impl MailHandler<GetTopicName> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(&mut self, message: GetTopicName) -> <GetTopicName as Mail>::Result {
        self.get_topic_name().to_string()
    }
}

pub struct WriteWithTimestamp {
    serialized_data: Vec<u8>,
    instance_serialized_key: DdsSerializedKey,
    handle: Option<InstanceHandle>,
    timestamp: Time,
}

impl WriteWithTimestamp {
    pub fn new(
        serialized_data: Vec<u8>,
        instance_serialized_key: DdsSerializedKey,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> Self {
        Self {
            serialized_data,
            instance_serialized_key,
            handle,
            timestamp,
        }
    }
}

impl Mail for WriteWithTimestamp {
    type Result = DdsResult<()>;
}

impl MailHandler<WriteWithTimestamp> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(&mut self, message: WriteWithTimestamp) -> <WriteWithTimestamp as Mail>::Result {
        self.write_w_timestamp(
            message.serialized_data,
            message.instance_serialized_key,
            message.handle,
            message.timestamp,
        );
        Ok(())
    }
}

pub struct UnregisterInstanceWithTimestamp {
    instance_serialized_key: Vec<u8>,
    handle: InstanceHandle,
    timestamp: Time,
}

impl UnregisterInstanceWithTimestamp {
    pub fn new(instance_serialized_key: Vec<u8>, handle: InstanceHandle, timestamp: Time) -> Self {
        Self {
            instance_serialized_key,
            handle,
            timestamp,
        }
    }
}

impl Mail for UnregisterInstanceWithTimestamp {
    type Result = DdsResult<()>;
}

impl MailHandler<UnregisterInstanceWithTimestamp> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: UnregisterInstanceWithTimestamp,
    ) -> <UnregisterInstanceWithTimestamp as Mail>::Result {
        self.unregister_instance_w_timestamp(
            message.instance_serialized_key,
            message.handle,
            message.timestamp,
        )
    }
}

pub struct LookupInstance {
    instance_serialized_key: DdsSerializedKey,
}

impl LookupInstance {
    pub fn new(instance_serialized_key: DdsSerializedKey) -> Self {
        Self {
            instance_serialized_key,
        }
    }
}

impl Mail for LookupInstance {
    type Result = DdsResult<Option<InstanceHandle>>;
}

impl MailHandler<LookupInstance> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(&mut self, message: LookupInstance) -> <LookupInstance as Mail>::Result {
        self.lookup_instance(message.instance_serialized_key)
    }
}

pub struct DisposeWithTimestamp {
    instance_serialized_key: Vec<u8>,
    handle: InstanceHandle,
    timestamp: Time,
}

impl DisposeWithTimestamp {
    pub fn new(instance_serialized_key: Vec<u8>, handle: InstanceHandle, timestamp: Time) -> Self {
        Self {
            instance_serialized_key,
            handle,
            timestamp,
        }
    }
}

impl Mail for DisposeWithTimestamp {
    type Result = DdsResult<()>;
}

impl MailHandler<DisposeWithTimestamp> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: DisposeWithTimestamp,
    ) -> <DisposeWithTimestamp as Mail>::Result {
        self.dispose_w_timestamp(
            message.instance_serialized_key,
            message.handle,
            message.timestamp,
        )
    }
}

pub struct IsEnabled;

impl Mail for IsEnabled {
    type Result = bool;
}

impl MailHandler<IsEnabled> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(&mut self, message: IsEnabled) -> <IsEnabled as Mail>::Result {
        self.is_enabled()
    }
}

pub struct AreAllChangesAcknowledge;

impl Mail for AreAllChangesAcknowledge {
    type Result = bool;
}

impl MailHandler<AreAllChangesAcknowledge> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: AreAllChangesAcknowledge,
    ) -> <AreAllChangesAcknowledge as Mail>::Result {
        self.are_all_changes_acknowledge()
    }
}

pub struct GetLivelinessLostStatus;

impl Mail for GetLivelinessLostStatus {
    type Result = LivelinessLostStatus;
}

impl MailHandler<GetLivelinessLostStatus> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: GetLivelinessLostStatus,
    ) -> <GetLivelinessLostStatus as Mail>::Result {
        self.get_liveliness_lost_status()
    }
}

pub struct GetOfferedDeadlineMissedStatus;

impl Mail for GetOfferedDeadlineMissedStatus {
    type Result = OfferedDeadlineMissedStatus;
}

impl MailHandler<GetOfferedDeadlineMissedStatus> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: GetOfferedDeadlineMissedStatus,
    ) -> <GetOfferedDeadlineMissedStatus as Mail>::Result {
        self.get_offered_deadline_missed_status()
    }
}

pub struct GetOfferedIncompatibleQosStatus;

impl Mail for GetOfferedIncompatibleQosStatus {
    type Result = OfferedIncompatibleQosStatus;
}

impl MailHandler<GetOfferedIncompatibleQosStatus> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: GetOfferedIncompatibleQosStatus,
    ) -> <GetOfferedIncompatibleQosStatus as Mail>::Result {
        self.get_offered_incompatible_qos_status()
    }
}

pub struct GetPublicationMatchedStatus;

impl Mail for GetPublicationMatchedStatus {
    type Result = PublicationMatchedStatus;
}

impl MailHandler<GetPublicationMatchedStatus> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: GetPublicationMatchedStatus,
    ) -> <GetPublicationMatchedStatus as Mail>::Result {
        self.get_publication_matched_status()
    }
}

pub struct GetMatchedSubscriptionData {
    handle: InstanceHandle,
}

impl GetMatchedSubscriptionData {
    pub fn new(handle: InstanceHandle) -> Self {
        Self { handle }
    }
}

impl Mail for GetMatchedSubscriptionData {
    type Result = Option<SubscriptionBuiltinTopicData>;
}

impl MailHandler<GetMatchedSubscriptionData> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: GetMatchedSubscriptionData,
    ) -> <GetMatchedSubscriptionData as Mail>::Result {
        self.get_matched_subscription_data(message.handle)
    }
}

pub struct GetMatchedSubscriptions;

impl Mail for GetMatchedSubscriptions {
    type Result = Vec<InstanceHandle>;
}

impl MailHandler<GetMatchedSubscriptions> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: GetMatchedSubscriptions,
    ) -> <GetMatchedSubscriptions as Mail>::Result {
        self.get_matched_subscriptions()
    }
}

pub struct GetQos;

impl Mail for GetQos {
    type Result = DataWriterQos;
}

impl MailHandler<GetQos> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(&mut self, message: GetQos) -> <GetQos as Mail>::Result {
        self.get_qos()
    }
}

pub struct SetQos {
    qos: DataWriterQos,
}

impl SetQos {
    pub fn new(qos: DataWriterQos) -> Self {
        Self { qos }
    }
}

impl Mail for SetQos {
    type Result = ();
}

impl MailHandler<SetQos> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(&mut self, message: SetQos) -> <SetQos as Mail>::Result {
        self.set_qos(message.qos)
    }
}

pub struct AsDiscoveredWriterData {
    topic_qos: TopicQos,
    publisher_qos: PublisherQos,
}

impl AsDiscoveredWriterData {
    pub fn new(topic_qos: TopicQos, publisher_qos: PublisherQos) -> Self {
        Self {
            topic_qos,
            publisher_qos,
        }
    }
}

impl Mail for AsDiscoveredWriterData {
    type Result = DiscoveredWriterData;
}

impl MailHandler<AsDiscoveredWriterData> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: AsDiscoveredWriterData,
    ) -> <AsDiscoveredWriterData as Mail>::Result {
        self.as_discovered_writer_data(&message.topic_qos, &message.publisher_qos)
    }
}
