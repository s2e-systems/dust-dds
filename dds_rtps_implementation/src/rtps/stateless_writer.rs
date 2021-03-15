use std::{ops::Deref, sync::Arc};

use rust_rtps::{
    behavior::{stateless_writer::RTPSReaderLocator, RTPSStatelessWriter, RTPSWriter},
    types::Locator,
};

use super::reader_locator::ReaderLocator;

pub struct StatelessWriter<T: RTPSWriter> {
    writer: Arc<T>,
    reader_locators: Vec<ReaderLocator<T>>,
}

impl<T: RTPSWriter> Deref for StatelessWriter<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl<T: RTPSWriter> RTPSStatelessWriter<T> for StatelessWriter<T> {
    type ReaderLocatorType = ReaderLocator<T>;

    fn new(writer: T) -> Self {
        Self {
            writer: Arc::new(writer),
            reader_locators: Vec::new(),
        }
    }

    fn reader_locators(&self) -> &[Self::ReaderLocatorType] {
        &self.reader_locators
    }

    fn reader_locator_add(&mut self, a_locator: Locator) {
        let reader_locator = Self::ReaderLocatorType::new(a_locator, false, self.writer.clone());
        self.reader_locators.push(reader_locator)
    }

    fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.retain(|x| &x.locator() != a_locator)
    }

    fn unsent_changes_reset(&mut self) {
        for r in &mut self.reader_locators.iter_mut() {
            *r = Self::ReaderLocatorType::new(
                r.locator(),
                r.expects_inline_qos(),
                self.writer.clone(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps::structure::{
        history_cache::RTPSHistoryCacheRead, RTPSCacheChange, RTPSEndpoint, RTPSEntity,
        RTPSHistoryCache,
    };

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
    impl<'a> RTPSHistoryCacheRead<'a> for MockHistoryCache {
        type CacheChangeType = MockCacheChange;
        type Item = &'a MockCacheChange;
    }

    impl RTPSHistoryCache for MockHistoryCache {
        type CacheChangeType = MockCacheChange;
        type HistoryCacheStorageType = Self;

        fn new() -> Self {
            todo!()
        }

        fn add_change(&self, _change: Self::CacheChangeType) {
            todo!()
        }

        fn remove_change(&self, _seq_num: rust_rtps::types::SequenceNumber) {
            todo!()
        }

        fn get_change<'a>(
            &'a self,
            _seq_num: rust_rtps::types::SequenceNumber,
        ) -> Option<<Self::HistoryCacheStorageType as RTPSHistoryCacheRead<'a>>::Item> {
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

        fn new(
            _guid: rust_rtps::types::GUID,
            _topic_kind: rust_rtps::types::TopicKind,
            _reliablility_level: rust_rtps::types::ReliabilityKind,
            _unicast_locator_list: &[Locator],
            _multicast_locator_list: &[Locator],
            _push_mode: bool,
            _heartbeat_period: rust_rtps::behavior::types::Duration,
            _nack_response_delay: rust_rtps::behavior::types::Duration,
            _nack_suppression_duration: rust_rtps::behavior::types::Duration,
            _data_max_sized_serialized: i32,
            _writer_cache: Self::HistoryCacheType,
        ) -> Self {
            todo!()
        }

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

        fn writer_cache(&self) -> &Self::HistoryCacheType {
            todo!()
        }

        fn new_change(
            &self,
            _kind: rust_rtps::types::ChangeKind,
            _data: <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType as RTPSCacheChange>::Data,
            _inline_qos: rust_rtps::messages::submessages::submessage_elements::ParameterList,
            _handle: rust_rtps::types::InstanceHandle,
        ) -> <Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType {
            todo!()
        }
    }

    #[test]
    fn reader_locator_add() {
        let writer = MockWriter;
        let mut stateless_writer = StatelessWriter::new(writer);

        let locator1 = Locator::new(0, 100, [1; 16]);
        let locator2 = Locator::new(0, 200, [2; 16]);

        stateless_writer.reader_locator_add(locator1);
        stateless_writer.reader_locator_add(locator2);

        // assert!(stateless_writer
        //     .reader_locators()
        //     .contains(&reader_locator1));
        // assert!(stateless_writer
        //     .reader_locators()
        //     .contains(&reader_locator2));
    }

    #[test]
    fn reader_locator_remove() {
        let writer = MockWriter;
        let mut stateless_writer = StatelessWriter::new(writer);

        let locator1 = Locator::new(0, 100, [1; 16]);
        let locator2 = Locator::new(0, 200, [2; 16]);

        stateless_writer.reader_locator_add(locator1);
        stateless_writer.reader_locator_add(locator2);
        stateless_writer.reader_locator_remove(&locator2);

        // assert!(stateless_writer
        //     .reader_locators()
        //     .contains(&reader_locator1));
        // assert!(!stateless_writer
        //     .reader_locators()
        //     .contains(&reader_locator2));
    }

    #[test]
    fn unsent_changes_reset() {
        let writer = MockWriter;
        let mut stateless_writer = StatelessWriter::new(writer);

        let locator1 = Locator::new(0, 100, [1; 16]);
        let locator2 = Locator::new(0, 200, [2; 16]);

        stateless_writer.reader_locator_add(locator1);
        stateless_writer.reader_locator_add(locator2);

        stateless_writer.unsent_changes_reset();
    }
}
