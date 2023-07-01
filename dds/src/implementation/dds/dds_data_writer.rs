use std::collections::{HashMap, HashSet};

use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData},
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_writer_data::{DiscoveredWriterData, WriterProxy},
        },
        rtps::{
            history_cache::{DataFragSubmessages, RtpsWriterCacheChange},
            messages::{
                overall_structure::{
                    RtpsMessageHeader, RtpsMessageRead, RtpsMessageWrite, RtpsSubmessageReadKind,
                    RtpsSubmessageWriteKind,
                },
                submessage_elements::{ParameterList, SequenceNumberSet},
                submessages::{
                    ack_nack::AckNackSubmessageRead, gap::GapSubmessageWrite,
                    info_destination::InfoDestinationSubmessageWrite,
                    info_timestamp::InfoTimestampSubmessageWrite,
                    nack_frag::NackFragSubmessageRead,
                },
                types::FragmentNumber,
            },
            reader_locator::{RtpsReaderLocator, WriterAssociatedReaderLocator},
            reader_proxy::{RtpsReaderProxy, WriterAssociatedReaderProxy},
            stateful_writer::RtpsStatefulWriter,
            stateless_writer::RtpsStatelessWriter,
            types::{
                ChangeKind, DurabilityKind, EntityId, EntityKey, Guid, GuidPrefix, Locator,
                ReliabilityKind, SequenceNumber, ENTITYID_UNKNOWN, GUID_UNKNOWN,
                USER_DEFINED_UNKNOWN,
            },
        },
        rtps_udp_psm::udp_transport::UdpTransportWrite,
        utils::{
            actor::{Actor, ActorAddress},
            shared_object::{DdsRwLock, DdsShared},
        },
    },
    infrastructure::{
        instance::{InstanceHandle, HANDLE_NIL},
        qos::{PublisherQos, TopicQos},
        qos_policy::{
            DurabilityQosPolicyKind, QosPolicyId, ReliabilityQosPolicyKind, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, INVALID_QOS_POLICY_ID,
            LATENCYBUDGET_QOS_POLICY_ID, LIVELINESS_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID,
            RELIABILITY_QOS_POLICY_ID,
        },
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, QosPolicyCount, StatusKind,
        },
        time::DurationKind,
    },
    topic_definition::type_support::{DdsSerializedKey, DdsType},
    {
        builtin_topics::SubscriptionBuiltinTopicData,
        infrastructure::{
            error::{DdsError, DdsResult},
            qos::DataWriterQos,
            time::{Duration, Time},
        },
    },
};

use super::{
    dds_data_writer_listener::DdsDataWriterListener, dds_domain_participant::DdsDomainParticipant,
    dds_publisher::DdsPublisher, message_receiver::MessageReceiver, nodes::DataWriterNode,
    status_condition_impl::StatusConditionImpl,
};

struct MatchedSubscriptions {
    matched_subscription_list: HashMap<InstanceHandle, SubscriptionBuiltinTopicData>,
    total_count: i32,
    total_count_last_read: i32,
    current_count_last_read: i32,
    last_subscription_handle: InstanceHandle,
}

impl MatchedSubscriptions {
    fn new() -> Self {
        Self {
            matched_subscription_list: HashMap::new(),
            total_count: 0,
            total_count_last_read: 0,
            current_count_last_read: 0,
            last_subscription_handle: HANDLE_NIL,
        }
    }

    fn add_matched_subscription(
        &mut self,
        handle: InstanceHandle,
        subscription_data: SubscriptionBuiltinTopicData,
    ) {
        self.matched_subscription_list
            .insert(handle, subscription_data);
        self.total_count += 1;
        self.last_subscription_handle = handle;
    }

    fn remove_matched_subscription(&mut self, handle: InstanceHandle) {
        self.matched_subscription_list.remove(&handle);
    }

    fn get_matched_subscriptions(&self) -> Vec<InstanceHandle> {
        self.matched_subscription_list
            .iter()
            .map(|(&h, _)| h)
            .collect()
    }

    pub fn get_matched_subscription_data(
        &self,
        handle: InstanceHandle,
    ) -> Option<&SubscriptionBuiltinTopicData> {
        self.matched_subscription_list.get(&handle)
    }

    fn get_publication_matched_status(&mut self) -> PublicationMatchedStatus {
        let current_count = self.matched_subscription_list.len() as i32;
        let status = PublicationMatchedStatus {
            total_count: self.total_count,
            total_count_change: self.total_count - self.total_count_last_read,
            last_subscription_handle: self.last_subscription_handle,
            current_count,
            current_count_change: current_count - self.current_count_last_read,
        };

        self.total_count_last_read = self.total_count;
        self.current_count_last_read = current_count;

        status
    }
}

struct IncompatibleSubscriptions {
    incompatible_subscription_list: HashSet<InstanceHandle>,
    total_count: i32,
    total_count_last_read: i32,
    last_policy_id: QosPolicyId,
    policies: Vec<QosPolicyCount>,
}

impl IncompatibleSubscriptions {
    fn new() -> Self {
        Self {
            incompatible_subscription_list: HashSet::new(),
            total_count: 0,
            total_count_last_read: 0,
            last_policy_id: INVALID_QOS_POLICY_ID,
            policies: Vec::new(),
        }
    }

    fn add_offered_incompatible_qos(
        &mut self,
        handle: InstanceHandle,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
    ) {
        self.total_count += 1;
        self.last_policy_id = incompatible_qos_policy_list[0];

        self.incompatible_subscription_list.insert(handle);
        for incompatible_qos_policy in incompatible_qos_policy_list.into_iter() {
            if let Some(policy_count) = self
                .policies
                .iter_mut()
                .find(|x| x.policy_id == incompatible_qos_policy)
            {
                policy_count.count += 1;
            } else {
                self.policies.push(QosPolicyCount {
                    policy_id: incompatible_qos_policy,
                    count: 1,
                })
            }
        }
    }

    fn get_incompatible_subscriptions(&self) -> Vec<InstanceHandle> {
        self.incompatible_subscription_list
            .iter()
            .cloned()
            .collect()
    }

    fn get_offered_incompatible_qos_status(&mut self) -> OfferedIncompatibleQosStatus {
        let status = OfferedIncompatibleQosStatus {
            total_count: self.total_count,
            total_count_change: self.total_count - self.total_count_last_read,
            last_policy_id: self.last_policy_id,
            policies: self.policies.clone(),
        };

        self.total_count_last_read = self.total_count;

        status
    }
}

pub struct DdsDataWriter<T> {
    rtps_writer: T,
    type_name: &'static str,
    topic_name: String,
    matched_subscriptions: MatchedSubscriptions,
    incompatible_subscriptions: IncompatibleSubscriptions,
    enabled: bool,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
    listener: Option<Actor<DdsDataWriterListener>>,
    status_kind: Vec<StatusKind>,
}

impl<T> DdsDataWriter<T> {
    pub fn new(
        rtps_writer: T,
        type_name: &'static str,
        topic_name: String,
        listener: Option<Actor<DdsDataWriterListener>>,
        status_kind: Vec<StatusKind>,
    ) -> Self {
        DdsDataWriter {
            rtps_writer,
            type_name,
            topic_name,
            matched_subscriptions: MatchedSubscriptions::new(),
            incompatible_subscriptions: IncompatibleSubscriptions::new(),
            enabled: false,
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
            listener,
            status_kind,
        }
    }

    pub fn get_publication_matched_status(&mut self) -> PublicationMatchedStatus {
        self.status_condition
            .write_lock()
            .remove_communication_state(StatusKind::PublicationMatched);
        self.matched_subscriptions.get_publication_matched_status()
    }

    pub fn add_matched_publication(
        &mut self,
        handle: InstanceHandle,
        subscription_data: SubscriptionBuiltinTopicData,
    ) {
        self.matched_subscriptions
            .add_matched_subscription(handle, subscription_data)
    }

    pub fn remove_matched_subscription(&mut self, handle: InstanceHandle) {
        self.matched_subscriptions
            .remove_matched_subscription(handle)
    }

    pub fn get_matched_subscriptions(&self) -> Vec<InstanceHandle> {
        self.matched_subscriptions.get_matched_subscriptions()
    }

    pub fn get_matched_subscription_data(
        &self,
        handle: InstanceHandle,
    ) -> Option<SubscriptionBuiltinTopicData> {
        self.matched_subscriptions
            .get_matched_subscription_data(handle)
            .cloned()
    }

    pub fn add_offered_incompatible_qos(
        &mut self,
        handle: InstanceHandle,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
    ) {
        self.incompatible_subscriptions
            .add_offered_incompatible_qos(handle, incompatible_qos_policy_list)
    }

    pub fn get_offered_incompatible_qos_status(&mut self) -> OfferedIncompatibleQosStatus {
        self.incompatible_subscriptions
            .get_offered_incompatible_qos_status()
    }

    pub fn get_offered_deadline_missed_status(&self) -> OfferedDeadlineMissedStatus {
        todo!()
    }

    pub fn get_liveliness_lost_status(&self) -> LivelinessLostStatus {
        todo!()
    }

    pub fn get_incompatible_subscriptions(&self) -> Vec<InstanceHandle> {
        self.incompatible_subscriptions
            .get_incompatible_subscriptions()
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_topic_name(&self) -> &str {
        &self.topic_name
    }

    pub fn get_type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }
}

impl DdsDataWriter<RtpsStatefulWriter> {
    pub fn guid(&self) -> Guid {
        self.rtps_writer.guid()
    }

    pub fn _unicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_writer.unicast_locator_list().to_vec()
    }

    pub fn _multicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_writer.multicast_locator_list().to_vec()
    }

    pub fn _push_mode(&self) -> bool {
        self.rtps_writer.push_mode()
    }

    pub fn heartbeat_period(&self) -> Duration {
        self.rtps_writer.heartbeat_period()
    }

    pub fn data_max_size_serialized(&self) -> usize {
        self.rtps_writer.data_max_size_serialized()
    }

    pub fn _new_change(
        &mut self,
        kind: ChangeKind,
        data: Vec<u8>,
        inline_qos: ParameterList,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> RtpsWriterCacheChange {
        self.rtps_writer
            .new_change(kind, data, inline_qos, handle, timestamp)
    }

    pub fn change_list(&self) -> &[RtpsWriterCacheChange] {
        self.rtps_writer.change_list()
    }

    pub fn _add_change(&mut self, change: RtpsWriterCacheChange) {
        self.rtps_writer.add_change(change)
    }

    pub fn remove_change<F>(&mut self, f: F)
    where
        F: FnMut(&RtpsWriterCacheChange) -> bool,
    {
        self.rtps_writer.remove_change(f)
    }

    pub fn matched_reader_add(&mut self, a_reader_proxy: RtpsReaderProxy) {
        self.rtps_writer.matched_reader_add(a_reader_proxy)
    }

    pub fn matched_reader_remove(&mut self, a_reader_guid: Guid) {
        self.rtps_writer.matched_reader_remove(a_reader_guid)
    }

    pub fn matched_reader_list(&mut self) -> Vec<WriterAssociatedReaderProxy> {
        self.rtps_writer.matched_reader_list()
    }

    pub fn get_qos(&self) -> DataWriterQos {
        self.rtps_writer.get_qos().clone()
    }

    pub fn set_qos(&mut self, qos: DataWriterQos) {
        self.rtps_writer.set_qos(qos);
    }

    pub fn process_rtps_message(&mut self, message: RtpsMessageRead) {
        let mut message_receiver = MessageReceiver::new(&message);
        while let Some(submessage) = message_receiver.next() {
            match &submessage {
                RtpsSubmessageReadKind::AckNack(acknack_submessage) => self
                    .on_acknack_submessage_received(
                        acknack_submessage,
                        message_receiver.source_guid_prefix(),
                    ),
                RtpsSubmessageReadKind::NackFrag(nackfrag_submessage) => self
                    .on_nack_frag_submessage_received(
                        nackfrag_submessage,
                        message_receiver.source_guid_prefix(),
                    ),
                _ => (),
            }
        }
    }

    fn on_acknack_submessage_received(
        &mut self,
        acknack_submessage: &AckNackSubmessageRead,
        source_guid_prefix: GuidPrefix,
    ) {
        if self.rtps_writer.get_qos().reliability.kind == ReliabilityQosPolicyKind::Reliable {
            let reader_guid = Guid::new(source_guid_prefix, acknack_submessage.reader_id());

            if let Some(reader_proxy) = self
                .rtps_writer
                .matched_reader_list()
                .iter_mut()
                .find(|x| x.remote_reader_guid() == reader_guid)
            {
                reader_proxy.receive_acknack(acknack_submessage);
            }
        }
    }

    fn on_nack_frag_submessage_received(
        &mut self,
        nackfrag_submessage: &NackFragSubmessageRead,
        source_guid_prefix: GuidPrefix,
    ) {
        if self.rtps_writer.get_qos().reliability.kind == ReliabilityQosPolicyKind::Reliable {
            let reader_guid = Guid::new(source_guid_prefix, nackfrag_submessage.reader_id());

            if let Some(reader_proxy) = self
                .rtps_writer
                .matched_reader_list()
                .iter_mut()
                .find(|x| x.remote_reader_guid() == reader_guid)
            {
                reader_proxy.receive_nack_frag(nackfrag_submessage);
            }
        }
    }

    pub fn register_instance_w_timestamp(
        &mut self,
        instance_serialized_key: DdsSerializedKey,
        timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_writer
            .register_instance_w_timestamp(instance_serialized_key, timestamp)
    }

    pub fn unregister_instance_w_timestamp(
        &mut self,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> DdsResult<()> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_writer
            .unregister_instance_w_timestamp(instance_serialized_key, handle, timestamp)
    }

    pub fn get_key_value<Foo>(&self, key_holder: &mut Foo, handle: InstanceHandle) -> DdsResult<()>
    where
        Foo: DdsType,
    {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        key_holder.set_key_fields_from_serialized_key(
            self.rtps_writer
                .get_key_value(handle)
                .ok_or(DdsError::BadParameter)?,
        )
    }

    pub fn lookup_instance(
        &self,
        instance_serialized_key: DdsSerializedKey,
    ) -> DdsResult<Option<InstanceHandle>> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        Ok(self.rtps_writer.lookup_instance(instance_serialized_key))
    }

    pub fn write_w_timestamp(
        &mut self,
        serialized_data: Vec<u8>,
        instance_serialized_key: DdsSerializedKey,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        self.rtps_writer.write_w_timestamp(
            serialized_data,
            instance_serialized_key,
            handle,
            timestamp,
        )?;

        Ok(())
    }

    pub fn dispose_w_timestamp(
        &mut self,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> DdsResult<()> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_writer
            .dispose_w_timestamp(instance_serialized_key, handle, timestamp)
    }

    pub fn are_all_changes_acknowledge(&mut self) -> bool {
        !self
            .rtps_writer
            .matched_reader_list()
            .iter()
            .any(|rp| rp.unacked_changes())
    }

    pub fn as_discovered_writer_data(
        &self,
        topic_qos: TopicQos,
        publisher_qos: PublisherQos,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
    ) -> DiscoveredWriterData {
        let writer_qos = self.rtps_writer.get_qos().clone();
        let unicast_locator_list = if self.rtps_writer.unicast_locator_list().is_empty() {
            default_unicast_locator_list
        } else {
            self.rtps_writer.unicast_locator_list().to_vec()
        };

        let multicast_locator_list = if self.rtps_writer.unicast_locator_list().is_empty() {
            default_multicast_locator_list
        } else {
            self.rtps_writer.multicast_locator_list().to_vec()
        };

        DiscoveredWriterData::new(
            PublicationBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: self.rtps_writer.guid().into(),
                },
                BuiltInTopicKey {
                    value: GUID_UNKNOWN.into(),
                },
                self.topic_name.clone(),
                self.type_name.to_string(),
                writer_qos.durability.clone(),
                writer_qos.deadline.clone(),
                writer_qos.latency_budget.clone(),
                writer_qos.liveliness.clone(),
                writer_qos.reliability.clone(),
                writer_qos.lifespan.clone(),
                writer_qos.user_data.clone(),
                writer_qos.ownership.clone(),
                writer_qos.destination_order,
                publisher_qos.presentation.clone(),
                publisher_qos.partition.clone(),
                topic_qos.topic_data,
                publisher_qos.group_data,
            ),
            WriterProxy::new(
                self.rtps_writer.guid(),
                EntityId::new(EntityKey::new([0; 3]), USER_DEFINED_UNKNOWN),
                unicast_locator_list,
                multicast_locator_list,
                None,
            ),
        )
    }

    pub fn send_message(
        &mut self,
        header: RtpsMessageHeader,
        udp_transport_write: ActorAddress<UdpTransportWrite>,
        now: Time,
    ) {
        // Remove stale changes before sending
        let timespan_duration = self.get_qos().lifespan.duration;
        self.remove_change(|cc| DurationKind::Finite(now - cc.timestamp()) > timespan_duration);

        let writer_id = self.guid().entity_id();
        let first_sn = self
            .change_list()
            .iter()
            .map(|x| x.sequence_number())
            .min()
            .unwrap_or_else(|| SequenceNumber::new(1));
        let last_sn = self
            .change_list()
            .iter()
            .map(|x| x.sequence_number())
            .max()
            .unwrap_or_else(|| SequenceNumber::new(0));
        let heartbeat_period = self.heartbeat_period();
        for reader_proxy in &mut self.matched_reader_list() {
            match reader_proxy.reliability() {
                ReliabilityKind::BestEffort => Self::send_message_best_effort_reader_proxy(
                    reader_proxy,
                    header,
                    &udp_transport_write,
                ),
                ReliabilityKind::Reliable => Self::send_message_reliable_reader_proxy(
                    reader_proxy,
                    header,
                    &udp_transport_write,
                    writer_id,
                    first_sn,
                    last_sn,
                    heartbeat_period,
                ),
            }
        }
    }

    fn send_message_best_effort_reader_proxy(
        reader_proxy: &mut WriterAssociatedReaderProxy,
        header: RtpsMessageHeader,
        udp_transport_write: &ActorAddress<UdpTransportWrite>,
    ) {
        let info_dst =
            Self::info_destination_submessage(reader_proxy.remote_reader_guid().prefix());
        let mut submessages = vec![info_dst];

        // a_change_seq_num := the_reader_proxy.next_unsent_change();
        // if ( a_change_seq_num > the_reader_proxy.higuest_sent_seq_num +1 ) {
        // GAP = new GAP(the_reader_locator.higuest_sent_seq_num + 1,
        // a_change_seq_num -1);
        // GAP.readerId := ENTITYID_UNKNOWN;
        // GAP.filteredCount := 0;
        // send GAP;
        // }
        // a_change := the_writer.writer_cache.get_change(a_change_seq_num );
        // if ( DDS_FILTER(the_reader_proxy, a_change) ) {
        // DATA = new DATA(a_change);
        // IF (the_reader_proxy.expectsInlineQos) {
        // DATA.inlineQos := the_rtps_writer.related_dds_writer.qos;
        // DATA.inlineQos += a_change.inlineQos;
        // }
        // DATA.readerId := ENTITYID_UNKNOWN;
        // send DATA;
        // }
        // else {
        // GAP = new GAP(a_change.sequenceNumber);
        // GAP.readerId := ENTITYID_UNKNOWN;
        // GAP.filteredCount := 1;
        // send GAP;
        // }
        // the_reader_proxy.higuest_sent_seq_num := a_change_seq_num;

        while reader_proxy.unsent_changes() {
            // Note: The readerId is set to the remote reader ID as described in 8.4.9.2.12 Transition T12
            // in confront to ENTITYID_UNKNOWN as described in 8.4.9.2.4 Transition T4
            let reader_id = reader_proxy.remote_reader_guid().entity_id();
            let a_change_seq_num = reader_proxy.next_unsent_change().expect("Should be Some");

            if a_change_seq_num > reader_proxy.highest_sent_seq_num() + 1 {
                let gap_set = (i64::from(reader_proxy.highest_sent_seq_num()) + 2
                    ..i64::from(a_change_seq_num) - 1)
                    .map(SequenceNumber::new)
                    .collect();
                let gap_submessage = GapSubmessageWrite::new(
                    reader_id,
                    reader_proxy.writer().guid().entity_id(),
                    reader_proxy.highest_sent_seq_num() + 1,
                    SequenceNumberSet {
                        base: reader_proxy.highest_sent_seq_num() + 1,
                        set: gap_set,
                    },
                );
                submessages.push(RtpsSubmessageWriteKind::Gap(gap_submessage));
            }
            let cache_change = reader_proxy
                .writer()
                .change_list()
                .iter()
                .find(|cc| cc.sequence_number() == a_change_seq_num);

            match cache_change {
                Some(cache_change)
                    if cache_change.sequence_number()
                        > reader_proxy.first_relevant_sample_seq_num() =>
                {
                    let timestamp = cache_change.timestamp();

                    if cache_change.data_value().len() > 1 {
                        let cache_change_frag = DataFragSubmessages::new(cache_change, reader_id);
                        for data_frag_submessage in cache_change_frag.into_iter() {
                            let info_dst = RtpsSubmessageWriteKind::InfoDestination(
                                InfoDestinationSubmessageWrite::new(
                                    reader_proxy.remote_reader_guid().prefix(),
                                ),
                            );

                            let info_timestamp = RtpsSubmessageWriteKind::InfoTimestamp(
                                InfoTimestampSubmessageWrite::new(
                                    false,
                                    crate::implementation::rtps::messages::types::Time::new(
                                        timestamp.sec(),
                                        timestamp.nanosec(),
                                    ),
                                ),
                            );
                            let data_frag = RtpsSubmessageWriteKind::DataFrag(data_frag_submessage);

                            let submessages = vec![info_dst, info_timestamp, data_frag];

                            udp_transport_write
                                .write(
                                    RtpsMessageWrite::new(header, submessages),
                                    reader_proxy.unicast_locator_list().to_vec(),
                                )
                                .unwrap();
                        }
                    } else {
                        submessages.push(RtpsSubmessageWriteKind::InfoTimestamp(
                            InfoTimestampSubmessageWrite::new(
                                false,
                                crate::implementation::rtps::messages::types::Time::new(
                                    timestamp.sec(),
                                    timestamp.nanosec(),
                                ),
                            ),
                        ));
                        submessages.push(RtpsSubmessageWriteKind::Data(
                            cache_change.as_data_submessage(reader_id),
                        ))
                    }
                }
                _ => {
                    let gap_submessage = GapSubmessageWrite::new(
                        reader_id,
                        reader_proxy.writer().guid().entity_id(),
                        a_change_seq_num,
                        SequenceNumberSet {
                            base: a_change_seq_num,
                            set: vec![],
                        },
                    );
                    submessages.push(RtpsSubmessageWriteKind::Gap(gap_submessage));
                }
            }

            reader_proxy.set_highest_sent_seq_num(a_change_seq_num);
        }

        // Send messages only if more than INFO_DST is added
        if submessages.len() > 1 {
            udp_transport_write
                .write(
                    RtpsMessageWrite::new(header, submessages),
                    reader_proxy.unicast_locator_list().to_vec(),
                )
                .ok();
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn send_message_reliable_reader_proxy(
        reader_proxy: &mut WriterAssociatedReaderProxy,
        header: RtpsMessageHeader,
        udp_transport_write: &ActorAddress<UdpTransportWrite>,
        writer_id: EntityId,
        first_sn: SequenceNumber,
        last_sn: SequenceNumber,
        heartbeat_period: Duration,
    ) {
        let reader_id = reader_proxy.remote_reader_guid().entity_id();

        let info_dst =
            Self::info_destination_submessage(reader_proxy.remote_reader_guid().prefix());

        let mut submessages = vec![info_dst];

        // Top part of the state machine - Figure 8.19 RTPS standard
        if reader_proxy.unsent_changes() {
            // Note: The readerId is set to the remote reader ID as described in 8.4.9.2.12 Transition T12
            // in confront to ENTITYID_UNKNOWN as described in 8.4.9.2.4 Transition T4

            while reader_proxy.unsent_changes() {
                let a_change_seq_num = reader_proxy.next_unsent_change().expect("Should be Some");

                if a_change_seq_num > reader_proxy.highest_sent_seq_num() + 1 {
                    let gap_set = (i64::from(reader_proxy.highest_sent_seq_num()) + 2
                        ..i64::from(a_change_seq_num) - 1)
                        .map(SequenceNumber::new)
                        .collect();
                    let gap_submessage = GapSubmessageWrite::new(
                        reader_id,
                        reader_proxy.writer().guid().entity_id(),
                        reader_proxy.highest_sent_seq_num() + 1,
                        SequenceNumberSet {
                            base: reader_proxy.highest_sent_seq_num() + 1,
                            set: gap_set,
                        },
                    );
                    submessages.push(RtpsSubmessageWriteKind::Gap(gap_submessage));
                }

                let cache_change = reader_proxy
                    .writer()
                    .change_list()
                    .iter()
                    .find(|cc| cc.sequence_number() == a_change_seq_num);

                match cache_change {
                    Some(cache_change)
                        if cache_change.sequence_number()
                            > reader_proxy.first_relevant_sample_seq_num() =>
                    {
                        let timestamp = cache_change.timestamp();

                        if cache_change.data_value().len() > 1 {
                            let cache_change_frag =
                                DataFragSubmessages::new(cache_change, reader_id);
                            for data_frag_submessage in cache_change_frag.into_iter() {
                                let info_dst = RtpsSubmessageWriteKind::InfoDestination(
                                    InfoDestinationSubmessageWrite::new(
                                        reader_proxy.remote_reader_guid().prefix(),
                                    ),
                                );

                                let info_timestamp = RtpsSubmessageWriteKind::InfoTimestamp(
                                    InfoTimestampSubmessageWrite::new(
                                        false,
                                        crate::implementation::rtps::messages::types::Time::new(
                                            timestamp.sec(),
                                            timestamp.nanosec(),
                                        ),
                                    ),
                                );
                                let data_frag =
                                    RtpsSubmessageWriteKind::DataFrag(data_frag_submessage);

                                let submessages = vec![info_dst, info_timestamp, data_frag];

                                udp_transport_write
                                    .write(
                                        RtpsMessageWrite::new(header, submessages),
                                        reader_proxy.unicast_locator_list().to_vec(),
                                    )
                                    .unwrap();
                            }
                        } else {
                            submessages.push(RtpsSubmessageWriteKind::InfoTimestamp(
                                InfoTimestampSubmessageWrite::new(
                                    false,
                                    crate::implementation::rtps::messages::types::Time::new(
                                        timestamp.sec(),
                                        timestamp.nanosec(),
                                    ),
                                ),
                            ));
                            submessages.push(RtpsSubmessageWriteKind::Data(
                                cache_change.as_data_submessage(reader_id),
                            ))
                        }
                    }
                    _ => {
                        let gap_submessage = GapSubmessageWrite::new(
                            reader_id,
                            reader_proxy.writer().guid().entity_id(),
                            a_change_seq_num,
                            SequenceNumberSet {
                                base: a_change_seq_num,
                                set: vec![],
                            },
                        );
                        submessages.push(RtpsSubmessageWriteKind::Gap(gap_submessage));
                    }
                }

                reader_proxy.set_highest_sent_seq_num(a_change_seq_num);
            }

            let heartbeat = reader_proxy
                .heartbeat_machine()
                .submessage(writer_id, first_sn, last_sn);
            submessages.push(heartbeat);
        } else if !reader_proxy.unacked_changes() {
            // Idle
        } else if reader_proxy
            .heartbeat_machine()
            .is_time_for_heartbeat(heartbeat_period)
        {
            let heartbeat = reader_proxy
                .heartbeat_machine()
                .submessage(writer_id, first_sn, last_sn);
            submessages.push(heartbeat);
        }

        // Middle-part of the state-machine - Figure 8.19 RTPS standard
        if !reader_proxy.requested_changes().is_empty() {
            let reader_id = reader_proxy.remote_reader_guid().entity_id();

            while let Some(next_requested_change_seq_num) = reader_proxy.next_requested_change() {
                // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
                // it's not done here to avoid the change being a mutable reference
                // Also the post-condition:
                // a_change BELONGS-TO the_reader_proxy.requested_changes() ) == FALSE
                // should be full-filled by next_requested_change()
                let cache_change = reader_proxy
                    .writer()
                    .change_list()
                    .iter()
                    .find(|cc| cc.sequence_number() == next_requested_change_seq_num);
                match cache_change {
                    Some(cache_change)
                        if cache_change.sequence_number()
                            > reader_proxy.first_relevant_sample_seq_num() =>
                    {
                        if cache_change.data_value().len() > 1 {
                            Self::directly_send_data_frag(
                                reader_proxy,
                                cache_change,
                                writer_id,
                                header,
                                first_sn,
                                last_sn,
                                udp_transport_write,
                            );
                            return;
                        } else {
                            submessages
                                .push(Self::info_timestamp_submessage(cache_change.timestamp()));
                            submessages.push(RtpsSubmessageWriteKind::Data(
                                cache_change.as_data_submessage(reader_id),
                            ))
                        }
                    }
                    _ => {
                        let gap_submessage = GapSubmessageWrite::new(
                            reader_id,
                            reader_proxy.writer().guid().entity_id(),
                            next_requested_change_seq_num,
                            SequenceNumberSet {
                                base: next_requested_change_seq_num + 1,
                                set: vec![],
                            },
                        );

                        submessages.push(RtpsSubmessageWriteKind::Gap(gap_submessage));
                    }
                }
            }
            let heartbeat = reader_proxy
                .heartbeat_machine()
                .submessage(writer_id, first_sn, last_sn);
            submessages.push(heartbeat);
        }
        // Send messages only if more or equal than INFO_DST and HEARTBEAT is added
        if submessages.len() >= 2 {
            udp_transport_write
                .write(
                    RtpsMessageWrite::new(header, submessages),
                    reader_proxy.unicast_locator_list().to_vec(),
                )
                .ok();
        }
    }

    fn _gap_submessage<'a>(
        writer_id: EntityId,
        gap_sequence_number: SequenceNumber,
    ) -> RtpsSubmessageWriteKind<'a> {
        RtpsSubmessageWriteKind::Gap(GapSubmessageWrite::new(
            ENTITYID_UNKNOWN,
            writer_id,
            gap_sequence_number,
            SequenceNumberSet {
                base: gap_sequence_number,
                set: vec![],
            },
        ))
    }

    fn info_timestamp_submessage<'a>(timestamp: Time) -> RtpsSubmessageWriteKind<'a> {
        RtpsSubmessageWriteKind::InfoTimestamp(InfoTimestampSubmessageWrite::new(
            false,
            crate::implementation::rtps::messages::types::Time::new(
                timestamp.sec(),
                timestamp.nanosec(),
            ),
        ))
    }

    fn info_destination_submessage<'a>(guid_prefix: GuidPrefix) -> RtpsSubmessageWriteKind<'a> {
        RtpsSubmessageWriteKind::InfoDestination(InfoDestinationSubmessageWrite::new(guid_prefix))
    }

    #[allow(clippy::too_many_arguments)]
    fn directly_send_data_frag(
        reader_proxy: &mut WriterAssociatedReaderProxy,
        cache_change: &RtpsWriterCacheChange,
        writer_id: EntityId,
        header: RtpsMessageHeader,
        first_sn: SequenceNumber,
        last_sn: SequenceNumber,
        udp_transport_write: &ActorAddress<UdpTransportWrite>,
    ) {
        let reader_id = reader_proxy.remote_reader_guid().entity_id();
        let timestamp = cache_change.timestamp();

        let cache_change_frag = DataFragSubmessages::new(cache_change, reader_id);
        let mut data_frag_submessage_list = cache_change_frag.into_iter().peekable();
        while let Some(data_frag_submessage) = data_frag_submessage_list.next() {
            let writer_sn = data_frag_submessage.writer_sn();
            let last_fragment_num = FragmentNumber::new(
                u32::from(data_frag_submessage.fragment_starting_num())
                    + data_frag_submessage.fragments_in_submessage() as u32
                    - 1,
            );

            let info_dst =
                Self::info_destination_submessage(reader_proxy.remote_reader_guid().prefix());
            let into_timestamp = Self::info_timestamp_submessage(timestamp);
            let data_frag = RtpsSubmessageWriteKind::DataFrag(data_frag_submessage);

            let is_last_fragment = data_frag_submessage_list.peek().is_none();
            let submessages = if is_last_fragment {
                let heartbeat_frag = reader_proxy.heartbeat_frag_machine().submessage(
                    writer_id,
                    writer_sn,
                    last_fragment_num,
                );
                vec![info_dst, into_timestamp, data_frag, heartbeat_frag]
            } else {
                let heartbeat = reader_proxy
                    .heartbeat_machine()
                    .submessage(writer_id, first_sn, last_sn);
                vec![info_dst, into_timestamp, data_frag, heartbeat]
            };
            udp_transport_write
                .write(
                    RtpsMessageWrite::new(header, submessages),
                    reader_proxy.unicast_locator_list().to_vec(),
                )
                .ok();
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_matched_reader(
        &mut self,
        discovered_reader_data: DiscoveredReaderData,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        publisher_qos: PublisherQos,
        data_writer_address: ActorAddress<DdsDataWriter<RtpsStatefulWriter>>,
        publisher_address: ActorAddress<DdsPublisher>,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) {
        let is_matched_topic_name = discovered_reader_data
            .subscription_builtin_topic_data()
            .topic_name()
            == self.get_topic_name();
        let is_matched_type_name = discovered_reader_data
            .subscription_builtin_topic_data()
            .get_type_name()
            == self.get_type_name();

        if is_matched_topic_name && is_matched_type_name {
            let incompatible_qos_policy_list = get_discovered_reader_incompatible_qos_policy_list(
                &self.get_qos(),
                discovered_reader_data.subscription_builtin_topic_data(),
                &publisher_qos,
            );
            let instance_handle = discovered_reader_data.get_serialized_key().into();

            if incompatible_qos_policy_list.is_empty() {
                let unicast_locator_list = if discovered_reader_data
                    .reader_proxy()
                    .unicast_locator_list()
                    .is_empty()
                {
                    default_unicast_locator_list
                } else {
                    discovered_reader_data
                        .reader_proxy()
                        .unicast_locator_list()
                        .to_vec()
                };

                let multicast_locator_list = if discovered_reader_data
                    .reader_proxy()
                    .multicast_locator_list()
                    .is_empty()
                {
                    default_multicast_locator_list
                } else {
                    discovered_reader_data
                        .reader_proxy()
                        .multicast_locator_list()
                        .to_vec()
                };

                let proxy_reliability = match discovered_reader_data
                    .subscription_builtin_topic_data()
                    .reliability()
                    .kind
                {
                    ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
                    ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
                };

                let proxy_durability = match discovered_reader_data
                    .subscription_builtin_topic_data()
                    .durability()
                    .kind
                {
                    DurabilityQosPolicyKind::Volatile => DurabilityKind::Volatile,
                    DurabilityQosPolicyKind::TransientLocal => DurabilityKind::TransientLocal,
                };

                let first_relevant_sample_seq_num = if proxy_durability == DurabilityKind::Volatile
                {
                    self.rtps_writer
                        .change_list()
                        .iter()
                        .map(|cc| cc.sequence_number())
                        .max()
                        .unwrap_or(SequenceNumber::new(0))
                } else {
                    SequenceNumber::new(0)
                };

                let reader_proxy = RtpsReaderProxy::new(
                    discovered_reader_data.reader_proxy().remote_reader_guid(),
                    discovered_reader_data
                        .reader_proxy()
                        .remote_group_entity_id(),
                    &unicast_locator_list,
                    &multicast_locator_list,
                    discovered_reader_data.reader_proxy().expects_inline_qos(),
                    true,
                    proxy_reliability,
                    proxy_durability,
                    first_relevant_sample_seq_num,
                );

                self.matched_reader_add(reader_proxy);

                if !self.get_matched_subscriptions().contains(&instance_handle)
                    || self.get_matched_subscription_data(instance_handle).as_ref()
                        != Some(discovered_reader_data.subscription_builtin_topic_data())
                {
                    self.add_matched_publication(
                        instance_handle,
                        discovered_reader_data
                            .subscription_builtin_topic_data()
                            .clone(),
                    );
                    self.matched_subscriptions.add_matched_subscription(
                        instance_handle,
                        discovered_reader_data
                            .subscription_builtin_topic_data()
                            .clone(),
                    );
                    self.on_publication_matched(
                        data_writer_address,
                        publisher_address,
                        participant_address,
                    )
                }
            } else {
                self.incompatible_subscriptions
                    .add_offered_incompatible_qos(instance_handle, incompatible_qos_policy_list);
                self.on_offered_incompatible_qos(
                    data_writer_address,
                    publisher_address,
                    participant_address,
                );
            }
        }
    }

    pub fn remove_matched_reader(
        &mut self,
        discovered_reader_handle: InstanceHandle,
        data_writer_address: ActorAddress<DdsDataWriter<RtpsStatefulWriter>>,
        publisher_address: ActorAddress<DdsPublisher>,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) {
        if let Some(r) = self.get_matched_subscription_data(discovered_reader_handle) {
            let handle = r.key().value.into();
            self.matched_reader_remove(handle);
            self.remove_matched_subscription(handle.into());

            self.on_publication_matched(data_writer_address, publisher_address, participant_address)
        }
    }

    fn on_publication_matched(
        &mut self,
        data_writer_address: ActorAddress<DdsDataWriter<RtpsStatefulWriter>>,
        publisher_address: ActorAddress<DdsPublisher>,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) {
        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::PublicationMatched);
        if self.listener.is_some() && self.status_kind.contains(&StatusKind::PublicationMatched) {
            let listener_address = self.listener.as_ref().unwrap().address().clone();
            let writer =
                DataWriterNode::new(data_writer_address, publisher_address, participant_address);
            let status = self.get_publication_matched_status();
            listener_address
                .trigger_on_publication_matched(writer, status)
                .expect("Should not fail to send message");
        } else if publisher_address.get_listener().unwrap().is_some()
            && publisher_address
                .status_kind()
                .unwrap()
                .contains(&StatusKind::PublicationMatched)
        {
            let status = self.get_publication_matched_status();
            let listener_address = publisher_address.get_listener().unwrap().unwrap();
            let writer =
                DataWriterNode::new(data_writer_address, publisher_address, participant_address);
            listener_address
                .trigger_on_publication_matched(writer, status)
                .expect("Should not fail to send message");
        } else if participant_address.get_listener().unwrap().is_some()
            && participant_address
                .status_kind()
                .unwrap()
                .contains(&StatusKind::PublicationMatched)
        {
            let status = self.get_publication_matched_status();
            let listener_address = participant_address.get_listener().unwrap().unwrap();
            let writer =
                DataWriterNode::new(data_writer_address, publisher_address, participant_address);
            listener_address
                .trigger_on_publication_matched(writer, status)
                .expect("Should not fail to send message");
        }
    }

    fn on_offered_incompatible_qos(
        &mut self,
        data_writer_address: ActorAddress<DdsDataWriter<RtpsStatefulWriter>>,
        publisher_address: ActorAddress<DdsPublisher>,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) {
        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::OfferedIncompatibleQos);
        if self.listener.is_some()
            && self
                .status_kind
                .contains(&StatusKind::OfferedIncompatibleQos)
        {
            let status = self.get_offered_incompatible_qos_status();
            let listener_address = self.listener.as_ref().unwrap().address();
            let writer =
                DataWriterNode::new(data_writer_address, publisher_address, participant_address);
            listener_address
                .trigger_on_offered_incompatible_qos(writer, status)
                .expect("Should not fail to send message");
        } else if publisher_address.get_listener().unwrap().is_some()
            && publisher_address
                .status_kind()
                .unwrap()
                .contains(&StatusKind::OfferedIncompatibleQos)
        {
            let status = self.get_offered_incompatible_qos_status();
            let listener_address = publisher_address.get_listener().unwrap().unwrap();
            let writer =
                DataWriterNode::new(data_writer_address, publisher_address, participant_address);
            listener_address
                .trigger_on_offered_incompatible_qos(writer, status)
                .expect("Should not fail to send message");
        } else if participant_address.get_listener().unwrap().is_some()
            && participant_address
                .status_kind()
                .unwrap()
                .contains(&StatusKind::OfferedIncompatibleQos)
        {
            let status = self.get_offered_incompatible_qos_status();
            let listener_address = participant_address.get_listener().unwrap().unwrap();
            let writer =
                DataWriterNode::new(data_writer_address, publisher_address, participant_address);
            listener_address
                .trigger_on_offered_incompatible_qos(writer, status)
                .expect("Should not fail to send message");
        }
    }
}

fn get_discovered_reader_incompatible_qos_policy_list(
    writer_qos: &DataWriterQos,
    discovered_reader_data: &SubscriptionBuiltinTopicData,
    publisher_qos: &PublisherQos,
) -> Vec<QosPolicyId> {
    let mut incompatible_qos_policy_list = Vec::new();
    if &writer_qos.durability < discovered_reader_data.durability() {
        incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
    }
    if publisher_qos.presentation.access_scope < discovered_reader_data.presentation().access_scope
        || publisher_qos.presentation.coherent_access
            != discovered_reader_data.presentation().coherent_access
        || publisher_qos.presentation.ordered_access
            != discovered_reader_data.presentation().ordered_access
    {
        incompatible_qos_policy_list.push(PRESENTATION_QOS_POLICY_ID);
    }
    if &writer_qos.deadline < discovered_reader_data.deadline() {
        incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
    }
    if &writer_qos.latency_budget < discovered_reader_data.latency_budget() {
        incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
    }
    if &writer_qos.liveliness < discovered_reader_data.liveliness() {
        incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
    }
    if writer_qos.reliability.kind < discovered_reader_data.reliability().kind {
        incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
    }
    if &writer_qos.destination_order < discovered_reader_data.destination_order() {
        incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
    }
    incompatible_qos_policy_list
}

impl DdsDataWriter<RtpsStatelessWriter> {
    pub fn guid(&self) -> Guid {
        self.rtps_writer.guid()
    }

    pub fn _unicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_writer.unicast_locator_list().to_vec()
    }

    pub fn _multicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_writer.multicast_locator_list().to_vec()
    }

    pub fn _push_mode(&self) -> bool {
        self.rtps_writer.push_mode()
    }

    pub fn _heartbeat_period(&self) -> Duration {
        self.rtps_writer.heartbeat_period()
    }

    pub fn _data_max_size_serialized(&self) -> usize {
        self.rtps_writer.data_max_size_serialized()
    }

    pub fn _new_change(
        &mut self,
        kind: ChangeKind,
        data: Vec<u8>,
        inline_qos: ParameterList,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> RtpsWriterCacheChange {
        self.rtps_writer
            .new_change(kind, data, inline_qos, handle, timestamp)
    }

    pub fn _add_change(&mut self, change: RtpsWriterCacheChange) {
        self.rtps_writer.add_change(change);
    }

    pub fn _remove_change<F>(&mut self, f: F)
    where
        F: FnMut(&RtpsWriterCacheChange) -> bool,
    {
        self.rtps_writer.remove_change(f)
    }

    pub fn reader_locator_add(&mut self, a_locator: RtpsReaderLocator) {
        self.rtps_writer.reader_locator_add(a_locator)
    }

    pub fn _reader_locator_remove(&mut self, a_locator: Locator) {
        self.rtps_writer.reader_locator_remove(a_locator)
    }

    pub fn reader_locator_list(&mut self) -> Vec<WriterAssociatedReaderLocator> {
        self.rtps_writer.reader_locator_list()
    }

    pub fn write_w_timestamp(
        &mut self,
        serialized_data: Vec<u8>,
        instance_serialized_key: DdsSerializedKey,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        self.rtps_writer.write_w_timestamp(
            serialized_data,
            instance_serialized_key,
            handle,
            timestamp,
        )?;

        Ok(())
    }

    pub fn send_message(
        &mut self,
        header: RtpsMessageHeader,
        udp_transport_write: ActorAddress<UdpTransportWrite>,
    ) {
        let writer_id = self.guid().entity_id();
        for mut rl in &mut self.reader_locator_list().into_iter() {
            Self::send_message_best_effort_reader_locator(
                &mut rl,
                header,
                &udp_transport_write,
                writer_id,
            );
        }
    }

    fn send_message_best_effort_reader_locator(
        reader_locator: &mut WriterAssociatedReaderLocator,
        header: RtpsMessageHeader,
        udp_transport_write: &ActorAddress<UdpTransportWrite>,
        writer_id: EntityId,
    ) {
        let mut submessages = Vec::new();
        while let Some(change) = reader_locator.next_unsent_change() {
            // The post-condition:
            // "( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
            // should be full-filled by next_unsent_change()
            if let Some(cache_change) = change.cache_change() {
                let info_ts_submessage = Self::info_timestamp_submessage(cache_change.timestamp());
                let data_submessage = cache_change.as_data_submessage(ENTITYID_UNKNOWN);
                submessages.push(info_ts_submessage);
                submessages.push(RtpsSubmessageWriteKind::Data(data_submessage));
            } else {
                let gap_submessage = Self::gap_submessage(writer_id, change.sequence_number());
                submessages.push(gap_submessage);
            }
        }
        if !submessages.is_empty() {
            udp_transport_write
                .write(
                    RtpsMessageWrite::new(header, submessages),
                    vec![reader_locator.locator()],
                )
                .ok();
        }
    }

    fn gap_submessage<'a>(
        writer_id: EntityId,
        gap_sequence_number: SequenceNumber,
    ) -> RtpsSubmessageWriteKind<'a> {
        RtpsSubmessageWriteKind::Gap(GapSubmessageWrite::new(
            ENTITYID_UNKNOWN,
            writer_id,
            gap_sequence_number,
            SequenceNumberSet {
                base: gap_sequence_number,
                set: vec![],
            },
        ))
    }

    fn info_timestamp_submessage<'a>(timestamp: Time) -> RtpsSubmessageWriteKind<'a> {
        RtpsSubmessageWriteKind::InfoTimestamp(InfoTimestampSubmessageWrite::new(
            false,
            crate::implementation::rtps::messages::types::Time::new(
                timestamp.sec(),
                timestamp.nanosec(),
            ),
        ))
    }
}
