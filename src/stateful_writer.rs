use std::collections::HashSet;

use crate::types::{SequenceNumber, GUID, Locator};
use crate::messages::types::Count;

pub struct ReaderProxy {
    remote_reader_guid: GUID,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    expects_inline_qos: bool,
    is_active: bool,

    //requested_changes: HashSet<CacheChange>,
    // unsent_changes: SequenceNumber,
    highest_sequence_number_sent: SequenceNumber,
    highest_sequence_number_acknowledged: SequenceNumber,
    sequence_numbers_requested: HashSet<SequenceNumber>,
    heartbeat_count: Count,
}

impl PartialEq for ReaderProxy {
    fn eq(&self, other: &Self) -> bool {
        self.remote_reader_guid == other.remote_reader_guid
    }
}

impl Eq for ReaderProxy {}

impl ReaderProxy {
    pub fn new(
        remote_reader_guid: GUID,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        expects_inline_qos: bool,
        is_active: bool) -> Self {
            ReaderProxy {
                remote_reader_guid,
                unicast_locator_list,
                multicast_locator_list,
                expects_inline_qos,
                is_active,
                highest_sequence_number_sent: SequenceNumber(0),
                highest_sequence_number_acknowledged: SequenceNumber(0),
                sequence_numbers_requested: HashSet::new(),
                heartbeat_count: Count(0),
        }
    }

    pub fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        self.highest_sequence_number_acknowledged = committed_seq_num;
    }
}

impl ::core::hash::Hash for ReaderProxy {
    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
        ::core::hash::Hash::hash(&self.remote_reader_guid, state);
    }
}

pub struct StatefulWriter {
    matched_readers: HashSet<ReaderProxy>,
}

impl StatefulWriter {
    pub fn new() -> Self {
        StatefulWriter {
            matched_readers: HashSet::new(),
        }
    }

    pub fn matched_reader_add(&mut self, a_reader_proxy: ReaderProxy) {
        self.matched_readers.insert(a_reader_proxy);
    }

    pub fn matched_reader_remove(&mut self, a_reader_proxy: &ReaderProxy) {
        self.matched_readers.remove(a_reader_proxy);
    }
    
    pub fn matched_reader_lookup(&self, a_reader_guid: &GUID) -> Option<&ReaderProxy> {
        self.matched_readers
            .iter()
            .find(|&x| &x.remote_reader_guid == a_reader_guid)
    }

    pub fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}

