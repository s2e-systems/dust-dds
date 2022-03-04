use rust_rtps_pim::{
    behavior::writer::reader_locator::{
        RtpsReaderLocatorAttributes, RtpsReaderLocatorConstructor, RtpsReaderLocatorOperations,
    },
    structure::{
        history_cache::RtpsHistoryCacheAttributes,
        types::{Locator, SequenceNumber},
    },
};

use super::rtps_history_cache_impl::RtpsHistoryCacheImpl;
pub struct RtpsReaderLocatorAttributesImpl {
    requested_changes: Vec<SequenceNumber>,
    unsent_changes: Vec<SequenceNumber>,
    locator: Locator,
    expects_inline_qos: bool,
    last_sent_sequence_number: SequenceNumber,
}

impl RtpsReaderLocatorAttributesImpl {
    pub fn unsent_changes_reset(&mut self) {
        self.unsent_changes = vec![];
    }
}

impl RtpsReaderLocatorConstructor for RtpsReaderLocatorAttributesImpl {
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

impl RtpsReaderLocatorAttributes for RtpsReaderLocatorAttributesImpl {
    fn locator(&self) -> Locator {
        self.locator
    }
    fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }
}

pub struct RtpsReaderLocatorOperationsImpl<'a> {
    pub reader_locator_attributes: &'a mut RtpsReaderLocatorAttributesImpl,
    writer_cache: &'a RtpsHistoryCacheImpl,
}

impl<'a> RtpsReaderLocatorOperationsImpl<'a> {
    pub fn new(
        reader_locator_attributes: &'a mut RtpsReaderLocatorAttributesImpl,
        writer_cache: &'a RtpsHistoryCacheImpl,
    ) -> Self {
        Self {
            reader_locator_attributes,
            writer_cache,
        }
    }
}

impl<'a> RtpsReaderLocatorOperations for RtpsReaderLocatorOperationsImpl<'a> {
    type CacheChangeType = SequenceNumber;
    type CacheChangeListType = Vec<SequenceNumber>;

    fn next_requested_change(&mut self) -> Option<Self::CacheChangeType> {
        if self.reader_locator_attributes.requested_changes.is_empty() {
            None
        } else {
            Some(self.reader_locator_attributes.requested_changes.remove(0))
        }
    }

    fn next_unsent_change(&mut self) -> Option<Self::CacheChangeType> {
        let unsent_change = self.unsent_changes().first().cloned();
        if let Some(unsent_change) = unsent_change {
            self.reader_locator_attributes.last_sent_sequence_number = unsent_change;
        };
        unsent_change
    }

    fn requested_changes_set(&mut self, req_seq_num_set: &[Self::CacheChangeType]) {
        self.reader_locator_attributes.requested_changes = req_seq_num_set.to_vec();
    }

    fn requested_changes(&self) -> Self::CacheChangeListType {
        self.reader_locator_attributes.requested_changes.clone()
    }
    fn unsent_changes(&self) -> Self::CacheChangeListType {
        self.writer_cache
            .changes()
            .iter()
            .filter_map(|f| {
                if f.sequence_number > self.reader_locator_attributes.last_sent_sequence_number {
                    Some(f.sequence_number)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::{
        cache_change::RtpsCacheChangeConstructor,
        history_cache::{RtpsHistoryCacheConstructor, RtpsHistoryCacheOperations},
        types::{ChangeKind, GUID_UNKNOWN, LOCATOR_INVALID},
    };

    use crate::rtps_impl::rtps_history_cache_impl::RtpsCacheChangeImpl;

    use super::*;

    #[test]
    fn reader_locator_next_unsent_change() {
        let mut hc = RtpsHistoryCacheImpl::new();
        hc.add_change(RtpsCacheChangeImpl::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            1,
            &[],
            &[],
        ));
        hc.add_change(RtpsCacheChangeImpl::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            2,
            &[],
            &[],
        ));
        let mut reader_locator_attributes =
            RtpsReaderLocatorAttributesImpl::new(LOCATOR_INVALID, false);
        let mut reader_locator_operations = RtpsReaderLocatorOperationsImpl {
            reader_locator_attributes: &mut reader_locator_attributes,
            writer_cache: &hc,
        };

        assert_eq!(reader_locator_operations.next_unsent_change(), Some(1));
        assert_eq!(reader_locator_operations.next_unsent_change(), Some(2));
        assert_eq!(reader_locator_operations.next_unsent_change(), None);
    }

    #[test]
    fn reader_locator_requested_changes_set() {
        let hc = RtpsHistoryCacheImpl::new();
        let mut reader_locator_attributes =
            RtpsReaderLocatorAttributesImpl::new(LOCATOR_INVALID, false);
        let mut reader_locator_operations = RtpsReaderLocatorOperationsImpl {
            reader_locator_attributes: &mut reader_locator_attributes,
            writer_cache: &hc,
        };
        let req_seq_num_set = vec![1, 2, 3];
        reader_locator_operations.requested_changes_set(&req_seq_num_set);

        let expected_requested_changes = vec![1, 2, 3];
        assert_eq!(
            reader_locator_operations.requested_changes(),
            expected_requested_changes
        )
    }

    #[test]
    fn reader_locator_unsent_changes() {
        let mut hc = RtpsHistoryCacheImpl::new();
        hc.add_change(RtpsCacheChangeImpl::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            2,
            &[],
            &[],
        ));
        hc.add_change(RtpsCacheChangeImpl::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            4,
            &[],
            &[],
        ));
        hc.add_change(RtpsCacheChangeImpl::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            6,
            &[],
            &[],
        ));
        let mut reader_locator_attributes =
            RtpsReaderLocatorAttributesImpl::new(LOCATOR_INVALID, false);
        let mut reader_locator_operations = RtpsReaderLocatorOperationsImpl {
            reader_locator_attributes: &mut reader_locator_attributes,
            writer_cache: &hc,
        };

        let unsent_changes = reader_locator_operations.unsent_changes();
        assert_eq!(unsent_changes, vec![2, 4, 6]);

        reader_locator_operations.next_unsent_change();
        let unsent_changes = reader_locator_operations.unsent_changes();
        assert_eq!(unsent_changes, vec![4, 6]);
    }
}
