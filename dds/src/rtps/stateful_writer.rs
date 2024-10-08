use crate::implementation::data_representation_builtin_endpoints::discovered_reader_data::ReaderProxy;

use super::{
    cache_change::RtpsCacheChange,
    reader_proxy::RtpsReaderProxy,
    types::{DurabilityKind, Guid, ReliabilityKind, SequenceNumber},
};

pub trait WriterHistoryCache {
    fn add_change(&mut self, cache_change: RtpsCacheChange);

    fn remove_change(&mut self, sequence_number: SequenceNumber);

    fn get_changes(&self) -> &[RtpsCacheChange];
}

pub trait TransportWriter {
    fn get_history_cache(&mut self) -> &mut dyn WriterHistoryCache;

    fn add_matched_reader(
        &mut self,
        reader_proxy: ReaderProxy,
        reliability_kind: ReliabilityKind,
        durability_kind: DurabilityKind,
    );

    fn delete_matched_reader(&mut self, reader_guid: Guid);
}

pub struct RtpsStatefulWriter {
    guid: Guid,
    changes: Vec<RtpsCacheChange>,
    matched_readers: Vec<RtpsReaderProxy>,
}

impl RtpsStatefulWriter {
    pub fn new(guid: Guid) -> Self {
        Self {
            guid,
            changes: Vec::new(),
            matched_readers: Vec::new(),
        }
    }

    pub fn guid(&self) -> Guid {
        self.guid
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
    }

    fn delete_matched_reader(&mut self, reader_guid: Guid) {
        self.matched_readers
            .retain(|rp| rp.remote_reader_guid() != reader_guid);
    }
}

impl WriterHistoryCache for RtpsStatefulWriter {
    fn add_change(&mut self, cache_change: RtpsCacheChange) {
        self.changes.push(cache_change);
    }

    fn remove_change(&mut self, sequence_number: SequenceNumber) {
        self.changes
            .retain(|cc| cc.sequence_number() != sequence_number);
    }

    fn get_changes(&self) -> &[RtpsCacheChange] {
        &self.changes
    }
}
