use std::collections::HashMap;

use crate::{
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::DiscoveredWriterData,
        },
        rtps::{
            endpoint::RtpsEndpoint,
            instance_handle_builder::InstanceHandleBuilder,
            messages::submessages::{DataSubmessage, HeartbeatSubmessage},
            reader::RtpsReader,
            stateful_reader::RtpsStatefulReader,
            transport::TransportWrite,
            types::{Guid, GuidPrefix},
        },
        utils::{
            shared_object::{DdsRwLock, DdsShared},
            timer::Timer,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::{InstanceHandle, HANDLE_NIL},
        qos_policy::ReliabilityQosPolicyKind,
        status::{
            LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
            SampleLostStatus, SampleRejectedStatus, SampleRejectedStatusKind, StatusKind,
            SubscriptionMatchedStatus,
        },
    },
    subscription::sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    topic_definition::type_support::DdsDeserialize,
};
use crate::{
    implementation::{
        rtps::{
            stateful_reader::{
                DEFAULT_HEARTBEAT_RESPONSE_DELAY, DEFAULT_HEARTBEAT_SUPPRESSION_DURATION,
            },
            types::TopicKind,
        },
        utils::timer::ThreadTimer,
    },
    infrastructure::{
        qos_policy::{HistoryQosPolicy, HistoryQosPolicyKind, ReliabilityQosPolicy},
        time::DURATION_ZERO,
    },
    subscription::data_reader::Sample,
    topic_definition::type_support::DdsType,
    {builtin_topics::PublicationBuiltinTopicData, infrastructure::qos::DataReaderQos},
};

use super::{
    message_receiver::MessageReceiver, participant_discovery::ParticipantDiscovery,
    status_condition_impl::StatusConditionImpl, topic_impl::TopicImpl,
};

pub struct BuiltinStatefulReader<Tim> {
    rtps_reader: DdsRwLock<RtpsStatefulReader>,
    topic: DdsShared<TopicImpl>,
    deadline_timer: DdsRwLock<Tim>,
    _liveliness_changed_status: DdsRwLock<LivelinessChangedStatus>,
    requested_deadline_missed_status: DdsRwLock<RequestedDeadlineMissedStatus>,
    _requested_incompatible_qos_status: DdsRwLock<RequestedIncompatibleQosStatus>,
    _sample_lost_status: DdsRwLock<SampleLostStatus>,
    _sample_rejected_status: DdsRwLock<SampleRejectedStatus>,
    _subscription_matched_status: DdsRwLock<SubscriptionMatchedStatus>,
    _matched_publication_list: DdsRwLock<HashMap<InstanceHandle, PublicationBuiltinTopicData>>,
    enabled: DdsRwLock<bool>,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
}

impl<Tim> BuiltinStatefulReader<Tim>
where
    Tim: Timer,
{
    pub fn new<Foo>(guid: Guid, topic: DdsShared<TopicImpl>) -> DdsShared<Self>
    where
        Foo: DdsType + for<'de> DdsDeserialize<'de>,
    {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAll,
                depth: 0,
            },
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::Reliable,
                max_blocking_time: DURATION_ZERO,
            },
            ..Default::default()
        };
        let deadline_duration = std::time::Duration::from_secs(qos.deadline.period.sec() as u64)
            + std::time::Duration::from_nanos(qos.deadline.period.nanosec() as u64);
        let topic_kind = TopicKind::WithKey;
        let heartbeat_response_delay = DEFAULT_HEARTBEAT_RESPONSE_DELAY;
        let heartbeat_suppression_duration = DEFAULT_HEARTBEAT_SUPPRESSION_DURATION;
        let expects_inline_qos = false;
        let unicast_locator_list = &[];
        let multicast_locator_list = &[];
        let sedp_builtin_publications_rtps_reader = RtpsStatefulReader::new(RtpsReader::new(
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
            InstanceHandleBuilder::new::<Foo>(),
        ));

        DdsShared::new(BuiltinStatefulReader {
            rtps_reader: DdsRwLock::new(sedp_builtin_publications_rtps_reader),
            topic,
            deadline_timer: DdsRwLock::new(Tim::new(deadline_duration)),
            _liveliness_changed_status: DdsRwLock::new(LivelinessChangedStatus {
                alive_count: 0,
                not_alive_count: 0,
                alive_count_change: 0,
                not_alive_count_change: 0,
                last_publication_handle: HANDLE_NIL,
            }),
            requested_deadline_missed_status: DdsRwLock::new(RequestedDeadlineMissedStatus {
                total_count: 0,
                total_count_change: 0,
                last_instance_handle: HANDLE_NIL,
            }),
            _requested_incompatible_qos_status: DdsRwLock::new(RequestedIncompatibleQosStatus {
                total_count: 0,
                total_count_change: 0,
                last_policy_id: 0,
                policies: Vec::new(),
            }),
            _sample_lost_status: DdsRwLock::new(SampleLostStatus {
                total_count: 0,
                total_count_change: 0,
            }),
            _sample_rejected_status: DdsRwLock::new(SampleRejectedStatus {
                total_count: 0,
                total_count_change: 0,
                last_reason: SampleRejectedStatusKind::NotRejected,
                last_instance_handle: HANDLE_NIL,
            }),
            _subscription_matched_status: DdsRwLock::new(SubscriptionMatchedStatus {
                total_count: 0,
                total_count_change: 0,
                last_publication_handle: HANDLE_NIL,
                current_count: 0,
                current_count_change: 0,
            }),
            _matched_publication_list: DdsRwLock::new(HashMap::new()),
            enabled: DdsRwLock::new(false),
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
        })
    }
}

impl<Tim> BuiltinStatefulReader<Tim> {
    pub fn add_matched_participant(&self, participant_discovery: &ParticipantDiscovery) {
        let mut rtps_reader_lock = self.rtps_reader.write_lock();

        if !rtps_reader_lock
            .matched_writers()
            .iter_mut()
            .any(|r| r.remote_writer_guid().prefix() == participant_discovery.guid_prefix())
        {
            let type_name = self.topic.get_type_name().unwrap();
            if type_name == DiscoveredWriterData::type_name() {
                participant_discovery
                    .discovered_participant_add_publications_reader(&mut *rtps_reader_lock);
            } else if type_name == DiscoveredReaderData::type_name() {
                participant_discovery
                    .discovered_participant_add_subscriptions_reader(&mut *rtps_reader_lock);
            } else if type_name == DiscoveredTopicData::type_name() {
                participant_discovery
                    .discovered_participant_add_topics_reader(&mut *rtps_reader_lock);
            }
        }
    }
}

impl DdsShared<BuiltinStatefulReader<ThreadTimer>> {
    pub fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) {
        let before_data_cache_len = self.rtps_reader.write_lock().reader_mut().changes().len();

        self.rtps_reader
            .write_lock()
            .on_data_submessage_received(data_submessage, message_receiver);

        let after_data_cache_len = self.rtps_reader.write_lock().reader_mut().changes().len();

        if before_data_cache_len < after_data_cache_len {
            let reader_shared = self.clone();
            self.deadline_timer.write_lock().on_deadline(move || {
                reader_shared
                    .requested_deadline_missed_status
                    .write_lock()
                    .total_count += 1;
                reader_shared
                    .requested_deadline_missed_status
                    .write_lock()
                    .total_count_change += 1;

                reader_shared
                    .status_condition
                    .write_lock()
                    .add_communication_state(StatusKind::RequestedDeadlineMissed);
            });

            self.status_condition
                .write_lock()
                .add_communication_state(StatusKind::DataAvailable);
        }
    }
}

impl<Tim> DdsShared<BuiltinStatefulReader<Tim>> {
    pub fn on_heartbeat_submessage_received(
        &self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        self.rtps_reader
            .write_lock()
            .on_heartbeat_submessage_received(heartbeat_submessage, source_guid_prefix);
    }
}

impl<Tim> DdsShared<BuiltinStatefulReader<Tim>>
where
    Tim: Timer,
{
    pub fn take<Foo>(
        &self,
        max_samples: i32,
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

        self.rtps_reader.write_lock().reader_mut().take(
            max_samples,
            sample_states,
            view_states,
            instance_states,
        )
    }
}

impl<Tim> DdsShared<BuiltinStatefulReader<Tim>> {
    pub fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;

        Ok(())
    }
}

impl<Tim> DdsShared<BuiltinStatefulReader<Tim>> {
    pub fn send_message(&self, transport: &mut impl TransportWrite) {
        self.rtps_reader.write_lock().send_message(transport);
    }
}
