use crate::implementation::data_representation_builtin_endpoints::{
    discovered_reader_data::ReaderProxy, discovered_writer_data::WriterProxy,
};

use super::{
    behavior_types::Duration,
    cache_change::RtpsCacheChange,
    message_sender::MessageSender,
    messages::{
        submessage_elements::{SequenceNumberSet, SerializedDataFragment},
        submessages::{
            ack_nack::AckNackSubmessage, data_frag::DataFragSubmessage, gap::GapSubmessage,
            info_destination::InfoDestinationSubmessage, info_timestamp::InfoTimestampSubmessage,
            nack_frag::NackFragSubmessage,
        },
        types::TIME_INVALID,
    },
    reader_proxy::RtpsReaderProxy,
    types::{
        ChangeKind, DurabilityKind, EntityId, Guid, GuidPrefix, ReliabilityKind, SequenceNumber,
        ENTITYID_UNKNOWN,
    },
};

pub trait WriterHistoryCache {
    fn add_change(&mut self, cache_change: RtpsCacheChange);

    fn remove_change(&mut self, sequence_number: SequenceNumber);
}

pub trait TransportWriter: Send + Sync {
    fn get_history_cache(&mut self) -> &mut dyn WriterHistoryCache;

    fn add_matched_reader(
        &mut self,
        reader_proxy: ReaderProxy,
        reliability_kind: ReliabilityKind,
        durability_kind: DurabilityKind,
    );

    fn delete_matched_reader(&mut self, reader_guid: Guid);

    fn are_all_changes_acknowledged(&self) -> bool;

    fn writer_proxy(&self) -> WriterProxy;
}

pub struct RtpsStatefulWriter {
    guid: Guid,
    changes: Vec<RtpsCacheChange>,
    matched_readers: Vec<RtpsReaderProxy>,
    message_sender: MessageSender,
    heartbeat_period: Duration,
    data_max_size_serialized: usize,
}

impl RtpsStatefulWriter {
    pub fn new(guid: Guid, message_sender: MessageSender) -> Self {
        Self {
            guid,
            changes: Vec::new(),
            matched_readers: Vec::new(),
            message_sender,
            heartbeat_period: Duration::new(0, 200_000_000).into(),
            data_max_size_serialized: 1344,
        }
    }

    pub fn guid(&self) -> Guid {
        self.guid
    }

    pub fn send_message(&mut self) {
        for reader_proxy in &mut self.matched_readers {
            match reader_proxy.reliability() {
                ReliabilityKind::BestEffort => send_message_to_reader_proxy_best_effort(
                    reader_proxy,
                    self.guid.entity_id(),
                    &self.changes,
                    self.data_max_size_serialized,
                    &self.message_sender,
                ),
                ReliabilityKind::Reliable => send_message_to_reader_proxy_reliable(
                    reader_proxy,
                    self.guid.entity_id(),
                    &self.changes,
                    self.changes.iter().map(|cc| cc.sequence_number()).min(),
                    self.changes.iter().map(|cc| cc.sequence_number()).max(),
                    self.data_max_size_serialized,
                    self.heartbeat_period.into(),
                    &self.message_sender,
                ),
            }
        }
    }

    pub fn on_acknack_submessage_received(
        &mut self,
        acknack_submessage: &AckNackSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        if &self.guid.entity_id() == acknack_submessage.writer_id() {
            let reader_guid = Guid::new(source_guid_prefix, *acknack_submessage.reader_id());

            if let Some(reader_proxy) = self
                .matched_readers
                .iter_mut()
                .find(|x| x.remote_reader_guid() == reader_guid)
            {
                if reader_proxy.reliability() == ReliabilityKind::Reliable {
                    if acknack_submessage.count() > reader_proxy.last_received_acknack_count() {
                        reader_proxy
                            .acked_changes_set(acknack_submessage.reader_sn_state().base() - 1);
                        reader_proxy
                            .requested_changes_set(acknack_submessage.reader_sn_state().set());

                        reader_proxy.set_last_received_acknack_count(acknack_submessage.count());

                        send_message_to_reader_proxy_reliable(
                            reader_proxy,
                            self.guid.entity_id(),
                            &self.changes,
                            self.changes.iter().map(|cc| cc.sequence_number()).min(),
                            self.changes.iter().map(|cc| cc.sequence_number()).max(),
                            self.data_max_size_serialized,
                            self.heartbeat_period.into(),
                            &self.message_sender,
                        );
                    }
                }
            }
        }
    }

    pub fn on_nack_frag_submessage_received(
        &mut self,
        nackfrag_submessage: &NackFragSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        let reader_guid = Guid::new(source_guid_prefix, nackfrag_submessage.reader_id());

        if let Some(reader_proxy) = self
            .matched_readers
            .iter_mut()
            .find(|x| x.remote_reader_guid() == reader_guid)
        {
            if reader_proxy.reliability() == ReliabilityKind::Reliable {
                if nackfrag_submessage.count() > reader_proxy.last_received_nack_frag_count() {
                    reader_proxy
                        .requested_changes_set(std::iter::once(nackfrag_submessage.writer_sn()));
                    reader_proxy.set_last_received_nack_frag_count(nackfrag_submessage.count());

                    send_message_to_reader_proxy_reliable(
                        reader_proxy,
                        self.guid.entity_id(),
                        &self.changes,
                        self.changes.iter().map(|cc| cc.sequence_number()).min(),
                        self.changes.iter().map(|cc| cc.sequence_number()).max(),
                        self.data_max_size_serialized,
                        self.heartbeat_period.into(),
                        &self.message_sender,
                    );
                }
            }
        }
    }
}

fn send_message_to_reader_proxy_best_effort(
    reader_proxy: &mut RtpsReaderProxy,
    writer_id: EntityId,
    changes: &[RtpsCacheChange],
    data_max_size_serialized: usize,
    message_sender: &MessageSender,
) {
    // a_change_seq_num := the_reader_proxy.next_unsent_change();
    // if ( a_change_seq_num > the_reader_proxy.higuest_sent_seq_num +1 ) {
    //      GAP = new GAP(the_reader_locator.higuest_sent_seq_num + 1, a_change_seq_num -1);
    //      GAP.readerId := ENTITYID_UNKNOWN;
    //      GAP.filteredCount := 0;
    //      send GAP;
    // }
    // a_change := the_writer.writer_cache.get_change(a_change_seq_num );
    // if ( DDS_FILTER(the_reader_proxy, a_change) ) {
    //      DATA = new DATA(a_change);
    //      IF (the_reader_proxy.expectsInlineQos) {
    //          DATA.inlineQos := the_rtps_writer.related_dds_writer.qos;
    //          DATA.inlineQos += a_change.inlineQos;
    //      }
    //      DATA.readerId := ENTITYID_UNKNOWN;
    //      send DATA;
    // }
    // else {
    //      GAP = new GAP(a_change.sequenceNumber);
    //      GAP.readerId := ENTITYID_UNKNOWN;
    //      GAP.filteredCount := 1;
    //      send GAP;
    // }
    // the_reader_proxy.higuest_sent_seq_num := a_change_seq_num;
    while let Some(next_unsent_change_seq_num) = reader_proxy.next_unsent_change(changes.iter()) {
        if next_unsent_change_seq_num > reader_proxy.highest_sent_seq_num() + 1 {
            let gap_start_sequence_number = reader_proxy.highest_sent_seq_num() + 1;
            let gap_end_sequence_number = next_unsent_change_seq_num - 1;
            let gap_submessage = Box::new(GapSubmessage::new(
                reader_proxy.remote_reader_guid().entity_id(),
                writer_id,
                gap_start_sequence_number,
                SequenceNumberSet::new(gap_end_sequence_number + 1, []),
            ));

            message_sender.write_message(
                &[gap_submessage],
                reader_proxy.unicast_locator_list().to_vec(),
            );

            reader_proxy.set_highest_sent_seq_num(next_unsent_change_seq_num);
        } else if let Some(cache_change) = changes
            .iter()
            .find(|cc| cc.sequence_number() == next_unsent_change_seq_num)
        {
            let number_of_fragments = cache_change
                .data_value()
                .len()
                .div_ceil(data_max_size_serialized);

            // Either send a DATAFRAG submessages or send a single DATA submessage
            if number_of_fragments > 1 {
                for frag_index in 0..number_of_fragments {
                    let info_dst = Box::new(InfoDestinationSubmessage::new(
                        reader_proxy.remote_reader_guid().prefix(),
                    ));

                    let info_timestamp = if let Some(timestamp) = cache_change.source_timestamp() {
                        Box::new(InfoTimestampSubmessage::new(false, timestamp))
                    } else {
                        Box::new(InfoTimestampSubmessage::new(true, TIME_INVALID))
                    };

                    let inline_qos_flag = true;
                    let key_flag = match cache_change.kind() {
                        ChangeKind::Alive => false,
                        ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => true,
                        _ => todo!(),
                    };
                    let non_standard_payload_flag = false;
                    let reader_id = reader_proxy.remote_reader_guid().entity_id();
                    let writer_sn = cache_change.sequence_number();
                    let fragment_starting_num = (frag_index + 1) as u32;
                    let fragments_in_submessage = 1;
                    let fragment_size = data_max_size_serialized as u16;
                    let data_size = cache_change.data_value().len() as u32;
                    let inline_qos = cache_change.inline_qos().clone();

                    let start = frag_index * data_max_size_serialized;
                    let end = std::cmp::min(
                        (frag_index + 1) * data_max_size_serialized,
                        cache_change.data_value().len(),
                    );

                    let serialized_payload =
                        SerializedDataFragment::new(cache_change.data_value().clone(), start..end);

                    let data_frag = Box::new(DataFragSubmessage::new(
                        inline_qos_flag,
                        non_standard_payload_flag,
                        key_flag,
                        reader_id,
                        writer_id,
                        writer_sn,
                        fragment_starting_num,
                        fragments_in_submessage,
                        fragment_size,
                        data_size,
                        inline_qos,
                        serialized_payload,
                    ));

                    message_sender.write_message(
                        &[info_dst, info_timestamp, data_frag],
                        reader_proxy.unicast_locator_list().to_vec(),
                    );
                }
            } else {
                let info_dst = Box::new(InfoDestinationSubmessage::new(
                    reader_proxy.remote_reader_guid().prefix(),
                ));

                let info_timestamp = if let Some(timestamp) = cache_change.source_timestamp() {
                    Box::new(InfoTimestampSubmessage::new(false, timestamp))
                } else {
                    Box::new(InfoTimestampSubmessage::new(true, TIME_INVALID))
                };

                let data_submessage =
                    Box::new(cache_change.as_data_submessage(
                        reader_proxy.remote_reader_guid().entity_id(),
                        writer_id,
                    ));

                message_sender.write_message(
                    &[info_dst, info_timestamp, data_submessage],
                    reader_proxy.unicast_locator_list().to_vec(),
                );
            }
        } else {
            message_sender.write_message(
                &[Box::new(GapSubmessage::new(
                    ENTITYID_UNKNOWN,
                    writer_id,
                    next_unsent_change_seq_num,
                    SequenceNumberSet::new(next_unsent_change_seq_num + 1, []),
                ))],
                reader_proxy.unicast_locator_list().to_vec(),
            );
        }

        reader_proxy.set_highest_sent_seq_num(next_unsent_change_seq_num);
    }
}

#[allow(clippy::too_many_arguments)]
fn send_message_to_reader_proxy_reliable(
    reader_proxy: &mut RtpsReaderProxy,
    writer_id: EntityId,
    changes: &[RtpsCacheChange],
    seq_num_min: Option<SequenceNumber>,
    seq_num_max: Option<SequenceNumber>,
    data_max_size_serialized: usize,
    heartbeat_period: Duration,
    message_sender: &MessageSender,
) {
    // Top part of the state machine - Figure 8.19 RTPS standard
    if reader_proxy.unsent_changes(changes.iter()) {
        while let Some(next_unsent_change_seq_num) = reader_proxy.next_unsent_change(changes.iter())
        {
            if next_unsent_change_seq_num > reader_proxy.highest_sent_seq_num() + 1 {
                let gap_start_sequence_number = reader_proxy.highest_sent_seq_num() + 1;
                let gap_end_sequence_number = next_unsent_change_seq_num - 1;
                let gap_submessage = Box::new(GapSubmessage::new(
                    reader_proxy.remote_reader_guid().entity_id(),
                    writer_id,
                    gap_start_sequence_number,
                    SequenceNumberSet::new(gap_end_sequence_number + 1, []),
                ));
                let first_sn = seq_num_min.unwrap_or(1);
                let last_sn = seq_num_max.unwrap_or(0);
                let heartbeat_submessage = Box::new(
                    reader_proxy
                        .heartbeat_machine()
                        .generate_new_heartbeat(writer_id, first_sn, last_sn),
                );
                message_sender.write_message(
                    &[gap_submessage, heartbeat_submessage],
                    reader_proxy.unicast_locator_list().to_vec(),
                );
            } else {
                send_change_message_reader_proxy_reliable(
                    reader_proxy,
                    writer_id,
                    changes,
                    seq_num_min,
                    seq_num_max,
                    data_max_size_serialized,
                    next_unsent_change_seq_num,
                    message_sender,
                );
            }
            reader_proxy.set_highest_sent_seq_num(next_unsent_change_seq_num);
        }
    } else if !reader_proxy.unacked_changes(seq_num_max) {
        // Idle
    } else if reader_proxy
        .heartbeat_machine()
        .is_time_for_heartbeat(heartbeat_period.into())
    {
        let first_sn = seq_num_min.unwrap_or(1);
        let last_sn = seq_num_max.unwrap_or(0);
        let heartbeat_submessage = Box::new(
            reader_proxy
                .heartbeat_machine()
                .generate_new_heartbeat(writer_id, first_sn, last_sn),
        );

        message_sender.write_message(
            &[heartbeat_submessage],
            reader_proxy.unicast_locator_list().to_vec(),
        );
    }

    // Middle-part of the state-machine - Figure 8.19 RTPS standard
    if !reader_proxy.requested_changes().is_empty() {
        while let Some(next_requested_change_seq_num) = reader_proxy.next_requested_change() {
            // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
            // it's not done here to avoid the change being a mutable reference
            // Also the post-condition:
            // a_change BELONGS-TO the_reader_proxy.requested_changes() ) == FALSE
            // should be full-filled by next_requested_change()
            send_change_message_reader_proxy_reliable(
                reader_proxy,
                writer_id,
                changes,
                seq_num_min,
                seq_num_max,
                data_max_size_serialized,
                next_requested_change_seq_num,
                message_sender,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn send_change_message_reader_proxy_reliable(
    reader_proxy: &mut RtpsReaderProxy,
    writer_id: EntityId,
    changes: &[RtpsCacheChange],
    seq_num_min: Option<SequenceNumber>,
    seq_num_max: Option<SequenceNumber>,
    data_max_size_serialized: usize,
    change_seq_num: SequenceNumber,
    message_sender: &MessageSender,
) {
    match changes
        .iter()
        .find(|cc| cc.sequence_number() == change_seq_num)
    {
        Some(cache_change) if change_seq_num > reader_proxy.first_relevant_sample_seq_num() => {
            let number_of_fragments = cache_change
                .data_value()
                .len()
                .div_ceil(data_max_size_serialized);

            // Either send a DATAFRAG submessages or send a single DATA submessage
            if number_of_fragments > 1 {
                for frag_index in 0..number_of_fragments {
                    let info_dst = Box::new(InfoDestinationSubmessage::new(
                        reader_proxy.remote_reader_guid().prefix(),
                    ));

                    let info_timestamp = if let Some(timestamp) = cache_change.source_timestamp() {
                        Box::new(InfoTimestampSubmessage::new(false, timestamp))
                    } else {
                        Box::new(InfoTimestampSubmessage::new(true, TIME_INVALID))
                    };

                    let inline_qos_flag = true;
                    let key_flag = match cache_change.kind() {
                        ChangeKind::Alive => false,
                        ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => true,
                        _ => todo!(),
                    };
                    let non_standard_payload_flag = false;
                    let reader_id = reader_proxy.remote_reader_guid().entity_id();
                    let writer_sn = cache_change.sequence_number();
                    let fragment_starting_num = (frag_index + 1) as u32;
                    let fragments_in_submessage = 1;
                    let fragment_size = data_max_size_serialized as u16;
                    let data_size = cache_change.data_value().len() as u32;
                    let inline_qos = cache_change.inline_qos().clone();

                    let start = frag_index * data_max_size_serialized;
                    let end = std::cmp::min(
                        (frag_index + 1) * data_max_size_serialized,
                        cache_change.data_value().len(),
                    );

                    let serialized_payload =
                        SerializedDataFragment::new(cache_change.data_value().clone(), start..end);

                    let data_frag = Box::new(DataFragSubmessage::new(
                        inline_qos_flag,
                        non_standard_payload_flag,
                        key_flag,
                        reader_id,
                        writer_id,
                        writer_sn,
                        fragment_starting_num,
                        fragments_in_submessage,
                        fragment_size,
                        data_size,
                        inline_qos,
                        serialized_payload,
                    ));

                    message_sender.write_message(
                        &[info_dst, info_timestamp, data_frag],
                        reader_proxy.unicast_locator_list().to_vec(),
                    );
                }
            } else {
                let info_dst = Box::new(InfoDestinationSubmessage::new(
                    reader_proxy.remote_reader_guid().prefix(),
                ));

                let info_timestamp = if let Some(timestamp) = cache_change.source_timestamp() {
                    Box::new(InfoTimestampSubmessage::new(false, timestamp))
                } else {
                    Box::new(InfoTimestampSubmessage::new(true, TIME_INVALID))
                };

                let data_submessage =
                    Box::new(cache_change.as_data_submessage(
                        reader_proxy.remote_reader_guid().entity_id(),
                        writer_id,
                    ));

                let first_sn = seq_num_min.unwrap_or(1);
                let last_sn = seq_num_max.unwrap_or(0);
                let heartbeat = Box::new(
                    reader_proxy
                        .heartbeat_machine()
                        .generate_new_heartbeat(writer_id, first_sn, last_sn),
                );

                message_sender.write_message(
                    &[info_dst, info_timestamp, data_submessage, heartbeat],
                    reader_proxy.unicast_locator_list().to_vec(),
                );
            }
        }
        _ => {
            let info_dst = Box::new(InfoDestinationSubmessage::new(
                reader_proxy.remote_reader_guid().prefix(),
            ));

            let gap_submessage = Box::new(GapSubmessage::new(
                ENTITYID_UNKNOWN,
                writer_id,
                change_seq_num,
                SequenceNumberSet::new(change_seq_num + 1, []),
            ));

            message_sender.write_message(
                &[info_dst, gap_submessage],
                reader_proxy.unicast_locator_list().to_vec(),
            );
        }
    }
}

impl TransportWriter for RtpsStatefulWriter {
    fn get_history_cache(&mut self) -> &mut dyn WriterHistoryCache {
        self
    }

    fn add_matched_reader(
        &mut self,
        reader_proxy: ReaderProxy,
        reliability_kind: ReliabilityKind,
        durability_kind: DurabilityKind,
    ) {
        let first_relevant_sample_seq_num = match durability_kind {
            DurabilityKind::Volatile => self
                .changes
                .iter()
                .map(|cc| cc.sequence_number)
                .max()
                .unwrap_or(0),
            DurabilityKind::TransientLocal
            | DurabilityKind::Transient
            | DurabilityKind::Persistent => 0,
        };
        let rtps_reader_proxy = RtpsReaderProxy::new(
            reader_proxy.remote_reader_guid,
            reader_proxy.remote_group_entity_id,
            &reader_proxy.unicast_locator_list,
            &reader_proxy.multicast_locator_list,
            reader_proxy.expects_inline_qos,
            true,
            reliability_kind,
            first_relevant_sample_seq_num,
        );
        self.matched_readers.push(rtps_reader_proxy);
        self.send_message();
    }

    fn delete_matched_reader(&mut self, reader_guid: Guid) {
        self.matched_readers
            .retain(|rp| rp.remote_reader_guid() != reader_guid);
    }

    fn are_all_changes_acknowledged(&self) -> bool {
        let max_seq_num = self.changes.iter().map(|cc| cc.sequence_number).max();
        !self
            .matched_readers
            .iter()
            .filter(|rp| rp.reliability() == ReliabilityKind::Reliable)
            .any(|rp| rp.unacked_changes(max_seq_num))
    }

    fn writer_proxy(&self) -> WriterProxy {
        WriterProxy {
            remote_writer_guid: self.guid,
            remote_group_entity_id: ENTITYID_UNKNOWN,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            data_max_size_serialized: Default::default(),
        }
    }
}

impl WriterHistoryCache for RtpsStatefulWriter {
    fn add_change(&mut self, cache_change: RtpsCacheChange) {
        self.changes.push(cache_change);
        self.send_message();
    }

    fn remove_change(&mut self, sequence_number: SequenceNumber) {
        self.changes
            .retain(|cc| cc.sequence_number() != sequence_number);
    }
}
