use crate::{
    implementation::rtps::{
        messages::{overall_structure::RtpsMessageHeader, submessages::GapSubmessage},
        stateful_reader::{
            StatefulReaderDataReceivedResult, DEFAULT_HEARTBEAT_RESPONSE_DELAY,
            DEFAULT_HEARTBEAT_SUPPRESSION_DURATION,
        },
        types::TopicKind,
    },
    infrastructure::qos::DataReaderQos,
    infrastructure::{
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            ReliabilityQosPolicy,
        },
        status::StatusKind,
        time::DURATION_ZERO,
    },
    subscription::data_reader::Sample,
    topic_definition::type_support::DdsType,
};
use crate::{
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::DiscoveredWriterData,
        },
        rtps::{
            endpoint::RtpsEndpoint,
            messages::submessages::{DataSubmessage, HeartbeatSubmessage},
            reader::RtpsReader,
            stateful_reader::RtpsStatefulReader,
            transport::TransportWrite,
            types::{Guid, GuidPrefix},
        },
        utils::shared_object::{DdsRwLock, DdsShared},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos_policy::ReliabilityQosPolicyKind,
    },
    subscription::sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    topic_definition::type_support::DdsDeserialize,
};

use super::{
    domain_participant_impl::{
        ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
    },
    message_receiver::MessageReceiver,
    participant_discovery::ParticipantDiscovery,
    status_condition_impl::StatusConditionImpl,
    topic_impl::TopicImpl,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BuiltInStatefulReaderDataSubmessageReceivedResult {
    NoChange,
    NewDataAvailable,
}

pub struct BuiltinStatefulReader {
    rtps_reader: DdsRwLock<RtpsStatefulReader>,
    topic: DdsShared<TopicImpl>,
    enabled: DdsRwLock<bool>,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
}

impl BuiltinStatefulReader {
    pub fn new<Foo>(guid: Guid, topic: DdsShared<TopicImpl>) -> DdsShared<Self>
    where
        Foo: DdsType + for<'de> DdsDeserialize<'de>,
    {
        let qos = DataReaderQos {
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal,
            },
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast,
                depth: 1,
            },
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::Reliable,
                max_blocking_time: DURATION_ZERO,
            },
            ..Default::default()
        };
        let topic_kind = TopicKind::WithKey;
        let heartbeat_response_delay = DEFAULT_HEARTBEAT_RESPONSE_DELAY;
        let heartbeat_suppression_duration = DEFAULT_HEARTBEAT_SUPPRESSION_DURATION;
        let expects_inline_qos = false;
        let unicast_locator_list = &[];
        let multicast_locator_list = &[];
        let sedp_builtin_publications_rtps_reader =
            RtpsStatefulReader::new(RtpsReader::new::<Foo>(
                RtpsEndpoint::new(
                    guid,
                    topic_kind,
                    unicast_locator_list,
                    multicast_locator_list,
                ),
                heartbeat_response_delay,
                heartbeat_suppression_duration,
                expects_inline_qos,
                qos,
            ));

        DdsShared::new(BuiltinStatefulReader {
            rtps_reader: DdsRwLock::new(sedp_builtin_publications_rtps_reader),
            topic,
            enabled: DdsRwLock::new(false),
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
        })
    }
}

impl DdsShared<BuiltinStatefulReader> {
    pub fn add_matched_participant(&self, participant_discovery: &ParticipantDiscovery) {
        let mut rtps_reader_lock = self.rtps_reader.write_lock();

        let type_name = self.topic.get_type_name();
        if type_name == DiscoveredWriterData::type_name() {
            participant_discovery
                .discovered_participant_add_publications_reader(&mut rtps_reader_lock);
        } else if type_name == DiscoveredReaderData::type_name() {
            participant_discovery
                .discovered_participant_add_subscriptions_reader(&mut rtps_reader_lock);
        } else if type_name == DiscoveredTopicData::type_name() {
            participant_discovery.discovered_participant_add_topics_reader(&mut rtps_reader_lock);
        }
    }

    pub fn remove_matched_participant(&self, participant_guid_prefix: GuidPrefix) {
        let type_name = self.topic.get_type_name();
        if type_name == DiscoveredWriterData::type_name() {
            self.rtps_reader
                .write_lock()
                .matched_writer_remove(Guid::new(
                    participant_guid_prefix,
                    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                ));
        } else if type_name == DiscoveredReaderData::type_name() {
            self.rtps_reader
                .write_lock()
                .matched_writer_remove(Guid::new(
                    participant_guid_prefix,
                    ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
                ));
        } else if type_name == DiscoveredTopicData::type_name() {
            self.rtps_reader
                .write_lock()
                .matched_writer_remove(Guid::new(
                    participant_guid_prefix,
                    ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
                ));
        }
    }

    pub fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) -> BuiltInStatefulReaderDataSubmessageReceivedResult {
        let r = self
            .rtps_reader
            .write_lock()
            .on_data_submessage_received(data_submessage, message_receiver);

        match r {
            StatefulReaderDataReceivedResult::NoMatchedWriterProxy
            | StatefulReaderDataReceivedResult::UnexpectedDataSequenceNumber
            | StatefulReaderDataReceivedResult::SampleRejected(_, _)
            | StatefulReaderDataReceivedResult::InvalidData(_) => {
                BuiltInStatefulReaderDataSubmessageReceivedResult::NoChange
            }

            StatefulReaderDataReceivedResult::NewSampleAdded(_)
            | StatefulReaderDataReceivedResult::NewSampleAddedAndSamplesLost(_) => {
                BuiltInStatefulReaderDataSubmessageReceivedResult::NewDataAvailable
            }
        }
    }

    pub fn on_heartbeat_submessage_received(
        &self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        self.rtps_reader
            .write_lock()
            .on_heartbeat_submessage_received(heartbeat_submessage, source_guid_prefix);
    }

    pub fn read<Foo>(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_reader.write_lock().reader_mut().read(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )
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
        Foo: for<'de> DdsDeserialize<'de>,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_reader
            .write_lock()
            .reader_mut()
            .read_next_instance(
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
            )
    }

    pub fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;

        Ok(())
    }

    pub fn send_message(&self, header: RtpsMessageHeader, transport: &mut impl TransportWrite) {
        self.rtps_reader
            .write_lock()
            .send_message(header, transport);
    }

    pub fn get_qos(&self) -> DataReaderQos {
        self.rtps_reader.read_lock().reader().get_qos().clone()
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_reader.read_lock().reader().guid().into()
    }

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }

    pub fn on_data_available(&self) {
        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::DataAvailable);
    }

    pub fn on_gap_submessage_received(
        &self,
        gap_submessage: &GapSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        self.rtps_reader
            .write_lock()
            .on_gap_submessage_received(gap_submessage, source_guid_prefix);
    }
}
