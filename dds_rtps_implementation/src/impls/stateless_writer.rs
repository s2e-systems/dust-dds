use std::ops::{Deref, DerefMut};

use rust_rtps::{
    behavior::{stateless_writer::RTPSReaderLocator, RTPSStatelessWriter, RTPSWriter},
    types::Locator,
};

pub struct StatelessWriter<W: RTPSWriter, R: RTPSReaderLocator> {
    writer: W,
    reader_locators: Vec<R>,
}

impl<W: RTPSWriter, R: RTPSReaderLocator> StatelessWriter<W, R> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            reader_locators: Vec::new(),
        }
    }
}

impl<W: RTPSWriter, R: RTPSReaderLocator> Deref for StatelessWriter<W, R> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl<W: RTPSWriter, R: RTPSReaderLocator> DerefMut for StatelessWriter<W, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

impl<W: RTPSWriter, R: RTPSReaderLocator> RTPSStatelessWriter<W> for StatelessWriter<W, R> {
    type ReaderLocatorType = R;

    fn reader_locators(&self) -> &[Self::ReaderLocatorType] {
        &self.reader_locators
    }

    fn reader_locator_add(&mut self, a_locator: Self::ReaderLocatorType) {
        self.reader_locators.push(a_locator)
    }

    fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.retain(|x| &x.locator() != a_locator)
    }

    fn unsent_changes_reset(&mut self) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps::structure::{RTPSCacheChange, RTPSEndpoint, RTPSEntity, RTPSHistoryCache};

    use super::*;

    struct MockCacheChange;

    impl RTPSCacheChange for MockCacheChange {
        type Data = ();

        fn new(
            _kind: rust_rtps::types::ChangeKind,
            _writer_guid: rust_rtps::types::GUID,
            _instance_handle: rust_rtps::types::InstanceHandle,
            _sequence_number: rust_rtps::types::SequenceNumber,
            _data_value: Self::Data,
            _inline_qos: rust_rtps::messages::submessages::submessage_elements::ParameterList,
        ) -> Self {
            todo!()
        }

        fn kind(&self) -> rust_rtps::types::ChangeKind {
            todo!()
        }

        fn writer_guid(&self) -> rust_rtps::types::GUID {
            todo!()
        }

        fn instance_handle(&self) -> &rust_rtps::types::InstanceHandle {
            todo!()
        }

        fn sequence_number(&self) -> rust_rtps::types::SequenceNumber {
            todo!()
        }

        fn data_value(&self) -> &Self::Data {
            todo!()
        }

        fn inline_qos(
            &self,
        ) -> &rust_rtps::messages::submessages::submessage_elements::ParameterList {
            todo!()
        }
    }
    struct MockHistoryCache;

    impl RTPSHistoryCache for MockHistoryCache {
        type CacheChangeType = MockCacheChange;

        fn new() -> Self {
            todo!()
        }

        fn add_change(&mut self, _change: Self::CacheChangeType) {
            todo!()
        }

        fn remove_change(&mut self, _seq_num: rust_rtps::types::SequenceNumber) {
            todo!()
        }

        fn get_change(
            &self,
            _seq_num: rust_rtps::types::SequenceNumber,
        ) -> Option<&Self::CacheChangeType> {
            todo!()
        }

        fn get_seq_num_min(&self) -> Option<rust_rtps::types::SequenceNumber> {
            todo!()
        }

        fn get_seq_num_max(&self) -> Option<rust_rtps::types::SequenceNumber> {
            todo!()
        }
    }

    struct MockWriter;
    impl RTPSEntity for MockWriter {
        fn guid(&self) -> rust_rtps::types::GUID {
            todo!()
        }
    }

    impl RTPSEndpoint for MockWriter {
        fn unicast_locator_list(&self) -> &[Locator] {
            todo!()
        }

        fn multicast_locator_list(&self) -> &[Locator] {
            todo!()
        }

        fn topic_kind(&self) -> rust_rtps::types::TopicKind {
            todo!()
        }

        fn reliability_level(&self) -> rust_rtps::types::ReliabilityKind {
            todo!()
        }
    }

    impl RTPSWriter for MockWriter {
        type HistoryCacheType = MockHistoryCache;

        fn push_mode(&self) -> bool {
            todo!()
        }

        fn heartbeat_period(&self) -> rust_rtps::behavior::types::Duration {
            todo!()
        }

        fn nack_response_delay(&self) -> rust_rtps::behavior::types::Duration {
            todo!()
        }

        fn nack_suppression_duration(&self) -> rust_rtps::behavior::types::Duration {
            todo!()
        }

        fn last_change_sequence_number(&self) -> rust_rtps::types::SequenceNumber {
            todo!()
        }

        fn data_max_sized_serialized(&self) -> i32 {
            todo!()
        }

        fn writer_cache(&mut self) -> &mut Self::HistoryCacheType {
            todo!()
        }

        fn new_change(
            &mut self,
            _kind: rust_rtps::types::ChangeKind,
            _data: <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType as RTPSCacheChange>::Data,
            _inline_qos: rust_rtps::messages::submessages::submessage_elements::ParameterList,
            _handle: rust_rtps::types::InstanceHandle,
        ) -> <Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType {
            todo!()
        }
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    struct MockReaderLocator {
        locator: Locator,
    }

    impl RTPSReaderLocator for MockReaderLocator {
        type CacheChangeRepresentation = MockReaderLocator;
        type CacheChangeRepresentationList = Vec<MockReaderLocator>;

        fn requested_changes(&self) -> Self::CacheChangeRepresentationList {
            todo!()
        }

        fn unsent_changes(
            &self,
            _writer_cache: &impl RTPSHistoryCache,
        ) -> Self::CacheChangeRepresentationList {
            todo!()
        }

        fn locator(&self) -> Locator {
            self.locator
        }

        fn expects_inline_qos(&self) -> bool {
            todo!()
        }

        fn next_requested_change(&mut self) -> Option<Self::CacheChangeRepresentation> {
            todo!()
        }

        fn next_unsent_change(
            &mut self,
            _writer_cache: &impl RTPSHistoryCache,
        ) -> Option<Self::CacheChangeRepresentation> {
            todo!()
        }

        fn requested_changes_set(
            &mut self,
            _req_seq_num_set: &[rust_rtps::types::SequenceNumber],
            _writer_cache: &impl RTPSHistoryCache,
        ) {
            todo!()
        }

        fn new(_locator: Locator, _expects_inline_qos: bool) -> Self {
            todo!()
        }
    }

    #[test]
    fn reader_locator_add() {
        let writer = MockWriter;
        let mut stateless_writer = StatelessWriter::new(writer);

        let locator1 = Locator::new(0, 100, [1; 16]);
        let locator2 = Locator::new(0, 200, [2; 16]);
        let reader_locator1 = MockReaderLocator { locator: locator1 };
        let reader_locator2 = MockReaderLocator { locator: locator2 };

        stateless_writer.reader_locator_add(reader_locator1);
        stateless_writer.reader_locator_add(reader_locator2);

        assert!(stateless_writer
            .reader_locators()
            .contains(&reader_locator1));
        assert!(stateless_writer
            .reader_locators()
            .contains(&reader_locator2));
    }

    #[test]
    fn reader_locator_remove() {
        let writer = MockWriter;
        let mut stateless_writer = StatelessWriter::new(writer);

        let locator1 = Locator::new(0, 100, [1; 16]);
        let locator2 = Locator::new(0, 200, [2; 16]);
        let reader_locator1 = MockReaderLocator { locator: locator1 };
        let reader_locator2 = MockReaderLocator { locator: locator2 };

        stateless_writer.reader_locator_add(reader_locator1);
        stateless_writer.reader_locator_add(reader_locator2);
        stateless_writer.reader_locator_remove(&locator2);

        assert!(stateless_writer
            .reader_locators()
            .contains(&reader_locator1));
        assert!(!stateless_writer
            .reader_locators()
            .contains(&reader_locator2));
    }

    #[test]
    fn unsent_changes_reset() {
        let writer = MockWriter;
        let mut stateless_writer = StatelessWriter::new(writer);

        let locator1 = Locator::new(0, 100, [1; 16]);
        let locator2 = Locator::new(0, 200, [2; 16]);
        let reader_locator1 = MockReaderLocator { locator: locator1 };
        let reader_locator2 = MockReaderLocator { locator: locator2 };

        stateless_writer.reader_locator_add(reader_locator1);
        stateless_writer.reader_locator_add(reader_locator2);

        stateless_writer.unsent_changes_reset();
    }
}
