use std::collections::{HashMap, HashSet};

use crate::{
    data_representation_builtin_endpoints::{
        discovered_reader_data::DiscoveredReaderData, discovered_topic_data::DiscoveredTopicData,
        discovered_writer_data::DiscoveredWriterData,
        spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
    },
    dds_type::{DdsDeserialize, DdsType},
    rtps_impl::{
        rtps_history_cache_impl::{RtpsCacheChangeImpl, RtpsHistoryCacheImpl},
        rtps_stateful_reader_impl::RtpsStatefulReaderImpl,
        rtps_stateless_reader_impl::RtpsStatelessReaderImpl,
        rtps_writer_proxy_impl::RtpsWriterProxyImpl,
    },
    transport::{RtpsMessage, RtpsSubmessageType, TransportWrite},
    utils::{
        discovery_traits::AddMatchedWriter,
        rtps_communication_traits::{
            ReceiveRtpsDataSubmessage, ReceiveRtpsHeartbeatSubmessage, SendRtpsMessage,
        },
        shared_object::{DdsRwLock, DdsShared, DdsWeak},
        timer::Timer,
    },
};
use dds_api::{
    builtin_topics::PublicationBuiltinTopicData,
    dcps_psm::{
        InstanceHandle, InstanceStateMask, LivelinessChangedStatus, QosPolicyCount,
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SampleRejectedStatusKind, SampleStateMask, StatusMask,
        SubscriptionMatchedStatus, Time, ViewStateMask, ALIVE_INSTANCE_STATE,
        DATA_AVAILABLE_STATUS, HANDLE_NIL, HANDLE_NIL_NATIVE, NEW_VIEW_STATE,
        NOT_ALIVE_DISPOSED_INSTANCE_STATE, NOT_READ_SAMPLE_STATE, READ_SAMPLE_STATE,
        REQUESTED_DEADLINE_MISSED_STATUS, SUBSCRIPTION_MATCHED_STATUS,
    },
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::DataReaderQos,
        qos_policy::{
            HistoryQosPolicyKind, DEADLINE_QOS_POLICY_ID, DESTINATIONORDER_QOS_POLICY_ID,
            DURABILITY_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID, LIVELINESS_QOS_POLICY_ID,
            OWNERSHIPSTRENGTH_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID, RELIABILITY_QOS_POLICY_ID,
        },
        read_condition::ReadCondition,
        sample_info::SampleInfo,
    },
    return_type::{DdsError, DdsResult},
    subscription::{
        data_reader::{
            DataReader, DataReaderGetSubscriber, DataReaderGetTopicDescription, FooDataReader,
            Sample,
        },
        data_reader_listener::DataReaderListener,
        query_condition::QueryCondition,
    },
    topic::topic_description::TopicDescription,
};
use rtps_pim::{
    behavior::{
        reader::{
            reader::RtpsReaderAttributes,
            stateful_reader::{RtpsStatefulReaderAttributes, RtpsStatefulReaderOperations},
            writer_proxy::{RtpsWriterProxyAttributes, RtpsWriterProxyConstructor},
        },
        stateful_reader_behavior::{
            RtpsStatefulReaderReceiveDataSubmessage, RtpsStatefulReaderReceiveHeartbeatSubmessage,
            RtpsStatefulReaderSendSubmessages,
        },
        stateless_reader_behavior::RtpsStatelessReaderReceiveDataSubmessage,
    },
    discovery::{
        participant_discovery::ParticipantDiscovery,
        spdp::spdp_discovered_participant_data::RtpsSpdpDiscoveredParticipantDataAttributes,
    },
    messages::{
        overall_structure::RtpsMessageHeader,
        submessage_elements::Parameter,
        submessages::{DataSubmessage, HeartbeatSubmessage},
    },
    structure::{
        cache_change::RtpsCacheChangeAttributes,
        entity::RtpsEntityAttributes,
        history_cache::{RtpsHistoryCacheAttributes, RtpsHistoryCacheOperations},
        types::{ChangeKind, Guid, GuidPrefix, SequenceNumber, PROTOCOLVERSION, VENDOR_ID_S2E},
    },
};

use super::{subscriber_impl::SubscriberImpl, topic_impl::TopicImpl};

pub trait AnyDataReaderListener<DR> {
    fn trigger_on_data_available(&mut self, reader: DR);
    fn trigger_on_sample_rejected(&mut self, reader: DR, status: SampleRejectedStatus);
    fn trigger_on_liveliness_changed(&mut self, reader: DR, status: LivelinessChangedStatus);
    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader: DR,
        status: RequestedDeadlineMissedStatus,
    );
    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader: DR,
        status: RequestedIncompatibleQosStatus,
    );
    fn trigger_on_subscription_matched(&mut self, reader: DR, status: SubscriptionMatchedStatus);
    fn trigger_on_sample_lost(&mut self, reader: DR, status: SampleLostStatus);
}

impl<Foo, DR> AnyDataReaderListener<DR> for Box<dyn DataReaderListener<Foo = Foo> + Send + Sync>
where
    DR: FooDataReader<Foo>,
{
    fn trigger_on_data_available(&mut self, reader: DR) {
        self.on_data_available(&reader)
    }

    fn trigger_on_sample_rejected(&mut self, reader: DR, status: SampleRejectedStatus) {
        self.on_sample_rejected(&reader, status)
    }

    fn trigger_on_liveliness_changed(&mut self, reader: DR, status: LivelinessChangedStatus) {
        self.on_liveliness_changed(&reader, status)
    }

    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader: DR,
        status: RequestedDeadlineMissedStatus,
    ) {
        self.on_requested_deadline_missed(&reader, status)
    }

    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader: DR,
        status: RequestedIncompatibleQosStatus,
    ) {
        self.on_requested_incompatible_qos(&reader, status)
    }

    fn trigger_on_subscription_matched(&mut self, reader: DR, status: SubscriptionMatchedStatus) {
        self.on_subscription_matched(&reader, status)
    }

    fn trigger_on_sample_lost(&mut self, reader: DR, status: SampleLostStatus) {
        self.on_sample_lost(&reader, status)
    }
}

pub enum RtpsReader {
    Stateless(RtpsStatelessReaderImpl),
    Stateful(RtpsStatefulReaderImpl),
}

impl RtpsReaderAttributes for RtpsReader {
    type HistoryCacheType = RtpsHistoryCacheImpl;

    fn heartbeat_response_delay(&self) -> rtps_pim::behavior::types::Duration {
        match self {
            RtpsReader::Stateless(reader) => reader.heartbeat_response_delay(),
            RtpsReader::Stateful(reader) => reader.heartbeat_response_delay(),
        }
    }

    fn heartbeat_suppression_duration(&self) -> rtps_pim::behavior::types::Duration {
        match self {
            RtpsReader::Stateless(reader) => reader.heartbeat_suppression_duration(),
            RtpsReader::Stateful(reader) => reader.heartbeat_suppression_duration(),
        }
    }

    fn reader_cache(&mut self) -> &mut Self::HistoryCacheType {
        match self {
            RtpsReader::Stateless(reader) => reader.reader_cache(),
            RtpsReader::Stateful(reader) => reader.reader_cache(),
        }
    }

    fn expects_inline_qos(&self) -> bool {
        match self {
            RtpsReader::Stateless(reader) => reader.expects_inline_qos(),
            RtpsReader::Stateful(reader) => reader.expects_inline_qos(),
        }
    }
}

impl RtpsEntityAttributes for RtpsReader {
    fn guid(&self) -> Guid {
        match self {
            RtpsReader::Stateless(r) => r.guid(),
            RtpsReader::Stateful(r) => r.guid(),
        }
    }
}

pub struct DataReaderImpl<Tim> {
    rtps_reader: DdsRwLock<RtpsReader>,
    qos: DdsRwLock<DataReaderQos>,
    topic: DdsShared<TopicImpl>,
    listener: DdsRwLock<Option<<DdsShared<Self> as Entity>::Listener>>,
    parent_subscriber: DdsWeak<SubscriberImpl>,
    samples_read: DdsRwLock<HashSet<SequenceNumber>>,
    deadline_timer: DdsRwLock<Tim>,
    status_change: DdsRwLock<StatusMask>,
    liveliness_changed_status: DdsRwLock<LivelinessChangedStatus>,
    requested_deadline_missed_status: DdsRwLock<RequestedDeadlineMissedStatus>,
    requested_incompatible_qos_status: DdsRwLock<RequestedIncompatibleQosStatus>,
    sample_lost_status: DdsRwLock<SampleLostStatus>,
    sample_rejected_status: DdsRwLock<SampleRejectedStatus>,
    subscription_matched_status: DdsRwLock<SubscriptionMatchedStatus>,
    matched_publication_list: DdsRwLock<HashMap<InstanceHandle, PublicationBuiltinTopicData>>,
    enabled: DdsRwLock<bool>,
}

impl<Tim> DataReaderImpl<Tim>
where
    Tim: Timer,
{
    pub fn new(
        qos: DataReaderQos,
        rtps_reader: RtpsReader,
        topic: DdsShared<TopicImpl>,
        listener: Option<<DdsShared<Self> as Entity>::Listener>,
        parent_subscriber: DdsWeak<SubscriberImpl>,
    ) -> DdsShared<Self> {
        let deadline_duration = std::time::Duration::from_secs(*qos.deadline.period.sec() as u64)
            + std::time::Duration::from_nanos(*qos.deadline.period.nanosec() as u64);

        DdsShared::new(DataReaderImpl {
            rtps_reader: DdsRwLock::new(rtps_reader),
            qos: DdsRwLock::new(qos),
            topic,
            listener: DdsRwLock::new(listener),
            parent_subscriber,
            samples_read: DdsRwLock::new(HashSet::new()),
            deadline_timer: DdsRwLock::new(Tim::new(deadline_duration)),
            status_change: DdsRwLock::new(0),
            liveliness_changed_status: DdsRwLock::new(LivelinessChangedStatus {
                alive_count: 0,
                not_alive_count: 0,
                alive_count_change: 0,
                not_alive_count_change: 0,
                last_publication_handle: HANDLE_NIL,
            }),
            requested_deadline_missed_status: DdsRwLock::new(RequestedDeadlineMissedStatus {
                total_count: 0,
                total_count_change: 0,
                last_instance_handle: HANDLE_NIL_NATIVE,
            }),
            requested_incompatible_qos_status: DdsRwLock::new(RequestedIncompatibleQosStatus {
                total_count: 0,
                total_count_change: 0,
                last_policy_id: 0,
                policies: Vec::new(),
            }),
            sample_lost_status: DdsRwLock::new(SampleLostStatus {
                total_count: 0,
                total_count_change: 0,
            }),
            sample_rejected_status: DdsRwLock::new(SampleRejectedStatus {
                total_count: 0,
                total_count_change: 0,
                last_reason: SampleRejectedStatusKind::NotRejected,
                last_instance_handle: HANDLE_NIL_NATIVE,
            }),
            subscription_matched_status: DdsRwLock::new(SubscriptionMatchedStatus {
                total_count: 0,
                total_count_change: 0,
                last_publication_handle: HANDLE_NIL_NATIVE,
                current_count: 0,
                current_count_change: 0,
            }),
            matched_publication_list: DdsRwLock::new(HashMap::new()),
            enabled: DdsRwLock::new(false),
        })
    }
}

fn read_sample<'a, Tim>(
    data_reader_attributes: &DataReaderImpl<Tim>,
    cache_change: &'a RtpsCacheChangeImpl,
) -> (&'a [u8], SampleInfo) {
    *data_reader_attributes.status_change.write_lock() &= !DATA_AVAILABLE_STATUS;

    let mut samples_read = data_reader_attributes.samples_read.write_lock();
    let data_value = cache_change.data_value();

    let sample_state = {
        let sn = cache_change.sequence_number();
        if samples_read.contains(&sn) {
            READ_SAMPLE_STATE
        } else {
            samples_read.insert(sn);
            NOT_READ_SAMPLE_STATE
        }
    };

    let (instance_state, valid_data) = match cache_change.kind() {
        ChangeKind::Alive => (ALIVE_INSTANCE_STATE, true),
        ChangeKind::NotAliveDisposed => (NOT_ALIVE_DISPOSED_INSTANCE_STATE, false),
        _ => unimplemented!(),
    };

    let sample_info = SampleInfo {
        sample_state,
        view_state: NEW_VIEW_STATE,
        instance_state,
        disposed_generation_count: 0,
        no_writers_generation_count: 0,
        sample_rank: 0,
        generation_rank: 0,
        absolute_generation_rank: 0,
        source_timestamp: Time { sec: 0, nanosec: 0 },
        instance_handle: HANDLE_NIL_NATIVE,
        publication_handle: HANDLE_NIL_NATIVE,
        valid_data,
    };

    (data_value, sample_info)
}

impl<Tim> DataReaderImpl<Tim> {
    pub fn add_matched_participant(
        &self,
        participant_discovery: &ParticipantDiscovery<SpdpDiscoveredParticipantData>,
    ) {
        let mut rtps_reader_lock = self.rtps_reader.write_lock();
        if let RtpsReader::Stateful(rtps_reader) = &mut *rtps_reader_lock {
            if !rtps_reader
                .matched_writers()
                .into_iter()
                .any(|r| r.remote_writer_guid().prefix == participant_discovery.guid_prefix())
            {
                let type_name = self.topic.get_type_name().unwrap();
                if type_name == DiscoveredWriterData::type_name() {
                    participant_discovery
                        .discovered_participant_add_publications_reader(rtps_reader);
                } else if type_name == DiscoveredReaderData::type_name() {
                    participant_discovery
                        .discovered_participant_add_subscriptions_reader(rtps_reader);
                } else if type_name == DiscoveredTopicData::type_name() {
                    participant_discovery.discovered_participant_add_topics_reader(rtps_reader);
                }
            }
        }
    }
}

impl<Tim> ReceiveRtpsDataSubmessage for DdsShared<DataReaderImpl<Tim>>
where
    Tim: Timer + Send + Sync + 'static,
{
    fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<Vec<Parameter>, &[u8]>,
        source_guid_prefix: GuidPrefix,
    ) {
        let before_data_cache_len;
        let after_data_cache_len;
        let mut rtps_reader = self.rtps_reader.write_lock();
        match &mut *rtps_reader {
            RtpsReader::Stateless(stateless_rtps_reader) => {
                before_data_cache_len = stateless_rtps_reader.reader_cache().changes().len();

                stateless_rtps_reader
                    .on_data_submessage_received(data_submessage, source_guid_prefix);

                after_data_cache_len = stateless_rtps_reader.reader_cache().changes().len();
            }
            RtpsReader::Stateful(stateful_rtps_reader) => {
                before_data_cache_len = stateful_rtps_reader.reader_cache().changes().len();

                stateful_rtps_reader
                    .on_data_submessage_received(data_submessage, source_guid_prefix);

                after_data_cache_len = stateful_rtps_reader.reader_cache().changes().len();
            }
        }
        // Call the listener after dropping the rtps_reader lock to avoid deadlock
        drop(rtps_reader);
        if before_data_cache_len < after_data_cache_len {
            DataReaderImpl::on_data_received(self.clone()).unwrap();
        }
    }
}

impl<Tim> ReceiveRtpsHeartbeatSubmessage for DdsShared<DataReaderImpl<Tim>> {
    fn on_heartbeat_submessage_received(
        &self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        let mut rtps_reader = self.rtps_reader.write_lock();
        if let RtpsReader::Stateful(stateful_rtps_reader) = &mut *rtps_reader {
            stateful_rtps_reader
                .on_heartbeat_submessage_received(heartbeat_submessage, source_guid_prefix);
        }
    }
}

impl<Tim> AddMatchedWriter for DdsShared<DataReaderImpl<Tim>>
where
    Tim: Timer,
{
    fn add_matched_writer(&self, discovered_writer_data: &DiscoveredWriterData) {
        let writer_info = &discovered_writer_data.publication_builtin_topic_data;
        let reader_topic_name = self.topic.get_name().unwrap();
        let reader_type_name = self.topic.get_type_name().unwrap();

        if writer_info.topic_name == reader_topic_name && writer_info.type_name == reader_type_name
        {
            let reader_qos_lock = self.qos.read_lock();
            let parent_subscriber_qos = self.get_subscriber().unwrap().get_qos().unwrap();

            let mut incompatible_qos_policy_list = Vec::new();

            if !(reader_qos_lock.durability >= writer_info.durability) {
                incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
            }
            if !(parent_subscriber_qos.presentation.access_scope
                <= writer_info.presentation.access_scope
                && parent_subscriber_qos.presentation.coherent_access
                    == writer_info.presentation.coherent_access
                && parent_subscriber_qos.presentation.ordered_access
                    == writer_info.presentation.ordered_access)
            {
                incompatible_qos_policy_list.push(PRESENTATION_QOS_POLICY_ID);
            }
            if !(reader_qos_lock.deadline <= writer_info.deadline) {
                incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
            }
            if !(reader_qos_lock.latency_budget <= writer_info.latency_budget) {
                incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
            }
            if !(reader_qos_lock.ownership == writer_info.ownership) {
                incompatible_qos_policy_list.push(OWNERSHIPSTRENGTH_QOS_POLICY_ID);
            }
            if !(reader_qos_lock.liveliness <= writer_info.liveliness) {
                incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
            }
            if !(reader_qos_lock.reliability.kind <= writer_info.reliability.kind) {
                incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
            }
            if !(reader_qos_lock.destination_order <= writer_info.destination_order) {
                incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
            }

            if incompatible_qos_policy_list.is_empty() {
                match &mut *self.rtps_reader.write_lock() {
                    RtpsReader::Stateless(_) => (),
                    RtpsReader::Stateful(r) => {
                        let writer_proxy = RtpsWriterProxyImpl::new(
                            discovered_writer_data.writer_proxy.remote_writer_guid,
                            discovered_writer_data
                                .writer_proxy
                                .unicast_locator_list
                                .as_ref(),
                            discovered_writer_data
                                .writer_proxy
                                .multicast_locator_list
                                .as_ref(),
                            discovered_writer_data.writer_proxy.data_max_size_serialized,
                            discovered_writer_data.writer_proxy.remote_group_entity_id,
                        );

                        r.matched_writer_add(writer_proxy);
                    }
                }
                self.matched_publication_list
                    .write_lock()
                    .insert(writer_info.key.value.into(), writer_info.clone());

                // Drop the subscription_matched_status_lock such that the listener can be triggered
                // if needed
                {
                    let mut subscription_matched_status_lock =
                        self.subscription_matched_status.write_lock();
                    subscription_matched_status_lock.total_count += 1;
                    subscription_matched_status_lock.total_count_change += 1;
                    subscription_matched_status_lock.current_count += 1;
                    subscription_matched_status_lock.current_count_change += 1;
                }

                if let Some(l) = self.listener.write_lock().as_mut() {
                    *self.status_change.write_lock() &= !SUBSCRIPTION_MATCHED_STATUS;
                    let subscription_matched_status =
                        self.get_subscription_matched_status().unwrap();
                    l.trigger_on_subscription_matched(self.clone(), subscription_matched_status)
                };
            } else {
                {
                    let mut requested_incompatible_qos_status_lock =
                        self.requested_incompatible_qos_status.write_lock();
                    requested_incompatible_qos_status_lock.total_count += 1;
                    requested_incompatible_qos_status_lock.total_count_change += 1;
                    requested_incompatible_qos_status_lock.last_policy_id =
                        incompatible_qos_policy_list[0];
                    for incompatible_qos_policy in incompatible_qos_policy_list.into_iter() {
                        if let Some(policy_count) = requested_incompatible_qos_status_lock
                            .policies
                            .iter_mut()
                            .find(|x| x.policy_id == incompatible_qos_policy)
                        {
                            policy_count.count += 1;
                        } else {
                            requested_incompatible_qos_status_lock
                                .policies
                                .push(QosPolicyCount {
                                    policy_id: incompatible_qos_policy,
                                    count: 1,
                                })
                        }
                    }
                }

                let mut listener_lock = self.listener.write_lock();
                if let Some(l) = listener_lock.as_mut() {
                    let requested_incompatible_qos_status =
                        self.get_requested_incompatible_qos_status().unwrap();
                    l.trigger_on_requested_incompatible_qos(
                        self.clone(),
                        requested_incompatible_qos_status,
                    )
                }
            }
        }
    }
}

impl<Tim> DataReaderImpl<Tim>
where
    Tim: Timer + Send + Sync + 'static,
{
    pub fn on_data_received(reader: DdsShared<Self>) -> DdsResult<()> {
        if reader.qos.read_lock().history.kind == HistoryQosPolicyKind::KeepLastHistoryQoS {
            let mut rtps_reader = reader.rtps_reader.write_lock();

            let cache_len = rtps_reader.reader_cache().changes().len() as i32;
            if cache_len > reader.qos.read_lock().history.depth {
                let mut seq_nums: Vec<_> = rtps_reader
                    .reader_cache()
                    .changes()
                    .iter()
                    .map(|c| c.sequence_number())
                    .collect();
                seq_nums.sort();

                let to_delete = &seq_nums
                    [0..(cache_len as usize - reader.qos.read_lock().history.depth as usize)];
                rtps_reader
                    .reader_cache()
                    .remove_change(|c| to_delete.contains(&c.sequence_number()));
            }
        }

        let reader_shared = reader.clone();
        reader.deadline_timer.write_lock().on_deadline(move || {
            reader_shared
                .requested_deadline_missed_status
                .write_lock()
                .total_count += 1;
            reader_shared
                .requested_deadline_missed_status
                .write_lock()
                .total_count_change += 1;

            *reader_shared.status_change.write_lock() |= REQUESTED_DEADLINE_MISSED_STATUS;
            reader_shared.listener.write_lock().as_mut().map(|l| {
                *reader_shared.status_change.write_lock() &= !REQUESTED_DEADLINE_MISSED_STATUS;
                l.trigger_on_requested_deadline_missed(
                    reader_shared.clone(),
                    reader_shared
                        .requested_deadline_missed_status
                        .read_lock()
                        .clone(),
                )
            });
        });

        *reader.status_change.write_lock() |= DATA_AVAILABLE_STATUS;
        if let Some(l) = reader.listener.write_lock().as_mut() {
            *reader.status_change.write_lock() &= !DATA_AVAILABLE_STATUS;
            l.trigger_on_data_available(reader.clone())
        };

        Ok(())
    }
}

impl<Tim> DataReaderGetSubscriber for DdsShared<DataReaderImpl<Tim>>
where
    Tim: Timer,
{
    type Subscriber = DdsShared<SubscriberImpl>;

    fn data_reader_get_subscriber(&self) -> DdsResult<Self::Subscriber> {
        Ok(self
            .parent_subscriber
            .upgrade()
            .expect("Failed to get parent subscriber of data reader"))
    }
}

impl<Tim> DataReaderGetTopicDescription for DdsShared<DataReaderImpl<Tim>>
where
    Tim: Timer,
{
    type TopicDescription = DdsShared<TopicImpl>;

    fn data_reader_get_topicdescription(&self) -> DdsResult<Self::TopicDescription> {
        Ok(self.topic.clone())
    }
}

impl<Tim, Foo> FooDataReader<Foo> for DdsShared<DataReaderImpl<Tim>>
where
    Tim: Timer,
    Foo: for<'de> DdsDeserialize<'de>,
{
    fn read(
        &self,
        max_samples: i32,
        sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
    ) -> DdsResult<Vec<Sample<Foo>>> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut rtps_reader = self.rtps_reader.write_lock();

        let samples = rtps_reader
            .reader_cache()
            .changes()
            .iter()
            .map(|sample| {
                let (mut data_value, sample_info) = read_sample(self, sample);
                let foo = DdsDeserialize::deserialize(&mut data_value)?;
                Ok(Sample {
                    data: Some(foo),
                    sample_info,
                })
            })
            .filter(|result| {
                if let Ok(sample) = result {
                    sample.sample_info.sample_state & sample_states != 0
                } else {
                    true
                }
            })
            .take(max_samples as usize)
            .collect::<DdsResult<Vec<_>>>()?;

        if samples.is_empty() {
            Err(DdsError::NoData)
        } else {
            Ok(samples)
        }
    }

    fn take(
        &self,
        _max_samples: i32,
        sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
    ) -> DdsResult<Vec<Sample<Foo>>> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut rtps_reader = self.rtps_reader.write_lock();

        let (samples, to_delete): (Vec<_>, Vec<_>) = rtps_reader
            .reader_cache()
            .changes()
            .iter()
            .map(|cache_change| match cache_change.kind() {
                ChangeKind::Alive => {
                    let (mut data_value, sample_info) = read_sample(self, cache_change);
                    let foo = DdsDeserialize::deserialize(&mut data_value)?;
                    let sample = Sample {
                        data: Some(foo),
                        sample_info,
                    };
                    Ok((sample, cache_change.sequence_number()))
                }
                ChangeKind::AliveFiltered => todo!(),
                ChangeKind::NotAliveDisposed => {
                    let (_, sample_info) = read_sample(self, cache_change);
                    let sample = Sample {
                        data: None,
                        sample_info,
                    };
                    Ok((sample, cache_change.sequence_number()))
                }
                ChangeKind::NotAliveUnregistered => todo!(),
            })
            .filter(|result| {
                if let Ok((sample, _)) = result {
                    sample.sample_info.sample_state & sample_states != 0
                } else {
                    true
                }
            })
            .collect::<DdsResult<Vec<_>>>()?
            .into_iter()
            .unzip();

        rtps_reader
            .reader_cache()
            .remove_change(|x| to_delete.contains(&x.sequence_number()));

        Ok(samples)
    }

    fn read_w_condition(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_condition: ReadCondition,
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn take_w_condition(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_condition: ReadCondition,
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn read_next_sample(
        &self,
        _data_value: &mut [Foo],
        _sample_info: &mut [SampleInfo],
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn take_next_sample(
        &self,
        _data_value: &mut [Foo],
        _sample_info: &mut [SampleInfo],
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn read_instance(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_handle: InstanceHandle,
        _sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn take_instance(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_handle: InstanceHandle,
        _sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn read_next_instance(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn take_next_instance(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn read_next_instance_w_condition(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _a_condition: ReadCondition,
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn take_next_instance_w_condition(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _a_condition: ReadCondition,
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn return_loan(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn get_key_value(&self, _key_holder: &mut Foo, _handle: InstanceHandle) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn lookup_instance(&self, _instance: &Foo) -> DdsResult<InstanceHandle> {
        todo!()
    }
}

impl<Tim> DataReader for DdsShared<DataReaderImpl<Tim>>
where
    Tim: Timer,
{
    fn create_readcondition(
        &self,
        _sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
    ) -> DdsResult<ReadCondition> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn create_querycondition(
        &self,
        _sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
        _query_expression: &'static str,
        _query_parameters: &[&'static str],
    ) -> DdsResult<QueryCondition> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn delete_readcondition(&self, _a_condition: ReadCondition) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn get_liveliness_changed_status(&self) -> DdsResult<LivelinessChangedStatus> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut liveliness_changed_status_lock = self.liveliness_changed_status.write_lock();
        let liveliness_changed_status = liveliness_changed_status_lock.clone();

        liveliness_changed_status_lock.alive_count_change = 0;
        liveliness_changed_status_lock.not_alive_count_change = 0;

        Ok(liveliness_changed_status)
    }

    fn get_requested_deadline_missed_status(&self) -> DdsResult<RequestedDeadlineMissedStatus> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let status = self.requested_deadline_missed_status.read_lock().clone();

        self.requested_deadline_missed_status
            .write_lock()
            .total_count_change = 0;

        Ok(status)
    }

    fn get_requested_incompatible_qos_status(&self) -> DdsResult<RequestedIncompatibleQosStatus> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut requested_incompatible_qos_status_lock =
            self.requested_incompatible_qos_status.write_lock();
        let requested_incompatible_qos_status = requested_incompatible_qos_status_lock.clone();

        requested_incompatible_qos_status_lock.total_count_change = 0;

        Ok(requested_incompatible_qos_status)
    }

    fn get_sample_lost_status(&self) -> DdsResult<SampleLostStatus> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut sample_lost_status_lock = self.sample_lost_status.write_lock();
        let sample_lost_status = sample_lost_status_lock.clone();

        sample_lost_status_lock.total_count_change = 0;

        Ok(sample_lost_status)
    }

    fn get_sample_rejected_status(&self) -> DdsResult<SampleRejectedStatus> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut sample_rejected_status_lock = self.sample_rejected_status.write_lock();
        let sample_rejected_status = sample_rejected_status_lock.clone();

        sample_rejected_status_lock.total_count_change = 0;

        Ok(sample_rejected_status)
    }

    fn get_subscription_matched_status(&self) -> DdsResult<SubscriptionMatchedStatus> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut subscription_matched_status_lock = self.subscription_matched_status.write_lock();
        let subscription_matched_status = subscription_matched_status_lock.clone();

        subscription_matched_status_lock.current_count_change = 0;
        subscription_matched_status_lock.total_count_change = 0;

        Ok(subscription_matched_status)
    }

    fn delete_contained_entities(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn wait_for_historical_data(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    fn get_matched_publication_data(
        &self,
        publication_handle: InstanceHandle,
    ) -> DdsResult<PublicationBuiltinTopicData> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.matched_publication_list
            .read_lock()
            .get(&publication_handle)
            .cloned()
            .ok_or(DdsError::BadParameter)
    }

    fn get_matched_publications(&self) -> DdsResult<Vec<InstanceHandle>> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        Ok(self
            .matched_publication_list
            .read_lock()
            .iter()
            .map(|(key, _)| key.clone())
            .collect())
    }
}

impl<Tim> Entity for DdsShared<DataReaderImpl<Tim>> {
    type Qos = DataReaderQos;
    type Listener = Box<dyn AnyDataReaderListener<Self> + Send + Sync>;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DdsResult<()> {
        let qos = qos.unwrap_or_default();

        qos.is_consistent()?;
        if *self.enabled.read_lock() {
            self.qos.read_lock().check_immutability(&qos)?;
        }

        *self.qos.write_lock() = qos;

        Ok(())
    }

    fn get_qos(&self) -> DdsResult<Self::Qos> {
        Ok(self.qos.read_lock().clone())
    }

    fn set_listener(&self, a_listener: Option<Self::Listener>, _mask: StatusMask) -> DdsResult<()> {
        *self.listener.write_lock() = a_listener;
        Ok(())
    }

    fn get_listener(&self) -> DdsResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        todo!()
    }

    fn get_status_changes(&self) -> DdsResult<StatusMask> {
        Ok(*self.status_change.read_lock())
    }

    fn enable(&self) -> DdsResult<()> {
        if !self.parent_subscriber.upgrade()?.is_enabled() {
            return Err(DdsError::PreconditionNotMet(
                "Parent subscriber disabled".to_string(),
            ));
        }

        *self.enabled.write_lock() = true;
        Ok(())
    }

    fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        Ok(self.rtps_reader.read_lock().guid().into())
    }
}

impl<Tim> SendRtpsMessage for DdsShared<DataReaderImpl<Tim>> {
    fn send_message(&self, transport: &mut impl TransportWrite) {
        if let RtpsReader::Stateful(stateful_rtps_reader) = &mut *self.rtps_reader.write_lock() {
            let mut acknacks = Vec::new();
            stateful_rtps_reader.send_submessages(|wp, acknack| {
                acknacks.push((
                    wp.unicast_locator_list().to_vec(),
                    vec![RtpsSubmessageType::AckNack(acknack)],
                ))
            });

            for (locator_list, acknacks) in acknacks {
                let header = RtpsMessageHeader {
                    protocol: rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
                    version: PROTOCOLVERSION,
                    vendor_id: VENDOR_ID_S2E,
                    guid_prefix: stateful_rtps_reader.guid().prefix(),
                };

                let message = RtpsMessage {
                    header,
                    submessages: acknacks,
                };

                for &locator in &locator_list {
                    transport.write(&message, locator);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data_representation_builtin_endpoints::discovered_writer_data::RtpsWriterProxy,
        data_representation_inline_qos::parameter_id_values::PID_STATUS_INFO,
        dds_impl::{data_reader_impl::RtpsReader, topic_impl::TopicImpl},
        dds_type::{DdsSerialize, DdsType, Endianness},
        rtps_impl::rtps_group_impl::RtpsGroupImpl,
        utils::shared_object::DdsShared,
    };
    use dds_api::{
        dcps_psm::{
            BuiltInTopicKey, ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE,
            NOT_ALIVE_DISPOSED_INSTANCE_STATE,
        },
        infrastructure::{
            qos::{SubscriberQos, TopicQos},
            qos_policy::{
                DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
                DurabilityServiceQosPolicy, GroupDataQosPolicy, HistoryQosPolicy,
                LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy,
                OwnershipStrengthQosPolicy, PartitionQosPolicy, PresentationQosPolicy,
                ReliabilityQosPolicy, ReliabilityQosPolicyKind, TopicDataQosPolicy,
                UserDataQosPolicy,
            },
        },
        return_type::DdsResult,
    };
    use mockall::mock;
    use rtps_pim::{
        behavior::{
            reader::{
                stateful_reader::RtpsStatefulReaderConstructor,
                stateless_reader::RtpsStatelessReaderConstructor,
            },
            types::DURATION_ZERO,
        },
        messages::{
            submessage_elements::{
                EntityIdSubmessageElement, ParameterListSubmessageElement,
                SequenceNumberSubmessageElement, SerializedDataSubmessageElement,
            },
            types::ParameterId,
        },
        structure::{
            cache_change::RtpsCacheChangeConstructor,
            group::RtpsGroupConstructor,
            history_cache::RtpsHistoryCacheConstructor,
            types::{EntityId, Guid, ENTITYID_UNKNOWN, GUIDPREFIX_UNKNOWN, GUID_UNKNOWN},
        },
    };
    use std::io::Write;

    pub struct ManualTimer {
        on_deadline: Option<Box<dyn FnMut() + Send + Sync>>,
    }

    impl ManualTimer {
        pub fn trigger(&mut self) {
            if let Some(f) = &mut self.on_deadline {
                f()
            }
            self.on_deadline = None;
        }
    }

    impl Timer for ManualTimer {
        fn new(_duration: std::time::Duration) -> Self {
            ManualTimer { on_deadline: None }
        }

        fn reset(&mut self) {
            self.on_deadline = None;
        }

        fn on_deadline<F>(&mut self, f: F)
        where
            F: FnMut() + Send + Sync + 'static,
        {
            self.on_deadline = Some(Box::new(f));
        }
    }

    struct UserData(u8);

    impl DdsType for UserData {
        fn type_name() -> &'static str {
            "UserData"
        }

        fn has_key() -> bool {
            false
        }
    }

    impl<'de> DdsDeserialize<'de> for UserData {
        fn deserialize(buf: &mut &'de [u8]) -> DdsResult<Self> {
            Ok(UserData(buf[0]))
        }
    }

    impl DdsSerialize for UserData {
        fn serialize<W: Write, E: Endianness>(&self, mut writer: W) -> DdsResult<()> {
            writer
                .write(&[self.0])
                .map(|_| ())
                .map_err(|e| DdsError::PreconditionNotMet(format!("{}", e)))
        }
    }

    fn cache_change(value: u8, sn: SequenceNumber) -> RtpsCacheChangeImpl {
        let cache_change = RtpsCacheChangeImpl::new(
            rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16],
            sn,
            vec![value],
            vec![],
        );

        cache_change
    }

    fn reader_with_changes<Tim: Timer>(
        changes: Vec<RtpsCacheChangeImpl>,
    ) -> DdsShared<DataReaderImpl<Tim>> {
        let mut stateful_reader = RtpsStatefulReaderImpl::new(
            GUID_UNKNOWN,
            rtps_pim::structure::types::TopicKind::NoKey,
            rtps_pim::structure::types::ReliabilityKind::BestEffort,
            &[],
            &[],
            DURATION_ZERO,
            DURATION_ZERO,
            false,
        );
        for change in changes {
            stateful_reader.reader_cache().add_change(change);
        }

        let data_reader = DataReaderImpl::new(
            DataReaderQos {
                history: HistoryQosPolicy {
                    kind: HistoryQosPolicyKind::KeepAllHistoryQos,
                    depth: 0,
                },
                ..Default::default()
            },
            RtpsReader::Stateful(stateful_reader),
            TopicImpl::new(
                GUID_UNKNOWN,
                Default::default(),
                "type_name",
                "topic_name",
                DdsWeak::new(),
            ),
            None,
            DdsWeak::new(),
        );
        *data_reader.enabled.write_lock() = true;
        data_reader
    }

    #[test]
    fn read_all_samples() {
        let reader = DdsShared::new(reader_with_changes::<ManualTimer>(vec![
            cache_change(1, 1),
            cache_change(0, 2),
            cache_change(2, 3),
            cache_change(5, 4),
        ]));

        let all_samples: Vec<Sample<UserData>> = reader
            .read(
                i32::MAX,
                ANY_SAMPLE_STATE,
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
            )
            .unwrap();
        assert_eq!(4, all_samples.len());

        assert_eq!(
            vec![1, 0, 2, 5],
            all_samples
                .into_iter()
                .map(|s| s.data.unwrap().0)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn read_only_unread() {
        let reader = reader_with_changes::<ManualTimer>(vec![cache_change(1, 1)]);

        let unread_samples = FooDataReader::<UserData>::read(
            &reader,
            i32::MAX,
            NOT_READ_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        )
        .unwrap();

        assert_eq!(1, unread_samples.len());

        assert!(FooDataReader::<UserData>::read(
            &reader,
            i32::MAX,
            NOT_READ_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        )
        .is_err());
    }

    #[test]
    fn on_missed_deadline_increases_total_count() {
        let reader = {
            let reader = reader_with_changes::<ManualTimer>(vec![]);
            *reader.qos.write_lock() = DataReaderQos {
                deadline: DeadlineQosPolicy {
                    period: dds_api::dcps_psm::Duration::new(1, 0),
                },
                ..Default::default()
            };
            reader
        };

        assert_eq!(
            0,
            reader
                .get_requested_deadline_missed_status()
                .unwrap()
                .total_count
        );

        DataReaderImpl::on_data_received(reader.clone()).unwrap();

        assert_eq!(
            0,
            reader
                .get_requested_deadline_missed_status()
                .unwrap()
                .total_count
        );

        reader.deadline_timer.write_lock().trigger();

        assert_eq!(
            1,
            reader
                .get_requested_deadline_missed_status()
                .unwrap()
                .total_count
        );
    }

    mock! {
        Listener {}
        impl AnyDataReaderListener<DdsShared<DataReaderImpl<ManualTimer>>> for Listener {
            fn trigger_on_data_available(&mut self, reader: DdsShared<DataReaderImpl<ManualTimer>>);
            fn trigger_on_sample_rejected(
                &mut self,
                reader: DdsShared<DataReaderImpl<ManualTimer>>,
                status: SampleRejectedStatus,
            );
            fn trigger_on_liveliness_changed(
                &mut self,
                reader: DdsShared<DataReaderImpl<ManualTimer>>,
                status: LivelinessChangedStatus,
            );
            fn trigger_on_requested_deadline_missed(
                &mut self,
                reader: DdsShared<DataReaderImpl<ManualTimer>>,
                status: RequestedDeadlineMissedStatus,
            );
            fn trigger_on_requested_incompatible_qos(
                &mut self,
                reader: DdsShared<DataReaderImpl<ManualTimer>>,
                status: RequestedIncompatibleQosStatus,
            );
            fn trigger_on_subscription_matched(
                &mut self,
                reader: DdsShared<DataReaderImpl<ManualTimer>>,
                status: SubscriptionMatchedStatus,
            );
            fn trigger_on_sample_lost(
                &mut self,
                reader: DdsShared<DataReaderImpl<ManualTimer>>,
                status: SampleLostStatus,
            );
        }
    }

    #[test]
    fn on_deadline_missed_calls_listener() {
        let reader = {
            let reader = reader_with_changes::<ManualTimer>(vec![]);
            *reader.qos.write_lock() = DataReaderQos {
                deadline: DeadlineQosPolicy {
                    period: dds_api::dcps_psm::Duration::new(1, 0),
                },
                ..Default::default()
            };
            reader
        };

        DataReaderImpl::on_data_received(reader.clone()).unwrap();

        let mut listener = MockListener::new();
        listener
            .expect_trigger_on_requested_deadline_missed()
            .once()
            .return_const(());
        reader.set_listener(Some(Box::new(listener)), 0).unwrap();

        reader.deadline_timer.write_lock().trigger();
    }

    #[test]
    fn receiving_data_triggers_status_change() {
        let reader = {
            let reader = reader_with_changes::<ManualTimer>(vec![]);
            *reader.qos.write_lock() = DataReaderQos {
                deadline: DeadlineQosPolicy {
                    period: dds_api::dcps_psm::Duration::new(1, 0),
                },
                ..Default::default()
            };
            reader
        };

        DataReaderImpl::on_data_received(reader.clone()).unwrap();

        assert!(reader.get_status_changes().unwrap() & DATA_AVAILABLE_STATUS > 0);
    }

    #[test]
    fn on_data_available_listener_resets_status_change() {
        let reader = {
            let reader = reader_with_changes::<ManualTimer>(vec![]);
            *reader.qos.write_lock() = DataReaderQos {
                deadline: DeadlineQosPolicy {
                    period: dds_api::dcps_psm::Duration::new(1, 0),
                },
                ..Default::default()
            };
            reader
        };

        let listener = {
            let mut listener = MockListener::new();
            listener
                .expect_trigger_on_data_available()
                .once()
                .return_const(());
            listener
        };
        reader.set_listener(Some(Box::new(listener)), 0).unwrap();

        DataReaderImpl::on_data_received(reader.clone()).unwrap();

        assert_eq!(
            0,
            reader.get_status_changes().unwrap() & DATA_AVAILABLE_STATUS
        );
    }

    #[test]
    fn deadline_missed_triggers_status_change() {
        let reader = {
            let reader = reader_with_changes::<ManualTimer>(vec![]);
            *reader.qos.write_lock() = DataReaderQos {
                deadline: DeadlineQosPolicy {
                    period: dds_api::dcps_psm::Duration::new(1, 0),
                },
                ..Default::default()
            };
            reader
        };

        DataReaderImpl::on_data_received(reader.clone()).unwrap();
        reader.deadline_timer.write_lock().trigger();

        assert!(reader.get_status_changes().unwrap() & REQUESTED_DEADLINE_MISSED_STATUS > 0);
    }

    #[test]
    fn on_deadline_missed_listener_resets_status_changed() {
        let reader = {
            let reader = reader_with_changes::<ManualTimer>(vec![]);
            *reader.qos.write_lock() = DataReaderQos {
                deadline: DeadlineQosPolicy {
                    period: dds_api::dcps_psm::Duration::new(1, 0),
                },
                ..Default::default()
            };
            reader
        };

        let listener = {
            let mut listener = Box::new(MockListener::new());
            listener
                .expect_trigger_on_requested_deadline_missed()
                .once()
                .return_const(());
            listener
                .expect_trigger_on_data_available()
                .once()
                .return_const(());
            listener
        };

        reader.set_listener(Some(listener), 0).unwrap();

        DataReaderImpl::on_data_received(reader.clone()).unwrap();
        reader.deadline_timer.write_lock().trigger();

        assert_eq!(
            0,
            reader.get_status_changes().unwrap() & REQUESTED_DEADLINE_MISSED_STATUS
        );
    }

    fn reader_with_max_depth<Tim: Timer>(
        max_depth: i32,
        changes: Vec<RtpsCacheChangeImpl>,
    ) -> DdsShared<DataReaderImpl<Tim>> {
        let mut history_cache = RtpsHistoryCacheImpl::new();
        for change in changes {
            history_cache.add_change(change);
        }

        let stateful_reader = RtpsStatefulReaderImpl::new(
            GUID_UNKNOWN,
            rtps_pim::structure::types::TopicKind::NoKey,
            rtps_pim::structure::types::ReliabilityKind::BestEffort,
            &[],
            &[],
            DURATION_ZERO,
            DURATION_ZERO,
            false,
        );

        DataReaderImpl::new(
            DataReaderQos {
                history: HistoryQosPolicy {
                    kind: HistoryQosPolicyKind::KeepLastHistoryQoS,
                    depth: max_depth,
                },
                ..Default::default()
            },
            RtpsReader::Stateful(stateful_reader),
            TopicImpl::new(
                GUID_UNKNOWN,
                Default::default(),
                "type_name",
                "topic_name",
                DdsWeak::new(),
            ),
            None,
            DdsWeak::new(),
        )
    }

    #[test]
    fn keep_last_qos() {
        let reader = {
            let reader = reader_with_max_depth::<ManualTimer>(
                2,
                vec![
                    cache_change(1, 1),
                    cache_change(2, 2),
                    cache_change(3, 3),
                    cache_change(4, 4),
                ],
            );

            reader
        };

        DataReaderImpl::on_data_received(reader.clone()).unwrap();
    }

    #[test]
    fn get_instance_handle() {
        let guid = Guid::new(
            GuidPrefix([4; 12]),
            EntityId {
                entity_key: [3; 3],
                entity_kind: 1,
            },
        );
        let dummy_topic = TopicImpl::new(GUID_UNKNOWN, TopicQos::default(), "", "", DdsWeak::new());

        let stateful_reader = RtpsStatefulReaderImpl::new(
            guid,
            rtps_pim::structure::types::TopicKind::NoKey,
            rtps_pim::structure::types::ReliabilityKind::BestEffort,
            &[],
            &[],
            DURATION_ZERO,
            DURATION_ZERO,
            false,
        );

        let data_reader: DdsShared<DataReaderImpl<ManualTimer>> = DataReaderImpl::new(
            DataReaderQos::default(),
            RtpsReader::Stateful(stateful_reader),
            dummy_topic,
            None,
            DdsWeak::new(),
        );
        *data_reader.enabled.write_lock() = true;

        let expected_instance_handle: [u8; 16] = guid.into();
        let instance_handle = data_reader.get_instance_handle().unwrap();
        assert_eq!(expected_instance_handle, instance_handle);
    }

    #[test]
    fn receive_disposed_data_submessage() {
        let dummy_topic = TopicImpl::new(GUID_UNKNOWN, TopicQos::default(), "", "", DdsWeak::new());

        let stateless_reader = RtpsStatelessReaderImpl::new(
            GUID_UNKNOWN,
            rtps_pim::structure::types::TopicKind::NoKey,
            rtps_pim::structure::types::ReliabilityKind::BestEffort,
            &[],
            &[],
            DURATION_ZERO,
            DURATION_ZERO,
            false,
        );

        let data_reader: DdsShared<DataReaderImpl<ManualTimer>> = DataReaderImpl::new(
            DataReaderQos::default(),
            RtpsReader::Stateless(stateless_reader),
            dummy_topic,
            None,
            DdsWeak::new(),
        );

        let data_submessage = DataSubmessage {
            endianness_flag: true,
            inline_qos_flag: true,
            data_flag: false,
            key_flag: true,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_sn: SequenceNumberSubmessageElement { value: 1 },
            inline_qos: ParameterListSubmessageElement {
                parameter: vec![Parameter {
                    parameter_id: ParameterId(PID_STATUS_INFO),
                    length: 4,
                    value: &[1, 0, 0, 0],
                }],
            },
            serialized_payload: SerializedDataSubmessageElement { value: &[1][..] },
        };
        *data_reader.enabled.write_lock() = true;

        data_reader.on_data_submessage_received(&data_submessage, GUIDPREFIX_UNKNOWN);
        let data: Vec<Sample<UserData>> = data_reader
            .take(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .unwrap();

        assert_eq!(data.len(), 1);
        assert!(data[0].data.is_none());
        assert_eq!(&data[0].sample_info.valid_data, &false);
        assert_eq!(
            &data[0].sample_info.instance_state,
            &NOT_ALIVE_DISPOSED_INSTANCE_STATE
        );
    }

    #[test]
    fn add_compatible_matched_writer() {
        let type_name = "test_type";
        let topic_name = "test_topic".to_string();
        let parent_subscriber = SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(GUID_UNKNOWN),
            DdsWeak::new(),
        );
        let test_topic = TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            type_name,
            &topic_name,
            DdsWeak::new(),
        );

        let rtps_reader = RtpsStatefulReaderImpl::new(
            GUID_UNKNOWN,
            rtps_pim::structure::types::TopicKind::WithKey,
            rtps_pim::structure::types::ReliabilityKind::BestEffort,
            &[],
            &[],
            DURATION_ZERO,
            DURATION_ZERO,
            false,
        );

        let data_reader = DataReaderImpl::<ManualTimer>::new(
            DataReaderQos::default(),
            RtpsReader::Stateful(rtps_reader),
            test_topic,
            None,
            parent_subscriber.downgrade(),
        );
        *data_reader.enabled.write_lock() = true;
        let publication_builtin_topic_data = PublicationBuiltinTopicData {
            key: BuiltInTopicKey { value: [2; 16] },
            participant_key: BuiltInTopicKey { value: [1; 16] },
            topic_name: topic_name.clone(),
            type_name: type_name.to_string(),
            durability: DurabilityQosPolicy::default(),
            deadline: DeadlineQosPolicy::default(),
            latency_budget: LatencyBudgetQosPolicy::default(),
            liveliness: LivelinessQosPolicy::default(),
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
                max_blocking_time: dds_api::dcps_psm::Duration::new(0, 0),
            },
            ownership: OwnershipQosPolicy::default(),
            destination_order: DestinationOrderQosPolicy::default(),
            user_data: UserDataQosPolicy::default(),
            presentation: PresentationQosPolicy::default(),
            partition: PartitionQosPolicy::default(),
            topic_data: TopicDataQosPolicy::default(),
            group_data: GroupDataQosPolicy::default(),
            durability_service: DurabilityServiceQosPolicy::default(),
            lifespan: LifespanQosPolicy::default(),
            ownership_strength: OwnershipStrengthQosPolicy::default(),
        };
        let discovered_writer_data = DiscoveredWriterData {
            writer_proxy: RtpsWriterProxy {
                remote_writer_guid: Guid {
                    prefix: GuidPrefix([2; 12]),
                    entity_id: EntityId {
                        entity_key: [2; 3],
                        entity_kind: 2,
                    },
                },
                remote_group_entity_id: ENTITYID_UNKNOWN,
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                data_max_size_serialized: None,
            },
            publication_builtin_topic_data: publication_builtin_topic_data.clone(),
        };
        data_reader.add_matched_writer(&discovered_writer_data);

        let subscription_matched_status = data_reader.get_subscription_matched_status().unwrap();
        assert_eq!(subscription_matched_status.current_count, 1);
        assert_eq!(subscription_matched_status.current_count_change, 1);
        assert_eq!(subscription_matched_status.total_count, 1);
        assert_eq!(subscription_matched_status.total_count_change, 1);

        let matched_publications = data_reader.get_matched_publications().unwrap();
        assert_eq!(matched_publications.len(), 1);
        assert_eq!(matched_publications[0], [2; 16]);
        let matched_publication_data = data_reader
            .get_matched_publication_data(matched_publications[0])
            .unwrap();
        assert_eq!(matched_publication_data, publication_builtin_topic_data);
    }

    #[test]
    fn add_incompatible_matched_writer() {
        let type_name = "test_type";
        let topic_name = "test_topic".to_string();
        let parent_subscriber = SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(GUID_UNKNOWN),
            DdsWeak::new(),
        );
        let test_topic = TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            type_name,
            &topic_name,
            DdsWeak::new(),
        );

        let rtps_reader = RtpsStatefulReaderImpl::new(
            GUID_UNKNOWN,
            rtps_pim::structure::types::TopicKind::WithKey,
            rtps_pim::structure::types::ReliabilityKind::BestEffort,
            &[],
            &[],
            DURATION_ZERO,
            DURATION_ZERO,
            false,
        );
        let mut data_reader_qos = DataReaderQos::default();
        data_reader_qos.reliability.kind = ReliabilityQosPolicyKind::ReliableReliabilityQos;
        let data_reader = DataReaderImpl::<ManualTimer>::new(
            data_reader_qos,
            RtpsReader::Stateful(rtps_reader),
            test_topic,
            None,
            parent_subscriber.downgrade(),
        );
        *data_reader.enabled.write_lock() = true;
        let publication_builtin_topic_data = PublicationBuiltinTopicData {
            key: BuiltInTopicKey { value: [2; 16] },
            participant_key: BuiltInTopicKey { value: [1; 16] },
            topic_name: topic_name.clone(),
            type_name: type_name.to_string(),
            durability: DurabilityQosPolicy::default(),
            deadline: DeadlineQosPolicy::default(),
            latency_budget: LatencyBudgetQosPolicy::default(),
            liveliness: LivelinessQosPolicy::default(),
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos,
                max_blocking_time: dds_api::dcps_psm::Duration::new(0, 0),
            },
            ownership: OwnershipQosPolicy::default(),
            destination_order: DestinationOrderQosPolicy::default(),
            user_data: UserDataQosPolicy::default(),
            presentation: PresentationQosPolicy::default(),
            partition: PartitionQosPolicy::default(),
            topic_data: TopicDataQosPolicy::default(),
            group_data: GroupDataQosPolicy::default(),
            durability_service: DurabilityServiceQosPolicy::default(),
            lifespan: LifespanQosPolicy::default(),
            ownership_strength: OwnershipStrengthQosPolicy::default(),
        };
        let discovered_writer_data = DiscoveredWriterData {
            writer_proxy: RtpsWriterProxy {
                remote_writer_guid: Guid {
                    prefix: GuidPrefix([2; 12]),
                    entity_id: EntityId {
                        entity_key: [2; 3],
                        entity_kind: 2,
                    },
                },
                remote_group_entity_id: ENTITYID_UNKNOWN,
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                data_max_size_serialized: None,
            },
            publication_builtin_topic_data: publication_builtin_topic_data.clone(),
        };
        data_reader.add_matched_writer(&discovered_writer_data);

        let matched_publications = data_reader.get_matched_publications().unwrap();
        assert_eq!(matched_publications.len(), 0);

        let requested_incompatible_qos_status =
            data_reader.get_requested_incompatible_qos_status().unwrap();
        assert_eq!(requested_incompatible_qos_status.total_count, 1);
        assert_eq!(requested_incompatible_qos_status.total_count_change, 1);
        assert_eq!(
            requested_incompatible_qos_status.last_policy_id,
            RELIABILITY_QOS_POLICY_ID
        );
        assert_eq!(
            requested_incompatible_qos_status.policies,
            vec![QosPolicyCount {
                policy_id: RELIABILITY_QOS_POLICY_ID,
                count: 1,
            }]
        )
    }
}
