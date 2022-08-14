use std::{
    collections::{HashMap, HashSet},
    convert::{TryFrom, TryInto},
};

use crate::{
    dds_type::DdsDeserialize,
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, ReaderProxy},
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::DiscoveredWriterData,
        },
        rtps::{
            messages::{
                overall_structure::RtpsMessageHeader,
                submessage_elements::{
                    GuidPrefixSubmessageElement, ProtocolVersionSubmessageElement,
                    VendorIdSubmessageElement,
                },
                submessages::{DataSubmessage, HeartbeatSubmessage},
                types::ProtocolId,
                RtpsMessage, RtpsSubmessageType,
            },
            reader::RtpsReader,
            reader_cache_change::RtpsReaderCacheChange,
            stateful_reader::RtpsStatefulReader,
            stateless_reader::RtpsStatelessReader,
            transport::TransportWrite,
            types::{
                ChangeKind, EntityId, Guid, GuidPrefix, SequenceNumber, ENTITYID_UNKNOWN,
                PROTOCOLVERSION, VENDOR_ID_S2E,
            },
            writer_proxy::RtpsWriterProxy,
        },
        utils::{
            discovery_traits::AddMatchedWriter,
            shared_object::{DdsRwLock, DdsShared, DdsWeak},
            timer::Timer,
        },
    },
    infrastructure::qos_policy::{DestinationOrderQosPolicyKind, ReliabilityQosPolicyKind},
};
use crate::{
    dds_type::DdsType,
    implementation::utils::timer::ThreadTimer,
    return_type::{DdsError, DdsResult},
    subscription::{
        data_reader::{DataReader, Sample},
        data_reader_listener::DataReaderListener,
        query_condition::QueryCondition,
    },
    {
        builtin_topics::{PublicationBuiltinTopicData, SubscriptionBuiltinTopicData},
        dcps_psm::{
            BuiltInTopicKey, InstanceHandle, InstanceStateMask, LivelinessChangedStatus,
            QosPolicyCount, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
            SampleLostStatus, SampleRejectedStatus, SampleRejectedStatusKind, SampleStateMask,
            StatusMask, SubscriptionMatchedStatus, ViewStateMask, ALIVE_INSTANCE_STATE,
            DATA_AVAILABLE_STATUS, HANDLE_NIL, NOT_ALIVE_DISPOSED_INSTANCE_STATE,
            NOT_READ_SAMPLE_STATE, READ_SAMPLE_STATE, REQUESTED_DEADLINE_MISSED_STATUS,
            SUBSCRIPTION_MATCHED_STATUS,
        },
        infrastructure::{
            entity::{Entity, StatusCondition},
            qos::DataReaderQos,
            qos_policy::{
                DEADLINE_QOS_POLICY_ID, DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID,
                LATENCYBUDGET_QOS_POLICY_ID, LIVELINESS_QOS_POLICY_ID,
                OWNERSHIPSTRENGTH_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID,
                RELIABILITY_QOS_POLICY_ID,
            },
            read_condition::ReadCondition,
            sample_info::SampleInfo,
        },
    },
};

use super::{
    domain_participant_impl::DomainParticipantImpl, message_receiver::MessageReceiver,
    participant_discovery::ParticipantDiscovery, subscriber_impl::SubscriberImpl,
    topic_impl::TopicImpl,
};

pub trait AnyDataReaderListener {
    fn trigger_on_data_available(&mut self, reader: &DdsShared<DataReaderImpl<ThreadTimer>>);
    fn trigger_on_sample_rejected(
        &mut self,
        reader: &DdsShared<DataReaderImpl<ThreadTimer>>,
        status: SampleRejectedStatus,
    );
    fn trigger_on_liveliness_changed(
        &mut self,
        reader: &DdsShared<DataReaderImpl<ThreadTimer>>,
        status: LivelinessChangedStatus,
    );
    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader: &DdsShared<DataReaderImpl<ThreadTimer>>,
        status: RequestedDeadlineMissedStatus,
    );
    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader: &DdsShared<DataReaderImpl<ThreadTimer>>,
        status: RequestedIncompatibleQosStatus,
    );
    fn trigger_on_subscription_matched(
        &mut self,
        reader: &DdsShared<DataReaderImpl<ThreadTimer>>,
        status: SubscriptionMatchedStatus,
    );
    fn trigger_on_sample_lost(
        &mut self,
        reader: &DdsShared<DataReaderImpl<ThreadTimer>>,
        status: SampleLostStatus,
    );
}

impl<Foo> AnyDataReaderListener for Box<dyn DataReaderListener<Foo = Foo> + Send + Sync> {
    fn trigger_on_data_available(&mut self, reader: &DdsShared<DataReaderImpl<ThreadTimer>>) {
        self.on_data_available(&DataReader::new(reader.downgrade()))
    }

    fn trigger_on_sample_rejected(
        &mut self,
        reader: &DdsShared<DataReaderImpl<ThreadTimer>>,
        status: SampleRejectedStatus,
    ) {
        self.on_sample_rejected(&DataReader::new(reader.downgrade()), status)
    }

    fn trigger_on_liveliness_changed(
        &mut self,
        reader: &DdsShared<DataReaderImpl<ThreadTimer>>,
        status: LivelinessChangedStatus,
    ) {
        self.on_liveliness_changed(&DataReader::new(reader.downgrade()), status)
    }

    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader: &DdsShared<DataReaderImpl<ThreadTimer>>,
        status: RequestedDeadlineMissedStatus,
    ) {
        self.on_requested_deadline_missed(&DataReader::new(reader.downgrade()), status)
    }

    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader: &DdsShared<DataReaderImpl<ThreadTimer>>,
        status: RequestedIncompatibleQosStatus,
    ) {
        self.on_requested_incompatible_qos(&DataReader::new(reader.downgrade()), status)
    }

    fn trigger_on_subscription_matched(
        &mut self,
        reader: &DdsShared<DataReaderImpl<ThreadTimer>>,
        status: SubscriptionMatchedStatus,
    ) {
        self.on_subscription_matched(&DataReader::new(reader.downgrade()), status)
    }

    fn trigger_on_sample_lost(
        &mut self,
        reader: &DdsShared<DataReaderImpl<ThreadTimer>>,
        status: SampleLostStatus,
    ) {
        self.on_sample_lost(&DataReader::new(reader.downgrade()), status)
    }
}

pub enum RtpsReaderKind {
    Stateless(RtpsStatelessReader),
    Stateful(RtpsStatefulReader),
}

impl RtpsReaderKind {
    pub fn reader(&self) -> &RtpsReader {
        match self {
            RtpsReaderKind::Stateless(r) => r.reader(),
            RtpsReaderKind::Stateful(r) => r.reader(),
        }
    }

    pub fn reader_mut(&mut self) -> &mut RtpsReader {
        match self {
            RtpsReaderKind::Stateless(r) => r.reader_mut(),
            RtpsReaderKind::Stateful(r) => r.reader_mut(),
        }
    }
}

pub struct DataReaderImpl<Tim> {
    rtps_reader: DdsRwLock<RtpsReaderKind>,
    topic: DdsShared<TopicImpl>,
    listener: DdsRwLock<Option<Box<dyn AnyDataReaderListener + Send + Sync>>>,
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
        rtps_reader: RtpsReaderKind,
        topic: DdsShared<TopicImpl>,
        listener: Option<Box<dyn AnyDataReaderListener + Send + Sync>>,
        parent_subscriber: DdsWeak<SubscriberImpl>,
    ) -> DdsShared<Self> {
        let qos = rtps_reader.reader().get_qos();
        let deadline_duration = std::time::Duration::from_secs(qos.deadline.period.sec() as u64)
            + std::time::Duration::from_nanos(qos.deadline.period.nanosec() as u64);

        DdsShared::new(DataReaderImpl {
            rtps_reader: DdsRwLock::new(rtps_reader),
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
                last_instance_handle: HANDLE_NIL,
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
                last_instance_handle: HANDLE_NIL,
            }),
            subscription_matched_status: DdsRwLock::new(SubscriptionMatchedStatus {
                total_count: 0,
                total_count_change: 0,
                last_publication_handle: HANDLE_NIL,
                current_count: 0,
                current_count_change: 0,
            }),
            matched_publication_list: DdsRwLock::new(HashMap::new()),
            enabled: DdsRwLock::new(false),
        })
    }
}

impl<Tim> DataReaderImpl<Tim> {
    pub fn add_matched_participant(&self, participant_discovery: &ParticipantDiscovery) {
        let mut rtps_reader_lock = self.rtps_reader.write_lock();
        if let RtpsReaderKind::Stateful(rtps_reader) = &mut *rtps_reader_lock {
            if !rtps_reader
                .matched_writers()
                .iter_mut()
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

impl DdsShared<DataReaderImpl<ThreadTimer>> {
    pub fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) {
        let sequence_number = data_submessage.writer_sn.value;
        let writer_guid = Guid::new(
            message_receiver.source_guid_prefix(),
            data_submessage.writer_id.value.into(),
        );

        let before_data_cache_len;
        let after_data_cache_len;
        let mut rtps_reader = self.rtps_reader.write_lock();
        match &mut *rtps_reader {
            RtpsReaderKind::Stateless(stateless_rtps_reader) => {
                before_data_cache_len = stateless_rtps_reader.reader_mut().changes().len();

                let data_reader_id: EntityId = data_submessage.reader_id.value.into();
                if data_reader_id == ENTITYID_UNKNOWN
                    || data_reader_id == stateless_rtps_reader.reader().guid().entity_id()
                {
                    stateless_rtps_reader
                        .reader_mut()
                        .on_data_submessage_received(data_submessage, message_receiver);
                }

                after_data_cache_len = stateless_rtps_reader.reader_mut().changes().len();
            }
            RtpsReaderKind::Stateful(stateful_rtps_reader) => {
                before_data_cache_len = stateful_rtps_reader.reader_mut().changes().len();

                if let Some(writer_proxy) = stateful_rtps_reader.matched_writer_lookup(writer_guid)
                {
                    if data_submessage.writer_sn.value < writer_proxy.first_available_seq_num
                        || data_submessage.writer_sn.value > writer_proxy.last_available_seq_num
                        || writer_proxy
                            .missing_changes()
                            .contains(&data_submessage.writer_sn.value)
                    {
                        let reliability_kind = stateful_rtps_reader
                            .reader()
                            .get_qos()
                            .reliability
                            .kind
                            .clone();
                        if let Some(writer_proxy) =
                            stateful_rtps_reader.matched_writer_lookup(writer_guid)
                        {
                            match reliability_kind {
                                ReliabilityQosPolicyKind::BestEffortReliabilityQos => {
                                    let expected_seq_num = writer_proxy.available_changes_max() + 1;
                                    if sequence_number >= expected_seq_num {
                                        writer_proxy.received_change_set(sequence_number);
                                        if sequence_number > expected_seq_num {
                                            writer_proxy.lost_changes_update(sequence_number);
                                        }

                                        stateful_rtps_reader
                                            .reader_mut()
                                            .on_data_submessage_received(
                                                data_submessage,
                                                message_receiver,
                                            );
                                    }
                                }
                                ReliabilityQosPolicyKind::ReliableReliabilityQos => {
                                    writer_proxy
                                        .received_change_set(data_submessage.writer_sn.value);
                                    stateful_rtps_reader
                                        .reader_mut()
                                        .on_data_submessage_received(
                                            data_submessage,
                                            message_receiver,
                                        );
                                }
                            }
                        }
                    }
                }

                after_data_cache_len = stateful_rtps_reader.reader_mut().changes().len();
            }
        }
        // Call the listener after dropping the rtps_reader lock to avoid deadlock
        drop(rtps_reader);
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

                *reader_shared.status_change.write_lock() |= REQUESTED_DEADLINE_MISSED_STATUS;
                if let Some(l) = reader_shared.listener.write_lock().as_mut() {
                    *reader_shared.status_change.write_lock() &= !REQUESTED_DEADLINE_MISSED_STATUS;
                    l.trigger_on_requested_deadline_missed(
                        &reader_shared,
                        reader_shared
                            .requested_deadline_missed_status
                            .read_lock()
                            .clone(),
                    )
                };
            });

            *self.status_change.write_lock() |= DATA_AVAILABLE_STATUS;
            if let Some(l) = self.listener.write_lock().as_mut() {
                *self.status_change.write_lock() &= !DATA_AVAILABLE_STATUS;
                l.trigger_on_data_available(self)
            };
        }
    }
}

impl<Tim> DdsShared<DataReaderImpl<Tim>> {
    pub fn on_heartbeat_submessage_received(
        &self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        let mut rtps_reader = self.rtps_reader.write_lock();
        if let RtpsReaderKind::Stateful(stateful_rtps_reader) = &mut *rtps_reader {
            stateful_rtps_reader
                .on_heartbeat_submessage_received(heartbeat_submessage, source_guid_prefix);
        }
    }
}

impl AddMatchedWriter for DdsShared<DataReaderImpl<ThreadTimer>> {
    fn add_matched_writer(&self, discovered_writer_data: &DiscoveredWriterData) {
        let writer_info = &discovered_writer_data.publication_builtin_topic_data;
        let reader_topic_name = self.topic.get_name().unwrap();
        let reader_type_name = self.topic.get_type_name().unwrap();

        if writer_info.topic_name == reader_topic_name && writer_info.type_name == reader_type_name
        {
            let mut rtps_reader_lock = self.rtps_reader.write_lock();
            let reader_qos = rtps_reader_lock.reader().get_qos();
            let parent_subscriber_qos = self.get_subscriber().unwrap().get_qos().unwrap();

            let mut incompatible_qos_policy_list = Vec::new();

            if reader_qos.durability < writer_info.durability {
                incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
            }
            if parent_subscriber_qos.presentation.access_scope
                > writer_info.presentation.access_scope
                || parent_subscriber_qos.presentation.coherent_access
                    != writer_info.presentation.coherent_access
                || parent_subscriber_qos.presentation.ordered_access
                    != writer_info.presentation.ordered_access
            {
                incompatible_qos_policy_list.push(PRESENTATION_QOS_POLICY_ID);
            }
            if reader_qos.deadline > writer_info.deadline {
                incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
            }
            if reader_qos.latency_budget > writer_info.latency_budget {
                incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
            }
            if reader_qos.ownership != writer_info.ownership {
                incompatible_qos_policy_list.push(OWNERSHIPSTRENGTH_QOS_POLICY_ID);
            }
            if reader_qos.liveliness > writer_info.liveliness {
                incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
            }
            if reader_qos.reliability.kind > writer_info.reliability.kind {
                incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
            }
            if reader_qos.destination_order > writer_info.destination_order {
                incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
            }

            if incompatible_qos_policy_list.is_empty() {
                match &mut *rtps_reader_lock {
                    RtpsReaderKind::Stateless(_) => (),
                    RtpsReaderKind::Stateful(r) => {
                        let writer_proxy = RtpsWriterProxy::new(
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
                    l.trigger_on_subscription_matched(self, subscription_matched_status)
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
                    l.trigger_on_requested_incompatible_qos(self, requested_incompatible_qos_status)
                }
            }
        }
    }
}

impl<Tim> DdsShared<DataReaderImpl<Tim>>
where
    Tim: Timer,
{
    pub fn read<Foo>(
        &self,
        max_samples: i32,
        sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut rtps_reader = self.rtps_reader.write_lock();

        let mut samples = rtps_reader
            .reader_mut()
            .changes()
            .iter()
            .map(|sample| {
                let (mut data_value, sample_info) = self.read_sample(sample);
                let value = DdsDeserialize::deserialize(&mut data_value)?;
                Ok(Sample {
                    data: Some(value),
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

        if rtps_reader.reader().get_qos().destination_order.kind
            == DestinationOrderQosPolicyKind::BySourceTimestampDestinationOrderQoS
        {
            samples.sort_by(|a, b| {
                a.sample_info
                    .source_timestamp
                    .as_ref()
                    .unwrap()
                    .cmp(b.sample_info.source_timestamp.as_ref().unwrap())
            });
        }

        if samples.is_empty() {
            Err(DdsError::NoData)
        } else {
            Ok(samples)
        }
    }

    pub fn take<Foo>(
        &self,
        _max_samples: i32,
        sample_states: SampleStateMask,
        _view_states: ViewStateMask,
        _instance_states: InstanceStateMask,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut rtps_reader = self.rtps_reader.write_lock();

        let (mut samples, to_delete): (Vec<_>, Vec<_>) = rtps_reader
            .reader_mut()
            .changes()
            .iter()
            .map(|cache_change| match cache_change.kind() {
                ChangeKind::Alive => {
                    let (mut data_value, sample_info) = self.read_sample(cache_change);
                    let value = DdsDeserialize::deserialize(&mut data_value)?;
                    let sample = Sample {
                        data: Some(value),
                        sample_info,
                    };
                    Ok((sample, cache_change.sequence_number()))
                }
                ChangeKind::AliveFiltered => todo!(),
                ChangeKind::NotAliveDisposed => {
                    let (_, sample_info) = self.read_sample(cache_change);
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
            .reader_mut()
            .remove_change(|x| to_delete.contains(&x.sequence_number()));

        if rtps_reader.reader().get_qos().destination_order.kind
            == DestinationOrderQosPolicyKind::BySourceTimestampDestinationOrderQoS
        {
            samples.sort_by(|a, b| {
                a.sample_info
                    .source_timestamp
                    .as_ref()
                    .unwrap()
                    .cmp(b.sample_info.source_timestamp.as_ref().unwrap())
            });
        }

        Ok(samples)
    }

    pub fn read_w_condition<Foo>(
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

    pub fn take_w_condition<Foo>(
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

    pub fn read_next_sample<Foo>(
        &self,
        _data_value: &mut [Foo],
        _sample_info: &mut [SampleInfo],
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn take_next_sample<Foo>(
        &self,
        _data_value: &mut [Foo],
        _sample_info: &mut [SampleInfo],
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn read_instance<Foo>(
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

    #[allow(clippy::too_many_arguments)]
    pub fn take_instance<Foo>(
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

    #[allow(clippy::too_many_arguments)]
    pub fn read_next_instance<Foo>(
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

    #[allow(clippy::too_many_arguments)]
    pub fn take_next_instance<Foo>(
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

    pub fn read_next_instance_w_condition<Foo>(
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

    pub fn take_next_instance_w_condition<Foo>(
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

    pub fn return_loan<Foo>(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn get_key_value<Foo>(
        &self,
        _key_holder: &mut Foo,
        _handle: InstanceHandle,
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn lookup_instance<Foo>(&self, _instance: &Foo) -> DdsResult<InstanceHandle> {
        todo!()
    }

    pub fn create_readcondition(
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

    pub fn create_querycondition(
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

    pub fn delete_readcondition(&self, _a_condition: ReadCondition) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn get_liveliness_changed_status(&self) -> DdsResult<LivelinessChangedStatus> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut liveliness_changed_status_lock = self.liveliness_changed_status.write_lock();
        let liveliness_changed_status = liveliness_changed_status_lock.clone();

        liveliness_changed_status_lock.alive_count_change = 0;
        liveliness_changed_status_lock.not_alive_count_change = 0;

        Ok(liveliness_changed_status)
    }

    pub fn get_requested_deadline_missed_status(&self) -> DdsResult<RequestedDeadlineMissedStatus> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let status = self.requested_deadline_missed_status.read_lock().clone();

        self.requested_deadline_missed_status
            .write_lock()
            .total_count_change = 0;

        Ok(status)
    }

    pub fn get_requested_incompatible_qos_status(
        &self,
    ) -> DdsResult<RequestedIncompatibleQosStatus> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut requested_incompatible_qos_status_lock =
            self.requested_incompatible_qos_status.write_lock();
        let requested_incompatible_qos_status = requested_incompatible_qos_status_lock.clone();

        requested_incompatible_qos_status_lock.total_count_change = 0;

        Ok(requested_incompatible_qos_status)
    }

    pub fn get_sample_lost_status(&self) -> DdsResult<SampleLostStatus> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut sample_lost_status_lock = self.sample_lost_status.write_lock();
        let sample_lost_status = sample_lost_status_lock.clone();

        sample_lost_status_lock.total_count_change = 0;

        Ok(sample_lost_status)
    }

    pub fn get_sample_rejected_status(&self) -> DdsResult<SampleRejectedStatus> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut sample_rejected_status_lock = self.sample_rejected_status.write_lock();
        let sample_rejected_status = sample_rejected_status_lock.clone();

        sample_rejected_status_lock.total_count_change = 0;

        Ok(sample_rejected_status)
    }

    pub fn get_subscription_matched_status(&self) -> DdsResult<SubscriptionMatchedStatus> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut subscription_matched_status_lock = self.subscription_matched_status.write_lock();
        let subscription_matched_status = subscription_matched_status_lock.clone();

        subscription_matched_status_lock.current_count_change = 0;
        subscription_matched_status_lock.total_count_change = 0;

        Ok(subscription_matched_status)
    }

    pub fn get_topicdescription(&self) -> DdsResult<DdsShared<TopicImpl>> {
        Ok(self.topic.clone())
    }

    pub fn get_subscriber(&self) -> DdsResult<DdsShared<SubscriberImpl>> {
        Ok(self
            .parent_subscriber
            .upgrade()
            .expect("Failed to get parent subscriber of data reader"))
    }

    pub fn delete_contained_entities(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn wait_for_historical_data(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn get_matched_publication_data(
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

    pub fn get_matched_publications(&self) -> DdsResult<Vec<InstanceHandle>> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        Ok(self
            .matched_publication_list
            .read_lock()
            .iter()
            .map(|(&key, _)| key)
            .collect())
    }

    fn read_sample<'a>(&self, cache_change: &'a RtpsReaderCacheChange) -> (&'a [u8], SampleInfo) {
        *self.status_change.write_lock() &= !DATA_AVAILABLE_STATUS;

        let mut samples_read = self.samples_read.write_lock();
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
            view_state: cache_change.view_state(),
            instance_state,
            disposed_generation_count: 0,
            no_writers_generation_count: 0,
            sample_rank: 0,
            generation_rank: 0,
            absolute_generation_rank: 0,
            source_timestamp: *cache_change.source_timestamp(),
            instance_handle: cache_change.instance_handle(),
            publication_handle: <[u8; 16]>::from(cache_change.writer_guid()).into(),
            valid_data,
        };

        (data_value, sample_info)
    }
}

impl<Tim> DdsShared<DataReaderImpl<Tim>> {
    pub fn set_qos(&self, qos: Option<DataReaderQos>) -> DdsResult<()> {
        let qos = qos.unwrap_or_default();

        let mut rtps_reader_lock = self.rtps_reader.write_lock();
        if *self.enabled.read_lock() {
            rtps_reader_lock
                .reader()
                .get_qos()
                .check_immutability(&qos)?;
        }

        rtps_reader_lock.reader_mut().set_qos(qos)?;

        Ok(())
    }

    pub fn get_qos(&self) -> DdsResult<DataReaderQos> {
        Ok(self.rtps_reader.read_lock().reader().get_qos().clone())
    }

    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn AnyDataReaderListener + Send + Sync>>,
        _mask: StatusMask,
    ) -> DdsResult<()> {
        *self.listener.write_lock() = a_listener;
        Ok(())
    }

    pub fn get_listener(&self) -> DdsResult<Option<Box<dyn AnyDataReaderListener + Send + Sync>>> {
        todo!()
    }

    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        todo!()
    }

    pub fn get_status_changes(&self) -> DdsResult<StatusMask> {
        Ok(*self.status_change.read_lock())
    }

    pub fn enable(&self, parent_participant: &DdsShared<DomainParticipantImpl>) -> DdsResult<()> {
        if !self.parent_subscriber.upgrade()?.is_enabled() {
            return Err(DdsError::PreconditionNotMet(
                "Parent subscriber disabled".to_string(),
            ));
        }

        parent_participant.announce_datareader(self.try_into()?);
        *self.enabled.write_lock() = true;

        Ok(())
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        Ok(<[u8; 16]>::from(self.rtps_reader.read_lock().reader().guid()).into())
    }
}

impl<Tim> TryFrom<&DdsShared<DataReaderImpl<Tim>>> for DiscoveredReaderData {
    type Error = DdsError;

    fn try_from(val: &DdsShared<DataReaderImpl<Tim>>) -> DdsResult<Self> {
        let rtps_reader_lock = val.rtps_reader.read_lock();
        let guid = rtps_reader_lock.reader().guid();
        let reader_qos = rtps_reader_lock.reader().get_qos();
        let topic_qos = val.topic.get_qos()?;
        let subscriber_qos = val.parent_subscriber.upgrade()?.get_qos()?;

        Ok(DiscoveredReaderData {
            reader_proxy: ReaderProxy {
                remote_reader_guid: guid,
                remote_group_entity_id: guid.entity_id,
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                expects_inline_qos: false,
            },

            subscription_builtin_topic_data: SubscriptionBuiltinTopicData {
                key: BuiltInTopicKey { value: guid.into() },
                participant_key: BuiltInTopicKey { value: [1; 16] },
                topic_name: val.topic.get_name().unwrap(),
                type_name: val.topic.get_type_name().unwrap().to_string(),
                durability: reader_qos.durability.clone(),
                deadline: reader_qos.deadline.clone(),
                latency_budget: reader_qos.latency_budget.clone(),
                liveliness: reader_qos.liveliness.clone(),
                reliability: reader_qos.reliability.clone(),
                ownership: reader_qos.ownership.clone(),
                destination_order: reader_qos.destination_order.clone(),
                user_data: reader_qos.user_data.clone(),
                time_based_filter: reader_qos.time_based_filter.clone(),
                presentation: subscriber_qos.presentation.clone(),
                partition: subscriber_qos.partition.clone(),
                topic_data: topic_qos.topic_data,
                group_data: subscriber_qos.group_data,
            },
        })
    }
}

impl<Tim> DdsShared<DataReaderImpl<Tim>> {
    pub fn send_message(&self, transport: &mut impl TransportWrite) {
        if let RtpsReaderKind::Stateful(stateful_rtps_reader) = &mut *self.rtps_reader.write_lock()
        {
            let mut acknacks = Vec::new();
            stateful_rtps_reader.send_submessages(|wp, acknack| {
                acknacks.push((
                    wp.unicast_locator_list().to_vec(),
                    vec![RtpsSubmessageType::AckNack(acknack)],
                ))
            });

            for (locator_list, acknacks) in acknacks {
                let header = RtpsMessageHeader {
                    protocol: ProtocolId::PROTOCOL_RTPS,
                    version: ProtocolVersionSubmessageElement {
                        value: PROTOCOLVERSION.into(),
                    },
                    vendor_id: VendorIdSubmessageElement {
                        value: VENDOR_ID_S2E,
                    },
                    guid_prefix: GuidPrefixSubmessageElement {
                        value: stateful_rtps_reader.reader().guid().prefix().into(),
                    },
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
        dcps_psm::{DURATION_ZERO, NEW_VIEW_STATE},
        dds_type::DdsSerialize,
        implementation::rtps::{
            endpoint::RtpsEndpoint,
            reader::RtpsReader,
            types::{EntityId, Guid, TopicKind, ENTITYID_UNKNOWN, GUID_UNKNOWN},
        },
        infrastructure::qos_policy::HistoryQosPolicyKind,
        {
            dcps_psm::{BuiltInTopicKey, ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
            infrastructure::{
                qos::{SubscriberQos, TopicQos},
                qos_policy::{
                    DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
                    DurabilityServiceQosPolicy, GroupDataQosPolicy, HistoryQosPolicy,
                    LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy,
                    OwnershipQosPolicy, OwnershipStrengthQosPolicy, PartitionQosPolicy,
                    PresentationQosPolicy, ReliabilityQosPolicy, ReliabilityQosPolicyKind,
                    TopicDataQosPolicy, UserDataQosPolicy,
                },
            },
        },
    };
    use crate::{
        dds_type::{DdsType, Endianness},
        implementation::{
            data_representation_builtin_endpoints::discovered_writer_data::WriterProxy,
            dds_impl::{data_reader_impl::RtpsReaderKind, topic_impl::TopicImpl},
            rtps::group::RtpsGroupImpl,
            utils::shared_object::DdsShared,
        },
    };

    use mockall::mock;
    use std::io::Write;

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

    fn cache_change(value: u8, sn: SequenceNumber) -> RtpsReaderCacheChange {
        let cache_change = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16].into(),
            sn,
            vec![value],
            vec![],
            None,
            NEW_VIEW_STATE,
        );

        cache_change
    }

    fn reader_with_changes(
        changes: Vec<RtpsReaderCacheChange>,
    ) -> DdsShared<DataReaderImpl<ThreadTimer>> {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAllHistoryQos,
                depth: 0,
            },
            ..Default::default()
        };
        let mut stateful_reader = RtpsStatefulReader::new(RtpsReader::new::<UserData>(
            RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]),
            DURATION_ZERO,
            DURATION_ZERO,
            false,
            qos,
        ));
        for change in changes {
            stateful_reader.reader_mut().add_change(change).unwrap();
        }

        let data_reader = DataReaderImpl::new(
            RtpsReaderKind::Stateful(stateful_reader),
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
        let reader = DdsShared::new(reader_with_changes(vec![
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
        let reader = reader_with_changes(vec![cache_change(1, 1)]);

        let unread_samples = reader
            .read::<UserData>(
                i32::MAX,
                NOT_READ_SAMPLE_STATE,
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
            )
            .unwrap();

        assert_eq!(1, unread_samples.len());

        assert!(reader
            .read::<UserData>(
                i32::MAX,
                NOT_READ_SAMPLE_STATE,
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
            )
            .is_err());
    }

    mock! {
        Listener {}
        impl AnyDataReaderListener for Listener {
            fn trigger_on_data_available(&mut self, reader: &DdsShared<DataReaderImpl<ThreadTimer>>);
            fn trigger_on_sample_rejected(
                &mut self,
                reader: &DdsShared<DataReaderImpl<ThreadTimer>>,
                status: SampleRejectedStatus,
            );
            fn trigger_on_liveliness_changed(
                &mut self,
                reader: &DdsShared<DataReaderImpl<ThreadTimer>>,
                status: LivelinessChangedStatus,
            );
            fn trigger_on_requested_deadline_missed(
                &mut self,
                reader: &DdsShared<DataReaderImpl<ThreadTimer>>,
                status: RequestedDeadlineMissedStatus,
            );
            fn trigger_on_requested_incompatible_qos(
                &mut self,
                reader: &DdsShared<DataReaderImpl<ThreadTimer>>,
                status: RequestedIncompatibleQosStatus,
            );
            fn trigger_on_subscription_matched(
                &mut self,
                reader: &DdsShared<DataReaderImpl<ThreadTimer>>,
                status: SubscriptionMatchedStatus,
            );
            fn trigger_on_sample_lost(
                &mut self,
                reader: &DdsShared<DataReaderImpl<ThreadTimer>>,
                status: SampleLostStatus,
            );
        }
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
        let qos = DataReaderQos::default();
        let stateful_reader = RtpsStatefulReader::new(RtpsReader::new::<UserData>(
            RtpsEndpoint::new(guid, TopicKind::NoKey, &[], &[]),
            DURATION_ZERO,
            DURATION_ZERO,
            false,
            qos,
        ));

        let data_reader: DdsShared<DataReaderImpl<ThreadTimer>> = DataReaderImpl::new(
            RtpsReaderKind::Stateful(stateful_reader),
            dummy_topic,
            None,
            DdsWeak::new(),
        );
        *data_reader.enabled.write_lock() = true;

        let expected_instance_handle: InstanceHandle = <[u8; 16]>::from(guid).into();
        let instance_handle = data_reader.get_instance_handle().unwrap();
        assert_eq!(expected_instance_handle, instance_handle);
    }

    #[test]
    fn add_compatible_matched_writer() {
        let type_name = "test_type";
        let topic_name = "test_topic".to_string();
        let parent_subscriber =
            SubscriberImpl::new(SubscriberQos::default(), RtpsGroupImpl::new(GUID_UNKNOWN));
        let test_topic = TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            type_name,
            &topic_name,
            DdsWeak::new(),
        );

        let rtps_reader = RtpsStatefulReader::new(RtpsReader::new::<UserData>(
            RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]),
            DURATION_ZERO,
            DURATION_ZERO,
            false,
            DataReaderQos::default(),
        ));

        let data_reader = DataReaderImpl::<ThreadTimer>::new(
            RtpsReaderKind::Stateful(rtps_reader),
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
                max_blocking_time: crate::dcps_psm::Duration::new(0, 0),
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
            writer_proxy: WriterProxy {
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
        assert_eq!(matched_publications[0], [2; 16].into());
        let matched_publication_data = data_reader
            .get_matched_publication_data(matched_publications[0])
            .unwrap();
        assert_eq!(matched_publication_data, publication_builtin_topic_data);
    }

    #[test]
    fn add_incompatible_matched_writer() {
        let type_name = "test_type";
        let topic_name = "test_topic".to_string();
        let parent_subscriber =
            SubscriberImpl::new(SubscriberQos::default(), RtpsGroupImpl::new(GUID_UNKNOWN));
        let test_topic = TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            type_name,
            &topic_name,
            DdsWeak::new(),
        );

        let mut data_reader_qos = DataReaderQos::default();
        data_reader_qos.reliability.kind = ReliabilityQosPolicyKind::ReliableReliabilityQos;

        let rtps_reader = RtpsStatefulReader::new(RtpsReader::new::<UserData>(
            RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]),
            DURATION_ZERO,
            DURATION_ZERO,
            false,
            data_reader_qos,
        ));

        let data_reader = DataReaderImpl::<ThreadTimer>::new(
            RtpsReaderKind::Stateful(rtps_reader),
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
                max_blocking_time: crate::dcps_psm::Duration::new(0, 0),
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
            writer_proxy: WriterProxy {
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
