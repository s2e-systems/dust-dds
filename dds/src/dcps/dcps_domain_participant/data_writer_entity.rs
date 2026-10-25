use crate::{
    dcps::xtypes_glue::key_and_instance_handle::{
        KeyHolderData, KeyHolderType, get_instance_handle_from_key_holder_data,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::DataWriterQos,
        qos_policy::{
            DataRepresentationQosPolicy, HistoryQosPolicyKind, Length, XCDR_DATA_REPRESENTATION,
            XCDR2_DATA_REPRESENTATION,
        },
        status::OfferedIncompatibleQosStatus,
        time::{Duration, DurationKind, Time},
    },
    rtps::{
        message_creator::write_submessages_into_transport, reader_proxy::RtpsReaderProxy,
        stateful_writer::RtpsStatefulWriter, stateless_writer::RtpsStatelessWriter,
    },
    rtps_messages::{
        submessage_elements::SequenceNumberSet,
        submessages::{
            ack_nack::AckNackSubmessage, gap::GapSubmessage,
            info_destination::InfoDestinationSubmessage, info_timestamp::InfoTimestampSubmessage,
            nack_frag::NackFragSubmessage,
        },
        types::TIME_INVALID,
    },
    transport::{
        interface::{RtpsParticipant, Write},
        types::{
            CacheChange, ChangeKind, DurabilityKind, ENTITYID_UNKNOWN, Guid, GuidPrefix,
            ReaderProxy, ReliabilityKind, SequenceNumber, TopicKind,
        },
    },
    xtypes::{
        dynamic_type::{DynamicData, DynamicType},
        serializer::{serialize_cdr1_be, serialize_cdr1_le, serialize_cdr2_be, serialize_cdr2_le},
    },
};
use alloc::{collections::VecDeque, sync::Arc, vec::Vec};

use super::rtps_traits::RtpsWriter;

pub struct RegisteredInstanceInfo {
    pub instance_handle: InstanceHandle,
    pub last_write_time: Option<Time>,
    pub samples: VecDeque<i64>,
}

#[derive(Default)]
pub struct IncompatibleSubscriptions {
    pub incompatible_subscription_list: Vec<InstanceHandle>,
    pub offered_incompatible_qos_status: OfferedIncompatibleQosStatus,
}

pub struct DataWriterEntity<T> {
    pub instance_handle: InstanceHandle,
    pub transport_writer: T,
    pub changes: Vec<CacheChange>,
    pub topic_name: Arc<str>,
    pub enabled: bool,
    pub last_change_sequence_number: i64,
    pub qos: DataWriterQos,
    pub registered_instance_info: Vec<RegisteredInstanceInfo>,
    pub key_holder_type: KeyHolderType,
}

impl<T: RtpsWriter> DataWriterEntity<T> {
    pub fn new(
        instance_handle: InstanceHandle,
        transport_writer: T,
        topic_name: Arc<str>,
        qos: DataWriterQos,
        key_holder_type: KeyHolderType,
    ) -> Self {
        Self {
            instance_handle,
            transport_writer,
            changes: Vec::new(),
            topic_name,
            enabled: false,
            last_change_sequence_number: 0,
            qos,
            registered_instance_info: Vec::new(),
            key_holder_type,
        }
    }

    pub fn remove_change(&mut self, sequence_number: SequenceNumber) {
        self.changes
            .retain(|cc| cc.sequence_number != sequence_number);
    }

    pub fn write_w_timestamp(
        &mut self,
        sample_instance_handle: InstanceHandle,
        serialized_data: Vec<u8>,
        sample_timestamp: Time,
        now: Time,
    ) -> DdsResult<()> {
        if !self
            .registered_instance_info
            .iter()
            .any(|x| x.instance_handle == sample_instance_handle)
        {
            if self.registered_instance_info.len() < self.qos.resource_limits.max_instances {
                self.registered_instance_info.push(RegisteredInstanceInfo {
                    instance_handle: sample_instance_handle,
                    last_write_time: None,
                    samples: VecDeque::new(),
                });
            } else {
                return Err(DdsError::OutOfResources);
            }
        }

        if let Length::Limited(max_samples_per_instance) =
            self.qos.resource_limits.max_samples_per_instance
        {
            // If the history Qos guarantess that the number of samples
            // is below the limit there is no need to check
            match self.qos.history.kind {
                HistoryQosPolicyKind::KeepLast(depth)
                    if depth as i32 <= max_samples_per_instance => {}
                _ => {
                    if let Some(s) = self
                        .registered_instance_info
                        .iter()
                        .find(|x| x.instance_handle == sample_instance_handle)
                    {
                        // Only Alive changes count towards the resource limits
                        if s.samples.len() >= max_samples_per_instance as usize {
                            return Err(DdsError::OutOfResources);
                        }
                    }
                }
            }
        }

        if let Length::Limited(max_samples) = self.qos.resource_limits.max_samples {
            let total_samples = self
                .registered_instance_info
                .iter()
                .fold(0, |acc, x| acc + x.samples.len());

            if total_samples >= max_samples as usize {
                return Err(DdsError::OutOfResources);
            }
        }

        self.last_change_sequence_number += 1;
        let change = CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: self.transport_writer.guid(),
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(sample_timestamp.into()),
            instance_handle: if self.key_holder_type.is_empty() {
                None
            } else {
                Some(sample_instance_handle.into())
            },
            data_value: serialized_data.into(),
        };

        let instance_info = self
            .registered_instance_info
            .iter_mut()
            .find(|x| x.instance_handle == sample_instance_handle)
            .expect("Instance info must exist");

        match &mut instance_info.last_write_time {
            Some(last_write_time) => {
                if *last_write_time < sample_timestamp {
                    *last_write_time = sample_timestamp;
                }
            }
            None => {
                instance_info.last_write_time = Some(sample_timestamp);
            }
        }

        instance_info.samples.push_back(change.sequence_number);

        if let DurationKind::Finite(lifespan_duration) = self.qos.lifespan.duration {
            let duration_until_expired = sample_timestamp - now + lifespan_duration;
            if duration_until_expired <= Duration::new(0, 0) {
                return Ok(());
            }
        }

        self.changes.push(change);

        Ok(())
    }

    pub fn dispose_w_timestamp(
        &mut self,
        instance_handle: InstanceHandle,
        serialized_key: Vec<u8>,
        timestamp: Time,
    ) -> DdsResult<()> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let Some(instance_info) = self
            .registered_instance_info
            .iter_mut()
            .find(|x| x.instance_handle == instance_handle)
        else {
            return Err(DdsError::BadParameter);
        };

        instance_info.last_write_time = None;

        self.last_change_sequence_number += 1;
        let cache_change = CacheChange {
            kind: ChangeKind::NotAliveDisposed,
            writer_guid: self.transport_writer.guid(),
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(timestamp.into()),
            instance_handle: Some(instance_handle.into()),
            data_value: serialized_key.into(),
        };
        self.changes.push(cache_change);

        Ok(())
    }

    pub fn register_w_timestamp(
        &mut self,
        dynamic_data: &DynamicData<'static>,
        type_support: &DynamicType<'static>,
        timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let key_holder_type = KeyHolderType::new(&dynamic_data.r#type());
        let key_holder_data = KeyHolderData::from_dynamic_data(dynamic_data, &key_holder_type)?;

        if TopicKind::from(type_support) == TopicKind::NoKey {
            return Err(DdsError::IllegalOperation);
        }

        let instance_handle = get_instance_handle_from_key_holder_data(&key_holder_data)?;

        if let Some(instance_info) = self
            .registered_instance_info
            .iter_mut()
            .find(|x| x.instance_handle == instance_handle)
        {
            instance_info.last_write_time = Some(timestamp);
        } else if self.registered_instance_info.len() < self.qos.resource_limits.max_instances {
            self.registered_instance_info.push(RegisteredInstanceInfo {
                instance_handle,
                last_write_time: Some(timestamp),
                samples: VecDeque::new(),
            });
        } else {
            return Err(DdsError::OutOfResources);
        }

        Ok(Some(instance_handle))
    }

    pub fn unregister_w_timestamp(
        &mut self,
        instance_handle: InstanceHandle,
        serialized_key: Vec<u8>,
        timestamp: Time,
    ) -> DdsResult<()> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let Some(instance_info) = self
            .registered_instance_info
            .iter_mut()
            .find(|x| x.instance_handle == instance_handle)
        else {
            return Err(DdsError::BadParameter);
        };

        instance_info.last_write_time = None;

        self.last_change_sequence_number += 1;
        let kind = if self
            .qos
            .writer_data_lifecycle
            .autodispose_unregistered_instances
        {
            ChangeKind::NotAliveDisposedUnregistered
        } else {
            ChangeKind::NotAliveUnregistered
        };
        let cache_change = CacheChange {
            kind,
            writer_guid: self.transport_writer.guid(),
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(timestamp.into()),
            instance_handle: Some(instance_handle.into()),
            data_value: serialized_key.into(),
        };
        self.changes.push(cache_change);
        Ok(())
    }
}

impl DataWriterEntity<RtpsStatelessWriter> {
    pub fn write_message(&mut self, transport_participant: &mut impl RtpsParticipant) {
        if self.changes.is_empty() {
            return;
        }

        let guid_prefix = self.transport_writer.guid().prefix();
        let entity_id = self.transport_writer.guid().entity_id();
        let changes = &self.changes;
        let reader_locators = self.transport_writer.reader_locator_list();

        if reader_locators.is_empty() {
            return;
        }

        for reader_locator in reader_locators {
            while let Some(unsent_change_seq_num) =
                reader_locator.next_unsent_change(changes.iter())
            {
                let locators = [reader_locator.locator()];
                let mut transport_writer = transport_participant.writer(&locators);

                if let Some(cache_change) = changes
                    .iter()
                    .find(|cc| cc.sequence_number == unsent_change_seq_num)
                {
                    let info_ts_submessage = cache_change
                        .source_timestamp
                        .map_or(InfoTimestampSubmessage::new(true, TIME_INVALID), |t| {
                            InfoTimestampSubmessage::new(false, t.into())
                        });

                    let inline_qos = match (
                        cache_change.status_info_parameter(),
                        cache_change.key_hash_parameter(),
                    ) {
                        (Some(s), Some(k)) => &[s, k][..],
                        (Some(s), None) => &[s][..],
                        (None, Some(k)) => &[k][..],
                        (None, None) => &[],
                    };
                    let data_submessage =
                        cache_change.as_data_submessage(ENTITYID_UNKNOWN, entity_id, inline_qos);

                    write_submessages_into_transport(
                        &[&info_ts_submessage, &data_submessage],
                        guid_prefix,
                        &mut transport_writer,
                    );
                    transport_writer.flush();
                } else {
                    let gap_submessage = GapSubmessage::new(
                        ENTITYID_UNKNOWN,
                        entity_id,
                        unsent_change_seq_num,
                        SequenceNumberSet::new(unsent_change_seq_num + 1, []),
                    );
                    write_submessages_into_transport(
                        &[&gap_submessage],
                        guid_prefix,
                        &mut transport_writer,
                    );
                    transport_writer.flush();
                }
                reader_locator.set_highest_sent_change_sn(unsent_change_seq_num);
            }
        }
    }
}

impl DataWriterEntity<RtpsStatefulWriter> {
    pub fn add_matched_reader(&mut self, reader_proxy: ReaderProxy) {
        let first_relevant_sample_seq_num = match reader_proxy.durability_kind {
            DurabilityKind::Volatile => self
                .changes
                .last()
                .map(|cc| cc.sequence_number + 1)
                .unwrap_or(1),
            DurabilityKind::TransientLocal
            | DurabilityKind::Transient
            | DurabilityKind::Persistent => 1,
        };
        let highest_sent_seq_num = match reader_proxy.reliability_kind {
            ReliabilityKind::BestEffort => 0,
            ReliabilityKind::Reliable => self
                .changes
                .last()
                .map(|cc| cc.sequence_number)
                .unwrap_or(0),
        };
        let rtps_reader_proxy = RtpsReaderProxy::new(
            reader_proxy.remote_reader_guid,
            reader_proxy.remote_group_entity_id,
            &reader_proxy.unicast_locator_list,
            &reader_proxy.multicast_locator_list,
            reader_proxy.expects_inline_qos,
            true,
            reader_proxy.reliability_kind,
            first_relevant_sample_seq_num,
            reader_proxy.durability_kind,
            highest_sent_seq_num,
        );
        self.transport_writer.add_matched_reader(rtps_reader_proxy);
    }

    pub fn delete_matched_reader(&mut self, reader_guid: Guid) {
        self.transport_writer.delete_matched_reader(reader_guid);
    }

    pub fn is_change_acknowledged(&self, sequence_number: SequenceNumber) -> bool {
        self.transport_writer
            .is_change_acknowledged(sequence_number)
    }

    pub fn time_until_next_heartbeat(&self, now: Time) -> Option<Duration> {
        if self.changes.is_empty() || self.transport_writer.matched_readers().is_empty() {
            return None;
        }
        let seq_num_max = self.changes.last()?.sequence_number;
        let mut min_time: Option<Duration> = None;
        for rp in self.transport_writer.matched_readers() {
            if rp.reliability() == ReliabilityKind::Reliable
                && seq_num_max > rp.highest_acked_seq_num()
            {
                if let Some(d) = rp.time_until_heartbeat(
                    now,
                    self.transport_writer.heartbeat_period().into(),
                    Some(seq_num_max),
                ) {
                    min_time = Some(min_time.map_or(d, |min| min.min(d)));
                }
            }
        }
        min_time
    }

    pub fn on_acknack_submessage_received(
        &mut self,
        acknack_submessage: &AckNackSubmessage,
        source_guid_prefix: GuidPrefix,
        now: Time,
    ) -> Option<SequenceNumber> {
        if &self.transport_writer.guid().entity_id() == acknack_submessage.writer_id() {
            let reader_guid = Guid::new(source_guid_prefix, *acknack_submessage.reader_id());

            if let Some(reader_proxy) = self
                .transport_writer
                .matched_readers_mut()
                .iter_mut()
                .find(|x| x.remote_reader_guid() == reader_guid)
            {
                if reader_proxy.reliability() == ReliabilityKind::Reliable
                    && acknack_submessage.count() > reader_proxy.last_received_acknack_count()
                {
                    let acked_changes = acknack_submessage.reader_sn_state().base() - 1;
                    reader_proxy.acked_changes_set(acked_changes);
                    reader_proxy
                        .requested_changes_set(acknack_submessage.reader_sn_state().set(), now);

                    reader_proxy.set_last_received_acknack_count(acknack_submessage.count());

                    let is_preemptive = acknack_submessage.reader_sn_state().base() <= 0
                        || (acknack_submessage.reader_sn_state().base() == 1
                            && reader_proxy.highest_acked_seq_num() == 0
                            && acknack_submessage.reader_sn_state().set().next().is_none());
                    if is_preemptive {
                        reader_proxy.heartbeat_machine().reset_heartbeat_time();
                    }

                    return Some(acked_changes);
                }
            }
        }
        None
    }

    pub fn on_nack_frag_submessage_received(
        &mut self,
        nackfrag_submessage: &NackFragSubmessage,
        source_guid_prefix: GuidPrefix,
        now: Time,
    ) {
        let reader_guid = Guid::new(source_guid_prefix, nackfrag_submessage.reader_id());

        if let Some(reader_proxy) = self
            .transport_writer
            .matched_readers_mut()
            .iter_mut()
            .find(|x| x.remote_reader_guid() == reader_guid)
        {
            if reader_proxy.reliability() == ReliabilityKind::Reliable
                && nackfrag_submessage.count() > reader_proxy.last_received_nack_frag_count()
            {
                reader_proxy.set_last_received_nack_frag_count(nackfrag_submessage.count());
                let change_seq_num = nackfrag_submessage.writer_sn();
                let frag_iter =
                    core::iter::once(nackfrag_submessage.fragment_number_state().base())
                        .chain(nackfrag_submessage.fragment_number_state().set());
                reader_proxy.requested_fragments_set(change_seq_num, frag_iter, now);
            }
        }
    }

    pub fn send_heartbeat(&mut self, transport_participant: &mut impl RtpsParticipant, now: Time) {
        let seq_num_min = self.changes.first().map(|cc| cc.sequence_number);
        let seq_num_max = self.changes.last().map(|cc| cc.sequence_number);
        let writer_id = self.transport_writer.guid().entity_id();
        let guid_prefix = self.transport_writer.guid().prefix();
        let heartbeat_period = self.transport_writer.heartbeat_period();

        for reader_proxy in self.transport_writer.matched_readers_mut() {
            if reader_proxy.reliability() == ReliabilityKind::Reliable
                && reader_proxy.unacked_changes(seq_num_max)
                && reader_proxy
                    .heartbeat_machine()
                    .is_time_for_heartbeat(now, heartbeat_period.into())
            {
                let first_sn = seq_num_min
                    .unwrap_or(1)
                    .max(reader_proxy.first_relevant_sample_seq_num());
                let last_sn = seq_num_max.unwrap_or(0).max(first_sn - 1);
                let heartbeat_submessage = reader_proxy
                    .heartbeat_machine()
                    .generate_new_heartbeat(writer_id, first_sn, last_sn, now, false);

                let info_dst =
                    InfoDestinationSubmessage::new(reader_proxy.remote_reader_guid().prefix());

                let mut writer = transport_participant.writer(reader_proxy.unicast_locator_list());
                write_submessages_into_transport(
                    &[&info_dst, &heartbeat_submessage],
                    guid_prefix,
                    &mut writer,
                );
                writer.flush();
            }
        }
    }

    pub fn send_requested_changes(
        &mut self,
        transport_participant: &mut impl RtpsParticipant,
        now: Time,
    ) {
        let seq_num_min = self.changes.first().map(|cc| cc.sequence_number);
        let seq_num_max = self.changes.last().map(|cc| cc.sequence_number);
        let writer_id = self.transport_writer.guid().entity_id();
        let guid_prefix = self.transport_writer.guid().prefix();
        let data_max_size_serialized = self.transport_writer.data_max_size_serialized();

        let changes = &self.changes;
        let matched_readers = self.transport_writer.matched_readers_mut();

        for reader_proxy in matched_readers {
            while let Some(next_requested_change_seq_num) = reader_proxy.next_requested_change() {
                if let Some(cache_change) = changes.iter().find(|cc| {
                    cc.sequence_number == next_requested_change_seq_num
                        && next_requested_change_seq_num
                            >= reader_proxy.first_relevant_sample_seq_num()
                }) {
                    let number_of_fragments = cache_change
                        .data_value
                        .len()
                        .div_ceil(data_max_size_serialized);

                    if number_of_fragments > 1 && cache_change.kind == ChangeKind::Alive {
                        for fragment_number in 0..number_of_fragments {
                            let data_frag = cache_change.as_data_frag_submessage(
                                reader_proxy.remote_reader_guid().entity_id(),
                                writer_id,
                                data_max_size_serialized,
                                fragment_number,
                            );

                            let info_dst = InfoDestinationSubmessage::new(
                                reader_proxy.remote_reader_guid().prefix(),
                            );
                            let info_timestamp =
                                if let Some(timestamp) = cache_change.source_timestamp {
                                    InfoTimestampSubmessage::new(false, timestamp.into())
                                } else {
                                    InfoTimestampSubmessage::new(true, TIME_INVALID)
                                };
                            let heartbeat = if fragment_number == number_of_fragments - 1 {
                                let first_sn = seq_num_min
                                    .unwrap_or(1)
                                    .max(reader_proxy.first_relevant_sample_seq_num());
                                let last_sn = seq_num_max.unwrap_or(0).max(first_sn - 1);
                                Some(reader_proxy.heartbeat_machine().generate_new_heartbeat(
                                    writer_id, first_sn, last_sn, now, false,
                                ))
                            } else {
                                None
                            };
                            let mut writer =
                                transport_participant.writer(reader_proxy.unicast_locator_list());
                            if let Some(heartbeat) = &heartbeat {
                                write_submessages_into_transport(
                                    &[&info_dst, &info_timestamp, &data_frag, heartbeat],
                                    guid_prefix,
                                    &mut writer,
                                );
                            } else {
                                write_submessages_into_transport(
                                    &[&info_dst, &info_timestamp, &data_frag],
                                    guid_prefix,
                                    &mut writer,
                                );
                            }
                            writer.flush();
                        }
                    } else {
                        let info_dst = InfoDestinationSubmessage::new(
                            reader_proxy.remote_reader_guid().prefix(),
                        );

                        let info_timestamp = if let Some(timestamp) = cache_change.source_timestamp
                        {
                            InfoTimestampSubmessage::new(false, timestamp.into())
                        } else {
                            InfoTimestampSubmessage::new(true, TIME_INVALID)
                        };

                        let inline_qos = match (
                            cache_change.status_info_parameter(),
                            cache_change.key_hash_parameter(),
                        ) {
                            (Some(s), Some(k)) => &[s, k][..],
                            (Some(s), None) => &[s][..],
                            (None, Some(k)) => &[k][..],
                            (None, None) => &[],
                        };
                        let data_submessage = cache_change.as_data_submessage(
                            reader_proxy.remote_reader_guid().entity_id(),
                            writer_id,
                            inline_qos,
                        );

                        let first_sn = seq_num_min
                            .unwrap_or(1)
                            .max(reader_proxy.first_relevant_sample_seq_num());
                        let last_sn = seq_num_max.unwrap_or(0).max(first_sn - 1);
                        let heartbeat = reader_proxy
                            .heartbeat_machine()
                            .generate_new_heartbeat(writer_id, first_sn, last_sn, now, false);

                        let mut writer =
                            transport_participant.writer(reader_proxy.unicast_locator_list());
                        write_submessages_into_transport(
                            &[&info_dst, &info_timestamp, &data_submessage, &heartbeat],
                            guid_prefix,
                            &mut writer,
                        );
                        writer.flush();
                    }
                } else {
                    let info_dst =
                        InfoDestinationSubmessage::new(reader_proxy.remote_reader_guid().prefix());

                    let gap_submessage = GapSubmessage::new(
                        ENTITYID_UNKNOWN,
                        writer_id,
                        next_requested_change_seq_num,
                        SequenceNumberSet::new(next_requested_change_seq_num + 1, []),
                    );

                    let mut writer =
                        transport_participant.writer(reader_proxy.unicast_locator_list());
                    write_submessages_into_transport(
                        &[&info_dst, &gap_submessage],
                        guid_prefix,
                        &mut writer,
                    );
                    writer.flush();
                }
            }
        }
    }

    pub fn send_requested_fragments(&mut self, transport_participant: &mut impl RtpsParticipant) {
        let writer_id = self.transport_writer.guid().entity_id();
        let guid_prefix = self.transport_writer.guid().prefix();
        let data_max_size_serialized = self.transport_writer.data_max_size_serialized();

        let changes = &self.changes;
        let matched_readers = self.transport_writer.matched_readers_mut();

        for reader_proxy in matched_readers {
            while let Some((change_seq_num, request_fragment_number)) =
                reader_proxy.next_requested_fragment()
            {
                if let Some(cache_change) = changes.iter().find(|cc| {
                    cc.sequence_number == change_seq_num
                        && change_seq_num >= reader_proxy.first_relevant_sample_seq_num()
                }) {
                    let number_of_fragments = cache_change
                        .data_value
                        .len()
                        .div_ceil(data_max_size_serialized);
                    let request_fragment_number = request_fragment_number as usize;
                    if (1..=number_of_fragments).contains(&request_fragment_number)
                        && cache_change.kind == ChangeKind::Alive
                    {
                        let reader_id = reader_proxy.remote_reader_guid().entity_id();
                        let data_frag = cache_change.as_data_frag_submessage(
                            reader_id,
                            writer_id,
                            data_max_size_serialized,
                            request_fragment_number - 1,
                        );

                        let info_dst = InfoDestinationSubmessage::new(
                            reader_proxy.remote_reader_guid().prefix(),
                        );
                        let info_timestamp = if let Some(timestamp) = cache_change.source_timestamp
                        {
                            InfoTimestampSubmessage::new(false, timestamp.into())
                        } else {
                            InfoTimestampSubmessage::new(true, TIME_INVALID)
                        };

                        let mut writer =
                            transport_participant.writer(reader_proxy.unicast_locator_list());
                        write_submessages_into_transport(
                            &[&info_dst, &info_timestamp, &data_frag],
                            guid_prefix,
                            &mut writer,
                        );
                        writer.flush();
                    }
                } else {
                    let info_dst =
                        InfoDestinationSubmessage::new(reader_proxy.remote_reader_guid().prefix());
                    let gap_submessage = GapSubmessage::new(
                        ENTITYID_UNKNOWN,
                        writer_id,
                        change_seq_num,
                        SequenceNumberSet::new(change_seq_num + 1, []),
                    );

                    let mut writer =
                        transport_participant.writer(reader_proxy.unicast_locator_list());
                    write_submessages_into_transport(
                        &[&info_dst, &gap_submessage],
                        guid_prefix,
                        &mut writer,
                    );
                    writer.flush();
                }
            }
        }
    }

    pub fn write_message(&mut self, transport_participant: &mut impl RtpsParticipant, now: Time) {
        if self.changes.is_empty() || self.transport_writer.matched_readers().is_empty() {
            return;
        }

        let seq_num_min = self.changes.first().map(|cc| cc.sequence_number);
        let seq_num_max = self.changes.last().map(|cc| cc.sequence_number);
        let writer_id = self.transport_writer.guid().entity_id();
        let guid_prefix = self.transport_writer.guid().prefix();
        let data_max_size_serialized = self.transport_writer.data_max_size_serialized();

        let changes = &self.changes;
        let matched_readers = self.transport_writer.matched_readers_mut();

        for reader_proxy in matched_readers {
            match reader_proxy.reliability() {
                ReliabilityKind::BestEffort => {
                    while let Some(next_unsent_change_seq_num) =
                        reader_proxy.next_unsent_change(changes)
                    {
                        if let Some(cache_change) = changes.iter().find(|cc| {
                            cc.sequence_number == next_unsent_change_seq_num
                                && next_unsent_change_seq_num
                                    >= reader_proxy.first_relevant_sample_seq_num()
                        }) {
                            let number_of_fragments = cache_change
                                .data_value
                                .len()
                                .div_ceil(data_max_size_serialized);

                            let info_dst = InfoDestinationSubmessage::new(
                                reader_proxy.remote_reader_guid().prefix(),
                            );

                            let info_timestamp =
                                if let Some(timestamp) = cache_change.source_timestamp {
                                    InfoTimestampSubmessage::new(false, timestamp.into())
                                } else {
                                    InfoTimestampSubmessage::new(true, TIME_INVALID)
                                };

                            if number_of_fragments > 1 {
                                for fragment_number in 0..number_of_fragments {
                                    let reader_id = reader_proxy.remote_reader_guid().entity_id();

                                    let data_frag = cache_change.as_data_frag_submessage(
                                        reader_id,
                                        writer_id,
                                        data_max_size_serialized,
                                        fragment_number,
                                    );
                                    let mut writer = transport_participant
                                        .writer(reader_proxy.unicast_locator_list());
                                    write_submessages_into_transport(
                                        &[&info_dst, &info_timestamp, &data_frag],
                                        guid_prefix,
                                        &mut writer,
                                    );
                                    writer.flush();
                                }
                            } else {
                                let inline_qos = match (
                                    cache_change.status_info_parameter(),
                                    cache_change.key_hash_parameter(),
                                ) {
                                    (Some(s), Some(k)) => &[s, k][..],
                                    (Some(s), None) => &[s][..],
                                    (None, Some(k)) => &[k][..],
                                    (None, None) => &[],
                                };
                                let data_submessage = cache_change.as_data_submessage(
                                    reader_proxy.remote_reader_guid().entity_id(),
                                    writer_id,
                                    inline_qos,
                                );

                                let mut writer = transport_participant
                                    .writer(reader_proxy.unicast_locator_list());
                                write_submessages_into_transport(
                                    &[&info_dst, &info_timestamp, &data_submessage],
                                    guid_prefix,
                                    &mut writer,
                                );
                                writer.flush();
                            }
                        }

                        reader_proxy.set_highest_sent_seq_num(next_unsent_change_seq_num);
                    }
                }
                ReliabilityKind::Reliable => {
                    if reader_proxy.unsent_changes(changes) {
                        while let Some(next_unsent_change_seq_num) =
                            reader_proxy.next_unsent_change(changes)
                        {
                            if next_unsent_change_seq_num > reader_proxy.highest_sent_seq_num() + 1
                            {
                                let gap_start_sequence_number =
                                    reader_proxy.highest_sent_seq_num() + 1;
                                let gap_end_sequence_number = next_unsent_change_seq_num - 1;
                                let gap_submessage = GapSubmessage::new(
                                    reader_proxy.remote_reader_guid().entity_id(),
                                    writer_id,
                                    gap_start_sequence_number,
                                    SequenceNumberSet::new(gap_end_sequence_number + 1, []),
                                );
                                let info_dst = InfoDestinationSubmessage::new(
                                    reader_proxy.remote_reader_guid().prefix(),
                                );
                                {
                                    let mut writer = transport_participant
                                        .writer(reader_proxy.unicast_locator_list());
                                    write_submessages_into_transport(
                                        &[&info_dst, &gap_submessage],
                                        guid_prefix,
                                        &mut writer,
                                    );
                                    writer.flush();
                                }
                                reader_proxy.set_highest_sent_seq_num(gap_end_sequence_number);
                            }

                            if let Some(cache_change) = changes.iter().find(|cc| {
                                cc.sequence_number == next_unsent_change_seq_num
                                    && next_unsent_change_seq_num
                                        >= reader_proxy.first_relevant_sample_seq_num()
                            }) {
                                let number_of_fragments = cache_change
                                    .data_value
                                    .len()
                                    .div_ceil(data_max_size_serialized);

                                if number_of_fragments > 1 && cache_change.kind == ChangeKind::Alive
                                {
                                    for fragment_number in 0..number_of_fragments {
                                        let reader_id =
                                            reader_proxy.remote_reader_guid().entity_id();

                                        let data_frag = cache_change.as_data_frag_submessage(
                                            reader_id,
                                            writer_id,
                                            data_max_size_serialized,
                                            fragment_number,
                                        );

                                        let info_dst = InfoDestinationSubmessage::new(
                                            reader_proxy.remote_reader_guid().prefix(),
                                        );
                                        let info_timestamp = if let Some(timestamp) =
                                            cache_change.source_timestamp
                                        {
                                            InfoTimestampSubmessage::new(false, timestamp.into())
                                        } else {
                                            InfoTimestampSubmessage::new(true, TIME_INVALID)
                                        };

                                        let heartbeat = if fragment_number
                                            == number_of_fragments - 1
                                        {
                                            let first_sn = seq_num_min
                                                .unwrap_or(1)
                                                .max(reader_proxy.first_relevant_sample_seq_num());
                                            let last_sn =
                                                seq_num_max.unwrap_or(0).max(first_sn - 1);
                                            Some(
                                                reader_proxy
                                                    .heartbeat_machine()
                                                    .generate_new_heartbeat(
                                                        writer_id, first_sn, last_sn, now, false,
                                                    ),
                                            )
                                        } else {
                                            None
                                        };
                                        let mut writer = transport_participant
                                            .writer(reader_proxy.unicast_locator_list());
                                        if let Some(heartbeat) = &heartbeat {
                                            write_submessages_into_transport(
                                                &[
                                                    &info_dst,
                                                    &info_timestamp,
                                                    &data_frag,
                                                    heartbeat,
                                                ],
                                                guid_prefix,
                                                &mut writer,
                                            );
                                        } else {
                                            write_submessages_into_transport(
                                                &[&info_dst, &info_timestamp, &data_frag],
                                                guid_prefix,
                                                &mut writer,
                                            );
                                        }
                                        writer.flush();
                                    }
                                } else {
                                    let info_dst = InfoDestinationSubmessage::new(
                                        reader_proxy.remote_reader_guid().prefix(),
                                    );
                                    let info_timestamp =
                                        if let Some(timestamp) = cache_change.source_timestamp {
                                            InfoTimestampSubmessage::new(false, timestamp.into())
                                        } else {
                                            InfoTimestampSubmessage::new(true, TIME_INVALID)
                                        };

                                    let inline_qos = match (
                                        cache_change.status_info_parameter(),
                                        cache_change.key_hash_parameter(),
                                    ) {
                                        (Some(s), Some(k)) => &[s, k][..],
                                        (Some(s), None) => &[s][..],
                                        (None, Some(k)) => &[k][..],
                                        (None, None) => &[],
                                    };
                                    let data_submessage = cache_change.as_data_submessage(
                                        reader_proxy.remote_reader_guid().entity_id(),
                                        writer_id,
                                        inline_qos,
                                    );

                                    let first_sn = seq_num_min
                                        .unwrap_or(1)
                                        .max(reader_proxy.first_relevant_sample_seq_num());
                                    let last_sn = seq_num_max.unwrap_or(0).max(first_sn - 1);
                                    let heartbeat =
                                        reader_proxy.heartbeat_machine().generate_new_heartbeat(
                                            writer_id, first_sn, last_sn, now, false,
                                        );

                                    let mut writer = transport_participant
                                        .writer(reader_proxy.unicast_locator_list());
                                    write_submessages_into_transport(
                                        &[&info_dst, &info_timestamp, &data_submessage, &heartbeat],
                                        guid_prefix,
                                        &mut writer,
                                    );
                                    writer.flush();
                                }
                            } else {
                                let info_dst = InfoDestinationSubmessage::new(
                                    reader_proxy.remote_reader_guid().prefix(),
                                );

                                let gap_submessage = GapSubmessage::new(
                                    ENTITYID_UNKNOWN,
                                    writer_id,
                                    next_unsent_change_seq_num,
                                    SequenceNumberSet::new(next_unsent_change_seq_num + 1, []),
                                );

                                let mut writer = transport_participant
                                    .writer(reader_proxy.unicast_locator_list());
                                write_submessages_into_transport(
                                    &[&info_dst, &gap_submessage],
                                    guid_prefix,
                                    &mut writer,
                                );
                                writer.flush();
                            }

                            reader_proxy.set_highest_sent_seq_num(next_unsent_change_seq_num);
                        }
                    }
                }
            }
        }
    }
}

pub fn serialize<'a>(
    dynamic_data: &DynamicData<'a>,
    representation: &DataRepresentationQosPolicy,
) -> DdsResult<Vec<u8>> {
    Ok(
        if representation.value.is_empty() || representation.value[0] == XCDR_DATA_REPRESENTATION {
            if cfg!(target_endian = "big") {
                serialize_cdr1_be(dynamic_data)
            } else {
                serialize_cdr1_le(dynamic_data)
            }
        } else if representation.value[0] == XCDR2_DATA_REPRESENTATION {
            if cfg!(target_endian = "big") {
                serialize_cdr2_be(dynamic_data)
            } else {
                serialize_cdr2_le(dynamic_data)
            }
        } else {
            panic!("Invalid data representation")
        }?,
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use crate::{
        dcps::xtypes_glue::key_and_instance_handle::KeyHolderType,
        infrastructure::{instance::InstanceHandle, qos::DataWriterQos, time::Time},
        rtps_messages::overall_structure::{RtpsMessageRead, RtpsSubmessageReadKind},
        transport::{
            interface::{RtpsParticipant, Write},
            types::{
                CacheChange, ChangeKind, DurabilityKind, ENTITYID_UNKNOWN, EntityId, Guid,
                LOCATOR_INVALID, Locator, ReaderProxy, ReliabilityKind,
            },
        },
    };

    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    enum RecordedSubmessage {
        DataFrag,
        Data,
        Gap,
        Heartbeat,
        Other,
    }

    struct MockTransportWriter<'a> {
        submessages: &'a Mutex<Vec<RecordedSubmessage>>,
        buffer: Vec<u8>,
    }

    impl<'a> Write for MockTransportWriter<'a> {
        fn write(&mut self, buf: &[u8]) {
            self.buffer.extend_from_slice(buf);
        }
        fn flush(&mut self) {
            if self.buffer.is_empty() {
                return;
            }
            let message = RtpsMessageRead::try_from(self.buffer.as_slice()).unwrap();
            for submessage in message.submessages() {
                let rec = match submessage {
                    RtpsSubmessageReadKind::DataFrag(_) => RecordedSubmessage::DataFrag,
                    RtpsSubmessageReadKind::Data(_) => RecordedSubmessage::Data,
                    RtpsSubmessageReadKind::Gap(_) => RecordedSubmessage::Gap,
                    RtpsSubmessageReadKind::Heartbeat(_) => RecordedSubmessage::Heartbeat,
                    _ => RecordedSubmessage::Other,
                };
                self.submessages.lock().unwrap().push(rec);
            }
            self.buffer.clear();
        }
    }

    #[derive(Default)]
    struct MockTransport {
        submessages: Mutex<Vec<RecordedSubmessage>>,
    }

    impl RtpsParticipant for MockTransport {
        type Writer<'a> = MockTransportWriter<'a>;

        fn guid(&self) -> Guid {
            Guid::new([1; 12], ENTITYID_UNKNOWN)
        }

        fn default_unicast_locator_list(&self) -> &[Locator] {
            &[]
        }

        fn metatraffic_unicast_locator_list(&self) -> &[Locator] {
            &[]
        }

        fn metatraffic_multicast_locator_list(&self) -> &[Locator] {
            &[]
        }

        fn default_multicast_locator_list(&self) -> &[Locator] {
            &[]
        }

        fn fragment_size(&self) -> usize {
            500
        }

        fn writer<'a>(&'a mut self, _locator: &'a [Locator]) -> Self::Writer<'a> {
            MockTransportWriter {
                submessages: &self.submessages,
                buffer: Vec::new(),
            }
        }

        async fn read(&mut self) -> &[u8] {
            &[]
        }
    }

    fn create_stateful_data_writer(
        guid: Guid,
        data_max_size_serialized: usize,
    ) -> DataWriterEntity<RtpsStatefulWriter> {
        let transport_writer = RtpsStatefulWriter::new(guid, data_max_size_serialized);
        DataWriterEntity::new(
            InstanceHandle::default(),
            transport_writer,
            "test_topic".into(),
            DataWriterQos::default(),
            KeyHolderType::default(),
        )
    }

    fn create_stateless_data_writer(guid: Guid) -> DataWriterEntity<RtpsStatelessWriter> {
        let transport_writer = RtpsStatelessWriter::new(guid);
        DataWriterEntity::new(
            InstanceHandle::default(),
            transport_writer,
            "test_topic".into(),
            DataWriterQos::default(),
            KeyHolderType::default(),
        )
    }

    #[test]
    fn test_stateless_writer_write_message() {
        let guid = Guid::new([1; 12], EntityId::new([1; 3], 1));
        let mut dw = create_stateless_data_writer(guid);
        dw.transport_writer.reader_locator_add(LOCATOR_INVALID);

        dw.changes.push(CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: guid,
            sequence_number: 1,
            source_timestamp: None,
            instance_handle: Some([10; 16]),
            data_value: vec![1, 2, 3].into(),
        });

        let mut transport = MockTransport::default();
        dw.write_message(&mut transport);

        let submsgs = transport.submessages.lock().unwrap();
        let data_count = submsgs
            .iter()
            .filter(|s| matches!(s, RecordedSubmessage::Data))
            .count();
        assert_eq!(data_count, 1);
    }

    #[test]
    fn test_stateful_writer_all_fragments_sent() {
        let data_max_size_serialized = 500;
        let guid = Guid::new([1; 12], EntityId::new([1; 3], 1));
        let mut dw = create_stateful_data_writer(guid, data_max_size_serialized);

        let remote_reader_guid = Guid::new([2; 12], EntityId::new([2; 3], 2));
        dw.add_matched_reader(ReaderProxy {
            remote_reader_guid,
            remote_group_entity_id: ENTITYID_UNKNOWN,
            reliability_kind: ReliabilityKind::Reliable,
            durability_kind: DurabilityKind::Volatile,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            expects_inline_qos: false,
        });

        let mut transport = MockTransport::default();
        dw.changes.push(CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: guid,
            sequence_number: 1,
            source_timestamp: None,
            instance_handle: Some([10; 16]),
            data_value: vec![8; 1300].into(),
        });

        dw.write_message(&mut transport, Time::new(1, 0));
        let total_fragments_sent = transport
            .submessages
            .lock()
            .unwrap()
            .iter()
            .filter(|s| matches!(s, RecordedSubmessage::DataFrag))
            .count();
        assert_eq!(total_fragments_sent, 3);
    }

    #[test]
    fn test_stateful_writer_best_effort_reader_no_gap_submessage_on_non_contiguous_sequence_numbers()
     {
        let data_max_size_serialized = 500;
        let writer_id = EntityId::new([1; 3], 1);
        let guid = Guid::new([1; 12], writer_id);
        let mut dw = create_stateful_data_writer(guid, data_max_size_serialized);

        let remote_reader_id = EntityId::new([2; 3], 2);
        let remote_reader_guid = Guid::new([2; 12], remote_reader_id);
        dw.add_matched_reader(ReaderProxy {
            remote_reader_guid,
            remote_group_entity_id: ENTITYID_UNKNOWN,
            reliability_kind: ReliabilityKind::BestEffort,
            durability_kind: DurabilityKind::Volatile,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            expects_inline_qos: false,
        });

        dw.changes.push(CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: guid,
            sequence_number: 2,
            source_timestamp: None,
            instance_handle: Some([10; 16]),
            data_value: vec![1, 2, 3].into(),
        });
        dw.changes.push(CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: guid,
            sequence_number: 5,
            source_timestamp: None,
            instance_handle: Some([10; 16]),
            data_value: vec![4, 5, 6].into(),
        });

        let mut transport = MockTransport::default();
        dw.write_message(&mut transport, Time::new(1, 0));

        let submsgs = transport.submessages.lock().unwrap();
        let gap_count = submsgs
            .iter()
            .filter(|s| matches!(s, RecordedSubmessage::Gap))
            .count();
        let data_count = submsgs
            .iter()
            .filter(|s| matches!(s, RecordedSubmessage::Data))
            .count();

        assert_eq!(
            gap_count, 0,
            "Best-effort reader proxy should not receive any GAP submessages"
        );
        assert_eq!(data_count, 2);
    }

    #[test]
    fn test_stateful_writer_reliable_reader_does_not_send_data_immediately_for_historical_data() {
        let data_max_size_serialized = 500;
        let writer_id = EntityId::new([1; 3], 1);
        let guid = Guid::new([1; 12], writer_id);
        let mut dw = create_stateful_data_writer(guid, data_max_size_serialized);

        dw.changes.push(CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: guid,
            sequence_number: 1,
            source_timestamp: None,
            instance_handle: Some([10; 16]),
            data_value: vec![1, 2, 3].into(),
        });

        let remote_reader_id = EntityId::new([2; 3], 2);
        let remote_reader_guid_prefix = [2; 12];
        let remote_reader_guid = Guid::new(remote_reader_guid_prefix, remote_reader_id);
        dw.add_matched_reader(ReaderProxy {
            remote_reader_guid,
            remote_group_entity_id: ENTITYID_UNKNOWN,
            reliability_kind: ReliabilityKind::Reliable,
            durability_kind: DurabilityKind::TransientLocal,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            expects_inline_qos: false,
        });

        let mut transport = MockTransport::default();

        dw.write_message(&mut transport, Time::new(1, 0));

        let submsgs = transport.submessages.lock().unwrap();
        let heartbeat_count = submsgs
            .iter()
            .filter(|s| matches!(s, RecordedSubmessage::Heartbeat))
            .count();
        let data_count = submsgs
            .iter()
            .filter(|s| matches!(s, RecordedSubmessage::Data))
            .count();

        assert_eq!(heartbeat_count, 0);
        assert_eq!(data_count, 0);
    }

    #[test]
    fn test_stateful_writer_best_effort_reader_receives_historical_data_immediately() {
        let data_max_size_serialized = 500;
        let writer_id = EntityId::new([1; 3], 1);
        let guid = Guid::new([1; 12], writer_id);
        let mut dw = create_stateful_data_writer(guid, data_max_size_serialized);

        dw.changes.push(CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: guid,
            sequence_number: 1,
            source_timestamp: None,
            instance_handle: Some([10; 16]),
            data_value: vec![1, 2, 3].into(),
        });

        let remote_reader_id = EntityId::new([2; 3], 2);
        let remote_reader_guid = Guid::new([2; 12], remote_reader_id);
        dw.add_matched_reader(ReaderProxy {
            remote_reader_guid,
            remote_group_entity_id: ENTITYID_UNKNOWN,
            reliability_kind: ReliabilityKind::BestEffort,
            durability_kind: DurabilityKind::TransientLocal,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            expects_inline_qos: false,
        });

        let mut transport = MockTransport::default();
        dw.write_message(&mut transport, Time::new(1, 0));

        let submsgs = transport.submessages.lock().unwrap();
        let data_count = submsgs
            .iter()
            .filter(|s| matches!(s, RecordedSubmessage::Data))
            .count();
        assert_eq!(data_count, 1);
    }

    #[test]
    fn test_single_fragment_sent_after_acknack_frag() {
        let data_max_size_serialized = 500;
        let writer_id = EntityId::new([1; 3], 1);
        let guid = Guid::new([1; 12], writer_id);
        let mut writer = create_stateful_data_writer(guid, data_max_size_serialized);

        let remote_reader_id = EntityId::new([2; 3], 2);
        let remote_reader_guid_prefix = [2; 12];
        let remote_reader_guid = Guid::new(remote_reader_guid_prefix, remote_reader_id);
        writer.add_matched_reader(ReaderProxy {
            remote_reader_guid,
            remote_group_entity_id: ENTITYID_UNKNOWN,
            reliability_kind: ReliabilityKind::Reliable,
            durability_kind: DurabilityKind::Volatile,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            expects_inline_qos: false,
        });
        writer.changes.push(CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: guid,
            sequence_number: 1,
            source_timestamp: None,
            instance_handle: Some([10; 16]),
            data_value: vec![8; 1300].into(),
        });
        writer.transport_writer.matched_readers_mut()[0].set_highest_sent_seq_num(1);

        let nackfrag_submessage = NackFragSubmessage::new(
            remote_reader_id,
            writer_id,
            1,
            crate::rtps_messages::submessage_elements::FragmentNumberSet::new(1, []),
            1,
        );
        writer.on_nack_frag_submessage_received(
            &nackfrag_submessage,
            remote_reader_guid_prefix,
            Time::new(1, 0),
        );
        assert_eq!(
            writer
                .transport_writer
                .time_until_next_repair(Time::new(1, 0)),
            Some(crate::infrastructure::time::Duration::new(0, 0))
        );
        assert_eq!(
            writer.transport_writer.matched_readers_mut()[0].next_requested_fragment(),
            Some((1, 1))
        );
    }
}
