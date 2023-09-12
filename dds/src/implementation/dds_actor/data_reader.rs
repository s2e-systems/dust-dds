use std::marker::PhantomData;

use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_writer_data::DiscoveredWriterData,
        },
        dds::{
            dds_data_reader::DdsDataReader, dds_domain_participant::DdsDomainParticipant,
            dds_domain_participant_listener::DdsDomainParticipantListener,
            dds_subscriber::DdsSubscriber, dds_subscriber_listener::DdsSubscriberListener,
            status_condition_impl::StatusConditionImpl,
        },
        rtps::{
            messages::overall_structure::{RtpsMessageHeader, RtpsMessageRead},
            types::Locator,
            writer_proxy::RtpsWriterProxy,
        },
        rtps_udp_psm::udp_transport::UdpTransportWrite,
        utils::{
            actor::{ActorAddress, CommandHandler, Mail, MailHandler},
            shared_object::{DdsRwLock, DdsShared},
        },
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataReaderQos, SubscriberQos, TopicQos},
        status::{StatusKind, SubscriptionMatchedStatus},
        time::Time,
    },
    subscription::{
        data_reader::Sample,
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic_definition::type_support::DdsRepresentation,
};

impl ActorAddress<DdsDataReader> {
    pub fn enable(&self) -> DdsResult<()> {
        struct Enable;

        impl Mail for Enable {
            type Result = ();
        }

        impl MailHandler<Enable> for DdsDataReader {
            fn handle(&mut self, _mail: Enable) -> <Enable as Mail>::Result {
                self.enable()
            }
        }
        self.send_blocking(Enable)
    }

    pub fn is_enabled(&self) -> DdsResult<bool> {
        struct IsEnabled;

        impl Mail for IsEnabled {
            type Result = bool;
        }

        impl MailHandler<IsEnabled> for DdsDataReader {
            fn handle(&mut self, _mail: IsEnabled) -> <IsEnabled as Mail>::Result {
                self.is_enabled()
            }
        }

        self.send_blocking(IsEnabled)
    }

    pub fn get_type_name(&self) -> DdsResult<String> {
        struct GetTypeName;

        impl Mail for GetTypeName {
            type Result = String;
        }

        impl MailHandler<GetTypeName> for DdsDataReader {
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

        impl MailHandler<GetTopicName> for DdsDataReader {
            fn handle(&mut self, _mail: GetTopicName) -> <GetTopicName as Mail>::Result {
                self.get_topic_name()
            }
        }

        self.send_blocking(GetTopicName)
    }

    pub fn get_statuscondition(&self) -> DdsResult<DdsShared<DdsRwLock<StatusConditionImpl>>> {
        struct GetStatusConditions;

        impl Mail for GetStatusConditions {
            type Result = DdsShared<DdsRwLock<StatusConditionImpl>>;
        }

        impl MailHandler<GetStatusConditions> for DdsDataReader {
            fn handle(
                &mut self,
                _mail: GetStatusConditions,
            ) -> <GetStatusConditions as Mail>::Result {
                self.get_statuscondition()
            }
        }
        self.send_blocking(GetStatusConditions)
    }

    pub fn get_subscription_matched_status(&self) -> DdsResult<SubscriptionMatchedStatus> {
        struct GetSubscriptionMatchedStatus;

        impl Mail for GetSubscriptionMatchedStatus {
            type Result = SubscriptionMatchedStatus;
        }

        impl MailHandler<GetSubscriptionMatchedStatus> for DdsDataReader {
            fn handle(
                &mut self,
                _mail: GetSubscriptionMatchedStatus,
            ) -> <GetSubscriptionMatchedStatus as Mail>::Result {
                self.get_subscription_matched_status()
            }
        }

        self.send_blocking(GetSubscriptionMatchedStatus)
    }

    pub fn get_matched_publications(&self) -> DdsResult<Vec<InstanceHandle>> {
        struct GetMatchedPublications;

        impl Mail for GetMatchedPublications {
            type Result = Vec<InstanceHandle>;
        }

        impl MailHandler<GetMatchedPublications> for DdsDataReader {
            fn handle(
                &mut self,
                _mail: GetMatchedPublications,
            ) -> <GetMatchedPublications as Mail>::Result {
                self.get_matched_publications()
            }
        }

        self.send_blocking(GetMatchedPublications)
    }

    pub fn get_matched_publication_data(
        &self,
        publication_handle: InstanceHandle,
    ) -> DdsResult<PublicationBuiltinTopicData> {
        struct GetMatchedPublicationData {
            publication_handle: InstanceHandle,
        }

        impl Mail for GetMatchedPublicationData {
            type Result = DdsResult<PublicationBuiltinTopicData>;
        }

        impl MailHandler<GetMatchedPublicationData> for DdsDataReader {
            fn handle(
                &mut self,
                mail: GetMatchedPublicationData,
            ) -> <GetMatchedPublicationData as Mail>::Result {
                self.get_matched_publication_data(mail.publication_handle)
            }
        }

        self.send_blocking(GetMatchedPublicationData { publication_handle })?
    }
}

impl ActorAddress<DdsDataReader> {
    pub fn add_matched_writer(
        &self,
        discovered_writer_data: DiscoveredWriterData,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        data_reader_address: ActorAddress<DdsDataReader>,
        subscriber_address: ActorAddress<DdsSubscriber>,
        participant_address: ActorAddress<DdsDomainParticipant>,
        subscriber_qos: SubscriberQos,
        subscriber_mask_listener: (Option<ActorAddress<DdsSubscriberListener>>, Vec<StatusKind>),
        participant_mask_listener: (
            Option<ActorAddress<DdsDomainParticipantListener>>,
            Vec<StatusKind>,
        ),
    ) -> DdsResult<()> {
        struct AddMatchedWriter {
            discovered_writer_data: DiscoveredWriterData,
            default_unicast_locator_list: Vec<Locator>,
            default_multicast_locator_list: Vec<Locator>,
            data_reader_address: ActorAddress<DdsDataReader>,
            subscriber_address: ActorAddress<DdsSubscriber>,
            participant_address: ActorAddress<DdsDomainParticipant>,
            subscriber_qos: SubscriberQos,
            subscriber_mask_listener:
                (Option<ActorAddress<DdsSubscriberListener>>, Vec<StatusKind>),
            participant_mask_listener: (
                Option<ActorAddress<DdsDomainParticipantListener>>,
                Vec<StatusKind>,
            ),
        }

        impl Mail for AddMatchedWriter {
            type Result = ();
        }

        impl MailHandler<AddMatchedWriter> for DdsDataReader {
            fn handle(&mut self, mail: AddMatchedWriter) -> <AddMatchedWriter as Mail>::Result {
                self.add_matched_writer(
                    mail.discovered_writer_data,
                    mail.default_unicast_locator_list,
                    mail.default_multicast_locator_list,
                    mail.data_reader_address,
                    mail.subscriber_address,
                    mail.participant_address,
                    mail.subscriber_qos,
                    mail.subscriber_mask_listener,
                    mail.participant_mask_listener,
                )
            }
        }

        self.send_blocking(AddMatchedWriter {
            discovered_writer_data,
            default_unicast_locator_list,
            default_multicast_locator_list,
            data_reader_address,
            subscriber_address,
            participant_address,
            subscriber_qos,
            subscriber_mask_listener,
            participant_mask_listener,
        })
    }

    pub fn remove_matched_writer(
        &self,
        discovered_writer_handle: InstanceHandle,
        data_reader_address: ActorAddress<DdsDataReader>,
        subscriber_address: ActorAddress<DdsSubscriber>,
        participant_address: ActorAddress<DdsDomainParticipant>,
        subscriber_mask_listener: (Option<ActorAddress<DdsSubscriberListener>>, Vec<StatusKind>),
        participant_mask_listener: (
            Option<ActorAddress<DdsDomainParticipantListener>>,
            Vec<StatusKind>,
        ),
    ) -> DdsResult<()> {
        struct RemoveMatchedWriter {
            discovered_writer_handle: InstanceHandle,
            data_reader_address: ActorAddress<DdsDataReader>,
            subscriber_address: ActorAddress<DdsSubscriber>,
            participant_address: ActorAddress<DdsDomainParticipant>,
            subscriber_mask_listener:
                (Option<ActorAddress<DdsSubscriberListener>>, Vec<StatusKind>),
            participant_mask_listener: (
                Option<ActorAddress<DdsDomainParticipantListener>>,
                Vec<StatusKind>,
            ),
        }

        impl Mail for RemoveMatchedWriter {
            type Result = ();
        }

        impl MailHandler<RemoveMatchedWriter> for DdsDataReader {
            fn handle(
                &mut self,
                mail: RemoveMatchedWriter,
            ) -> <RemoveMatchedWriter as Mail>::Result {
                self.remove_matched_writer(
                    mail.discovered_writer_handle,
                    mail.data_reader_address,
                    mail.subscriber_address,
                    mail.participant_address,
                    mail.subscriber_mask_listener,
                    mail.participant_mask_listener,
                )
            }
        }

        self.send_blocking(RemoveMatchedWriter {
            discovered_writer_handle,
            data_reader_address,
            subscriber_address,
            participant_address,
            subscriber_mask_listener,
            participant_mask_listener,
        })
    }

    pub fn set_qos(&self, qos: DataReaderQos) -> DdsResult<()> {
        struct SetQos {
            qos: DataReaderQos,
        }

        impl Mail for SetQos {
            type Result = DdsResult<()>;
        }

        impl MailHandler<SetQos> for DdsDataReader {
            fn handle(&mut self, mail: SetQos) -> <SetQos as Mail>::Result {
                self.set_qos(mail.qos)
            }
        }

        self.send_blocking(SetQos { qos })?
    }

    pub fn get_qos(&self) -> DdsResult<DataReaderQos> {
        struct GetQos;

        impl Mail for GetQos {
            type Result = DataReaderQos;
        }

        impl MailHandler<GetQos> for DdsDataReader {
            fn handle(&mut self, _mail: GetQos) -> <GetQos as Mail>::Result {
                self.get_qos()
            }
        }

        self.send_blocking(GetQos)
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

        impl CommandHandler<SendMessage> for DdsDataReader {
            fn handle(&mut self, mail: SendMessage) {
                self.send_message(mail.header, mail.udp_transport_write)
            }
        }

        self.send_command(SendMessage {
            header,
            udp_transport_write,
        })
    }

    pub fn matched_writer_add(&self, a_writer_proxy: RtpsWriterProxy) -> DdsResult<()> {
        struct MatchedWriterAdd {
            a_writer_proxy: RtpsWriterProxy,
        }

        impl Mail for MatchedWriterAdd {
            type Result = ();
        }

        impl MailHandler<MatchedWriterAdd> for DdsDataReader {
            fn handle(&mut self, mail: MatchedWriterAdd) -> <MatchedWriterAdd as Mail>::Result {
                self.matched_writer_add(mail.a_writer_proxy)
            }
        }

        self.send_blocking(MatchedWriterAdd { a_writer_proxy })
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        struct GetInstanceHandle;

        impl Mail for GetInstanceHandle {
            type Result = InstanceHandle;
        }

        impl MailHandler<GetInstanceHandle> for DdsDataReader {
            fn handle(&mut self, _mail: GetInstanceHandle) -> <GetInstanceHandle as Mail>::Result {
                self.guid().into()
            }
        }

        self.send_blocking(GetInstanceHandle)
    }

    pub fn read_next_instance<Foo>(
        &self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: DdsRepresentation + for<'de> serde::Deserialize<'de> + Send + 'static,
    {
        struct ReadNextInstance<Foo> {
            phantom: PhantomData<Foo>,
            max_samples: i32,
            previous_handle: Option<InstanceHandle>,
            sample_states: Vec<SampleStateKind>,
            view_states: Vec<ViewStateKind>,
            instance_states: Vec<InstanceStateKind>,
        }

        impl<Foo> Mail for ReadNextInstance<Foo> {
            type Result = DdsResult<Vec<Sample<Foo>>>;
        }

        impl<Foo> MailHandler<ReadNextInstance<Foo>> for DdsDataReader
        where
            Foo: DdsRepresentation + for<'de> serde::Deserialize<'de> + Send + 'static,
        {
            fn handle(
                &mut self,
                mail: ReadNextInstance<Foo>,
            ) -> <ReadNextInstance<Foo> as Mail>::Result {
                self.read_next_instance(
                    mail.max_samples,
                    mail.previous_handle,
                    &mail.sample_states,
                    &mail.view_states,
                    &mail.instance_states,
                )
            }
        }

        self.send_blocking(ReadNextInstance {
            phantom: PhantomData,
            max_samples,
            previous_handle,
            sample_states: sample_states.to_vec(),
            view_states: view_states.to_vec(),
            instance_states: instance_states.to_vec(),
        })?
    }

    pub fn take<Foo>(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: DdsRepresentation + for<'de> serde::Deserialize<'de> + Send + 'static,
    {
        struct Take<Foo> {
            phantom: PhantomData<Foo>,
            max_samples: i32,
            sample_states: Vec<SampleStateKind>,
            view_states: Vec<ViewStateKind>,
            instance_states: Vec<InstanceStateKind>,
            specific_instance_handle: Option<InstanceHandle>,
        }

        impl<Foo> Mail for Take<Foo> {
            type Result = DdsResult<Vec<Sample<Foo>>>;
        }

        impl<Foo> MailHandler<Take<Foo>> for DdsDataReader
        where
            Foo: DdsRepresentation + for<'de> serde::Deserialize<'de> + Send + 'static,
        {
            fn handle(&mut self, mail: Take<Foo>) -> <Take<Foo> as Mail>::Result {
                self.take(
                    mail.max_samples,
                    &mail.sample_states,
                    &mail.view_states,
                    &mail.instance_states,
                    mail.specific_instance_handle,
                )
            }
        }

        self.send_blocking(Take {
            phantom: PhantomData,
            max_samples,
            sample_states: sample_states.to_vec(),
            view_states: view_states.to_vec(),
            instance_states: instance_states.to_vec(),
            specific_instance_handle,
        })?
    }

    pub fn take_next_instance<Foo>(
        &self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: DdsRepresentation + for<'de> serde::Deserialize<'de> + Send + 'static,
    {
        struct TakeNextInstance<Foo> {
            phantom_data: PhantomData<Foo>,
            max_samples: i32,
            previous_handle: Option<InstanceHandle>,
            sample_states: Vec<SampleStateKind>,
            view_states: Vec<ViewStateKind>,
            instance_states: Vec<InstanceStateKind>,
        }

        impl<Foo> Mail for TakeNextInstance<Foo> {
            type Result = DdsResult<Vec<Sample<Foo>>>;
        }

        impl<Foo> MailHandler<TakeNextInstance<Foo>> for DdsDataReader
        where
            Foo: DdsRepresentation + for<'de> serde::Deserialize<'de> + Send + 'static,
        {
            fn handle(
                &mut self,
                mail: TakeNextInstance<Foo>,
            ) -> <TakeNextInstance<Foo> as Mail>::Result {
                self.take_next_instance(
                    mail.max_samples,
                    mail.previous_handle,
                    &mail.sample_states,
                    &mail.view_states,
                    &mail.instance_states,
                )
            }
        }

        self.send_blocking(TakeNextInstance {
            phantom_data: PhantomData,
            max_samples,
            previous_handle,
            sample_states,
            view_states,
            instance_states,
        })?
    }

    pub fn is_historical_data_received(&self) -> DdsResult<bool> {
        struct IsHistoricalDataReceived;

        impl Mail for IsHistoricalDataReceived {
            type Result = DdsResult<bool>;
        }

        impl MailHandler<IsHistoricalDataReceived> for DdsDataReader {
            fn handle(
                &mut self,
                _mail: IsHistoricalDataReceived,
            ) -> <IsHistoricalDataReceived as Mail>::Result {
                self.is_historical_data_received()
            }
        }

        self.send_blocking(IsHistoricalDataReceived)?
    }

    pub fn as_discovered_reader_data(
        &self,
        topic_qos: TopicQos,
        subscriber_qos: SubscriberQos,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
    ) -> DdsResult<DiscoveredReaderData> {
        struct AsDiscoveredReaderData {
            topic_qos: TopicQos,
            subscriber_qos: SubscriberQos,
            default_unicast_locator_list: Vec<Locator>,
            default_multicast_locator_list: Vec<Locator>,
        }

        impl Mail for AsDiscoveredReaderData {
            type Result = DiscoveredReaderData;
        }

        impl MailHandler<AsDiscoveredReaderData> for DdsDataReader {
            fn handle(
                &mut self,
                mail: AsDiscoveredReaderData,
            ) -> <AsDiscoveredReaderData as Mail>::Result {
                self.as_discovered_reader_data(
                    mail.topic_qos,
                    mail.subscriber_qos,
                    mail.default_unicast_locator_list,
                    mail.default_multicast_locator_list,
                )
            }
        }

        self.send_blocking(AsDiscoveredReaderData {
            topic_qos,
            subscriber_qos,
            default_unicast_locator_list,
            default_multicast_locator_list,
        })
    }

    pub fn update_communication_status(
        &self,
        now: Time,
        data_reader_address: ActorAddress<DdsDataReader>,
        subscriber_address: ActorAddress<DdsSubscriber>,
        participant_address: ActorAddress<DdsDomainParticipant>,
        subscriber_mask_listener: (Option<ActorAddress<DdsSubscriberListener>>, Vec<StatusKind>),
        participant_mask_listener: (
            Option<ActorAddress<DdsDomainParticipantListener>>,
            Vec<StatusKind>,
        ),
    ) -> DdsResult<()> {
        struct UpdateCommunicationStatus {
            now: Time,
            data_reader_address: ActorAddress<DdsDataReader>,
            subscriber_address: ActorAddress<DdsSubscriber>,
            participant_address: ActorAddress<DdsDomainParticipant>,
            subscriber_mask_listener:
                (Option<ActorAddress<DdsSubscriberListener>>, Vec<StatusKind>),
            participant_mask_listener: (
                Option<ActorAddress<DdsDomainParticipantListener>>,
                Vec<StatusKind>,
            ),
        }

        impl CommandHandler<UpdateCommunicationStatus> for DdsDataReader {
            fn handle(&mut self, mail: UpdateCommunicationStatus) {
                self.update_communication_status(
                    mail.now,
                    mail.data_reader_address,
                    mail.subscriber_address,
                    mail.participant_address,
                    mail.subscriber_mask_listener,
                    mail.participant_mask_listener,
                )
            }
        }

        self.send_command(UpdateCommunicationStatus {
            now,
            data_reader_address,
            subscriber_address,
            participant_address,
            subscriber_mask_listener,
            participant_mask_listener,
        })
    }
}

impl ActorAddress<DdsDataReader> {
    pub fn read<Foo>(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: DdsRepresentation + for<'de> serde::Deserialize<'de> + Send + 'static,
    {
        struct Read<Foo> {
            phantom: PhantomData<Foo>,
            max_samples: i32,
            sample_states: Vec<SampleStateKind>,
            view_states: Vec<ViewStateKind>,
            instance_states: Vec<InstanceStateKind>,
            specific_instance_handle: Option<InstanceHandle>,
        }

        impl<Foo> Mail for Read<Foo> {
            type Result = DdsResult<Vec<Sample<Foo>>>;
        }

        impl<Foo> MailHandler<Read<Foo>> for DdsDataReader
        where
            Foo: DdsRepresentation + for<'de> serde::Deserialize<'de> + Send + 'static,
        {
            fn handle(&mut self, mail: Read<Foo>) -> <Read<Foo> as Mail>::Result {
                self.read(
                    mail.max_samples,
                    &mail.sample_states,
                    &mail.view_states,
                    &mail.instance_states,
                    mail.specific_instance_handle,
                )
            }
        }

        self.send_blocking(Read {
            phantom: PhantomData,
            max_samples,
            sample_states: sample_states.to_vec(),
            view_states: view_states.to_vec(),
            instance_states: instance_states.to_vec(),
            specific_instance_handle,
        })?
    }

    pub fn process_rtps_message(
        &self,
        message: RtpsMessageRead,
        reception_timestamp: Time,
        data_reader_address: ActorAddress<DdsDataReader>,
        subscriber_address: ActorAddress<DdsSubscriber>,
        participant_address: ActorAddress<DdsDomainParticipant>,
        subscriber_status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
        subscriber_mask_listener: (Option<ActorAddress<DdsSubscriberListener>>, Vec<StatusKind>),
        participant_mask_listener: (
            Option<ActorAddress<DdsDomainParticipantListener>>,
            Vec<StatusKind>,
        ),
    ) -> DdsResult<()> {
        struct ProcessRtpsMessage {
            message: RtpsMessageRead,
            reception_timestamp: Time,
            data_reader_address: ActorAddress<DdsDataReader>,
            subscriber_address: ActorAddress<DdsSubscriber>,
            participant_address: ActorAddress<DdsDomainParticipant>,
            subscriber_status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
            subscriber_mask_listener:
                (Option<ActorAddress<DdsSubscriberListener>>, Vec<StatusKind>),
            participant_mask_listener: (
                Option<ActorAddress<DdsDomainParticipantListener>>,
                Vec<StatusKind>,
            ),
        }

        impl CommandHandler<ProcessRtpsMessage> for DdsDataReader {
            fn handle(&mut self, mail: ProcessRtpsMessage) {
                self.process_rtps_message(
                    mail.message,
                    mail.reception_timestamp,
                    mail.data_reader_address,
                    mail.subscriber_address,
                    mail.participant_address,
                    mail.subscriber_status_condition,
                    mail.subscriber_mask_listener,
                    mail.participant_mask_listener,
                )
            }
        }

        self.send_command(ProcessRtpsMessage {
            message,
            reception_timestamp,
            data_reader_address,
            subscriber_address,
            participant_address,
            subscriber_status_condition,
            subscriber_mask_listener,
            participant_mask_listener,
        })
    }
}
