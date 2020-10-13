use std::collections::{VecDeque};

use rust_dds_interface::qos::DataReaderQos;

use crate::structure::HistoryCache;
use crate::types::{ReliabilityKind, TopicKind, GUID, Locator, GuidPrefix };
use crate::types::constants::ENTITYID_UNKNOWN;
use crate::messages::RtpsSubmessage;
use crate::messages::message_receiver::Receiver;
use crate::messages::submessages::{Data, };
use crate::behavior::cache_change_from_data;


pub struct StatelessReader {
    // From RTPS Entity
    guid: GUID,

    // From RTPS Enpoint:    
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    reliability_level: ReliabilityKind,
    topic_kind: TopicKind,

    // From RTPS Reader:
    // Heartbeats are not relevant to stateless readers (only to stateful readers),
    // hence the heartbeat_ members are not included here
    // heartbeat_response_delay: Duration,
    // heartbeat_suppression_duration: Duration,
    reader_cache: HistoryCache,
    expects_inline_qos: bool,

    // Additional field:
    received_messages: VecDeque<(GuidPrefix, RtpsSubmessage)>,
}

impl StatelessReader {

    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        // reliability_level: ReliabilityKind, // Only BestEffort is supported
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        reader_qos: &DataReaderQos,
    ) -> Self {

        let expects_inline_qos = false;

        StatelessReader {
            guid,
            topic_kind,
            reliability_level: ReliabilityKind::BestEffort,
            unicast_locator_list,
            multicast_locator_list,
            reader_cache: HistoryCache::new(&reader_qos.resource_limits),
            expects_inline_qos,
            received_messages: VecDeque::new(),
        }
    }

    pub fn run(&mut self) {
        self.waiting_state();
    }


    fn waiting_state(&mut self) {
        while let Some((guid_prefix, received_message)) =  self.received_messages.pop_front() {
            match received_message {
                RtpsSubmessage::Data(data) => self.transition_t2(guid_prefix, data),
                _ => (),
            };
        }
    }

    fn transition_t2(&mut self, guid_prefix: GuidPrefix, data: Data) {
        let cache_change = cache_change_from_data(data, &guid_prefix);
        self.reader_cache.add_change(cache_change).unwrap();
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
}

impl Receiver for StatelessReader {
    fn push_receive_message(&mut self, src_locator: Locator, source_guid_prefix: GuidPrefix, submessage: RtpsSubmessage) {
        assert!(self.is_submessage_destination(&src_locator, &source_guid_prefix, &submessage));

        self.received_messages.push_back((source_guid_prefix, submessage));
    }

    fn is_submessage_destination(&self, src_locator: &Locator, _src_guid_prefix: &GuidPrefix, submessage: &RtpsSubmessage) -> bool {
        let reader_id = match submessage {
            RtpsSubmessage::Data(data) => data.reader_id(),
            _ => return false,
        };
        let is_in_locator_lists = self.multicast_locator_list.contains(src_locator) || self.unicast_locator_list.contains(src_locator);
        is_in_locator_lists && (self.guid.entity_id() == reader_id || reader_id == ENTITYID_UNKNOWN)
    }   
 
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use crate::types::constants::*;
    use crate::serialized_payload::ParameterList;
    use crate::messages::Endianness;
    use crate::messages::submessages::Data;
    use crate::messages::submessages::data_submessage::Payload;
    use crate::inline_qos_types::KeyHash;

    #[test]
    fn best_effort_stateless_reader_run() {
        let data_reader_qos = DataReaderQos::default();
        let mut reader = StatelessReader::new(
            GUID::new([0;12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER),
            TopicKind::WithKey,
            vec![Locator::new(0, 7400, [0;16])],
            vec![],
            &data_reader_qos
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

        reader.push_receive_message(LOCATOR_INVALID,[2;12], RtpsSubmessage::Data(data1));

        assert_eq!(reader.reader_cache().changes().len(), 0);
        // let message = RtpsMessage::new(, submessages);
        
        reader.run();

        assert_eq!(reader.reader_cache().changes().len(), 1);
    }
}
