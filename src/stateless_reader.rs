use std::collections::VecDeque;
use std::sync::Mutex;

use crate::cache::HistoryCache;
use crate::types::{ReliabilityKind, TopicKind, GUID, Locator, GuidPrefix };
use crate::messages::receiver::ReaderReceiveMessage;
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

    received_messages: Mutex<VecDeque<(GuidPrefix, ReaderReceiveMessage)>>,
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

    pub fn history_cache(&self) -> &HistoryCache {
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

    pub fn push_receive_message(&self, source_guid_prefix: GuidPrefix, message: ReaderReceiveMessage) {
        self.received_messages.lock().unwrap().push_back((source_guid_prefix, message));
    }

    pub fn pop_receive_message(&self) -> Option<(GuidPrefix, ReaderReceiveMessage)> {
        self.received_messages.lock().unwrap().pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use crate::types::constants::*;
    use crate::messages::{Data, Payload, Endianness, ParameterList };
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

        reader.push_receive_message([2;12], ReaderReceiveMessage::Data(data1));

        assert_eq!(reader.history_cache().changes().len(), 0);
        // let message = RtpsMessage::new(, submessages);
        
        reader.run();

        assert_eq!(reader.history_cache().changes().len(), 1);
    }
}
