use rtps_pim::{
    behavior::writer::reader_locator::{
        RtpsReaderLocatorAttributes, RtpsReaderLocatorConstructor, RtpsReaderLocatorOperations,
    },
    messages::types::Count,
    structure::{
        history_cache::RtpsHistoryCacheAttributes,
        types::{Locator, SequenceNumber},
    },
};

use crate::rtps_history_cache_impl::RtpsCacheChangeImpl;

use super::rtps_history_cache_impl::RtpsHistoryCacheImpl;
pub struct RtpsReaderLocatorAttributesImpl {
    pub requested_changes: Vec<SequenceNumber>,
    pub unsent_changes: Vec<SequenceNumber>,
    locator: Locator,
    expects_inline_qos: bool,
    last_sent_sequence_number: SequenceNumber,
    pub last_received_acknack_count: Count,
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
            last_received_acknack_count: Count(0),
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
    type CacheChangeType = &'a RtpsCacheChangeImpl;
    type CacheChangeListType = Vec<SequenceNumber>;

    fn next_requested_change(&mut self) -> Option<Self::CacheChangeType> {
        // "next_seq_num := MIN {change.sequenceNumber
        //     SUCH-THAT change IN this.requested_changes()};
        // return change IN this.requested_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"

        let next_seq_num = self
            .reader_locator_attributes
            .requested_changes
            .iter()
            .min()
            .cloned()?;

        // 8.4.8.2.4 Transition T4
        // "After the transition, the following post-conditions hold:
        //   ( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
        self.reader_locator_attributes
            .unsent_changes
            .retain(|c| *c != next_seq_num);

        self.writer_cache
            .changes()
            .iter()
            .find(|c| c.sequence_number == next_seq_num)
    }

    fn next_unsent_change(&mut self) -> Option<Self::CacheChangeType> {
        // "next_seq_num := MIN { change.sequenceNumber
        //     SUCH-THAT change IN this.unsent_changes() };
        // return change IN this.unsent_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"

        let next_seq_num = self
            .reader_locator_attributes
            .unsent_changes
            .iter()
            .min()
            .cloned()?;

        // 8.4.8.2.10 Transition T10
        // "After the transition, the following post-conditions hold:
        //   ( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
        self.reader_locator_attributes
            .unsent_changes
            .retain(|c| *c != next_seq_num);

        self.writer_cache
            .changes()
            .iter()
            .find(|c| c.sequence_number == next_seq_num)
    }

    fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]) {
        self.reader_locator_attributes.requested_changes = req_seq_num_set.to_vec();
    }

    fn requested_changes(&self) -> Self::CacheChangeListType {
        self.reader_locator_attributes.requested_changes.clone()
    }
    fn unsent_changes(&self) -> Self::CacheChangeListType {
        self.reader_locator_attributes.unsent_changes.clone()
    }
}

#[cfg(test)]
mod tests {
    use rtps_pim::structure::{
        cache_change::RtpsCacheChangeConstructor,
        history_cache::{RtpsHistoryCacheConstructor, RtpsHistoryCacheOperations},
        types::{ChangeKind, GUID_UNKNOWN, LOCATOR_INVALID},
    };

    use crate::rtps_history_cache_impl::{RtpsCacheChangeImpl, RtpsData, RtpsParameterList};

    use super::*;

    #[test]
    fn reader_locator_next_unsent_change() {
        let mut hc = RtpsHistoryCacheImpl::new();
        hc.add_change(RtpsCacheChangeImpl::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            1,
            RtpsData(vec![]),
            RtpsParameterList(vec![]),
        ));
        hc.add_change(RtpsCacheChangeImpl::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            2,
            RtpsData(vec![]),
            RtpsParameterList(vec![]),
        ));
        let mut reader_locator_attributes =
            RtpsReaderLocatorAttributesImpl::new(LOCATOR_INVALID, false);
        let mut reader_locator_operations = RtpsReaderLocatorOperationsImpl {
            reader_locator_attributes: &mut reader_locator_attributes,
            writer_cache: &hc,
        };

        // assert_eq!(reader_locator_operations.next_unsent_change(), Some(1));
        // assert_eq!(reader_locator_operations.next_unsent_change(), Some(2));
        // assert_eq!(reader_locator_operations.next_unsent_change(), None);
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

    // #[test]
    // fn reader_locator_unsent_changes() {
    //     let mut hc = RtpsHistoryCacheImpl::new();
    //     hc.add_change(RtpsCacheChangeImpl::new(
    //         ChangeKind::Alive,
    //         GUID_UNKNOWN,
    //         0,
    //         2,
    //         RtpsData(vec![]),
    //         RtpsParameterList(vec![]),
    //     ));
    //     hc.add_change(RtpsCacheChangeImpl::new(
    //         ChangeKind::Alive,
    //         GUID_UNKNOWN,
    //         0,
    //         4,
    //         RtpsData(vec![]),
    //         RtpsParameterList(vec![]),
    //     ));
    //     hc.add_change(RtpsCacheChangeImpl::new(
    //         ChangeKind::Alive,
    //         GUID_UNKNOWN,
    //         0,
    //         6,
    //         RtpsData(vec![]),
    //         RtpsParameterList(vec![]),
    //     ));
    //     let mut reader_locator_attributes =
    //         RtpsReaderLocatorAttributesImpl::new(LOCATOR_INVALID, false);
    //     let mut reader_locator_operations = RtpsReaderLocatorOperationsImpl {
    //         reader_locator_attributes: &mut reader_locator_attributes,
    //         writer_cache: &hc,
    //     };

    //     let unsent_changes = reader_locator_operations.unsent_changes();
    //     assert_eq!(unsent_changes, vec![2, 4, 6]);

    //     reader_locator_operations.next_unsent_change();
    //     let unsent_changes = reader_locator_operations.unsent_changes();
    //     assert_eq!(unsent_changes, vec![4, 6]);
    // }
}
