use crate::implementation::data_representation_builtin_endpoints::discovered_reader_data::ReaderProxy;

use super::{
    cache_change::RtpsCacheChange,
    reader_locator::RtpsReaderLocator,
    stateful_writer::{TransportWriter, WriterHistoryCache},
    types::{DurabilityKind, Guid, Locator, ReliabilityKind, SequenceNumber},
};

pub struct RtpsStatelessWriter {
    guid: Guid,
    changes: Vec<RtpsCacheChange>,
    reader_locators: Vec<RtpsReaderLocator>,
}

impl RtpsStatelessWriter {
    pub fn new(guid: Guid) -> Self {
        Self {
            guid,
            changes: Vec::new(),
            reader_locators: Vec::new(),
        }
    }

    pub fn guid(&self) -> Guid {
        self.guid
    }

    pub fn reader_locator_add(&mut self, locator: Locator) {
        self.reader_locators
            .push(RtpsReaderLocator::new(locator, false));
    }
}

impl TransportWriter for RtpsStatelessWriter {
    fn get_history_cache(&mut self) -> &mut dyn WriterHistoryCache {
        self
    }

    fn add_matched_reader(
        &mut self,
        reader_proxy: ReaderProxy,
        _reliability_kind: ReliabilityKind,
        _durability_kind: DurabilityKind,
    ) {
        for unicast_locator in reader_proxy.unicast_locator_list {
            self.reader_locator_add(unicast_locator);
        }
        for multicast_locator in reader_proxy.multicast_locator_list {
            self.reader_locator_add(multicast_locator);
        }
    }

    fn delete_matched_reader(&mut self, _reader_guid: Guid) {
        // Do nothing
    }
}

impl WriterHistoryCache for RtpsStatelessWriter {
    fn add_change(&mut self, cache_change: RtpsCacheChange) {
        self.changes.push(cache_change);
    }

    fn remove_change(&mut self, sequence_number: SequenceNumber) {
        self.changes
            .retain(|cc| cc.sequence_number() != sequence_number);
    }

    fn get_changes(&self) -> &[RtpsCacheChange] {
        &self.changes
    }
}
