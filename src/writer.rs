use crate::cache::{CacheChange, HistoryCache};
use crate::endpoint::Endpoint;
use crate::proxy::ReaderProxy;
use crate::types::{
    ChangeKind, Duration, InstanceHandle, Locator, LocatorList, ParameterList, SequenceNumber, GUID,
};
use std::collections::{HashMap, HashSet};

pub struct ReaderLocator {
    //requested_changes: HashSet<CacheChange>,
    // unsent_changes: SequenceNumber,
    locator: Locator,
    expects_inline_qos: bool,
    highest_sequence_number_sent: SequenceNumber,
    sequence_numbers_requested: HashSet<SequenceNumber>,
}

impl ReaderLocator {
    pub fn new(locator: Locator, expects_inline_qos: bool) -> Self {
        ReaderLocator {
            locator,
            expects_inline_qos,
            highest_sequence_number_sent: 0,
            sequence_numbers_requested: HashSet::new(),
        }
    }

    pub fn next_unsent_change<'a>(
        &self,
        history_cache: &'a HistoryCache,
    ) -> Option<&'a CacheChange> {
        history_cache
            .get_changes()
            .iter()
            .filter(|cc| cc.get_sequence_number() > &self.highest_sequence_number_sent)
            .min()
    }
    pub fn next_requested_change<'a>(
        &self,
        history_cache: &'a HistoryCache,
    ) -> Option<&'a CacheChange> {
        let min_requested_sequence_number = self.sequence_numbers_requested.iter().min()?;
        history_cache
            .get_changes()
            .iter()
            .find(|cc| cc.get_sequence_number() == min_requested_sequence_number)
    }

    pub fn unsent_changes<'a>(&self, history_cache: &'a HistoryCache) -> HashSet<&'a CacheChange> {
        history_cache
            .get_changes()
            .iter()
            .filter(|cc| cc.get_sequence_number() > &self.highest_sequence_number_sent)
            .collect()
    }

    pub fn requested_changes<'a>(
        &self,
        history_cache: &'a HistoryCache,
    ) -> HashSet<&'a CacheChange> {
        let mut requested_changes = HashSet::new();
        for rsn in self.sequence_numbers_requested.iter() {
            if let Some(cc) = history_cache
                .get_changes()
                .iter()
                .find(|cc| cc.get_sequence_number() == rsn)
            {
                requested_changes.insert(cc);
            }
        }
        requested_changes
    }

    pub fn requested_changes_set(&mut self, req_seq_num_set: HashSet<SequenceNumber>) {
        for rsn in req_seq_num_set.iter() {
            self.sequence_numbers_requested.insert(*rsn);
        }
    }

    pub fn set_highest_sequence_number_sent(&mut self, n: SequenceNumber) {
        self.highest_sequence_number_sent = n;
    }
}

pub struct Writer {
    endpoint: Endpoint,
    push_mode: bool,
    heartbeat_period: Duration,
    nack_response_delay: Duration,
    nack_suppression_duration: Duration,
    last_change_sequence_number: SequenceNumber,
    writer_cache: HistoryCache,
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
            writer_cache: HistoryCache::new(),
            data_max_sized_serialized: None,
        }
    }

    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Option<Vec<u8>>,
        inline_qos: Option<ParameterList>,
        handle: InstanceHandle,
    ) -> CacheChange {
        self.last_change_sequence_number = self.last_change_sequence_number + 1;
        CacheChange::new(
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
    reader_locators: HashMap<Locator, ReaderLocator>,
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
            reader_locators: HashMap::new(),
        }
    }

    pub fn reader_locator_add(&mut self, a_locator: Locator) {
        self.reader_locators.insert(
            a_locator,
            ReaderLocator::new(a_locator, /*expects_inline_qos:*/ false),
        );
    }

    pub fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.remove(a_locator);
    }

    pub fn unsent_changes_reset(&mut self) {
        for (_, rl) in self.reader_locators.iter_mut() {
            rl.set_highest_sequence_number_sent(0);
        }
    }
}

pub struct StatefulWriter {
    writer: Writer,
    matched_readers: HashMap<GUID, ReaderProxy>,
}

impl StatefulWriter {
    pub fn new(
        endpoint: Endpoint,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
    ) -> Self {
        StatefulWriter {
            writer: Writer::new(
                endpoint,
                push_mode,
                heartbeat_period,
                nack_response_delay,
                nack_suppression_duration,
            ),
            matched_readers: HashMap::new(),
        }
    }

    pub fn matched_reader_add(&mut self, a_reader_proxy: ReaderProxy)
    {
        self.matched_readers.insert(*a_reader_proxy.remote_reader_guid(), a_reader_proxy);
    }

    pub fn matched_reader_remove(&mut self, a_reader_proxy: &ReaderProxy)
    {
        self.matched_readers.remove(a_reader_proxy.remote_reader_guid());
    }

    pub fn matched_reader_lookup(&self, a_reader_guid : &GUID) -> Option<&ReaderProxy>
    {
        self.matched_readers.get(a_reader_guid)
    }

    pub fn is_acked_by_all(&self, a_change: &CacheChange) -> bool {
        self.matched_readers.iter().all(|(_,reader)|reader.is_acked(*a_change.get_sequence_number()))
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
