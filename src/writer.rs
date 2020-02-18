use crate::cache::{
    CacheChange, CacheChangeOperations, HistoryCache, WriterCacheChange, WriterHistoryCache,
};
use crate::endpoint::Endpoint;
use crate::types::{
    ChangeKind, Duration, InstanceHandle, Locator, LocatorList, ParameterList, SequenceNumber,
};
use std::collections::HashSet;

pub struct ReaderLocator<'a> {
    requested_changes: HashSet<CacheChange>,
    unsent_changes: SequenceNumber,
    locator: Locator,
    expects_inline_qos: bool,
    cache_changes: &'a WriterHistoryCache,
}

impl<'a> ReaderLocator<'a> {
    pub fn new(
        locator: Locator,
        expects_inline_qos: bool,
        cache_changes: &'a WriterHistoryCache,
    ) -> Self {
        let mut unsent_changes = HashSet::new();

        for (_, change) in cache_changes.get_changes().iter() {
            unsent_changes.insert(change.cache_change().clone());
        }

        ReaderLocator {
            requested_changes: HashSet::new(),
            unsent_changes: 0,
            locator,
            expects_inline_qos,
            cache_changes,
        }
    }

    pub fn next_unsent_change(&self) -> Option<SequenceNumber> {
        unimplemented!();
        // let cache_change_lock = self.cache_changes.get_changes().lock().unwrap();
        // let min_unsent_cache_change = cache_change_lock.iter()
        //     .filter(|x| x.1.cache_change.sequence_number > self.unsent_changes).min();

        // match min_unsent_cache_change {
        //     Some((_,cache_change)) => Some(cache_change.cache_change.sequence_number),
        //     None => None,
        // }
    }
}

pub struct Writer {
    endpoint: Endpoint,
    push_mode: bool,
    heartbeat_period: Duration,
    nack_response_delay: Duration,
    nack_suppression_duration: Duration,
    last_change_sequence_number: SequenceNumber,
    writer_cache: WriterHistoryCache,
    data_max_sized_serialized: Option<i32>,
}

impl Writer {
    pub fn new(
        endpoint: Endpoint,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
    ) -> Self {
        Writer {
            endpoint,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            last_change_sequence_number: 0,
            writer_cache: WriterHistoryCache::new(),
            data_max_sized_serialized: None,
        }
    }

    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Option<Vec<u8>>,
        inline_qos: Option<ParameterList>,
        handle: InstanceHandle,
    ) -> WriterCacheChange {
        self.last_change_sequence_number = self.last_change_sequence_number + 1;
        WriterCacheChange::new(
            kind,
            *self.endpoint.guid(),
            handle,
            self.last_change_sequence_number,
            inline_qos,
            data,
        )
    }
}

struct StatelessWriter {
    writer: Writer,
    reader_locators: HashSet<Locator>,
}

impl StatelessWriter {
    pub fn new(
        endpoint: Endpoint,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
    ) -> Self {
        StatelessWriter {
            writer: Writer::new(
                endpoint,
                push_mode,
                heartbeat_period,
                nack_response_delay,
                nack_suppression_duration,
            ),
            reader_locators: HashSet::new(),
        }
    }

    pub fn reader_locator_add(&mut self, a_locator: Locator) {
        self.reader_locators.insert(a_locator);
    }

    pub fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.remove(a_locator);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer_new_change() {
        // let endpoint = Endpoint::new()
    }
}
