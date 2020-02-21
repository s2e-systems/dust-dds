use crate::cache::{CacheChange, HistoryCache};
use crate::endpoint::Endpoint;
use crate::types::{
    ChangeKind, Duration, InstanceHandle, Locator, LocatorList, ParameterList, SequenceNumber,
};
use std::collections::HashSet;

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
