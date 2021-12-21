use rust_dds_api::dcps_psm::{InstanceStateKind, SampleStateKind, ViewStateKind};
use rust_rtps_pim::{
    messages::{submessage_elements::Parameter, types::Time},
    structure::{
        cache_change::{RtpsCacheChange, RtpsCacheChangeAttributes},
        history_cache::{
            RtpsHistoryCacheAddChange, RtpsHistoryCacheConstructor, RtpsHistoryCacheGetChange,
            RtpsHistoryCacheOperations,
        },
        types::{ChangeKind, Guid, InstanceHandle, SequenceNumber},
    },
};

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
        todo!()
    }

    fn writer_guid(&self) -> &Guid {
        todo!()
    }

    fn instance_handle(&self) -> &InstanceHandle {
        todo!()
    }

    fn sequence_number(&self) -> &SequenceNumber {
        todo!()
    }

    fn data_value(&self) -> &Self::DataType {
        &self.data
    }

    fn inline_qos(&self) -> &Self::ParameterListType {
        todo!()
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

impl<'a, T> RtpsHistoryCacheAddChange<'a> for ReaderHistoryCache<T>
where
    T: for<'b> DdsDeserialize<'b>,
{
    type ParameterListType = &'a [Parameter<&'a [u8]>];
    type DataType = &'a [u8];

    fn add_change(&mut self, change: RtpsCacheChange<Self::ParameterListType, Self::DataType>) {
        let instance_state_kind = match change.kind {
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

        let data = DdsDeserialize::deserialize(&mut change.data_value.as_ref()).unwrap();

        let local_change = ReaderCacheChange {
            kind: change.kind,
            writer_guid: change.writer_guid,
            sequence_number: change.sequence_number,
            instance_handle: change.instance_handle,
            data,
            _source_timestamp: self.source_timestamp,
            _reception_timestamp: Some(reception_timestamp),
            _sample_state_kind: SampleStateKind::NotRead,
            _view_state_kind: ViewStateKind::New,
            _instance_state_kind: instance_state_kind,
        };

        self.changes.push(local_change)
    }
}

impl<T> RtpsHistoryCacheGetChange for ReaderHistoryCache<T> {
    type CacheChangeType = ReaderCacheChange<T>;

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

// pub trait ReaderHistoryCacheGetChange<'a, T> {
//     fn get_reader_history_cache_get_change(
//         &'a self,
//     ) -> &dyn RtpsHistoryCacheGetChange<CacheChangeType = ReaderCacheChange<T>>;
// }

#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::types::GUID_UNKNOWN;

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
        let change = RtpsCacheChange {
            kind: rust_rtps_pim::structure::types::ChangeKind::Alive,
            writer_guid: GUID_UNKNOWN,
            instance_handle: 0,
            sequence_number: 1,
            data_value: &[][..],
            inline_qos: &[][..],
        };
        hc.add_change(change);
        assert!(hc.get_change(&1).is_some());
    }

    #[test]
    fn remove_change() {
        let mut hc: ReaderHistoryCache<MockDdsDeserialize> = ReaderHistoryCache::new();
        let change = RtpsCacheChange {
            kind: rust_rtps_pim::structure::types::ChangeKind::Alive,
            writer_guid: GUID_UNKNOWN,
            instance_handle: 0,
            sequence_number: 1,
            data_value: &[][..],
            inline_qos: &[][..],
        };
        hc.add_change(change);
        hc.remove_change(&1);
        assert!(hc.get_change(&1).is_none());
    }

    #[test]
    fn get_change() {
        let mut hc: ReaderHistoryCache<MockDdsDeserialize> = ReaderHistoryCache::new();
        let change = RtpsCacheChange {
            kind: rust_rtps_pim::structure::types::ChangeKind::Alive,
            writer_guid: GUID_UNKNOWN,
            instance_handle: 0,
            sequence_number: 1,
            data_value: &[][..],
            inline_qos: &[][..],
        };
        hc.add_change(change);
        assert!(hc.get_change(&1).is_some());
        assert!(hc.get_change(&2).is_none());
    }

    #[test]
    fn get_seq_num_min() {
        let mut hc: ReaderHistoryCache<MockDdsDeserialize> = ReaderHistoryCache::new();
        let change1 = RtpsCacheChange {
            kind: rust_rtps_pim::structure::types::ChangeKind::Alive,
            writer_guid: GUID_UNKNOWN,
            instance_handle: 0,
            sequence_number: 1,
            data_value: &[][..],
            inline_qos: &[][..],
        };
        let change2 = RtpsCacheChange {
            kind: rust_rtps_pim::structure::types::ChangeKind::Alive,
            writer_guid: GUID_UNKNOWN,
            instance_handle: 0,
            sequence_number: 2,
            data_value: &[][..],
            inline_qos: &[][..],
        };
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_min(), Some(1));
    }

    #[test]
    fn get_seq_num_max() {
        let mut hc: ReaderHistoryCache<MockDdsDeserialize> = ReaderHistoryCache::new();
        let change1 = RtpsCacheChange {
            kind: rust_rtps_pim::structure::types::ChangeKind::Alive,
            writer_guid: GUID_UNKNOWN,
            instance_handle: 0,
            sequence_number: 1,
            data_value: &[][..],
            inline_qos: &[][..],
        };
        let change2 = RtpsCacheChange {
            kind: rust_rtps_pim::structure::types::ChangeKind::Alive,
            writer_guid: GUID_UNKNOWN,
            instance_handle: 0,
            sequence_number: 2,
            data_value: &[][..],
            inline_qos: &[][..],
        };
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_max(), Some(2));
    }
}
