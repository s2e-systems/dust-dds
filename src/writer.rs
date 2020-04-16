use crate::cache::{CacheChange, HistoryCache};
use crate::endpoint::Endpoint;
use crate::entity::Entity;
use crate::proxy::ReaderProxy;
use crate::types::{
    ChangeKind, Duration, InstanceHandle, Locator, LocatorList, ParameterList, ReliabilityKind,
    SequenceNumber, TopicKind, GUID,
};

use std::collections::{HashMap, HashSet};

pub struct ReaderLocator {
    //requested_changes: HashSet<CacheChange>,
    // unsent_changes: SequenceNumber,
    expects_inline_qos: bool,
    highest_sequence_number_sent: SequenceNumber,
    sequence_numbers_requested: HashSet<SequenceNumber>,
}

impl ReaderLocator {
    pub fn new(expects_inline_qos: bool) -> Self {
        ReaderLocator {
            expects_inline_qos,
            highest_sequence_number_sent: 0,
            sequence_numbers_requested: HashSet::new(),
        }
    }
}

pub trait WriterOperations {
    fn writer(&mut self) -> &mut Writer;
    
    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Option<Vec<u8>>,
        inline_qos: Option<ParameterList>,
        handle: InstanceHandle,
    ) -> CacheChange {
        self.writer().new_change(kind, data, inline_qos, handle)
    }
}

pub struct Writer {
    /// Entity base class (contains the GUID)
    entity: Entity,

    // Endpoint base class:
    /// Used to indicate whether the Endpoint supports instance lifecycle management operations. Indicates whether the Endpoint is associated with a DataType that has defined some fields as containing the DDS key.
    topic_kind: TopicKind,
    /// The level of reliability supported by the Endpoint.
    reliability_level: ReliabilityKind,
    /// List of unicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty
    unicast_locator_list: LocatorList,
    /// List of multicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty.
    multicast_locator_list: LocatorList,

    //Writer class:
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
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: LocatorList,
        multicast_locator_list: LocatorList,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
    ) -> Self {
        Writer {
            entity: Entity { guid: guid },
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
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
            self.entity.guid,
            handle,
            self.last_change_sequence_number,
            inline_qos,
            data,
        )
    }

    pub fn history_cache(&mut self) -> &mut HistoryCache {
        &mut self.writer_cache
    }
}

pub struct StatelessWriter {
    writer: Writer,
    pub reader_locators: HashMap<Locator, ReaderLocator>,
}

impl StatelessWriter {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: LocatorList,
        multicast_locator_list: LocatorList,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
    ) -> Self {
        StatelessWriter {
            writer: Writer::new(
                guid,
                topic_kind,
                reliability_level,
                unicast_locator_list,
                multicast_locator_list,
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
            ReaderLocator::new(false /*expects_inline_qos*/),
        );
    }

    pub fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.remove(a_locator);
    }

    pub fn unsent_changes_reset(&mut self) {
        for (_, rl) in self.reader_locators.iter_mut() {
            // rl.set_highest_sequence_number_sent(0);
        }
    }

    pub fn next_unsent_change(&mut self, a_locator: Locator) -> Option<SequenceNumber>
    {
        let reader_locator = self.reader_locators.get_mut(&a_locator)?;

        let next_unsent_sequence_number = reader_locator.highest_sequence_number_sent + 1;
        if next_unsent_sequence_number > self.writer.last_change_sequence_number {
            None
        } else {
            reader_locator.highest_sequence_number_sent = next_unsent_sequence_number;
            Some(next_unsent_sequence_number)
        }
    }

    pub fn unsent_changes(&self, a_locator: Locator) -> HashSet<SequenceNumber> {
        let reader_locator = self.reader_locators.get(&a_locator).unwrap();

        let mut unsent_changes_set = HashSet::new();

        for unsent_sequence_number in reader_locator.highest_sequence_number_sent+1..=self.writer.last_change_sequence_number {
            unsent_changes_set.insert(unsent_sequence_number);
        }

        unsent_changes_set
    }

    pub fn requested_changes_set(&mut self, a_locator: Locator, req_seq_num_set: HashSet<SequenceNumber>) {
        let reader_locator = self.reader_locators.get_mut(&a_locator).unwrap();
        reader_locator.sequence_numbers_requested = req_seq_num_set;
    }

    // pub fn set_highest_sequence_number_sent(&mut self, n: SequenceNumber) {
    //     self.highest_sequence_number_sent = n;
    // }

    // fn next_unsent_change<'a, 'b>(reader_locator: &'b ReaderLocator, history_cache: &'a HistoryCache) -> Option<&'a CacheChange> {
    //     history_cache
    //         .get_changes()
    //         .iter()
    //         .filter(|cc| cc.get_sequence_number() > &reader_locator.highest_sequence_number_sent)
    //         .min()
    // }

    pub fn get_data_to_send(&mut self, a_locator: Locator) /*Vec of data */{
        unimplemented!();
    }
}


impl WriterOperations for StatelessWriter {
    fn writer(&mut self) -> &mut Writer {
        &mut self.writer
    }
}

pub struct StatefulWriter {
    writer: Writer,
    matched_readers: HashMap<GUID, ReaderProxy>,
}

impl StatefulWriter {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: LocatorList,
        multicast_locator_list: LocatorList,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
    ) -> Self {
        StatefulWriter {
            writer: Writer::new(
                guid,
                topic_kind,
                reliability_level,
                unicast_locator_list,
                multicast_locator_list,
                push_mode,
                heartbeat_period,
                nack_response_delay,
                nack_suppression_duration,
            ),
            matched_readers: HashMap::new(),
        }
    }

    pub fn matched_reader_add(&mut self, a_reader_proxy: ReaderProxy) {
        self.matched_readers
            .insert(*a_reader_proxy.remote_reader_guid(), a_reader_proxy);
    }

    pub fn matched_reader_remove(&mut self, a_reader_proxy: &ReaderProxy) {
        self.matched_readers
            .remove(a_reader_proxy.remote_reader_guid());
    }

    pub fn matched_reader_lookup(&self, a_reader_guid: &GUID) -> Option<&ReaderProxy> {
        self.matched_readers.get(a_reader_guid)
    }

    pub fn is_acked_by_all(&self, a_change: &CacheChange) -> bool {
        self.matched_readers
            .iter()
            .all(|(_, reader)| reader.is_acked(*a_change.get_sequence_number()))
    }
}

impl WriterOperations for StatefulWriter {
    fn writer(&mut self) -> &mut Writer {
        &mut self.writer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn test_writer_new_change() {
        let mut writer = StatelessWriter::new(
             GUID::new([0;12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
             TopicKind::WithKey,
             ReliabilityKind::BestEffort,
             vec![Locator::new(0, 7400, [0;16])], /*unicast_locator_list*/
             vec![], /*multicast_locator_list*/
             false, /*push_mode*/
             DURATION_ZERO,  /* heartbeat_period */
             DURATION_ZERO, /* nack_response_delay */
             DURATION_ZERO, /* nack_suppression_duration */
            );

        let cache_change_seq1 = writer.new_change(
            ChangeKind::Alive,
            Some(vec![1,2,3]), /*data*/
            None, /*inline_qos*/
            [1;16], /*handle*/
        );

        let cache_change_seq2 = writer.new_change(
            ChangeKind::NotAliveUnregistered,
            None, /*data*/
            None, /*inline_qos*/
            [1;16], /*handle*/
        );

        assert_eq!(cache_change_seq1.get_sequence_number(), &1);
        assert_eq!(cache_change_seq1.get_change_kind(), &ChangeKind::Alive);
        assert_eq!(cache_change_seq1.get_inline_qos(), &None);
        assert_eq!(cache_change_seq1.get_instance_handle(), &[1;16]);

        assert_eq!(cache_change_seq2.get_sequence_number(), &2);
        assert_eq!(cache_change_seq2.get_change_kind(), &ChangeKind::NotAliveUnregistered);
        assert_eq!(cache_change_seq2.get_inline_qos(), &None);
        assert_eq!(cache_change_seq2.get_instance_handle(), &[1;16]);
    }

    #[test]
    fn test_stateless_writer_unsent_changes() {
        let mut writer = StatelessWriter::new(
             GUID::new([0;12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
             TopicKind::WithKey,
             ReliabilityKind::BestEffort,
             vec![Locator::new(0, 7400, [0;16])], /*unicast_locator_list*/
             vec![], /*multicast_locator_list*/
             false, /*push_mode*/
             DURATION_ZERO,  /* heartbeat_period */
             DURATION_ZERO, /* nack_response_delay */
             DURATION_ZERO, /* nack_suppression_duration */
            );

        let locator = Locator::new(0, 7400, [1;16]);

        writer.reader_locator_add(locator);

        // Verify next unsent change without any changes created
        assert_eq!(writer.unsent_changes(locator), HashSet::new());
        assert_eq!(writer.next_unsent_change(locator), None);

        let cache_change_seq1 = writer.new_change(
            ChangeKind::Alive,
            Some(vec![1,2,3]), /*data*/
            None, /*inline_qos*/
            [1;16], /*handle*/
        );
        
        let cache_change_seq2 = writer.new_change(
            ChangeKind::NotAliveUnregistered,
            None, /*data*/
            None, /*inline_qos*/
            [1;16], /*handle*/
        );

        let hash_set_2_changes : HashSet<SequenceNumber> = [1,2].iter().cloned().collect();
        assert_eq!(writer.unsent_changes(locator), hash_set_2_changes);
        assert_eq!(writer.next_unsent_change(locator), Some(1));

        let hash_set_1_change : HashSet<SequenceNumber> = [2].iter().cloned().collect();
        assert_eq!(writer.unsent_changes(locator), hash_set_1_change);
        assert_eq!(writer.next_unsent_change(locator), Some(2));

        assert_eq!(writer.unsent_changes(locator), HashSet::new());    
        assert_eq!(writer.next_unsent_change(locator), None);
        assert_eq!(writer.next_unsent_change(locator), None);        

        let cache_change_seq3 = writer.new_change(
            ChangeKind::Alive,
            Some(vec![1,2,3]), /*data*/
            None, /*inline_qos*/
            [1;16], /*handle*/
        );

        assert_eq!(writer.next_unsent_change(locator), Some(3));
    }
}
