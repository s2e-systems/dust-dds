use rust_dds_api::dcps_psm::{InstanceStateKind, SampleStateKind, ViewStateKind};
use rust_rtps_pim::{
    messages::types::Time,
    structure::{
        cache_change::{RtpsCacheChangeAttributes, RtpsCacheChangeConstructor},
        history_cache::{
            RtpsHistoryCacheAddChange, RtpsHistoryCacheConstructor, RtpsHistoryCacheGetChange,
            RtpsHistoryCacheOperations,
        },
        types::{ChangeKind, Guid, InstanceHandle, SequenceNumber},
    },
};
use rust_rtps_psm::messages::submessage_elements::Parameter;

use crate::dds_type::DdsDeserialize;

pub struct ReaderCacheChange<T> {
    kind: ChangeKind,
    writer_guid: Guid,
    sequence_number: SequenceNumber,
    instance_handle: InstanceHandle,
    data: T,
    _source_timestamp: Option<Time>,
    _reception_timestamp: Option<Time>,
    _sample_state_kind: SampleStateKind,
    _view_state_kind: ViewStateKind,
    _instance_state_kind: InstanceStateKind,
}

impl<T> RtpsCacheChangeAttributes for ReaderCacheChange<T> {
    type DataType = T;
    type ParameterListType = ();

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
        todo!()
    }
}

impl<Foo> RtpsCacheChangeConstructor for ReaderCacheChange<Foo>
where
    Foo: for<'a> DdsDeserialize<'a>,
{
    type DataType = [u8];
    type ParameterListType = [Parameter<Vec<u8>>];

    fn new(
        kind: &ChangeKind,
        writer_guid: &Guid,
        instance_handle: &InstanceHandle,
        sequence_number: &SequenceNumber,
        mut data_value: &Self::DataType,
        _inline_qos: &Self::ParameterListType,
    ) -> Self {
        let instance_state_kind = match kind {
            ChangeKind::Alive => InstanceStateKind::Alive,
            ChangeKind::AliveFiltered => InstanceStateKind::Alive,
            ChangeKind::NotAliveDisposed => InstanceStateKind::NotAliveDisposed,
            ChangeKind::NotAliveUnregistered => todo!(),
        };

        let current_time = std::time::SystemTime::now();
        let reception_timestamp = Time(
            current_time
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );

        let data = DdsDeserialize::deserialize(&mut data_value).unwrap();

        ReaderCacheChange {
            kind: *kind,
            writer_guid: *writer_guid,
            sequence_number: *sequence_number,
            instance_handle: *instance_handle,
            data,
            _source_timestamp: None,
            _reception_timestamp: Some(reception_timestamp),
            _sample_state_kind: SampleStateKind::NotRead,
            _view_state_kind: ViewStateKind::New,
            _instance_state_kind: instance_state_kind,
        }
    }
}

pub struct ReaderHistoryCache<T> {
    changes: Vec<ReaderCacheChange<T>>,
    source_timestamp: Option<Time>,
}

impl<T> ReaderHistoryCache<T> {
    /// Set the Rtps history cache impl's info.
    pub fn set_source_timestamp(&mut self, info: Option<Time>) {
        self.source_timestamp = info;
    }
}

impl<T> RtpsHistoryCacheConstructor for ReaderHistoryCache<T> {
    fn new() -> Self {
        Self {
            changes: Vec::new(),
            source_timestamp: None,
        }
    }
}

impl<Foo> RtpsHistoryCacheAddChange for ReaderHistoryCache<Foo> {
    type CacheChangeType = ReaderCacheChange<Foo>;

    fn add_change(&mut self, change: Self::CacheChangeType) {
        self.changes.push(change)
    }
}

impl<Foo> RtpsHistoryCacheGetChange for ReaderHistoryCache<Foo> {
    type CacheChangeType = ReaderCacheChange<Foo>;

    fn get_change(&self, seq_num: &SequenceNumber) -> Option<&Self::CacheChangeType> {
        self.changes
            .iter()
            .find(|&cc| &cc.sequence_number == seq_num)
    }
}

impl<T> RtpsHistoryCacheOperations for ReaderHistoryCache<T> {
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
    use rust_rtps_pim::structure::{cache_change::RtpsCacheChangeConstructor, types::GUID_UNKNOWN};

    use super::*;
    struct MockDdsDeserialize;

    impl DdsDeserialize<'_> for MockDdsDeserialize {
        fn deserialize(_buf: &mut &'_ [u8]) -> rust_dds_api::return_type::DDSResult<Self> {
            Ok(Self)
        }
    }

    #[test]
    fn add_change() {
        let mut hc: ReaderHistoryCache<MockDdsDeserialize> = ReaderHistoryCache::new();
        let change = ReaderCacheChange::new(
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
        let mut hc: ReaderHistoryCache<MockDdsDeserialize> = ReaderHistoryCache::new();
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
        assert!(hc.get_change(&1).is_none());
    }

    #[test]
    fn get_change() {
        let mut hc: ReaderHistoryCache<MockDdsDeserialize> = ReaderHistoryCache::new();
        let change = ReaderCacheChange::new(
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
        let mut hc: ReaderHistoryCache<MockDdsDeserialize> = ReaderHistoryCache::new();
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
        let mut hc: ReaderHistoryCache<MockDdsDeserialize> = ReaderHistoryCache::new();
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
        assert_eq!(hc.get_seq_num_max(), Some(2));
    }
}
