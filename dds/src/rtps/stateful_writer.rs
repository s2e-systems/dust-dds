use super::{behavior_types::Duration, reader_proxy::RtpsReaderProxy};
use crate::{
    infrastructure::time::Time,
    rtps_messages::submessages::{ack_nack::AckNackSubmessage, nack_frag::NackFragSubmessage},
    transport::types::{
        CacheChange, DurabilityKind, Guid, GuidPrefix, ReaderProxy, ReliabilityKind, SequenceNumber,
    },
};
use alloc::vec::Vec;

pub struct RtpsStatefulWriter {
    guid: Guid,
    changes: Vec<CacheChange>,
    matched_readers: Vec<RtpsReaderProxy>,
    heartbeat_period: Duration,
    nack_response_delay: Duration,
    data_max_size_serialized: usize,
}

impl RtpsStatefulWriter {
    pub fn new(guid: Guid, data_max_size_serialized: usize) -> Self {
        Self {
            guid,
            changes: Vec::new(),
            matched_readers: Vec::new(),
            heartbeat_period: Duration::from_millis(200),
            nack_response_delay: Duration::from_millis(0),
            data_max_size_serialized,
        }
    }

    pub fn guid(&self) -> Guid {
        self.guid
    }

    pub fn data_max_size_serialized(&self) -> usize {
        self.data_max_size_serialized
    }

    pub fn add_change(&mut self, cache_change: CacheChange) {
        self.changes.push(cache_change);
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

    pub fn time_until_next_heartbeat(
        &self,
        now: crate::infrastructure::time::Time,
    ) -> Option<crate::infrastructure::time::Duration> {
        if self.changes.is_empty() || self.matched_readers.is_empty() {
            return None;
        }
        let seq_num_max = self.changes.last()?.sequence_number;
        let mut min_time: Option<crate::infrastructure::time::Duration> = None;
        for rp in &self.matched_readers {
            if rp.reliability() == ReliabilityKind::Reliable
                && seq_num_max > rp.highest_acked_seq_num()
            {
                if let Some(d) =
                    rp.time_until_heartbeat(now, self.heartbeat_period.into(), Some(seq_num_max))
                {
                    min_time = Some(min_time.map_or(d, |min| min.min(d)));
                }
            }
        }
        min_time
    }

    pub fn time_until_next_repair(
        &self,
        now: crate::infrastructure::time::Time,
    ) -> Option<crate::infrastructure::time::Duration> {
        let mut min_time: Option<crate::infrastructure::time::Duration> = None;
        for rp in &self.matched_readers {
            if let Some(d) = rp.time_until_repair(now, self.nack_response_delay.into()) {
                min_time = Some(min_time.map_or(d, |min| min.min(d)));
            }
        }
        min_time
    }

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

    /// Process the received AckNack RTPS submessage. This method return an Option indicating the sequence number of the acknowledged change
    /// or None if no change has been acknowledged.
    pub fn on_acknack_submessage_received(
        &mut self,
        acknack_submessage: &AckNackSubmessage,
        source_guid_prefix: GuidPrefix,
        now: Time,
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
            .matched_readers
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

    pub fn changes(&self) -> &[CacheChange] {
        &self.changes
    }

    pub fn changes_mut(&mut self) -> &mut Vec<CacheChange> {
        &mut self.changes
    }

    pub fn heartbeat_period(&self) -> Duration {
        self.heartbeat_period
    }

    pub fn matched_readers(&self) -> &[RtpsReaderProxy] {
        &self.matched_readers
    }

    pub fn matched_readers_mut(&mut self) -> &mut [RtpsReaderProxy] {
        &mut self.matched_readers
    }

    pub fn changes_and_matched_readers_mut(&mut self) -> (&[CacheChange], &mut [RtpsReaderProxy]) {
        (&self.changes, &mut self.matched_readers)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        infrastructure::time::Time,
        rtps_messages::submessage_elements::FragmentNumberSet,
        transport::types::{ChangeKind, DurabilityKind, ENTITYID_UNKNOWN, EntityId},
    };

    use super::*;

    #[test]
    fn test_single_fragment_sent_after_acknack_frag() {
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
        writer.add_change(CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: guid,
            sequence_number: 1,
            source_timestamp: None,
            instance_handle: Some([10; 16]),
            data_value: vec![8; 1300].into(),
        });
        writer.matched_readers_mut()[0].set_highest_sent_seq_num(1);

        let nackfrag_submessage = NackFragSubmessage::new(
            remote_reader_id,
            writer_id,
            1,
            FragmentNumberSet::new(1, []),
            1,
        );
        writer.on_nack_frag_submessage_received(
            &nackfrag_submessage,
            remote_reader_guid_prefix,
            Time::new(1, 0),
        );
        assert_eq!(
            writer.time_until_next_repair(Time::new(1, 0)),
            Some(crate::infrastructure::time::Duration::new(0, 0))
        );
        assert_eq!(
            writer.matched_readers_mut()[0].next_requested_fragment(),
            Some((1, 1))
        );
    }
}
