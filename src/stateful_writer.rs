use std::collections::{BTreeSet, HashMap};

use crate::types::{ChangeKind, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind, GUID, };
use crate::behavior_types::Duration;
use crate::messages::types::Count;
use crate::serdes::EndianessFlag;
use crate::cache::{CacheChange, HistoryCache};
use crate::inline_qos::{InlineQosParameter, InlineQosParameterList, };
use crate::inline_qos_types::{KeyHash, StatusInfo, };
use crate::messages::{Data, InfoTs, Payload, RtpsMessage, RtpsSubmessage, Header, };
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

    pub fn next_unsent_change(&mut self, a_reader_guid: &GUID) -> Option<SequenceNumber> {
        let reader_proxy = self.matched_readers.get_mut(a_reader_guid).unwrap();

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

    fn has_unsent_changes(&self, a_reader_guid: &GUID) -> bool {
        let reader_proxy = self.matched_readers.get(a_reader_guid).unwrap();

        if reader_proxy.highest_sequence_number_sent + 1 >= self.last_change_sequence_number {
            false
        } else {
            true
        }
    }

    pub fn get_data_to_send(&mut self, a_reader_guid: &GUID) -> RtpsMessage {
        let mut message = RtpsMessage::new(
            Header::new(*self.guid.prefix()), 
            Vec::new() );

        if self.has_unsent_changes(a_reader_guid) {
            let time = Time::now();
            let infots = InfoTs::new(Some(time), EndianessFlag::LittleEndian);

            message.push(RtpsSubmessage::InfoTs(infots));

            while let Some(next_unsent_seq_num) = self.next_unsent_change(a_reader_guid) {
                if let Some(cache_change) = self
                    .writer_cache
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
                        ENTITYID_UNKNOWN,
                        *self.guid.entity_id(),
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
        }

        message
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
    fn test_writer_new_change() {
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
    fn test_best_effort_statefult_writer_get_data_to_send() {
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
        let writer_data = writer.get_data_to_send(&reader_guid);
        assert_eq!(writer_data.submessages().len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &writer_data.submessages()[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Data(data_message_1) = &writer_data.submessages()[1] {
            assert_eq!(data_message_1.reader_id(), &ENTITYID_UNKNOWN);
            assert_eq!(data_message_1.writer_id(), &ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
            assert_eq!(data_message_1.writer_sn(), &SequenceNumber(1));
            assert_eq!(data_message_1.serialized_payload(), &Some(SerializedPayload(vec![1, 2, 3])));

        } else {
            panic!("Wrong message type");
        };

        if let RtpsSubmessage::Data(data_message_2) = &writer_data.submessages()[2] {
            assert_eq!(data_message_2.reader_id(), &ENTITYID_UNKNOWN);
            assert_eq!(data_message_2.writer_id(), &ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
            assert_eq!(data_message_2.writer_sn(), &SequenceNumber(2));
            assert_eq!(data_message_2.serialized_payload(), &Some(SerializedPayload(vec![4, 5, 6])));
        } else {
            panic!("Wrong message type");
        };

        // Test that nothing more is sent after the first time
        let writer_data = writer.get_data_to_send(&reader_guid);
        assert_eq!(writer_data.submessages().len(), 0);
    }

}