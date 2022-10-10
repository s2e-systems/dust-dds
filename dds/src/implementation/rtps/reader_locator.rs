use super::{
    history_cache::{RtpsWriterCacheChange, WriterHistoryCache},
    messages::{
        submessage_elements::TimestampSubmessageElement,
        submessages::{AckNackSubmessage, DataSubmessage, GapSubmessage, InfoTimestampSubmessage},
    },
    types::{Count, Locator, SequenceNumber},
};

pub struct RtpsReaderLocator {
    requested_changes: Vec<SequenceNumber>,
    unsent_changes: Vec<SequenceNumber>,
    locator: Locator,
    expects_inline_qos: bool,
    last_received_acknack_count: Count,
}

impl RtpsReaderLocator {
    pub fn new(locator: Locator, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            expects_inline_qos,
            requested_changes: vec![],
            unsent_changes: vec![],
            last_received_acknack_count: Count(0),
        }
    }

    pub fn unsent_changes_reset(&mut self) {
        self.unsent_changes = vec![];
    }

    pub fn unsent_changes_mut(&mut self) -> &mut Vec<SequenceNumber> {
        &mut self.unsent_changes
    }

    pub fn requested_changes_mut(&mut self) -> &mut Vec<SequenceNumber> {
        &mut self.requested_changes
    }

    pub fn locator(&self) -> Locator {
        self.locator
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    /// 8.4.8.2.5 Transition T6
    /// Implementation does not include the part corresponding to searching the reader locator
    /// on the stateless writer
    pub fn receive_acknack(&mut self, acknack_submessage: &AckNackSubmessage) {
        if acknack_submessage.count.value > self.last_received_acknack_count.0 {
            self.requested_changes_set(acknack_submessage.reader_sn_state.set.as_ref());
            self.last_received_acknack_count.0 = acknack_submessage.count.value;
        }
    }

    pub fn next_requested_change<'a>(
        &mut self,
        writer_cache: &'a WriterHistoryCache,
    ) -> RtpsReaderLocatorCacheChange<'a> {
        // "next_seq_num := MIN {change.sequenceNumber
        //     SUCH-THAT change IN this.requested_changes()};
        // return change IN this.requested_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"

        let next_seq_num = self.requested_changes.iter().min().cloned().unwrap();

        // 8.4.8.2.4 Transition T4
        // "After the transition, the following post-conditions hold:
        //   ( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
        self.unsent_changes.retain(|c| *c != next_seq_num);

        let cache_change = writer_cache
            .changes()
            .iter()
            .find(|c| c.sequence_number() == next_seq_num);

        RtpsReaderLocatorCacheChange { cache_change }
    }

    pub fn next_unsent_change<'a>(
        &mut self,
        writer_cache: &'a WriterHistoryCache,
    ) -> RtpsReaderLocatorCacheChange<'a> {
        // "next_seq_num := MIN { change.sequenceNumber
        //     SUCH-THAT change IN this.unsent_changes() };
        // return change IN this.unsent_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"

        let next_seq_num = self.unsent_changes.iter().min().cloned().unwrap();

        // 8.4.8.2.10 Transition T10
        // "After the transition, the following post-conditions hold:
        //   ( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
        self.unsent_changes.retain(|c| *c != next_seq_num);

        let cache_change = writer_cache
            .changes()
            .iter()
            .find(|c| c.sequence_number() == next_seq_num);

        RtpsReaderLocatorCacheChange { cache_change }
    }

    pub fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]) {
        self.requested_changes = req_seq_num_set.to_vec();
    }

    pub fn requested_changes(&self) -> Vec<SequenceNumber> {
        self.requested_changes.clone()
    }

    pub fn unsent_changes(&self) -> Vec<SequenceNumber> {
        self.unsent_changes.clone()
    }
}

pub struct RtpsReaderLocatorCacheChange<'a> {
    cache_change: Option<&'a RtpsWriterCacheChange>,
}

impl RtpsReaderLocatorCacheChange<'_> {
    pub fn is_in_cache(&self) -> bool {
        self.cache_change.is_some()
    }
}

impl<'a> From<RtpsReaderLocatorCacheChange<'a>> for GapSubmessage {
    fn from(_val: RtpsReaderLocatorCacheChange<'a>) -> Self {
        todo!()
    }
}

impl<'a> From<RtpsReaderLocatorCacheChange<'a>> for (InfoTimestampSubmessage, DataSubmessage<'a>) {
    fn from(val: RtpsReaderLocatorCacheChange<'a>) -> Self {
        let cache_change = val
            .cache_change
            .expect("Can only convert to data if it exists in the writer cache");
        let info_ts_submessage = InfoTimestampSubmessage {
            endianness_flag: true,
            invalidate_flag: false,
            timestamp: TimestampSubmessageElement {
                value: cache_change.timestamp().into(),
            },
        };
        let data_submessage = cache_change.into();
        (info_ts_submessage, data_submessage)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        dcps_psm::{HANDLE_NIL, TIME_INVALID},
        implementation::rtps::{
            history_cache::RtpsWriterCacheChange,
            types::{ChangeKind, GUID_UNKNOWN, LOCATOR_INVALID},
        },
    };

    use super::*;

    #[test]
    fn reader_locator_next_unsent_change() {
        let mut hc = WriterHistoryCache::new();
        hc.add_change(RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            1,
            TIME_INVALID,
            vec![],
            vec![],
        ));
        hc.add_change(RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            2,
            TIME_INVALID,
            vec![],
            vec![],
        ));
        let mut reader_locator_attributes = RtpsReaderLocator::new(LOCATOR_INVALID, false);
        reader_locator_attributes.unsent_changes = vec![1, 2];

        assert_eq!(
            reader_locator_attributes
                .next_unsent_change(&hc)
                .cache_change
                .unwrap()
                .sequence_number(),
            1
        );
        assert_eq!(
            reader_locator_attributes
                .next_unsent_change(&hc)
                .cache_change
                .unwrap()
                .sequence_number(),
            2
        );
    }

    #[test]
    fn reader_locator_requested_changes_set() {
        let mut reader_locator_attributes = RtpsReaderLocator::new(LOCATOR_INVALID, false);

        let req_seq_num_set = vec![1, 2, 3];
        reader_locator_attributes.requested_changes_set(&req_seq_num_set);

        let expected_requested_changes = vec![1, 2, 3];
        assert_eq!(
            reader_locator_attributes.requested_changes(),
            expected_requested_changes
        )
    }
}
