use std::collections::{BTreeSet, HashMap, };
use crate::types::{Locator, ReliabilityKind, SequenceNumber, TopicKind, GUID, };
use crate::behavior_types::Duration;

pub struct WriterProxy {
    remote_writer_guid: GUID,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    // data_max_size_serialized: Long,
    // changes_from_writer: CacheChange[*],     
    // remoteGroupEntityId: EntityId_t,
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
        }
    }

    pub fn available_changes_max(&self) -> &SequenceNumber {
        todo!()
    }

    pub fn irrelevant_change_set(&mut self, a_seq_num: SequenceNumber) {
        todo!()
    }

    pub fn lost_changes_update(&mut self, first_available_seq_num: SequenceNumber) {
        todo!()
    }

    pub fn missing_changes(&self) -> &BTreeSet<SequenceNumber> {
        todo!()
    }

    pub fn missing_changes_update(&mut self, last_available_seq_num: SequenceNumber) {
        todo!()
    }

    pub fn received_change_set(&mut self, a_seq_num: SequenceNumber) {
        todo!()
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
