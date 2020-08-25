use std::collections::VecDeque;
use std::sync::Mutex;

use crate::structure::history_cache::HistoryCache;
use crate::types::{ReliabilityKind, TopicKind, GUID, Locator, GuidPrefix };
use crate::types::constants::{ENTITYID_UNKNOWN};
use crate::messages::RtpsSubmessage;
use crate::messages::message_receiver::Receiver;
use crate::behavior::stateless_reader::BestEffortStatelessReaderBehavior;

pub struct StatelessReader {
    // Heartbeats are not relevant to stateless readers (only to readers),
    // hence the heartbeat_ members are not included here
    // heartbeat_response_delay: Duration,
    // heartbeat_suppression_duration: Duration,
    reader_cache: HistoryCache,
    expects_inline_qos: bool,
    // Enpoint members:
    /// Entity base class (contains the GUID)
    guid: GUID,
    /// Used to indicate whether the Endpoint supports instance lifecycle management operations. Indicates whether the Endpoint is associated with a DataType that has defined some fields as containing the DDS key.
    topic_kind: TopicKind,
    /// The level of reliability supported by the Endpoint.
    reliability_level: ReliabilityKind,
    /// List of unicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty
    unicast_locator_list: Vec<Locator>,
    /// List of multicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty.
    multicast_locator_list: Vec<Locator>,

    received_messages: Mutex<VecDeque<(GuidPrefix, RtpsSubmessage)>>,
}

impl StatelessReader {

    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        // reliability_level: ReliabilityKind, // Only BestEffort is supported
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        expects_inline_qos: bool,
    ) -> Self {
        StatelessReader {
            guid,
            topic_kind,
            reliability_level: ReliabilityKind::BestEffort,
            unicast_locator_list,
            multicast_locator_list,
            reader_cache: HistoryCache::new(),
            expects_inline_qos,
            received_messages: Mutex::new(VecDeque::new()),
        }
    }

    pub fn guid(&self) -> &GUID {
        &self.guid
    }

    pub fn reader_cache(&self) -> &HistoryCache {
        &self.reader_cache
    }

    pub fn unicast_locator_list(&self) -> &Vec<Locator> {
        &self.unicast_locator_list
    }

    pub fn multicast_locator_list(&self) -> &Vec<Locator> {
        &self.multicast_locator_list
    }

    pub fn run(&self) {
        match self.reliability_level {
            ReliabilityKind::BestEffort => BestEffortStatelessReaderBehavior::run(self),
            ReliabilityKind::Reliable => panic!("Reliable stateless reader not allowed!"),
        }
    }
}

impl Receiver for StatelessReader {
    fn push_receive_message(&self, source_guid_prefix: GuidPrefix, message: RtpsSubmessage) {
        self.received_messages.lock().unwrap().push_back((source_guid_prefix, message));
    }
    
    fn pop_receive_message(&self, _guid: &GUID) -> Option<(GuidPrefix, RtpsSubmessage)> {
        self.received_messages.lock().unwrap().pop_front()
    }

    fn is_submessage_destination(&self, src_locator: &Locator, _src_guid_prefix: &GuidPrefix, submessage: &RtpsSubmessage) -> bool {
        // The stateless reader receives only Data and Gap messages
        let reader_guid_prefix = match submessage {
            RtpsSubmessage::Data(data) => data.reader_id(),
            RtpsSubmessage::Gap(gap) => gap.reader_id(),
            _ => return false,
        }.0;

        // Messages are received by the stateless reader if they come from the expected source locator and
        // if the destination entity_id matches the reader id or if it is unknown
        if (self.unicast_locator_list().iter().find(|&loc| loc == src_locator).is_some() || self.multicast_locator_list().iter().find(|&loc| loc == src_locator).is_some()) 
        && (self.guid().entity_id() == reader_guid_prefix || reader_guid_prefix == ENTITYID_UNKNOWN) {
                true
        } else {
            false
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use crate::types::constants::*;
    use crate::messages::{ParameterList, Endianness };
    use crate::messages::submessages::Data;
    use crate::messages::submessages::data_submessage::Payload;
    use crate::inline_qos_types::{KeyHash};

    #[test]
    fn best_effort_stateless_reader_run() {
        let reader = StatelessReader::new(
            GUID::new([0;12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER),
            TopicKind::WithKey,
            vec![Locator::new(0, 7400, [0;16])],
            vec![],
            false,
           );

        let mut inline_qos = ParameterList::new();
        inline_qos.push(KeyHash([1;16]));

        let data1 = Data::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN,
            ENTITYID_UNKNOWN,
            1,
            Some(inline_qos),
            Payload::Data(vec![0,1,2]),
        );

        reader.push_receive_message([2;12], RtpsSubmessage::Data(data1));

        assert_eq!(reader.reader_cache().changes().len(), 0);
        // let message = RtpsMessage::new(, submessages);
        
        reader.run();

        assert_eq!(reader.reader_cache().changes().len(), 1);
    }
}
