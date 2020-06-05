use std::collections::{BTreeSet, HashMap};
use std::time::{Instant};
use std::convert::TryInto;

use crate::types::{ChangeKind, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind, GUID, };
use crate::behavior_types::Duration;
use crate::messages::types::Count;
use crate::serdes::EndianessFlag;
use crate::cache::{CacheChange, HistoryCache};
use crate::inline_qos::{InlineQosParameter, InlineQosParameterList, };
use crate::inline_qos_types::{KeyHash, StatusInfo, };
use crate::messages::{Data, InfoTs, Heartbeat, Payload, RtpsMessage, RtpsSubmessage, };
use crate::types::constants::ENTITYID_UNKNOWN;
use crate::messages::types::{Time, };
use crate::serialized_payload::SerializedPayload;
use crate::messages::submessage_elements::ParameterList;

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

    time_last_sent_data: Instant,
    time_nack_received: Instant,
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
                time_last_sent_data: Instant::now(),
                time_nack_received: Instant::now(),
        }
    }

    fn next_unsent_change(&mut self, last_change_sequence_number: SequenceNumber) -> Option<SequenceNumber> {
        let next_unsent_sequence_number = self.highest_sequence_number_sent + 1;
        if next_unsent_sequence_number > last_change_sequence_number {
            None
        } else {
            self.highest_sequence_number_sent = next_unsent_sequence_number;
            Some(next_unsent_sequence_number)
        }
    }

    pub fn unsent_changes(&self, last_change_sequence_number: SequenceNumber) -> BTreeSet<SequenceNumber> {
        let mut unsent_changes_set = BTreeSet::new();

        // The for loop is made with the underlying sequence number type because it is not possible to implement the Step trait on Stable yet
        for unsent_sequence_number in
            self.highest_sequence_number_sent.0 + 1..=last_change_sequence_number.0
        {
            unsent_changes_set.insert(SequenceNumber(unsent_sequence_number));
        }

        unsent_changes_set
    }

    pub fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        self.highest_sequence_number_acknowledged = committed_seq_num;
    }

    pub fn unacked_changes(&self, last_change_sequence_number: SequenceNumber) -> BTreeSet<SequenceNumber> {
        let mut unacked_changes_set = BTreeSet::new();

        // The for loop is made with the underlying sequence number type because it is not possible to implement the Step trait on Stable yet
        for unsent_sequence_number in
            self.highest_sequence_number_acknowledged.0 + 1..=last_change_sequence_number.0
        {
            unacked_changes_set.insert(SequenceNumber(unsent_sequence_number));
        }

        unacked_changes_set
    }

    pub fn requested_changes_set(&mut self, req_seq_num_set: BTreeSet<SequenceNumber>) {
        let mut new_set = req_seq_num_set;
        self.sequence_numbers_requested.append(&mut new_set);
    }

    pub fn requested_changes(&self) -> BTreeSet<SequenceNumber> {
        self.sequence_numbers_requested.clone()
    }

    pub fn next_requested_change(&mut self) -> Option<SequenceNumber> {
        let next_requested_change = *self.sequence_numbers_requested.iter().next()?;

        self.sequence_numbers_requested.remove(&next_requested_change);

        Some(next_requested_change)
    }

    fn run_best_effort(&mut self, writer_guid: &GUID, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) -> Option<RtpsMessage> {
        if !self.unsent_changes(last_change_sequence_number).is_empty() {
            Some(self.run_pushing_state(writer_guid, history_cache, last_change_sequence_number))
        } else {
            None
        }
    }

    fn run_reliable(&mut self, writer_guid: &GUID, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber, heartbeat_period: Duration, _nack_response_delay: Duration) -> Option<RtpsMessage> {
        if self.unacked_changes(last_change_sequence_number).is_empty() {
            // Idle
            None
        } else if !self.unsent_changes(last_change_sequence_number).is_empty() {
            Some(self.run_pushing_state(writer_guid, history_cache, last_change_sequence_number))
        } else if !self.unacked_changes(last_change_sequence_number).is_empty() {
            self.run_announcing_state(writer_guid, history_cache, last_change_sequence_number, heartbeat_period)
        } else {
            None
        }

        //TODO: Process the received message for setting the acknack
        // if !self.requested_changes().is_empty() {
        //     if self.duration_since_nack_received() > nack_response_delay {
        //         // self.run_repairing_state()
        //     }
        // }
    }

    fn run_pushing_state(&mut self, writer_guid: &GUID, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) -> RtpsMessage {
        let mut message = RtpsMessage::new(*writer_guid.prefix());

        let time = Time::now();
        let infots = InfoTs::new(Some(time), EndianessFlag::LittleEndian);
        message.push(RtpsSubmessage::InfoTs(infots));

        while let Some(next_unsent_seq_num) = self.next_unsent_change(last_change_sequence_number) {
            if let Some(cache_change) = history_cache
                .get_change_with_sequence_number(&next_unsent_seq_num)
            {
                let change_kind = *cache_change.change_kind();

                let mut inline_qos_parameter_list : InlineQosParameterList = ParameterList::new();

                let payload = match change_kind {
                    ChangeKind::Alive => {
                        inline_qos_parameter_list.push(InlineQosParameter::StatusInfo(StatusInfo::from(change_kind)));
                        inline_qos_parameter_list.push(InlineQosParameter::KeyHash(KeyHash(*cache_change.instance_handle())));
                        Payload::Data(SerializedPayload(cache_change.data().unwrap().to_vec()))
                    },
                    ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered | ChangeKind::AliveFiltered => {
                        inline_qos_parameter_list.push(InlineQosParameter::StatusInfo(StatusInfo::from(change_kind)));
                        Payload::Key(SerializedPayload(cache_change.instance_handle().to_vec()))
                    }
                };

                let data = Data::new(
                    EndianessFlag::LittleEndian.into(),
                    *self.remote_reader_guid.entity_id(),
                    *writer_guid.entity_id(),
                    *cache_change.sequence_number(),
                    Some(inline_qos_parameter_list), 
                    payload,
                );

                message.push(RtpsSubmessage::Data(data));
            } else {
                panic!("GAP not implemented yet");
                // let gap = Gap::new(ENTITYID_UNKNOWN /*reader_id*/,ENTITYID_UNKNOWN /*writer_id*/, 0 /*gap_start*/, BTreeMap::new() /*gap_list*/);

                // message.push(RtpsSubmessage::Gap(gap));
            }
        }

        self.time_last_sent_data_reset();

        message
    }

    fn run_announcing_state(&mut self, writer_guid: &GUID, history_cache: &HistoryCache,  last_change_sequence_number: SequenceNumber, heartbeat_period: Duration) -> Option<RtpsMessage> {

        if self.duration_since_last_sent_data() > heartbeat_period {
            let mut message = RtpsMessage::new(*writer_guid.prefix());

            let time = Time::now();
            let infots = InfoTs::new(Some(time), EndianessFlag::LittleEndian);

            message.push(RtpsSubmessage::InfoTs(infots));

            let first_sn = if let Some(seq_num) = history_cache.get_seq_num_min() {
                seq_num
            } else {
                last_change_sequence_number + 1
            };

            let heartbeat = Heartbeat::new(
                ENTITYID_UNKNOWN,
                *writer_guid.entity_id(),
                first_sn,
                last_change_sequence_number,
                self.heartbeat_count,
                true,
                false,
                EndianessFlag::LittleEndian,
            );

            message.push(RtpsSubmessage::Heartbeat(heartbeat));

            self.increment_heartbeat_count();
            self.time_last_sent_data_reset();
            
            Some(message)
        } else {
            None
        }
    }

    fn time_last_sent_data_reset(&mut self) {
        self.time_last_sent_data = Instant::now();
    }

    fn duration_since_last_sent_data(&self) -> Duration {
        self.time_last_sent_data.elapsed().try_into().unwrap()
    }

    fn duration_since_nack_received(&self) -> Duration {
        self.time_nack_received.elapsed().try_into().unwrap()
    }

    fn increment_heartbeat_count(&mut self) {
        self.heartbeat_count += 1;
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

    pub fn run(&mut self, a_reader_guid: &GUID, _received_message: Option<&RtpsMessage>) -> Option<RtpsMessage> {
        let reader_proxy = self.matched_readers.get_mut(a_reader_guid).unwrap();

        match self.reliability_level {
            ReliabilityKind::BestEffort => reader_proxy.run_best_effort(&self.guid, &self.writer_cache, self.last_change_sequence_number),
            ReliabilityKind::Reliable => reader_proxy.run_reliable(&self.guid, &self.writer_cache, self.last_change_sequence_number, self.heartbeat_period, self.nack_response_delay),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::*;
    use crate::behavior_types::constants::DURATION_ZERO;
    use crate::types::*;
    use std::thread::sleep;

    #[test]
    fn stateful_writer_new_change() {
        let mut writer = StatefulWriter::new(
            GUID::new(GuidPrefix([0; 12]), ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            vec![Locator::new(0, 7400, [0; 16])], /*unicast_locator_list*/
            vec![],                               /*multicast_locator_list*/
            false,                                /*push_mode*/
            DURATION_ZERO,                        /* heartbeat_period */
            DURATION_ZERO,                        /* nack_response_delay */
            DURATION_ZERO,                        /* nack_suppression_duration */
        );

        let cache_change_seq1 = writer.new_change(
            ChangeKind::Alive,
            Some(vec![1, 2, 3]), /*data*/
            None,                /*inline_qos*/
            [1; 16],             /*handle*/
        );

        let cache_change_seq2 = writer.new_change(
            ChangeKind::NotAliveUnregistered,
            None,    /*data*/
            None,    /*inline_qos*/
            [1; 16], /*handle*/
        );

        assert_eq!(cache_change_seq1.sequence_number(), &SequenceNumber(1));
        assert_eq!(cache_change_seq1.change_kind(), &ChangeKind::Alive);
        assert_eq!(cache_change_seq1.inline_qos(), &None);
        assert_eq!(cache_change_seq1.instance_handle(), &[1; 16]);

        assert_eq!(cache_change_seq2.sequence_number(), &SequenceNumber(2));
        assert_eq!(
            cache_change_seq2.change_kind(),
            &ChangeKind::NotAliveUnregistered
        );
        assert_eq!(cache_change_seq2.inline_qos(), &None);
        assert_eq!(cache_change_seq2.instance_handle(), &[1; 16]);
    }

    #[test]
    fn reader_proxy_unsent_changes_operations() {
        let remote_reader_guid = GUID::new(GuidPrefix([1,2,3,4,5,6,7,8,9,10,11,12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        // Check that a reader proxy that has no changes marked as sent doesn't reports no changes
        let no_change_in_writer_sequence_number = SequenceNumber(0);
        assert_eq!(reader_proxy.next_unsent_change(no_change_in_writer_sequence_number), None);
        assert!(reader_proxy.unsent_changes(no_change_in_writer_sequence_number).is_empty());

        // Check the behaviour for a reader proxy starting with no changes sent and two changes in writer
        let two_changes_in_writer_sequence_number = SequenceNumber(2);
        assert_eq!(reader_proxy.unsent_changes(two_changes_in_writer_sequence_number).len(), 2);
        assert!(reader_proxy.unsent_changes(two_changes_in_writer_sequence_number).contains(&SequenceNumber(1)));
        assert!(reader_proxy.unsent_changes(two_changes_in_writer_sequence_number).contains(&SequenceNumber(2)));

        assert_eq!(reader_proxy.next_unsent_change(two_changes_in_writer_sequence_number), Some(SequenceNumber(1)));
        assert_eq!(reader_proxy.unsent_changes(two_changes_in_writer_sequence_number).len(), 1);
        assert!(reader_proxy.unsent_changes(two_changes_in_writer_sequence_number).contains(&SequenceNumber(2)));

        assert_eq!(reader_proxy.next_unsent_change(two_changes_in_writer_sequence_number), Some(SequenceNumber(2)));
        assert!(reader_proxy.unsent_changes(two_changes_in_writer_sequence_number).is_empty());

        assert_eq!(reader_proxy.next_unsent_change(two_changes_in_writer_sequence_number), None);
    }

    #[test]
    fn reader_proxy_requested_changes_operations() {
        let remote_reader_guid = GUID::new(GuidPrefix([1,2,3,4,5,6,7,8,9,10,11,12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        // Check that a reader proxy that has no changes marked as sent doesn't reports no changes
        assert!(reader_proxy.requested_changes().is_empty());
        assert_eq!(reader_proxy.next_requested_change(), None);

        // Insert some requested changes
        let mut requested_changes = BTreeSet::new();
        requested_changes.insert(SequenceNumber(2));
        requested_changes.insert(SequenceNumber(3));
        requested_changes.insert(SequenceNumber(6));
        reader_proxy.requested_changes_set(requested_changes);

        // Verify that the changes were correctly inserted and are removed in the correct order
        assert_eq!(reader_proxy.requested_changes().len(), 3);
        assert!(reader_proxy.requested_changes().contains(&SequenceNumber(2)));
        assert!(reader_proxy.requested_changes().contains(&SequenceNumber(3)));
        assert!(reader_proxy.requested_changes().contains(&SequenceNumber(6)));

        assert_eq!(reader_proxy.next_requested_change(), Some(SequenceNumber(2)));
        assert_eq!(reader_proxy.next_requested_change(), Some(SequenceNumber(3)));
        assert_eq!(reader_proxy.requested_changes().len(), 1);
        assert!(reader_proxy.requested_changes().contains(&SequenceNumber(6)));
        assert_eq!(reader_proxy.next_requested_change(), Some(SequenceNumber(6)));
        assert_eq!(reader_proxy.next_requested_change(), None);


        // Verify that if requested changes are inserted when there are already requested changes
        // that the sets are not replaced
        let mut requested_changes_1 = BTreeSet::new();
        requested_changes_1.insert(SequenceNumber(2));
        requested_changes_1.insert(SequenceNumber(3));
        reader_proxy.requested_changes_set(requested_changes_1);

        let mut requested_changes_2 = BTreeSet::new();
        requested_changes_2.insert(SequenceNumber(2)); // Repeated number
        requested_changes_2.insert(SequenceNumber(7));
        requested_changes_2.insert(SequenceNumber(9));
        reader_proxy.requested_changes_set(requested_changes_2);
        
        assert_eq!(reader_proxy.requested_changes().len(), 4);
        assert!(reader_proxy.requested_changes().contains(&SequenceNumber(2)));
        assert!(reader_proxy.requested_changes().contains(&SequenceNumber(3)));
        assert!(reader_proxy.requested_changes().contains(&SequenceNumber(7)));
        assert!(reader_proxy.requested_changes().contains(&SequenceNumber(9)));
    }

    #[test]
    fn reader_proxy_unacked_changes_operations() {
        let remote_reader_guid = GUID::new(GuidPrefix([1,2,3,4,5,6,7,8,9,10,11,12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let no_change_in_writer = SequenceNumber(0);
        assert!(reader_proxy.unacked_changes(no_change_in_writer).is_empty());

        let two_changes_in_writer = SequenceNumber(2);
        assert_eq!(reader_proxy.unacked_changes(two_changes_in_writer).len(), 2);
        assert!(reader_proxy.unacked_changes(two_changes_in_writer).contains(&SequenceNumber(1)));
        assert!(reader_proxy.unacked_changes(two_changes_in_writer).contains(&SequenceNumber(2)));

        reader_proxy.acked_changes_set(SequenceNumber(1));
        assert_eq!(reader_proxy.unacked_changes(two_changes_in_writer).len(), 1);
        assert!(reader_proxy.unacked_changes(two_changes_in_writer).contains(&SequenceNumber(2)));
    }

    #[test]
    fn run_pushing_state_only_data_messages() {
        let writer_guid = GUID::new(GuidPrefix([2;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let mut history_cache = HistoryCache::new();

        let instance_handle = [1;16];

        let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(1), None, Some(vec![1,2,3]));
        let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(2), None, Some(vec![2,3,4]));
        history_cache.add_change(cache_change_seq1);
        history_cache.add_change(cache_change_seq2);
        let last_change_sequence_number  = SequenceNumber(2);

        let remote_reader_guid = GUID::new(GuidPrefix([1,2,3,4,5,6,7,8,9,10,11,12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let message = reader_proxy.run_pushing_state(&writer_guid, &history_cache, last_change_sequence_number);
        assert_eq!(message.submessages().len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &message.submessages()[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Data(data_message_1) = &message.submessages()[1] {
            assert_eq!(data_message_1.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_1.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_1.writer_sn(), &SequenceNumber(1));
            assert_eq!(data_message_1.serialized_payload(), &Some(SerializedPayload(vec![1, 2, 3])));

        } else {
            panic!("Wrong message type");
        };

        if let RtpsSubmessage::Data(data_message_2) = &message.submessages()[2] {
            assert_eq!(data_message_2.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_2.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_2.writer_sn(), &SequenceNumber(2));
            assert_eq!(data_message_2.serialized_payload(), &Some(SerializedPayload(vec![2, 3, 4])));
        } else {
            panic!("Wrong message type");
        };

    }

    #[test]
    fn run_pushing_state_only_gap_message() {
        let writer_guid = GUID::new(GuidPrefix([2;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let history_cache = HistoryCache::new();

        // Don't add any change to the history cache so that gap message has to be sent
        // let instance_handle = [1;16];


        // let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(1), None, Some(vec![1,2,3]));
        // let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(2), None, Some(vec![2,3,4]));
        // history_cache.add_change(cache_change_seq1);
        // history_cache.add_change(cache_change_seq2);

        let last_change_sequence_number  = SequenceNumber(2);

        let remote_reader_guid = GUID::new(GuidPrefix([1,2,3,4,5,6,7,8,9,10,11,12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let message = reader_proxy.run_pushing_state(&writer_guid, &history_cache, last_change_sequence_number);
        assert_eq!(message.submessages().len(), 2);
        if let RtpsSubmessage::InfoTs(message_1) = &message.submessages()[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Gap(_gap_message) = &message.submessages()[1] {

        } else {
            panic!("Wrong message type");
        };

       
    }
    
    #[test]
    fn best_effort_stateful_writer_run() {
        let mut writer = StatefulWriter::new(
            GUID::new(GuidPrefix([0; 12]), ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            vec![Locator::new(0, 7400, [0; 16])], 
            vec![],                               
            false,                                
            DURATION_ZERO,                        
            DURATION_ZERO,                        
            DURATION_ZERO,                        
        );

        let reader_guid = GUID::new(GuidPrefix([1;12]), ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
        let reader_proxy = ReaderProxy::new(reader_guid, vec![], vec![], false, true);

        writer.matched_reader_add(reader_proxy);

        let cache_change_seq1 = writer.new_change(
            ChangeKind::Alive,
            Some(vec![1, 2, 3]), 
            None,                
            [1; 16],             
        );

        let cache_change_seq2 = writer.new_change(
            ChangeKind::Alive,
            Some(vec![4, 5, 6]), 
            None,                
            [1; 16],             
        );

        writer.history_cache().add_change(cache_change_seq1);
        writer.history_cache().add_change(cache_change_seq2);

        // let reader_proxy = writer.matched_reader_lookup(& reader_guid).unwrap();
        let writer_data = writer.run(&reader_guid, None).unwrap();
        assert_eq!(writer_data.submessages().len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &writer_data.submessages()[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Data(data_message_1) = &writer_data.submessages()[1] {
            assert_eq!(data_message_1.reader_id(), &ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
            assert_eq!(data_message_1.writer_id(), &ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
            assert_eq!(data_message_1.writer_sn(), &SequenceNumber(1));
            assert_eq!(data_message_1.serialized_payload(), &Some(SerializedPayload(vec![1, 2, 3])));

        } else {
            panic!("Wrong message type");
        };

        if let RtpsSubmessage::Data(data_message_2) = &writer_data.submessages()[2] {
            assert_eq!(data_message_2.reader_id(), &ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
            assert_eq!(data_message_2.writer_id(), &ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
            assert_eq!(data_message_2.writer_sn(), &SequenceNumber(2));
            assert_eq!(data_message_2.serialized_payload(), &Some(SerializedPayload(vec![4, 5, 6])));
        } else {
            panic!("Wrong message type");
        };

        // Test that nothing more is sent after the first time
        let writer_data = writer.run(&reader_guid, None);
        assert_eq!(writer_data.is_none(), true);
    }

    
}