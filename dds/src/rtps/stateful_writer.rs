use super::{behavior_types::Duration, reader_proxy::RtpsReaderProxy};
use crate::{
    rtps_messages::{
        overall_structure::RtpsMessageWrite,
        submessage_elements::SequenceNumberSet,
        submessages::{
            ack_nack::AckNackSubmessage, gap::GapSubmessage,
            info_destination::InfoDestinationSubmessage, info_timestamp::InfoTimestampSubmessage,
            nack_frag::NackFragSubmessage,
        },
        types::TIME_INVALID,
    },
    runtime::Clock,
    transport::{
        interface::WriteMessage,
        types::{
            CacheChange, ChangeKind, DurabilityKind, ENTITYID_UNKNOWN, EntityId, Guid, GuidPrefix,
            ReaderProxy, ReliabilityKind, SequenceNumber,
        },
    },
};
use alloc::vec::Vec;

pub struct RtpsStatefulWriter {
    guid: Guid,
    changes: Vec<CacheChange>,
    matched_readers: Vec<RtpsReaderProxy>,
    heartbeat_period: Duration,
    data_max_size_serialized: usize,
}

impl RtpsStatefulWriter {
    pub fn new(guid: Guid, data_max_size_serialized: usize) -> Self {
        Self {
            guid,
            changes: Vec::new(),
            matched_readers: Vec::new(),
            heartbeat_period: Duration::from_millis(200),
            data_max_size_serialized,
        }
    }

    pub fn guid(&self) -> Guid {
        self.guid
    }

    pub fn data_max_size_serialized(&self) -> usize {
        self.data_max_size_serialized
    }

    pub fn cached_changes_count(&self) -> usize {
        self.changes.len()
    }

    pub async fn add_change(
        &mut self,
        cache_change: CacheChange,
        message_writer: &(impl WriteMessage + ?Sized),
        clock: &impl Clock,
    ) {
        self.changes.push(cache_change);
        self.write_message(message_writer, clock).await;
    }

    pub fn remove_change(&mut self, sequence_number: SequenceNumber) {
        self.changes
            .retain(|cc| cc.sequence_number != sequence_number);
    }

    pub fn is_change_acknowledged(&self, sequence_number: SequenceNumber) -> bool {
        !self
            .matched_readers
            .iter()
            .filter(|rp| rp.reliability() == ReliabilityKind::Reliable)
            .any(|rp| rp.unacked_changes(Some(sequence_number)))
    }

    pub fn add_matched_reader(&mut self, reader_proxy: ReaderProxy) {
        let first_relevant_sample_seq_num = match reader_proxy.durability_kind {
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
            reader_proxy.reliability_kind,
            first_relevant_sample_seq_num,
            reader_proxy.durability_kind,
        );
        if let Some(rp) = self
            .matched_readers
            .iter_mut()
            .find(|rp| rp.remote_reader_guid() == reader_proxy.remote_reader_guid)
        {
            *rp = rtps_reader_proxy;
        } else {
            self.matched_readers.push(rtps_reader_proxy);
        }
    }

    pub fn delete_matched_reader(&mut self, reader_guid: Guid) {
        self.matched_readers
            .retain(|reader_proxy| reader_proxy.remote_reader_guid() != reader_guid);
    }

    pub async fn write_message(
        &mut self,
        message_writer: &(impl WriteMessage + ?Sized),
        clock: &impl Clock,
    ) {
        for reader_proxy in &mut self.matched_readers {
            reader_proxy
                .write_message(
                    self.guid.entity_id(),
                    &self.changes,
                    self.data_max_size_serialized,
                    self.heartbeat_period,
                    message_writer,
                    clock,
                    self.guid.prefix(),
                )
                .await
        }
    }

    /// Process the received AckNack RTPS submessage. This method return an Option indicating the sequence number of the acknowledged change
    /// or None if no change has been acknowledged.
    pub async fn on_acknack_submessage_received(
        &mut self,
        acknack_submessage: &AckNackSubmessage,
        source_guid_prefix: GuidPrefix,
        message_writer: &(impl WriteMessage + ?Sized),
        clock: &impl Clock,
    ) -> Option<SequenceNumber> {
        if &self.guid.entity_id() == acknack_submessage.writer_id() {
            let reader_guid = Guid::new(source_guid_prefix, *acknack_submessage.reader_id());

            if let Some(reader_proxy) = self
                .matched_readers
                .iter_mut()
                .find(|x| x.remote_reader_guid() == reader_guid)
            {
                if reader_proxy.reliability() == ReliabilityKind::Reliable
                    && acknack_submessage.count() > reader_proxy.last_received_acknack_count()
                {
                    let acked_changes = acknack_submessage.reader_sn_state().base() - 1;
                    reader_proxy.acked_changes_set(acked_changes);
                    reader_proxy.requested_changes_set(acknack_submessage.reader_sn_state().set());

                    reader_proxy.set_last_received_acknack_count(acknack_submessage.count());

                    reader_proxy
                        .write_message_reliable(
                            self.guid.entity_id(),
                            &self.changes,
                            self.data_max_size_serialized,
                            self.heartbeat_period,
                            message_writer,
                            clock,
                            self.guid.prefix(),
                        )
                        .await;
                    return Some(acked_changes);
                }
            }
        }
        None
    }

    pub async fn on_nack_frag_submessage_received(
        &mut self,
        nackfrag_submessage: &NackFragSubmessage,
        source_guid_prefix: GuidPrefix,
        message_writer: &(impl WriteMessage + ?Sized),
    ) {
        let reader_guid = Guid::new(source_guid_prefix, nackfrag_submessage.reader_id());

        if let Some(reader_proxy) = self
            .matched_readers
            .iter_mut()
            .find(|x| x.remote_reader_guid() == reader_guid)
        {
            if reader_proxy.reliability() == ReliabilityKind::Reliable
                && nackfrag_submessage.count() > reader_proxy.last_received_nack_frag_count()
            {
                reader_proxy.set_last_received_nack_frag_count(nackfrag_submessage.count());
                let change_seq_num = nackfrag_submessage.writer_sn();
                if let Some(cache_change) = self
                    .changes
                    .iter()
                    .find(|cc| cc.sequence_number == change_seq_num)
                {
                    let number_of_fragments = cache_change
                        .data_value
                        .len()
                        .div_ceil(self.data_max_size_serialized);

                    for request_fragment_number in
                        core::iter::once(nackfrag_submessage.fragment_number_state().base())
                            .chain(nackfrag_submessage.fragment_number_state().set())
                    {
                        let request_fragment_number = request_fragment_number as usize;
                        // Either send a DATAFRAG submessages or send a single DATA submessage
                        if (request_fragment_number) < number_of_fragments
                            && cache_change.kind == ChangeKind::Alive
                        {
                            let writer_id = self.guid.entity_id();
                            let reader_id = reader_proxy.remote_reader_guid().entity_id();
                            let data_frag = cache_change.as_data_frag_submessage(
                                reader_id,
                                writer_id,
                                self.data_max_size_serialized,
                                request_fragment_number,
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

                            let rtps_message = RtpsMessageWrite::from_submessages(
                                &[&info_dst, &info_timestamp, &data_frag],
                                self.guid.prefix(),
                            );
                            message_writer
                                .write_message(
                                    rtps_message.buffer(),
                                    reader_proxy.unicast_locator_list(),
                                )
                                .await
                        }
                    }
                } else {
                    let writer_id = self.guid.entity_id();
                    let info_dst =
                        InfoDestinationSubmessage::new(reader_proxy.remote_reader_guid().prefix());
                    let gap_submessage = GapSubmessage::new(
                        ENTITYID_UNKNOWN,
                        writer_id,
                        change_seq_num,
                        SequenceNumberSet::new(change_seq_num + 1, []),
                    );

                    let rtps_message = RtpsMessageWrite::from_submessages(
                        &[&info_dst, &gap_submessage],
                        self.guid.prefix(),
                    );
                    message_writer
                        .write_message(rtps_message.buffer(), reader_proxy.unicast_locator_list())
                        .await
                }
            }
        }
    }
}

impl RtpsReaderProxy {
    #[allow(clippy::too_many_arguments)]
    async fn write_message(
        &mut self,
        writer_id: EntityId,
        changes: &[CacheChange],
        data_max_size_serialized: usize,
        heartbeat_period: Duration,
        message_writer: &(impl WriteMessage + ?Sized),
        clock: &impl Clock,
        guid_prefix: GuidPrefix,
    ) {
        match self.reliability() {
            ReliabilityKind::BestEffort => {
                self.write_message_best_effort(
                    writer_id,
                    changes,
                    data_max_size_serialized,
                    message_writer,
                    guid_prefix,
                )
                .await
            }
            ReliabilityKind::Reliable => {
                self.write_message_reliable(
                    writer_id,
                    changes,
                    data_max_size_serialized,
                    heartbeat_period,
                    message_writer,
                    clock,
                    guid_prefix,
                )
                .await
            }
        }
    }

    async fn write_message_best_effort(
        &mut self,
        writer_id: EntityId,
        changes: &[CacheChange],
        data_max_size_serialized: usize,
        message_writer: &(impl WriteMessage + ?Sized),
        guid_prefix: GuidPrefix,
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
        while let Some(next_unsent_change_seq_num) = self.next_unsent_change(changes.iter()) {
            if next_unsent_change_seq_num > self.highest_sent_seq_num() + 1 {
                let gap_start_sequence_number = self.highest_sent_seq_num() + 1;
                let gap_end_sequence_number = next_unsent_change_seq_num - 1;
                let gap_submessage = GapSubmessage::new(
                    self.remote_reader_guid().entity_id(),
                    writer_id,
                    gap_start_sequence_number,
                    SequenceNumberSet::new(gap_end_sequence_number + 1, []),
                );
                let rtps_message =
                    RtpsMessageWrite::from_submessages(&[&gap_submessage], guid_prefix);
                message_writer
                    .write_message(rtps_message.buffer(), self.unicast_locator_list())
                    .await;
                // Set highest_sent to gap_end so next iteration sends the actual data
                self.set_highest_sent_seq_num(gap_end_sequence_number);
                // Don't set to next_unsent yet - let the loop continue to send the actual data
            }
            if let Some(cache_change) = changes
                .iter()
                .find(|cc| cc.sequence_number == next_unsent_change_seq_num)
            {
                let number_of_fragments = cache_change
                    .data_value
                    .len()
                    .div_ceil(data_max_size_serialized);

                let info_dst = InfoDestinationSubmessage::new(self.remote_reader_guid().prefix());

                let info_timestamp = if let Some(timestamp) = cache_change.source_timestamp {
                    InfoTimestampSubmessage::new(false, timestamp.into())
                } else {
                    InfoTimestampSubmessage::new(true, TIME_INVALID)
                };
                // Either send a DATAFRAG submessages or send a single DATA submessage
                if number_of_fragments > 1 {
                    for fragment_number in 0..number_of_fragments {
                        let reader_id = self.remote_reader_guid().entity_id();

                        let data_frag = cache_change.as_data_frag_submessage(
                            reader_id,
                            writer_id,
                            data_max_size_serialized,
                            fragment_number,
                        );
                        let rtps_message = RtpsMessageWrite::from_submessages(
                            &[&info_dst, &info_timestamp, &data_frag],
                            guid_prefix,
                        );
                        message_writer
                            .write_message(rtps_message.buffer(), self.unicast_locator_list())
                            .await
                    }
                } else {
                    let data_submessage = cache_change
                        .as_data_submessage(self.remote_reader_guid().entity_id(), writer_id);

                    let rtps_message = RtpsMessageWrite::from_submessages(
                        &[&info_dst, &info_timestamp, &data_submessage],
                        guid_prefix,
                    );
                    message_writer
                        .write_message(rtps_message.buffer(), self.unicast_locator_list())
                        .await
                }
            } else {
                let gap_submessage = GapSubmessage::new(
                    ENTITYID_UNKNOWN,
                    writer_id,
                    next_unsent_change_seq_num,
                    SequenceNumberSet::new(next_unsent_change_seq_num + 1, []),
                );
                let rtps_message =
                    RtpsMessageWrite::from_submessages(&[&gap_submessage], guid_prefix);
                message_writer
                    .write_message(rtps_message.buffer(), self.unicast_locator_list())
                    .await
            }

            self.set_highest_sent_seq_num(next_unsent_change_seq_num);
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn write_message_reliable(
        &mut self,
        writer_id: EntityId,
        changes: &[CacheChange],
        data_max_size_serialized: usize,
        heartbeat_period: Duration,
        message_writer: &(impl WriteMessage + ?Sized),
        clock: &impl Clock,
        guid_prefix: GuidPrefix,
    ) {
        let now = clock.now();
        let seq_num_min = changes.iter().map(|cc| cc.sequence_number).min();
        let seq_num_max = changes.iter().map(|cc| cc.sequence_number).max();
        // Top part of the state machine - Figure 8.19 RTPS standard
        if self.unsent_changes(changes.iter()) {
            while let Some(next_unsent_change_seq_num) = self.next_unsent_change(changes.iter()) {
                if next_unsent_change_seq_num > self.highest_sent_seq_num() + 1 {
                    let gap_start_sequence_number = self.highest_sent_seq_num() + 1;
                    let gap_end_sequence_number = next_unsent_change_seq_num - 1;
                    let gap_submessage = GapSubmessage::new(
                        self.remote_reader_guid().entity_id(),
                        writer_id,
                        gap_start_sequence_number,
                        SequenceNumberSet::new(gap_end_sequence_number + 1, []),
                    );
                    let first_sn = seq_num_min.unwrap_or(1);
                    let last_sn = seq_num_max.unwrap_or(0);
                    let heartbeat_submessage = self
                        .heartbeat_machine()
                        .generate_new_heartbeat(writer_id, first_sn, last_sn, now, false);
                    let info_dst =
                        InfoDestinationSubmessage::new(self.remote_reader_guid().prefix());
                    let rtps_message = RtpsMessageWrite::from_submessages(
                        &[&info_dst, &gap_submessage, &heartbeat_submessage],
                        guid_prefix,
                    );
                    message_writer
                        .write_message(rtps_message.buffer(), self.unicast_locator_list())
                        .await
                } else {
                    let seq_num_min = changes.iter().map(|cc| cc.sequence_number).min();
                    let seq_num_max = changes.iter().map(|cc| cc.sequence_number).max();
                    if let Some(cache_change) = changes.iter().find(|cc| {
                        cc.sequence_number == next_unsent_change_seq_num
                            && next_unsent_change_seq_num > self.first_relevant_sample_seq_num()
                    }) {
                        let number_of_fragments = cache_change
                            .data_value
                            .len()
                            .div_ceil(data_max_size_serialized);

                        // Either send a DATAFRAG submessages or send a single DATA submessage
                        if number_of_fragments > 1 && cache_change.kind == ChangeKind::Alive {
                            for fragment_number in 0..number_of_fragments {
                                let reader_id = self.remote_reader_guid().entity_id();
                                let data_frag = cache_change.as_data_frag_submessage(
                                    reader_id,
                                    writer_id,
                                    data_max_size_serialized,
                                    fragment_number,
                                );

                                let info_dst = InfoDestinationSubmessage::new(
                                    self.remote_reader_guid().prefix(),
                                );
                                let info_timestamp =
                                    if let Some(timestamp) = cache_change.source_timestamp {
                                        InfoTimestampSubmessage::new(false, timestamp.into())
                                    } else {
                                        InfoTimestampSubmessage::new(true, TIME_INVALID)
                                    };

                                let rtps_message = if fragment_number == number_of_fragments - 1 {
                                    let first_sn = seq_num_min.unwrap_or(1);
                                    let last_sn = seq_num_max.unwrap_or(0);
                                    let heartbeat =
                                        self.heartbeat_machine().generate_new_heartbeat(
                                            writer_id, first_sn, last_sn, now, false,
                                        );
                                    RtpsMessageWrite::from_submessages(
                                        &[&info_dst, &info_timestamp, &data_frag, &heartbeat],
                                        guid_prefix,
                                    )
                                } else {
                                    RtpsMessageWrite::from_submessages(
                                        &[&info_dst, &info_timestamp, &data_frag],
                                        guid_prefix,
                                    )
                                };
                                message_writer
                                    .write_message(
                                        rtps_message.buffer(),
                                        self.unicast_locator_list(),
                                    )
                                    .await
                            }
                        } else {
                            let info_dst =
                                InfoDestinationSubmessage::new(self.remote_reader_guid().prefix());

                            let info_timestamp =
                                if let Some(timestamp) = cache_change.source_timestamp {
                                    InfoTimestampSubmessage::new(false, timestamp.into())
                                } else {
                                    InfoTimestampSubmessage::new(true, TIME_INVALID)
                                };

                            let data_submessage = cache_change.as_data_submessage(
                                self.remote_reader_guid().entity_id(),
                                writer_id,
                            );

                            let first_sn = seq_num_min.unwrap_or(1);
                            let last_sn = seq_num_max.unwrap_or(0);
                            let heartbeat = self
                                .heartbeat_machine()
                                .generate_new_heartbeat(writer_id, first_sn, last_sn, now, false);

                            let rtps_message = RtpsMessageWrite::from_submessages(
                                &[&info_dst, &info_timestamp, &data_submessage, &heartbeat],
                                guid_prefix,
                            );
                            message_writer
                                .write_message(rtps_message.buffer(), self.unicast_locator_list())
                                .await
                        }
                    } else {
                        let info_dst =
                            InfoDestinationSubmessage::new(self.remote_reader_guid().prefix());

                        let gap_submessage = GapSubmessage::new(
                            ENTITYID_UNKNOWN,
                            writer_id,
                            next_unsent_change_seq_num,
                            SequenceNumberSet::new(next_unsent_change_seq_num + 1, []),
                        );

                        let rtps_message = RtpsMessageWrite::from_submessages(
                            &[&info_dst, &gap_submessage],
                            guid_prefix,
                        );
                        message_writer
                            .write_message(rtps_message.buffer(), self.unicast_locator_list())
                            .await
                    }
                }
                self.set_highest_sent_seq_num(next_unsent_change_seq_num);
            }
        } else if !self.unacked_changes(seq_num_max) {
            // Idle
        } else if self
            .heartbeat_machine()
            .is_time_for_heartbeat(now, heartbeat_period.into())
        {
            let first_sn = seq_num_min.unwrap_or(1);
            let last_sn = seq_num_max.unwrap_or(0);
            let heartbeat_submessage = self
                .heartbeat_machine()
                .generate_new_heartbeat(writer_id, first_sn, last_sn, now, false);

            let info_dst = InfoDestinationSubmessage::new(self.remote_reader_guid().prefix());

            let rtps_message = RtpsMessageWrite::from_submessages(
                &[&info_dst, &heartbeat_submessage],
                guid_prefix,
            );
            message_writer
                .write_message(rtps_message.buffer(), self.unicast_locator_list())
                .await
        }

        // Middle-part of the state-machine - Figure 8.19 RTPS standard
        if !self.requested_changes().is_empty() {
            while let Some(next_requested_change_seq_num) = self.next_requested_change() {
                // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
                // it's not done here to avoid the change being a mutable reference
                // Also the post-condition:
                // a_change BELONGS-TO the_reader_proxy.requested_changes() ) == FALSE
                // should be full-filled by next_requested_change()
                let now = clock.now();
                let seq_num_min = changes.iter().map(|cc| cc.sequence_number).min();
                let seq_num_max = changes.iter().map(|cc| cc.sequence_number).max();
                if let Some(cache_change) = changes.iter().find(|cc| {
                    cc.sequence_number == next_requested_change_seq_num
                        && next_requested_change_seq_num > self.first_relevant_sample_seq_num()
                }) {
                    let number_of_fragments = cache_change
                        .data_value
                        .len()
                        .div_ceil(data_max_size_serialized);

                    // Either send a DATAFRAG submessages or send a single DATA submessage
                    if number_of_fragments > 1 && cache_change.kind == ChangeKind::Alive {
                        let fragment_number = 0;
                        let reader_id = self.remote_reader_guid().entity_id();
                        let data_frag = cache_change.as_data_frag_submessage(
                            reader_id,
                            writer_id,
                            data_max_size_serialized,
                            fragment_number,
                        );

                        let info_dst =
                            InfoDestinationSubmessage::new(self.remote_reader_guid().prefix());
                        let info_timestamp = if let Some(timestamp) = cache_change.source_timestamp
                        {
                            InfoTimestampSubmessage::new(false, timestamp.into())
                        } else {
                            InfoTimestampSubmessage::new(true, TIME_INVALID)
                        };
                        let first_sn = seq_num_min.unwrap_or(1);
                        let last_sn = seq_num_max.unwrap_or(0);
                        let heartbeat = self
                            .heartbeat_machine()
                            .generate_new_heartbeat(writer_id, first_sn, last_sn, now, false);

                        let rtps_message = RtpsMessageWrite::from_submessages(
                            &[&info_dst, &info_timestamp, &data_frag, &heartbeat],
                            guid_prefix,
                        );
                        message_writer
                            .write_message(rtps_message.buffer(), self.unicast_locator_list())
                            .await;
                    } else {
                        let info_dst =
                            InfoDestinationSubmessage::new(self.remote_reader_guid().prefix());

                        let info_timestamp = if let Some(timestamp) = cache_change.source_timestamp
                        {
                            InfoTimestampSubmessage::new(false, timestamp.into())
                        } else {
                            InfoTimestampSubmessage::new(true, TIME_INVALID)
                        };

                        let data_submessage = cache_change
                            .as_data_submessage(self.remote_reader_guid().entity_id(), writer_id);

                        let first_sn = seq_num_min.unwrap_or(1);
                        let last_sn = seq_num_max.unwrap_or(0);
                        let heartbeat = self
                            .heartbeat_machine()
                            .generate_new_heartbeat(writer_id, first_sn, last_sn, now, false);

                        let rtps_message = RtpsMessageWrite::from_submessages(
                            &[&info_dst, &info_timestamp, &data_submessage, &heartbeat],
                            guid_prefix,
                        );
                        message_writer
                            .write_message(rtps_message.buffer(), self.unicast_locator_list())
                            .await;
                    }
                } else {
                    let info_dst =
                        InfoDestinationSubmessage::new(self.remote_reader_guid().prefix());

                    let gap_submessage = GapSubmessage::new(
                        ENTITYID_UNKNOWN,
                        writer_id,
                        next_requested_change_seq_num,
                        SequenceNumberSet::new(next_requested_change_seq_num + 1, []),
                    );

                    let rtps_message = RtpsMessageWrite::from_submessages(
                        &[&info_dst, &gap_submessage],
                        guid_prefix,
                    );
                    message_writer
                        .write_message(rtps_message.buffer(), self.unicast_locator_list())
                        .await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use crate::{
        infrastructure::time::Time,
        rtps_messages::{
            overall_structure::{RtpsMessageRead, RtpsSubmessageReadKind},
            submessage_elements::FragmentNumberSet,
        },
        std_runtime::executor::block_on,
    };

    use super::*;

    #[test]
    fn test_all_fragments_sent() {
        struct MockClock {}
        impl Clock for MockClock {
            fn now(&self) -> crate::infrastructure::time::Time {
                Time::new(1, 0)
            }
        }
        struct MockWriter {
            total_fragments_sent: Mutex<usize>,
        }
        impl WriteMessage for MockWriter {
            fn write_message(
                &self,
                datagram: &[u8],
                _locator_list: &[crate::transport::types::Locator],
            ) -> core::pin::Pin<Box<dyn Future<Output = ()> + Send>> {
                let message = RtpsMessageRead::try_from(datagram).unwrap();
                assert!(matches!(
                    message.submessages()[2],
                    RtpsSubmessageReadKind::DataFrag(_)
                ));
                *self.total_fragments_sent.lock().unwrap() += 1;
                Box::pin(async {})
            }
        }

        let data_max_size_serialized = 500;
        let guid = Guid::new([1; 12], EntityId::new([1; 3], 1));
        let mut writer = RtpsStatefulWriter::new(guid, data_max_size_serialized);

        let remote_reader_guid = Guid::new([2; 12], EntityId::new([2; 3], 2));
        writer.add_matched_reader(ReaderProxy {
            remote_reader_guid,
            remote_group_entity_id: ENTITYID_UNKNOWN,
            reliability_kind: ReliabilityKind::Reliable,
            durability_kind: DurabilityKind::Volatile,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            expects_inline_qos: false,
        });

        let message_writer = MockWriter {
            total_fragments_sent: Mutex::new(0),
        };
        block_on(writer.add_change(
            CacheChange {
                kind: ChangeKind::Alive,
                writer_guid: guid,
                sequence_number: 1,
                source_timestamp: None,
                instance_handle: Some([10; 16]),
                data_value: vec![8; 1300].into(),
            },
            &message_writer,
            &MockClock {},
        ));
        assert_eq!(*message_writer.total_fragments_sent.lock().unwrap(), 3);
    }

    #[test]
    fn test_single_fragment_sent_after_acknack_frag() {
        struct MockClock {}
        impl Clock for MockClock {
            fn now(&self) -> crate::infrastructure::time::Time {
                Time::new(1, 0)
            }
        }
        struct MockWriter {
            total_fragments_sent: Mutex<usize>,
        }
        impl WriteMessage for MockWriter {
            fn write_message(
                &self,
                datagram: &[u8],
                _locator_list: &[crate::transport::types::Locator],
            ) -> core::pin::Pin<Box<dyn Future<Output = ()> + Send>> {
                let message = RtpsMessageRead::try_from(datagram).unwrap();
                assert!(matches!(
                    message.submessages()[2],
                    RtpsSubmessageReadKind::DataFrag(_)
                ));
                *self.total_fragments_sent.lock().unwrap() += 1;
                Box::pin(async {})
            }
        }

        let data_max_size_serialized = 500;
        let writer_id = EntityId::new([1; 3], 1);
        let guid = Guid::new([1; 12], writer_id);
        let mut writer = RtpsStatefulWriter::new(guid, data_max_size_serialized);

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
        let message_writer = MockWriter {
            total_fragments_sent: Mutex::new(0),
        };
        block_on(writer.add_change(
            CacheChange {
                kind: ChangeKind::Alive,
                writer_guid: guid,
                sequence_number: 1,
                source_timestamp: None,
                instance_handle: Some([10; 16]),
                data_value: vec![8; 1300].into(),
            },
            &message_writer,
            &MockClock {},
        ));

        let nackfrag_submessage = NackFragSubmessage::new(
            remote_reader_id,
            writer_id,
            1,
            FragmentNumberSet::new(1, []),
            1,
        );
        let message_writer = MockWriter {
            total_fragments_sent: Mutex::new(0),
        };
        block_on(writer.on_nack_frag_submessage_received(
            &nackfrag_submessage,
            remote_reader_guid_prefix,
            &message_writer,
        ));

        assert_eq!(*message_writer.total_fragments_sent.lock().unwrap(), 1);
    }

    /// Test that when a late-joining reader connects and there are gaps in sequence numbers
    /// (e.g., old samples were removed from cache), the writer sends both:
    /// 1. A GAP message for the missing sequence numbers
    /// 2. A DATA message for the available sample
    ///
    /// This tests a bug fix where the DATA message was not sent after the GAP because
    /// the code used `else if` instead of `if`, causing the DATA branch to be skipped.
    #[test]
    fn test_gap_followed_by_data_for_late_joining_reader() {
        struct MockClock {}
        impl Clock for MockClock {
            fn now(&self) -> crate::infrastructure::time::Time {
                Time::new(1, 0)
            }
        }

        #[derive(Default)]
        struct MessageTracker {
            gap_count: Mutex<usize>,
            data_count: Mutex<usize>,
            gap_ranges: Mutex<Vec<(SequenceNumber, SequenceNumber)>>,
            data_seq_nums: Mutex<Vec<SequenceNumber>>,
        }

        impl WriteMessage for MessageTracker {
            fn write_message(
                &self,
                datagram: &[u8],
                _locator_list: &[crate::transport::types::Locator],
            ) -> core::pin::Pin<Box<dyn Future<Output = ()> + Send>> {
                let message = RtpsMessageRead::try_from(datagram).unwrap();
                for submessage in message.submessages() {
                    match submessage {
                        RtpsSubmessageReadKind::Gap(gap) => {
                            *self.gap_count.lock().unwrap() += 1;
                            let start = gap.gap_start();
                            // gap_list base is the first seq after the gap
                            let end = gap.gap_list().base() - 1;
                            self.gap_ranges.lock().unwrap().push((start, end));
                        }
                        RtpsSubmessageReadKind::Data(data) => {
                            *self.data_count.lock().unwrap() += 1;
                            self.data_seq_nums.lock().unwrap().push(data.writer_sn());
                        }
                        _ => {}
                    }
                }
                Box::pin(async {})
            }
        }

        let data_max_size_serialized = 500;
        let guid = Guid::new([1; 12], EntityId::new([1; 3], 1));
        let mut writer = RtpsStatefulWriter::new(guid, data_max_size_serialized);

        // Step 1: Add a Volatile reader first (won't try to get historical data)
        let volatile_reader_guid = Guid::new([2; 12], EntityId::new([2; 3], 2));
        writer.add_matched_reader(ReaderProxy {
            remote_reader_guid: volatile_reader_guid,
            remote_group_entity_id: ENTITYID_UNKNOWN,
            reliability_kind: ReliabilityKind::BestEffort,
            durability_kind: DurabilityKind::Volatile,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            expects_inline_qos: false,
        });

        // Step 2: Add changes with sequence numbers 1-5
        let dummy_writer = MessageTracker::default();
        for seq in 1..=5 {
            block_on(writer.add_change(
                CacheChange {
                    kind: ChangeKind::Alive,
                    writer_guid: guid,
                    sequence_number: seq,
                    source_timestamp: None,
                    instance_handle: Some([10; 16]),
                    data_value: vec![8; 100].into(),
                },
                &dummy_writer,
                &MockClock {},
            ));
        }

        // Step 3: Remove changes 1-4, leaving only seq 5 in the cache
        for seq in 1..=4 {
            writer.remove_change(seq);
        }
        assert_eq!(writer.cached_changes_count(), 1);

        // Step 4: Remove the volatile reader and add a TransientLocal reader
        // TransientLocal readers expect to receive all available historical data
        writer.delete_matched_reader(volatile_reader_guid);

        let transient_local_reader_guid = Guid::new([3; 12], EntityId::new([3; 3], 3));
        writer.add_matched_reader(ReaderProxy {
            remote_reader_guid: transient_local_reader_guid,
            remote_group_entity_id: ENTITYID_UNKNOWN,
            reliability_kind: ReliabilityKind::BestEffort,
            durability_kind: DurabilityKind::TransientLocal,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            expects_inline_qos: false,
        });

        // Step 5: Call write_message and track what's sent
        let message_tracker = MessageTracker::default();
        block_on(writer.write_message(&message_tracker, &MockClock {}));

        // Verify: Should have sent 1 GAP (for seq 1-4) and 1 DATA (for seq 5)
        let gap_count = *message_tracker.gap_count.lock().unwrap();
        let data_count = *message_tracker.data_count.lock().unwrap();
        let gap_ranges = message_tracker.gap_ranges.lock().unwrap().clone();
        let data_seq_nums = message_tracker.data_seq_nums.lock().unwrap().clone();

        assert_eq!(
            gap_count, 1,
            "Expected 1 GAP message for missing sequence numbers 1-4"
        );
        assert_eq!(
            data_count, 1,
            "Expected 1 DATA message for sequence number 5 (this was the bug - DATA was not sent after GAP)"
        );

        // Verify the GAP covers seq 1-4
        assert_eq!(gap_ranges.len(), 1);
        assert_eq!(
            gap_ranges[0],
            (1, 4),
            "GAP should cover sequence numbers 1-4"
        );

        // Verify the DATA is for seq 5
        assert_eq!(data_seq_nums.len(), 1);
        assert_eq!(data_seq_nums[0], 5, "DATA should be for sequence number 5");
    }
}
