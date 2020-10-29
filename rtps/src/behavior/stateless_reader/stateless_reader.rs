use std::collections::VecDeque;

use rust_dds_interface::qos::DataReaderQos;

use crate::structure::{HistoryCache, RtpsEndpoint, RtpsEntity};
use crate::types::{ReliabilityKind, TopicKind, GUID, Locator, GuidPrefix };
use crate::types::constants::ENTITYID_UNKNOWN;
use crate::messages::RtpsSubmessage;
use crate::messages::submessages::Data;
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
    input_queue: VecDeque<(GuidPrefix, RtpsSubmessage)>,
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
            input_queue: VecDeque::new(),
        }
    }

    pub fn run(&mut self) {
        self.waiting_state();
    }


    fn waiting_state(&mut self) {
        let popped_queue = self.input_queue.pop_front();
        if let Some((guid_prefix, received_message)) = popped_queue {
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

    pub fn reader_cache(&self) -> &HistoryCache {
        &self.reader_cache
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

impl RtpsEntity for StatelessReader {
    fn guid(&self) -> GUID {
        self.guid
    }
}

impl RtpsEndpoint for StatelessReader {
    fn try_push_message(&mut self, src_locator: Locator, src_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>) {
        if let Some(inner_submessage) = submessage {
            if self.is_submessage_destination(&src_locator, &src_guid_prefix, inner_submessage) {
                self.input_queue.push_back((src_guid_prefix, submessage.take().unwrap()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ChangeKind;
    use crate::types::constants::{ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR};
    use crate::serialized_payload::ParameterList;
    use crate::messages::Endianness;
    use crate::messages::submessages::Data;
    use crate::messages::submessages::data_submessage::Payload;
    use crate::inline_qos_types::KeyHash;
    use crate::structure::CacheChange;
    use crate::behavior::change_kind_to_status_info;
    
    #[test]
    fn run() {
        let data_reader_qos = DataReaderQos::default();
        let reader_guid_prefix = [0;12];
        let source_locator = Locator::new(0, 7400, [0;16]);
        let mut reader = StatelessReader::new(
            GUID::new(reader_guid_prefix, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER),
            TopicKind::WithKey,
            vec![source_locator],
            vec![],
            &data_reader_qos
           );

        let mut inline_qos = ParameterList::new();
        let instance_handle = [1;16];
        inline_qos.push(KeyHash(instance_handle));
        inline_qos.push(change_kind_to_status_info(ChangeKind::Alive));

        let data1 = Data::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            1,
            Some(inline_qos),
            Payload::Data(vec![0,1,2]),
        );

        let source_guid_prefix  = [2;12];
        reader.try_push_message(source_locator,source_guid_prefix, &mut Some(RtpsSubmessage::Data(data1)));

        let expected_cache_change = CacheChange::new(
            ChangeKind::Alive,
            GUID::new(source_guid_prefix, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
            instance_handle,
            1,
            Some(vec![0,1,2]),
            None);

        assert_eq!(reader.reader_cache.changes().len(), 0);
        reader.run();
        assert_eq!(reader.reader_cache.changes().len(), 1);
        assert!(reader.reader_cache.changes().contains(&expected_cache_change));
    }

    #[test]
    fn submessage_destination() {
        let data_reader_qos = DataReaderQos::default();
        let reader_guid_prefix = [0;12];
        let source_locator_unicast1 = Locator::new(0, 7400, [0;16]);
        let source_locator_unicast2 = Locator::new(0, 7400, [1;16]);
        let source_locator_multicast = Locator::new(0, 7401, [2;16]);
        let reader = StatelessReader::new(
            GUID::new(reader_guid_prefix, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER),
            TopicKind::WithKey,
            vec![source_locator_unicast1, source_locator_unicast2],
            vec![source_locator_multicast],
            &data_reader_qos
           );
        
        let data_to_unknown_reader = RtpsSubmessage::Data(Data::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            1,
            None,
            Payload::Data(vec![0,1,2]),
        ));

        let data_to_this_reader = RtpsSubmessage::Data(Data::new(
            Endianness::LittleEndian,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            1,
            None,
            Payload::Data(vec![0,1,2]),
        ));

        let data_to_other_reader = RtpsSubmessage::Data(Data::new(
            Endianness::LittleEndian,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            1,
            None,
            Payload::Data(vec![0,1,2]),
        ));

        let source_guid_prefix = [1;12];

        // Check that messages from different valid locators are received
        assert!(reader.is_submessage_destination(&source_locator_unicast1, &source_guid_prefix, &data_to_unknown_reader));
        assert!(reader.is_submessage_destination(&source_locator_unicast2, &source_guid_prefix, &data_to_unknown_reader));
        assert!(reader.is_submessage_destination(&source_locator_multicast, &source_guid_prefix, &data_to_unknown_reader));

        // Check that messages with reader id unknown and the correct reader id are received
        assert!(reader.is_submessage_destination(&source_locator_unicast1, &source_guid_prefix, &data_to_unknown_reader));
        assert!(reader.is_submessage_destination(&source_locator_unicast1, &source_guid_prefix, &data_to_this_reader));

        // Check that messages with other source locator and mean for other reader are NOT received
        let other_source_locator = Locator::new(1, 1111, [11;16]);
        assert!(!reader.is_submessage_destination(&other_source_locator, &source_guid_prefix, &data_to_unknown_reader));
        assert!(!reader.is_submessage_destination(&source_locator_unicast1, &source_guid_prefix, &data_to_other_reader));
    }
}
