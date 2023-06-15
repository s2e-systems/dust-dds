use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_writer_data::DiscoveredWriterData,
        },
        dds::dds_data_writer::DdsDataWriter,
        rtps::{
            messages::overall_structure::{RtpsMessageHeader, RtpsMessageRead},
            reader_locator::RtpsReaderLocator,
            reader_proxy::RtpsReaderProxy,
            stateful_writer::RtpsStatefulWriter,
            stateless_writer::RtpsStatelessWriter,
            types::{Guid, Locator},
        },
        rtps_udp_psm::udp_transport::UdpTransportWrite,
        utils::actor::{ActorAddress, CommandHandler, Mail, MailHandler},
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

impl<T> ActorAddress<DdsDataWriter<T>> {
    pub fn enable(&self) -> DdsResult<()> {
        struct Enable;

        impl Mail for Enable {
            type Result = ();
        }

        impl<T> MailHandler<Enable> for DdsDataWriter<T> {
            fn handle(&mut self, _mail: Enable) -> <Enable as Mail>::Result {
                self.enable();
            }
        }

        self.send_blocking(Enable)
    }

    pub fn get_type_name(&self) -> DdsResult<&'static str> {
        struct GetTypeName;

        impl Mail for GetTypeName {
            type Result = &'static str;
        }

        impl<T> MailHandler<GetTypeName> for DdsDataWriter<T> {
            fn handle(&mut self, _mail: GetTypeName) -> <GetTypeName as Mail>::Result {
                self.get_type_name()
            }
        }

        self.send_blocking(GetTypeName)
    }

    pub fn get_topic_name(&self) -> DdsResult<String> {
        struct GetTopicName;

        impl Mail for GetTopicName {
            type Result = String;
        }

        impl<T> MailHandler<GetTopicName> for DdsDataWriter<T> {
            fn handle(&mut self, _mail: GetTopicName) -> <GetTopicName as Mail>::Result {
                self.get_topic_name().to_string()
            }
        }

        self.send_blocking(GetTopicName)
    }

    pub fn get_matched_subscriptions(&self) -> DdsResult<Vec<InstanceHandle>> {
        struct GetMatchedSubscriptions;

        impl Mail for GetMatchedSubscriptions {
            type Result = Vec<InstanceHandle>;
        }

        impl<T> MailHandler<GetMatchedSubscriptions> for DdsDataWriter<T> {
            fn handle(
                &mut self,
                _mail: GetMatchedSubscriptions,
            ) -> <GetMatchedSubscriptions as Mail>::Result {
                self.get_matched_subscriptions()
            }
        }

        self.send_blocking(GetMatchedSubscriptions)
    }

    pub fn is_enabled(&self) -> DdsResult<bool> {
        struct IsEnabled;

        impl Mail for IsEnabled {
            type Result = bool;
        }

        impl<T> MailHandler<IsEnabled> for DdsDataWriter<T> {
            fn handle(&mut self, _mail: IsEnabled) -> <IsEnabled as Mail>::Result {
                self.is_enabled()
            }
        }

        self.send_blocking(IsEnabled)
    }

    pub fn get_liveliness_lost_status(&self) -> DdsResult<LivelinessLostStatus> {
        struct GetLivelinessLostStatus;

        impl Mail for GetLivelinessLostStatus {
            type Result = LivelinessLostStatus;
        }

        impl<T> MailHandler<GetLivelinessLostStatus> for DdsDataWriter<T> {
            fn handle(
                &mut self,
                _mail: GetLivelinessLostStatus,
            ) -> <GetLivelinessLostStatus as Mail>::Result {
                self.get_liveliness_lost_status()
            }
        }
        self.send_blocking(GetLivelinessLostStatus)
    }

    pub fn get_offered_deadline_missed_status(&self) -> DdsResult<OfferedDeadlineMissedStatus> {
        struct GetOfferedDeadlineMissedStatus;

        impl Mail for GetOfferedDeadlineMissedStatus {
            type Result = OfferedDeadlineMissedStatus;
        }

        impl<T> MailHandler<GetOfferedDeadlineMissedStatus> for DdsDataWriter<T> {
            fn handle(
                &mut self,
                _mail: GetOfferedDeadlineMissedStatus,
            ) -> <GetOfferedDeadlineMissedStatus as Mail>::Result {
                self.get_offered_deadline_missed_status()
            }
        }
        self.send_blocking(GetOfferedDeadlineMissedStatus)
    }

    pub fn get_offered_incompatible_qos_status(&self) -> DdsResult<OfferedIncompatibleQosStatus> {
        struct GetOfferedIncompatibleQosStatus;

        impl Mail for GetOfferedIncompatibleQosStatus {
            type Result = OfferedIncompatibleQosStatus;
        }

        impl<T> MailHandler<GetOfferedIncompatibleQosStatus> for DdsDataWriter<T> {
            fn handle(
                &mut self,
                _mail: GetOfferedIncompatibleQosStatus,
            ) -> <GetOfferedIncompatibleQosStatus as Mail>::Result {
                self.get_offered_incompatible_qos_status()
            }
        }
        self.send_blocking(GetOfferedIncompatibleQosStatus)
    }

    pub fn get_publication_matched_status(&self) -> DdsResult<PublicationMatchedStatus> {
        struct GetPublicationMatchedStatus;

        impl Mail for GetPublicationMatchedStatus {
            type Result = PublicationMatchedStatus;
        }

        impl<T> MailHandler<GetPublicationMatchedStatus> for DdsDataWriter<T> {
            fn handle(
                &mut self,
                _mail: GetPublicationMatchedStatus,
            ) -> <GetPublicationMatchedStatus as Mail>::Result {
                self.get_publication_matched_status()
            }
        }
        self.send_blocking(GetPublicationMatchedStatus)
    }

    pub fn get_matched_subscription_data(
        &self,
        handle: InstanceHandle,
    ) -> DdsResult<Option<SubscriptionBuiltinTopicData>> {
        struct GetMatchedSubscriptionData {
            handle: InstanceHandle,
        }

        impl Mail for GetMatchedSubscriptionData {
            type Result = Option<SubscriptionBuiltinTopicData>;
        }

        impl<T> MailHandler<GetMatchedSubscriptionData> for DdsDataWriter<T> {
            fn handle(
                &mut self,
                mail: GetMatchedSubscriptionData,
            ) -> <GetMatchedSubscriptionData as Mail>::Result {
                self.get_matched_subscription_data(mail.handle)
            }
        }
        self.send_blocking(GetMatchedSubscriptionData { handle })
    }
}

impl ActorAddress<DdsDataWriter<RtpsStatefulWriter>> {
    pub fn add_matched_reader(
        &self,
        discovered_reader_data: DiscoveredReaderData,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        publisher_qos: PublisherQos,
    ) -> DdsResult<()> {
        struct AddMatchedReader {
            discovered_reader_data: DiscoveredReaderData,
            default_unicast_locator_list: Vec<Locator>,
            default_multicast_locator_list: Vec<Locator>,
            publisher_qos: PublisherQos,
        }

        impl Mail for AddMatchedReader {
            type Result = ();
        }

        impl MailHandler<AddMatchedReader> for DdsDataWriter<RtpsStatefulWriter> {
            fn handle(&mut self, mail: AddMatchedReader) -> <AddMatchedReader as Mail>::Result {
                self.add_matched_reader(
                    mail.discovered_reader_data,
                    mail.default_unicast_locator_list,
                    mail.default_multicast_locator_list,
                    mail.publisher_qos,
                )
            }
        }

        self.send_blocking(AddMatchedReader {
            discovered_reader_data,
            default_unicast_locator_list,
            default_multicast_locator_list,
            publisher_qos,
        })
    }

    pub fn process_rtps_message(&self, message: RtpsMessageRead) -> DdsResult<()> {
        struct ProcessRtpsMessage {
            message: RtpsMessageRead,
        }

        impl Mail for ProcessRtpsMessage {
            type Result = ();
        }

        impl MailHandler<ProcessRtpsMessage> for DdsDataWriter<RtpsStatefulWriter> {
            fn handle(&mut self, mail: ProcessRtpsMessage) -> <ProcessRtpsMessage as Mail>::Result {
                self.process_rtps_message(mail.message)
            }
        }

        self.send_blocking(ProcessRtpsMessage { message })
    }

    pub fn send_message(
        &self,
        header: RtpsMessageHeader,
        udp_transport_write: ActorAddress<UdpTransportWrite>,
    ) -> DdsResult<()> {
        struct SendMessage {
            header: RtpsMessageHeader,
            udp_transport_write: ActorAddress<UdpTransportWrite>,
        }

        impl CommandHandler<SendMessage> for DdsDataWriter<RtpsStatefulWriter> {
            fn handle(&mut self, mail: SendMessage) {
                self.send_message(mail.header, mail.udp_transport_write)
            }
        }

        self.send_command(SendMessage {
            header,
            udp_transport_write,
        })
    }

    pub fn write_w_timestamp(
        &self,
        serialized_data: Vec<u8>,
        instance_serialized_key: DdsSerializedKey,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        struct WriteWithTimestamp {
            serialized_data: Vec<u8>,
            instance_serialized_key: DdsSerializedKey,
            handle: Option<InstanceHandle>,
            timestamp: Time,
        }

        impl Mail for WriteWithTimestamp {
            type Result = DdsResult<()>;
        }

        impl MailHandler<WriteWithTimestamp> for DdsDataWriter<RtpsStatefulWriter> {
            fn handle(&mut self, mail: WriteWithTimestamp) -> <WriteWithTimestamp as Mail>::Result {
                self.write_w_timestamp(
                    mail.serialized_data,
                    mail.instance_serialized_key,
                    mail.handle,
                    mail.timestamp,
                )
            }
        }

        self.send_blocking(WriteWithTimestamp {
            serialized_data,
            instance_serialized_key,
            handle,
            timestamp,
        })?
    }

    pub fn unregister_instance_w_timestamp(
        &self,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> DdsResult<()> {
        struct UnregisterInstanceWithTimestamp {
            instance_serialized_key: Vec<u8>,
            handle: InstanceHandle,
            timestamp: Time,
        }

        impl Mail for UnregisterInstanceWithTimestamp {
            type Result = DdsResult<()>;
        }

        impl MailHandler<UnregisterInstanceWithTimestamp> for DdsDataWriter<RtpsStatefulWriter> {
            fn handle(
                &mut self,
                mail: UnregisterInstanceWithTimestamp,
            ) -> <UnregisterInstanceWithTimestamp as Mail>::Result {
                self.unregister_instance_w_timestamp(
                    mail.instance_serialized_key,
                    mail.handle,
                    mail.timestamp,
                )
            }
        }

        self.send_blocking(UnregisterInstanceWithTimestamp {
            instance_serialized_key,
            handle,
            timestamp,
        })?
    }
    pub fn dispose_w_timestamp(
        &self,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> DdsResult<()> {
        struct DisposeWithTimestamp {
            instance_serialized_key: Vec<u8>,
            handle: InstanceHandle,
            timestamp: Time,
        }

        impl Mail for DisposeWithTimestamp {
            type Result = DdsResult<()>;
        }

        impl MailHandler<DisposeWithTimestamp> for DdsDataWriter<RtpsStatefulWriter> {
            fn handle(
                &mut self,
                mail: DisposeWithTimestamp,
            ) -> <DisposeWithTimestamp as Mail>::Result {
                self.dispose_w_timestamp(mail.instance_serialized_key, mail.handle, mail.timestamp)
            }
        }

        self.send_blocking(DisposeWithTimestamp {
            instance_serialized_key,
            handle,
            timestamp,
        })?
    }

    pub fn lookup_instance(
        &self,
        instance_serialized_key: DdsSerializedKey,
    ) -> DdsResult<Option<InstanceHandle>> {
        struct LookupInstance {
            instance_serialized_key: DdsSerializedKey,
        }

        impl Mail for LookupInstance {
            type Result = DdsResult<Option<InstanceHandle>>;
        }

        impl MailHandler<LookupInstance> for DdsDataWriter<RtpsStatefulWriter> {
            fn handle(&mut self, mail: LookupInstance) -> <LookupInstance as Mail>::Result {
                self.lookup_instance(mail.instance_serialized_key)
            }
        }
        self.send_blocking(LookupInstance {
            instance_serialized_key,
        })?
    }

    pub fn get_qos(&self) -> DdsResult<DataWriterQos> {
        struct GetQos;

        impl Mail for GetQos {
            type Result = DataWriterQos;
        }

        impl MailHandler<GetQos> for DdsDataWriter<RtpsStatefulWriter> {
            fn handle(&mut self, _mail: GetQos) -> <GetQos as Mail>::Result {
                self.get_qos()
            }
        }
        self.send_blocking(GetQos)
    }

    pub fn set_qos(self, qos: DataWriterQos) -> DdsResult<()> {
        struct SetQos {
            qos: DataWriterQos,
        }

        impl Mail for SetQos {
            type Result = ();
        }

        impl MailHandler<SetQos> for DdsDataWriter<RtpsStatefulWriter> {
            fn handle(&mut self, mail: SetQos) -> <SetQos as Mail>::Result {
                self.set_qos(mail.qos)
            }
        }
        self.send_blocking(SetQos { qos })
    }

    pub fn as_discovered_writer_data(
        &self,
        topic_qos: TopicQos,
        publisher_qos: PublisherQos,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
    ) -> DdsResult<DiscoveredWriterData> {
        struct AsDiscoveredWriterData {
            topic_qos: TopicQos,
            publisher_qos: PublisherQos,
            default_unicast_locator_list: Vec<Locator>,
            default_multicast_locator_list: Vec<Locator>,
        }

        impl Mail for AsDiscoveredWriterData {
            type Result = DiscoveredWriterData;
        }

        impl MailHandler<AsDiscoveredWriterData> for DdsDataWriter<RtpsStatefulWriter> {
            fn handle(
                &mut self,
                mail: AsDiscoveredWriterData,
            ) -> <AsDiscoveredWriterData as Mail>::Result {
                self.as_discovered_writer_data(
                    mail.topic_qos,
                    mail.publisher_qos,
                    mail.default_unicast_locator_list,
                    mail.default_multicast_locator_list,
                )
            }
        }
        self.send_blocking(AsDiscoveredWriterData {
            topic_qos,
            publisher_qos,
            default_unicast_locator_list,
            default_multicast_locator_list,
        })
    }

    pub fn are_all_changes_acknowledge(&self) -> DdsResult<bool> {
        struct AreAllChangesAcknowledge;

        impl Mail for AreAllChangesAcknowledge {
            type Result = bool;
        }

        impl MailHandler<AreAllChangesAcknowledge> for DdsDataWriter<RtpsStatefulWriter> {
            fn handle(
                &mut self,
                _mail: AreAllChangesAcknowledge,
            ) -> <AreAllChangesAcknowledge as Mail>::Result {
                self.are_all_changes_acknowledge()
            }
        }
        self.send_blocking(AreAllChangesAcknowledge)
    }

    pub fn matched_reader_add(&self, a_reader_proxy: RtpsReaderProxy) -> DdsResult<()> {
        struct MatchedReaderAdd {
            a_reader_proxy: RtpsReaderProxy,
        }

        impl Mail for MatchedReaderAdd {
            type Result = ();
        }

        impl MailHandler<MatchedReaderAdd> for DdsDataWriter<RtpsStatefulWriter> {
            fn handle(&mut self, mail: MatchedReaderAdd) -> <MatchedReaderAdd as Mail>::Result {
                self.matched_reader_add(mail.a_reader_proxy)
            }
        }

        self.send_blocking(MatchedReaderAdd { a_reader_proxy })
    }

    pub fn guid(&self) -> DdsResult<Guid> {
        struct GetGuid;

        impl Mail for GetGuid {
            type Result = Guid;
        }

        impl MailHandler<GetGuid> for DdsDataWriter<RtpsStatefulWriter> {
            fn handle(&mut self, _mail: GetGuid) -> <GetGuid as Mail>::Result {
                self.guid()
            }
        }

        self.send_blocking(GetGuid)
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        struct GetInstanceHandle;

        impl Mail for GetInstanceHandle {
            type Result = InstanceHandle;
        }

        impl MailHandler<GetInstanceHandle> for DdsDataWriter<RtpsStatefulWriter> {
            fn handle(&mut self, _mail: GetInstanceHandle) -> <GetInstanceHandle as Mail>::Result {
                self.guid().into()
            }
        }

        self.send_blocking(GetInstanceHandle)
    }
}

impl ActorAddress<DdsDataWriter<RtpsStatelessWriter>> {
    pub fn send_message(
        &self,
        header: RtpsMessageHeader,
        udp_transport_write: ActorAddress<UdpTransportWrite>,
    ) -> DdsResult<()> {
        struct SendMessage {
            header: RtpsMessageHeader,
            udp_transport_write: ActorAddress<UdpTransportWrite>,
        }

        impl Mail for SendMessage {
            type Result = ();
        }

        impl MailHandler<SendMessage> for DdsDataWriter<RtpsStatelessWriter> {
            fn handle(&mut self, mail: SendMessage) -> <SendMessage as Mail>::Result {
                self.send_message(mail.header, mail.udp_transport_write)
            }
        }

        self.send_blocking(SendMessage {
            header,
            udp_transport_write,
        })
    }

    pub fn write_w_timestamp(
        &self,
        serialized_data: Vec<u8>,
        instance_serialized_key: DdsSerializedKey,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        struct WriteWithTimestamp {
            serialized_data: Vec<u8>,
            instance_serialized_key: DdsSerializedKey,
            handle: Option<InstanceHandle>,
            timestamp: Time,
        }

        impl Mail for WriteWithTimestamp {
            type Result = DdsResult<()>;
        }

        impl MailHandler<WriteWithTimestamp> for DdsDataWriter<RtpsStatelessWriter> {
            fn handle(&mut self, mail: WriteWithTimestamp) -> <WriteWithTimestamp as Mail>::Result {
                self.write_w_timestamp(
                    mail.serialized_data,
                    mail.instance_serialized_key,
                    mail.handle,
                    mail.timestamp,
                )
            }
        }

        self.send_blocking(WriteWithTimestamp {
            serialized_data,
            instance_serialized_key,
            handle,
            timestamp,
        })?
    }

    pub fn reader_locator_add(&self, a_locator: RtpsReaderLocator) -> DdsResult<()> {
        struct ReaderLocatorAdd {
            a_locator: RtpsReaderLocator,
        }

        impl Mail for ReaderLocatorAdd {
            type Result = ();
        }

        impl MailHandler<ReaderLocatorAdd> for DdsDataWriter<RtpsStatelessWriter> {
            fn handle(&mut self, mail: ReaderLocatorAdd) -> <ReaderLocatorAdd as Mail>::Result {
                self.reader_locator_add(mail.a_locator)
            }
        }

        self.send_blocking(ReaderLocatorAdd { a_locator })
    }
}
