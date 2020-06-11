use crate::cache::HistoryCache;
use crate::types::{ReliabilityKind, TopicKind, GUID, Locator, };
use crate::messages::RtpsMessage;
use crate::behavior::StatelessReaderBehavior;

#[derive(Debug)]
pub enum StatelessReaderError {
    InlineQosNotFound,
    StatusInfoNotFound,
    KeyHashNotFound,
    InvalidStatusInfo,
    InvalidDataKeyFlagCombination,
    InvalidKeyHashPayload,
}

pub type StatelessReaderResult<T> = std::result::Result<T, StatelessReaderError>;

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
}

impl StatelessReader {

    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        expects_inline_qos: bool,
    ) -> Self {
        assert!(reliability_level == ReliabilityKind::BestEffort);
        StatelessReader {
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            reader_cache: HistoryCache::new(),
            expects_inline_qos,
        }
    }

    pub fn history_cache(&self) -> &HistoryCache {
        &self.reader_cache
    }

    pub fn run(&mut self, received_message: Option<&RtpsMessage>) {
        match self.reliability_level {
            ReliabilityKind::BestEffort => StatelessReaderBehavior::run_best_effort(&mut self.reader_cache, received_message),
            ReliabilityKind::Reliable => panic!("Reliable stateless reader not allowed!"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use crate::types::constants::*;
    use crate::serialized_payload::SerializedPayload;
    use crate::messages::submessage_elements::{Parameter, ParameterList, };
    use crate::messages::{Data, Payload, RtpsSubmessage };
    use crate::serdes::Endianness;
    use crate::inline_qos_types::{KeyHash};

    #[test]
    fn best_effort_stateless_reader_run() {
        let data1 = Data::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN,
            ENTITYID_UNKNOWN,
            SequenceNumber(1),
            Some(ParameterList::new(vec![Parameter::new(KeyHash([1;16]), Endianness::LittleEndian)])),
            Payload::Data(SerializedPayload(vec![0,1,2])),
        );

        let mut submessages = Vec::new();
        submessages.push(RtpsSubmessage::Data(data1));

        let mut reader = StatelessReader::new(
            GUID::new(GuidPrefix([0;12]), ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER),
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            vec![Locator::new(0, 7400, [0;16])],
            vec![],
            false,
           );

        assert_eq!(reader.history_cache().get_changes().len(), 0);
        let message = RtpsMessage::new(GuidPrefix([2;12]), submessages);
        
        reader.run(Some(&message));

        assert_eq!(reader.history_cache().get_changes().len(), 1);
    }
}
