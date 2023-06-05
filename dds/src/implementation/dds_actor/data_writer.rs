use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    implementation::{
        data_representation_builtin_endpoints::discovered_writer_data::DiscoveredWriterData,
        dds::dds_data_writer::DdsDataWriter,
        rtps::stateful_writer::RtpsStatefulWriter,
        utils::actor::{Handler, Message},
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

impl Message for Enable {
    type Result = ();
}

impl<T> Handler<Enable> for DdsDataWriter<T> {
    fn handle(&mut self, _message: Enable) -> <Enable as Message>::Result {
        self.enable();
    }
}

pub struct GetTypeName;

impl Message for GetTypeName {
    type Result = &'static str;
}

impl<T> Handler<GetTypeName> for DdsDataWriter<T> {
    fn handle(&mut self, _message: GetTypeName) -> <GetTypeName as Message>::Result {
        self.get_type_name()
    }
}

pub struct GetTopicName;

impl Message for GetTopicName {
    type Result = String;
}

impl Handler<GetTopicName> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(&mut self, message: GetTopicName) -> <GetTopicName as Message>::Result {
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

impl Message for WriteWithTimestamp {
    type Result = DdsResult<()>;
}

impl Handler<WriteWithTimestamp> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(&mut self, message: WriteWithTimestamp) -> <WriteWithTimestamp as Message>::Result {
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

impl Message for UnregisterInstanceWithTimestamp {
    type Result = DdsResult<()>;
}

impl Handler<UnregisterInstanceWithTimestamp> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: UnregisterInstanceWithTimestamp,
    ) -> <UnregisterInstanceWithTimestamp as Message>::Result {
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

impl Message for LookupInstance {
    type Result = DdsResult<Option<InstanceHandle>>;
}

impl Handler<LookupInstance> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(&mut self, message: LookupInstance) -> <LookupInstance as Message>::Result {
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

impl Message for DisposeWithTimestamp {
    type Result = DdsResult<()>;
}

impl Handler<DisposeWithTimestamp> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: DisposeWithTimestamp,
    ) -> <DisposeWithTimestamp as Message>::Result {
        self.dispose_w_timestamp(
            message.instance_serialized_key,
            message.handle,
            message.timestamp,
        )
    }
}

pub struct IsEnabled;

impl Message for IsEnabled {
    type Result = bool;
}

impl Handler<IsEnabled> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(&mut self, message: IsEnabled) -> <IsEnabled as Message>::Result {
        self.is_enabled()
    }
}

pub struct AreAllChangesAcknowledge;

impl Message for AreAllChangesAcknowledge {
    type Result = bool;
}

impl Handler<AreAllChangesAcknowledge> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: AreAllChangesAcknowledge,
    ) -> <AreAllChangesAcknowledge as Message>::Result {
        self.are_all_changes_acknowledge()
    }
}

pub struct GetLivelinessLostStatus;

impl Message for GetLivelinessLostStatus {
    type Result = LivelinessLostStatus;
}

impl Handler<GetLivelinessLostStatus> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: GetLivelinessLostStatus,
    ) -> <GetLivelinessLostStatus as Message>::Result {
        self.get_liveliness_lost_status()
    }
}

pub struct GetOfferedDeadlineMissedStatus;

impl Message for GetOfferedDeadlineMissedStatus {
    type Result = OfferedDeadlineMissedStatus;
}

impl Handler<GetOfferedDeadlineMissedStatus> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: GetOfferedDeadlineMissedStatus,
    ) -> <GetOfferedDeadlineMissedStatus as Message>::Result {
        self.get_offered_deadline_missed_status()
    }
}

pub struct GetOfferedIncompatibleQosStatus;

impl Message for GetOfferedIncompatibleQosStatus {
    type Result = OfferedIncompatibleQosStatus;
}

impl Handler<GetOfferedIncompatibleQosStatus> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: GetOfferedIncompatibleQosStatus,
    ) -> <GetOfferedIncompatibleQosStatus as Message>::Result {
        self.get_offered_incompatible_qos_status()
    }
}

pub struct GetPublicationMatchedStatus;

impl Message for GetPublicationMatchedStatus {
    type Result = PublicationMatchedStatus;
}

impl Handler<GetPublicationMatchedStatus> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: GetPublicationMatchedStatus,
    ) -> <GetPublicationMatchedStatus as Message>::Result {
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

impl Message for GetMatchedSubscriptionData {
    type Result = Option<SubscriptionBuiltinTopicData>;
}

impl Handler<GetMatchedSubscriptionData> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: GetMatchedSubscriptionData,
    ) -> <GetMatchedSubscriptionData as Message>::Result {
        self.get_matched_subscription_data(message.handle)
    }
}

pub struct GetMatchedSubscriptions;

impl Message for GetMatchedSubscriptions {
    type Result = Vec<InstanceHandle>;
}

impl Handler<GetMatchedSubscriptions> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: GetMatchedSubscriptions,
    ) -> <GetMatchedSubscriptions as Message>::Result {
        self.get_matched_subscriptions()
    }
}

pub struct GetQos;

impl Message for GetQos {
    type Result = DataWriterQos;
}

impl Handler<GetQos> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(&mut self, message: GetQos) -> <GetQos as Message>::Result {
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

impl Message for SetQos {
    type Result = ();
}

impl Handler<SetQos> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(&mut self, message: SetQos) -> <SetQos as Message>::Result {
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

impl Message for AsDiscoveredWriterData {
    type Result = DiscoveredWriterData;
}

impl Handler<AsDiscoveredWriterData> for DdsDataWriter<RtpsStatefulWriter> {
    fn handle(
        &mut self,
        message: AsDiscoveredWriterData,
    ) -> <AsDiscoveredWriterData as Message>::Result {
        self.as_discovered_writer_data(&message.topic_qos, &message.publisher_qos)
    }
}
