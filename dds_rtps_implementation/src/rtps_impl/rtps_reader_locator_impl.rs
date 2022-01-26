use rust_rtps_pim::{
    behavior::writer::reader_locator::{
        RtpsReaderLocatorAttributes, RtpsReaderLocatorConstructor, RtpsReaderLocatorOperations,
    },
    structure::{
        history_cache::RtpsHistoryAttributes,
        types::{Locator, SequenceNumber},
    },
};

use super::rtps_writer_history_cache_impl::WriterHistoryCache;
pub struct RtpsReaderLocatorImpl {
    requested_changes: Vec<SequenceNumber>,
    unsent_changes: Vec<SequenceNumber>,
    locator: Locator,
    expects_inline_qos: bool,
    last_sent_sequence_number: SequenceNumber,
}

impl RtpsReaderLocatorImpl {
    pub fn unsent_changes_reset(&mut self) {
        self.unsent_changes = vec![];
    }
}

impl RtpsReaderLocatorConstructor for RtpsReaderLocatorImpl {
    type CacheChangeType = SequenceNumber;
    fn new(locator: Locator, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            expects_inline_qos,
            requested_changes: vec![],
            unsent_changes: vec![],
            last_sent_sequence_number: 0,
        }
    }
}

impl RtpsReaderLocatorAttributes for RtpsReaderLocatorImpl {
    type CacheChangeType = SequenceNumber;
    type HistoryCacheType = WriterHistoryCache;

    fn requested_changes(&self) -> &[Self::CacheChangeType] {
        &self.requested_changes
    }
    fn unsent_changes(
        &mut self,
        history_cache: &Self::HistoryCacheType,
    ) -> &[Self::CacheChangeType] {
        self.unsent_changes = history_cache
            .changes()
            .iter()
            .filter_map(|f| {
                if f.sequence_number > self.last_sent_sequence_number {
                    Some(f.sequence_number)
                } else {
                    None
                }
            })
            .collect();
        &self.unsent_changes
    }
    fn locator(&self) -> &Locator {
        &self.locator
    }
    fn expects_inline_qos(&self) -> &bool {
        &self.expects_inline_qos
    }
}

impl RtpsReaderLocatorOperations for RtpsReaderLocatorImpl {
    type CacheChangeType = SequenceNumber;
    type HistoryCacheType = WriterHistoryCache;

    fn next_requested_change(&mut self) -> Option<Self::CacheChangeType> {
        if self.requested_changes.is_empty() {
            None
        } else {
            Some(self.requested_changes.remove(0))
        }
    }

    fn next_unsent_change(
        &mut self,
        history_cache: &Self::HistoryCacheType,
    ) -> Option<Self::CacheChangeType> {
        let unsent_change = self.unsent_changes(history_cache).first().cloned();
        if let Some(unsent_change) = unsent_change {
            self.last_sent_sequence_number = unsent_change;
        };
        unsent_change
    }

    fn requested_changes_set(&mut self, req_seq_num_set: &[Self::CacheChangeType]) {
        self.requested_changes = req_seq_num_set.to_vec();
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::{
        cache_change::RtpsCacheChangeConstructor,
        history_cache::{RtpsHistoryCacheConstructor, RtpsHistoryCacheOperations},
        types::{ChangeKind, GUID_UNKNOWN, LOCATOR_INVALID},
    };

    use crate::rtps_impl::rtps_writer_history_cache_impl::WriterCacheChange;

    use super::*;

    #[test]
    fn reader_locator_next_unsent_change() {
        let mut hc = WriterHistoryCache::new();
        hc.add_change(WriterCacheChange::new(
            &ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &1,
            &[],
            &[],
        ));
        hc.add_change(WriterCacheChange::new(
            &ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &2,
            &[],
            &[],
        ));
        let mut reader_locator = RtpsReaderLocatorImpl::new(LOCATOR_INVALID, false);

        assert_eq!(reader_locator.next_unsent_change(&hc), Some(1));
        assert_eq!(reader_locator.next_unsent_change(&hc), Some(2));
        assert_eq!(reader_locator.next_unsent_change(&hc), None);
    }

    #[test]
    fn reader_locator_requested_changes_set() {
        let mut reader_locator = RtpsReaderLocatorImpl::new(LOCATOR_INVALID, false);

        let req_seq_num_set = vec![1, 2, 3];
        reader_locator.requested_changes_set(&req_seq_num_set);

        let expected_requested_changes = vec![1, 2, 3];
        assert_eq!(
            reader_locator.requested_changes(),
            expected_requested_changes
        )
    }

    #[test]
    fn reader_locator_unsent_changes() {
        let mut hc = WriterHistoryCache::new();
        hc.add_change(WriterCacheChange::new(
            &ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &2,
            &[],
            &[],
        ));
        hc.add_change(WriterCacheChange::new(
            &ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &4,
            &[],
            &[],
        ));
        hc.add_change(WriterCacheChange::new(
            &ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &6,
            &[],
            &[],
        ));
        let mut reader_locator = RtpsReaderLocatorImpl::new(LOCATOR_INVALID, false);

        let unsent_changes = reader_locator.unsent_changes(&hc);
        assert_eq!(unsent_changes, vec![2, 4, 6]);

        reader_locator.next_unsent_change(&hc);
        let unsent_changes = reader_locator.unsent_changes(&hc);
        assert_eq!(unsent_changes, vec![4, 6]);
    }
}
