use std::collections::{BTreeSet, HashMap};

use crate::types::{ChangeKind, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind, GUID, };
use crate::behavior_types::Duration;
use crate::messages::types::Count;
use crate::cache::{CacheChange, HistoryCache};
use crate::inline_qos::{InlineQosParameterList, };

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
    sequence_numbers_requested: BTreeSet<SequenceNumber>,
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
                sequence_numbers_requested: BTreeSet::new(),
                heartbeat_count: Count(0),
        }
    }
}

pub struct StatefulWriter {
    /// Entity base class (contains the GUID)
    guid: GUID,
    // entity: Entity,

    // Endpoint base class:
    /// Used to indicate whether the Endpoint supports instance lifecycle management operations. Indicates whether the Endpoint is associated with a DataType that has defined some fields as containing the DDS key.
    topic_kind: TopicKind,
    /// The level of reliability supported by the Endpoint.
    reliability_level: ReliabilityKind,
    /// List of unicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty
    unicast_locator_list: Vec<Locator>,
    /// List of multicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty.
    multicast_locator_list: Vec<Locator>,

    //Writer class:
    push_mode: bool,
    heartbeat_period: Duration,
    nack_response_delay: Duration,
    nack_suppression_duration: Duration,
    last_change_sequence_number: SequenceNumber,
    writer_cache: HistoryCache,
    data_max_sized_serialized: Option<i32>,

    matched_readers: HashMap<GUID, ReaderProxy>,
}

impl StatefulWriter {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,) -> Self {
        StatefulWriter {
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            last_change_sequence_number: SequenceNumber(0),
            writer_cache: HistoryCache::new(),
            data_max_sized_serialized: None,
            matched_readers: HashMap::new(),
        }
    }

    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Option<Vec<u8>>,
        inline_qos: Option<InlineQosParameterList>,
        handle: InstanceHandle,
    ) -> CacheChange {
        self.last_change_sequence_number = self.last_change_sequence_number + 1;
        CacheChange::new(
            kind,
            self.guid,
            handle,
            self.last_change_sequence_number,
            inline_qos,
            data,
        )
    }

    pub fn history_cache(&mut self) -> &mut HistoryCache {
        &mut self.writer_cache
    }

    pub fn matched_reader_add(&mut self, a_reader_proxy: ReaderProxy) {
        self.matched_readers.insert(a_reader_proxy.remote_reader_guid, a_reader_proxy);
    }

    pub fn matched_reader_remove(&mut self, a_reader_proxy: &ReaderProxy) {
        self.matched_readers.remove(&a_reader_proxy.remote_reader_guid);
    }
    
    pub fn matched_reader_lookup(&self, a_reader_guid: &GUID) -> Option<&ReaderProxy> {
        self.matched_readers.get(a_reader_guid)
    }

    pub fn is_acked_by_all(&self) -> bool {
        todo!()
    }

    pub fn next_unsent_change(&mut self, a_reader_proxy: &ReaderProxy) -> Option<SequenceNumber> {
        let reader_proxy = self.matched_readers.get_mut(&a_reader_proxy.remote_reader_guid).unwrap();

        let next_unsent_sequence_number = reader_proxy.highest_sequence_number_sent + 1;
        if next_unsent_sequence_number > self.last_change_sequence_number {
            None
        } else {
            reader_proxy.highest_sequence_number_sent = next_unsent_sequence_number;
            Some(next_unsent_sequence_number)
        }
    }

    pub fn unsent_changes(&self, a_reader_proxy: &ReaderProxy) -> BTreeSet<SequenceNumber> {
        let reader_proxy = self.matched_readers.get(&a_reader_proxy.remote_reader_guid).unwrap();

        let mut unsent_changes_set = BTreeSet::new();

        // The for loop is made with the underlying sequence number type because it is not possible to implement the Step trait on Stable yet
        for unsent_sequence_number in
            reader_proxy.highest_sequence_number_sent.0 + 1..=self.last_change_sequence_number.0
        {
            unsent_changes_set.insert(SequenceNumber(unsent_sequence_number));
        }

        unsent_changes_set
    }

    pub fn acked_changes_set(&mut self, a_reader_proxy: &ReaderProxy, committed_seq_num: SequenceNumber) {
        let reader_proxy = self.matched_readers.get_mut(&a_reader_proxy.remote_reader_guid).unwrap();

        reader_proxy.highest_sequence_number_acknowledged = committed_seq_num;
    }

    pub fn unacked_changes(&self, a_reader_proxy: &ReaderProxy) -> BTreeSet<SequenceNumber> {
        let reader_proxy = self.matched_readers.get(&a_reader_proxy.remote_reader_guid).unwrap();

        let mut unacked_changes_set = BTreeSet::new();

        // The for loop is made with the underlying sequence number type because it is not possible to implement the Step trait on Stable yet
        for unsent_sequence_number in
            reader_proxy.highest_sequence_number_acknowledged.0 + 1..=self.last_change_sequence_number.0
        {
            unacked_changes_set.insert(SequenceNumber(unsent_sequence_number));
        }

        unacked_changes_set
    }

    pub fn requested_changes_set(&mut self, a_reader_proxy: &ReaderProxy, req_seq_num_set: BTreeSet<SequenceNumber>) {
        let reader_proxy = self.matched_readers.get_mut(&a_reader_proxy.remote_reader_guid).unwrap();

        reader_proxy.sequence_numbers_requested = req_seq_num_set;
    }

    pub fn requested_changes(&self, a_reader_proxy: &ReaderProxy) -> BTreeSet<SequenceNumber> {
        let reader_proxy = self.matched_readers.get(&a_reader_proxy.remote_reader_guid).unwrap();

        reader_proxy.sequence_numbers_requested.clone()
    }

    pub fn next_requested_change(&mut self, a_reader_proxy: &ReaderProxy) -> Option<SequenceNumber> {
        let reader_proxy = self.matched_readers.get_mut(&a_reader_proxy.remote_reader_guid).unwrap();

        let next_requested_change = *reader_proxy.sequence_numbers_requested.iter().next()?;

        reader_proxy.sequence_numbers_requested.remove(&next_requested_change);

        Some(next_requested_change)
    }
}

