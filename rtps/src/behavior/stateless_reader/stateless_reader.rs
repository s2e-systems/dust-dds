use std::collections::{HashMap, };

use rust_dds_interface::qos::DataReaderQos;

use crate::structure::HistoryCache;
use crate::types::{ReliabilityKind, TopicKind, GUID, Locator, GuidPrefix };
use crate::messages::RtpsSubmessage;
use crate::messages::message_receiver::Receiver;
use super::writer_locator::WriterLocator;
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
    writer_locators: HashMap<Locator, WriterLocator>,
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
            writer_locators: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        for (_writer_guid, writer_locator) in self.writer_locators.iter_mut(){
            writer_locator.run(&self.reader_cache)
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
}

impl Receiver for StatelessReader {
    fn push_receive_message(&mut self, _src_locator: Locator, _source_guid_prefix: GuidPrefix, _message: RtpsSubmessage) {
        todo!()
    }

    fn is_submessage_destination(&self, _src_locator: &Locator, _src_guid_prefix: &GuidPrefix, _submessage: &RtpsSubmessage) -> bool {
        todo!()       
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
