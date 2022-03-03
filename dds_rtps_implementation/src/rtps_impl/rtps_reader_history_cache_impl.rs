use rust_rtps_pim::{
    messages::{submessage_elements::Parameter, types::Time},
    structure::{
        cache_change::{RtpsCacheChangeAttributes, RtpsCacheChangeConstructor},
        history_cache::{
            RtpsHistoryCacheAttributes, RtpsHistoryCacheConstructor, RtpsHistoryCacheOperations,
        },
        types::{ChangeKind, Guid, InstanceHandle, SequenceNumber},
    },
};

use super::rtps_writer_history_cache_impl::RtpsParameterList;
pub struct ReaderCacheChange {
    kind: ChangeKind,
    writer_guid: Guid,
    sequence_number: SequenceNumber,
    instance_handle: InstanceHandle,
    data: Vec<u8>,
    inline_qos: RtpsParameterList,
}

impl RtpsCacheChangeAttributes<'_> for ReaderCacheChange {
    type DataType = [u8];
    type ParameterListType = RtpsParameterList;

    fn kind(&self) -> &ChangeKind {
        &self.kind
    }

    fn writer_guid(&self) -> &Guid {
        &self.writer_guid
    }

    fn instance_handle(&self) -> &InstanceHandle {
        &self.instance_handle
    }

    fn sequence_number(&self) -> &SequenceNumber {
        &self.sequence_number
    }

    fn data_value(&self) -> &Self::DataType {
        &self.data
    }

    fn inline_qos(&self) -> &Self::ParameterListType {
        &self.inline_qos
    }
}

impl<'b> RtpsCacheChangeConstructor<'b> for ReaderCacheChange {
    type DataType = [u8];
    type ParameterListType = [Parameter<'b>];

    fn new(
        kind: &ChangeKind,
        writer_guid: &Guid,
        instance_handle: &InstanceHandle,
        sequence_number: &SequenceNumber,
        data_value: &Self::DataType,
        inline_qos: &Self::ParameterListType,
    ) -> Self {
        ReaderCacheChange {
            kind: *kind,
            writer_guid: *writer_guid,
            sequence_number: *sequence_number,
            instance_handle: *instance_handle,
            data: data_value.to_vec(),
            inline_qos: inline_qos.iter().collect(),
        }
    }
}

pub struct ReaderHistoryCache {
    changes: Vec<ReaderCacheChange>,
    source_timestamp: Option<Time>,
}

impl ReaderHistoryCache {
    /// Set the Rtps history cache impl's info.
    pub fn set_source_timestamp(&mut self, info: Option<Time>) {
        self.source_timestamp = info;
    }
}

impl RtpsHistoryCacheConstructor for ReaderHistoryCache {
    fn new() -> Self {
        Self {
            changes: Vec::new(),
            source_timestamp: None,
        }
    }
}

impl RtpsHistoryCacheAttributes for ReaderHistoryCache {
    type CacheChangeType = ReaderCacheChange;

    fn changes(&self) -> &[Self::CacheChangeType] {
        &self.changes
    }
}

impl RtpsHistoryCacheOperations for ReaderHistoryCache {
    type CacheChangeType = ReaderCacheChange;

    fn remove_change(&mut self, seq_num: &SequenceNumber) {
        self.changes.retain(|cc| &cc.sequence_number != seq_num)
    }

    fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.changes
            .iter()
            .map(|cc| cc.sequence_number)
            .min()
            .clone()
    }

    fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.changes
            .iter()
            .map(|cc| cc.sequence_number)
            .max()
            .clone()
    }

    fn add_change(&mut self, change: Self::CacheChangeType) {
        self.changes.push(change)
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::{cache_change::RtpsCacheChangeConstructor, types::GUID_UNKNOWN};

    use super::*;

    #[test]
    fn remove_change() {
        let mut hc = ReaderHistoryCache::new();
        let change = ReaderCacheChange::new(
            &rust_rtps_pim::structure::types::ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &1,
            &vec![],
            &vec![],
        );
        hc.add_change(change);
        hc.remove_change(&1);
        assert!(hc.changes().is_empty());
    }

    #[test]
    fn get_seq_num_min() {
        let mut hc = ReaderHistoryCache::new();
        let change1 = ReaderCacheChange::new(
            &rust_rtps_pim::structure::types::ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &1,
            &vec![],
            &vec![],
        );
        let change2 = ReaderCacheChange::new(
            &rust_rtps_pim::structure::types::ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &2,
            &vec![],
            &vec![],
        );
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_min(), Some(1));
    }

    #[test]
    fn get_seq_num_max() {
        let mut hc = ReaderHistoryCache::new();
        let change1 = ReaderCacheChange::new(
            &rust_rtps_pim::structure::types::ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &1,
            &vec![],
            &[],
        );
        let change2 = ReaderCacheChange::new(
            &rust_rtps_pim::structure::types::ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &2,
            &vec![],
            &[],
        );
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_max(), Some(2));
    }
}
