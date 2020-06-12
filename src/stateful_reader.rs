use std::collections::{BTreeSet, HashMap, };
use crate::types::{Locator, ReliabilityKind, SequenceNumber, TopicKind, GUID, };
use crate::behavior::types::Duration;

pub struct WriterProxy {
    remote_writer_guid: GUID,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    // data_max_size_serialized: Long,
    // changes_from_writer: CacheChange[*],     
    // remoteGroupEntityId: EntityId_t,

    max_available_change: SequenceNumber,
    unknown_changes: BTreeSet<SequenceNumber>,
    lost_changes: BTreeSet<SequenceNumber>,
    missing_changes: BTreeSet<SequenceNumber>,
}

impl WriterProxy {
    pub fn new(
        remote_writer_guid: GUID,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>, 
        ) -> Self {
            Self {
                remote_writer_guid,
                unicast_locator_list,
                multicast_locator_list,
                max_available_change: SequenceNumber(0),
                unknown_changes: BTreeSet::new(),
                lost_changes: BTreeSet::new(),
                missing_changes: BTreeSet::new(),
        }
    }

    pub fn available_changes_max(&self) -> &SequenceNumber {
        &self.max_available_change
    }

    pub fn irrelevant_change_set(&mut self, _a_seq_num: SequenceNumber) {
        todo!()
    }

    pub fn lost_changes_update(&mut self, first_available_seq_num: SequenceNumber) {
        let remaining_unknown = self.unknown_changes.split_off(&first_available_seq_num);
        self.lost_changes.append(&mut self.unknown_changes);
        self.unknown_changes = remaining_unknown;
        
        let remaining_missing = self.missing_changes.split_off(&first_available_seq_num);
        self.lost_changes.append(&mut self.missing_changes);
        self.missing_changes = remaining_missing;

        if first_available_seq_num > self.max_available_change {
            for seq_num in self.max_available_change.0 .. first_available_seq_num.0 {
                self.lost_changes.insert(SequenceNumber(seq_num));
            }
            self.max_available_change = first_available_seq_num;
        }
    }

    pub fn missing_changes(&self) -> &BTreeSet<SequenceNumber> {
        &self.missing_changes
    }

    pub fn missing_changes_update(&mut self, last_available_seq_num: SequenceNumber) {
        let remaining_unknown = self.unknown_changes.split_off(&last_available_seq_num);
        self.lost_changes.append(&mut self.unknown_changes);
        self.unknown_changes = remaining_unknown;

        if last_available_seq_num > self.max_available_change {
            for seq_num in self.max_available_change.0 ..= last_available_seq_num.0 {
                self.lost_changes.insert(SequenceNumber(seq_num));
            }
            self.max_available_change = last_available_seq_num + 1 ;
        }
    }

    pub fn received_change_set(&mut self, a_seq_num: SequenceNumber) {
        if a_seq_num > self.max_available_change{
            for seq_num in  self.max_available_change.0+1 .. a_seq_num.0 {
                self.unknown_changes.insert(SequenceNumber(seq_num));
            }
            self.max_available_change = a_seq_num;
        } else {
            self.unknown_changes.remove(&a_seq_num);
        }
    }
}

pub struct StatefulReader {
    // From Entity base class
    guid: GUID,
    // entity: Entity,

    // From Endpoint base class:
    topic_kind: TopicKind,
    reliability_level: ReliabilityKind,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,

    // From Reader base class:
    expects_inline_qos: bool,
    heartbeat_response_delay: Duration,

    // Fields
    matched_writers: HashMap<GUID, WriterProxy>,
}

impl StatefulReader {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        expects_inline_qos: bool,
        heartbeat_response_delay: Duration,        
        ) -> Self {
        Self {
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            heartbeat_response_delay,       
            matched_writers: HashMap::new(),
        }
    }

    pub fn matched_writer_add(&mut self, a_writer_proxy: WriterProxy) {
        self.matched_writers.insert(a_writer_proxy.remote_writer_guid, a_writer_proxy);
    }

    pub fn matched_writer_remove(&mut self, a_writer_proxy: &WriterProxy) {
        self.matched_writers.remove(&a_writer_proxy.remote_writer_guid);
    }
    
    pub fn matched_writer_lookup(&self, a_writer_guid: &GUID) -> Option<&WriterProxy> {
        self.matched_writers.get(a_writer_guid)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::GuidPrefix;
    use crate::types::constants::ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER;

    #[test]
    fn received_changes_set_sequential_ordered() {
        let remote_writer_guid = GUID::new(GuidPrefix([1;12]), ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER );
        let mut writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);

        // Originally the writer proxy doesn't know of any available changes
        assert_eq!(writer_proxy.available_changes_max(), &SequenceNumber(0));

        // If the next sequence number is received the unknown changes are empty
        writer_proxy.received_change_set(SequenceNumber(1));
        assert_eq!(writer_proxy.available_changes_max(), &SequenceNumber(1));
        assert!(writer_proxy.unknown_changes.is_empty());

        // If the next sequence number is received the unknown changes are empty
        writer_proxy.received_change_set(SequenceNumber(2));
        assert_eq!(writer_proxy.available_changes_max(), &SequenceNumber(2));
        assert!(writer_proxy.unknown_changes.is_empty());
    }

    #[test]
    fn received_changes_set_unordered() {
        let remote_writer_guid = GUID::new(GuidPrefix([1;12]), ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER );
        let mut writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);

        // Originally the writer proxy doesn't know of any available changes
        assert_eq!(writer_proxy.available_changes_max(), &SequenceNumber(0));

        // If the next sequence number jumps two number then change 1 and 2 are unknown
        writer_proxy.received_change_set(SequenceNumber(3));
        assert_eq!(writer_proxy.available_changes_max(), &SequenceNumber(3));
        assert_eq!(writer_proxy.unknown_changes, [SequenceNumber(1), SequenceNumber(2)].iter().cloned().collect());

        // If sequence number 2 is received the only unknown change is 1
        writer_proxy.received_change_set(SequenceNumber(2));
        assert_eq!(writer_proxy.available_changes_max(), &SequenceNumber(3));
        assert_eq!(writer_proxy.unknown_changes, [SequenceNumber(1)].iter().cloned().collect());

        // If sequence number 1 is received after there are no more unknown changes
        writer_proxy.received_change_set(SequenceNumber(1));
        assert_eq!(writer_proxy.available_changes_max(), &SequenceNumber(3));
        assert!(writer_proxy.unknown_changes.is_empty());
    }
}
