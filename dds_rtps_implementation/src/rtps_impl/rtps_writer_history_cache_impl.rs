use rust_dds_api::dcps_psm::{InstanceStateKind, ViewStateKind};
use rust_rtps_pim::{
    messages::types::Time,
    structure::{
        cache_change::{RtpsCacheChangeAttributes, RtpsCacheChangeConstructor},
        history_cache::{
            RtpsHistoryAttributes, RtpsHistoryCacheConstructor, RtpsHistoryCacheOperations,
        },
        types::{ChangeKind, Guid, InstanceHandle, SequenceNumber},
    },
};
use rust_rtps_psm::messages::submessage_elements::{Parameter, ParameterOwned};

pub struct WriterCacheChange {
    pub kind: ChangeKind,
    pub writer_guid: Guid,
    pub sequence_number: SequenceNumber,
    pub instance_handle: InstanceHandle,
    pub data: Vec<u8>,
    pub _source_timestamp: Option<Time>,
    pub _view_state_kind: ViewStateKind,
    pub _instance_state_kind: InstanceStateKind,
    pub inline_qos: Vec<ParameterOwned>,
}

impl<'a> RtpsCacheChangeConstructor<'a> for WriterCacheChange {
    type DataType = [u8];
    type ParameterListType = [Parameter<'a>];

    fn new(
        kind: &ChangeKind,
        writer_guid: &Guid,
        instance_handle: &InstanceHandle,
        sequence_number: &SequenceNumber,
        data_value: &Self::DataType,
        _inline_qos: &Self::ParameterListType,
    ) -> Self {
        Self {
            kind: *kind,
            writer_guid: *writer_guid,
            sequence_number: *sequence_number,
            instance_handle: *instance_handle,
            data: data_value.to_vec(),
            _source_timestamp: None,
            _view_state_kind: ViewStateKind::New,
            _instance_state_kind: InstanceStateKind::Alive,
            inline_qos: vec![],
        }
    }
}

impl RtpsCacheChangeAttributes for WriterCacheChange {
    type DataType = [u8];
    type ParameterListType = [ParameterOwned];

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
        self.data.as_ref()
    }

    fn inline_qos(&self) -> &Self::ParameterListType {
        self.inline_qos.as_ref()
    }
}

pub struct WriterHistoryCache {
    changes: Vec<WriterCacheChange>,
    source_timestamp: Option<Time>,
}

impl WriterHistoryCache {
    /// Set the Rtps history cache impl's info.
    pub fn set_source_timestamp(&mut self, info: Option<Time>) {
        self.source_timestamp = info;
    }
}

impl RtpsHistoryCacheConstructor for WriterHistoryCache {
    fn new() -> Self {
        Self {
            changes: Vec::new(),
            source_timestamp: None,
        }
    }
}

// impl RtpsHistoryCacheAddChange for WriterHistoryCache {
//     type CacheChangeType = WriterCacheChange;

//     fn add_change(&mut self, change: Self::CacheChangeType) {
//         self.changes.push(change)
//     }
// }

impl RtpsHistoryAttributes for WriterHistoryCache {
    type CacheChangeType = WriterCacheChange;

    fn get_change(&self, seq_num: &SequenceNumber) -> Option<&Self::CacheChangeType> {
        self.changes
            .iter()
            .find(|&cc| &cc.sequence_number == seq_num)
    }
}

impl RtpsHistoryCacheOperations for WriterHistoryCache {
    type CacheChangeType = WriterCacheChange;

    fn add_change(&mut self, change: Self::CacheChangeType) {
        self.changes.push(change)
    }

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
}

#[cfg(test)]
mod tests {

    use super::*;
    use rust_rtps_pim::structure::types::GUID_UNKNOWN;

    #[test]
    fn add_change() {
        let mut hc = WriterHistoryCache::new();
        let change = WriterCacheChange::new(
            &rust_rtps_pim::structure::types::ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &1,
            &vec![],
            &vec![],
        );
        hc.add_change(change);
        assert!(hc.get_change(&1).is_some());
    }

    #[test]
    fn remove_change() {
        let mut hc = WriterHistoryCache::new();
        let change = WriterCacheChange::new(
            &rust_rtps_pim::structure::types::ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &1,
            &vec![],
            &vec![],
        );
        hc.add_change(change);
        hc.remove_change(&1);
        assert!(hc.get_change(&1).is_none());
    }

    #[test]
    fn get_change() {
        let mut hc = WriterHistoryCache::new();
        let change = WriterCacheChange::new(
            &rust_rtps_pim::structure::types::ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &1,
            &vec![],
            &vec![],
        );
        hc.add_change(change);
        assert!(hc.get_change(&1).is_some());
        assert!(hc.get_change(&2).is_none());
    }

    #[test]
    fn get_seq_num_min() {
        let mut hc = WriterHistoryCache::new();
        let change1 = WriterCacheChange::new(
            &rust_rtps_pim::structure::types::ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &1,
            &vec![],
            &vec![],
        );
        let change2 = WriterCacheChange::new(
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
        let mut hc = WriterHistoryCache::new();
        let change1 = WriterCacheChange::new(
            &rust_rtps_pim::structure::types::ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &1,
            &vec![],
            &vec![],
        );
        let change2 = WriterCacheChange::new(
            &rust_rtps_pim::structure::types::ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &2,
            &vec![],
            &vec![],
        );
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_max(), Some(2));
    }
}
