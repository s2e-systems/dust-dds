use crate::structure::{HistoryCache, RtpsEndpoint, RtpsEntity, HistoryCacheResourceLimits};
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
}

impl StatelessReader {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        expects_inline_qos: bool,
        resource_limits: HistoryCacheResourceLimits,
    ) -> Self {

        assert!(reliability_level == ReliabilityKind::BestEffort, "Only BestEffort is supported on stateless reader");

        Self {
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            reader_cache: HistoryCache::new(resource_limits),
            expects_inline_qos,
        }
    }

    fn waiting_state(&mut self, src_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>) {
        if let Some(inner_submessage) = submessage {
            if let RtpsSubmessage::Data(data) = inner_submessage { 
                if self.guid.entity_id() == data.reader_id() || data.reader_id() == ENTITYID_UNKNOWN {
                    if let RtpsSubmessage::Data(data) = submessage.take().unwrap() {
                        self.transition_t2(src_guid_prefix, data)
                    }
                }
            }              
        }
    }

    fn transition_t2(&mut self, guid_prefix: GuidPrefix, data: Data) {
        let cache_change = cache_change_from_data(data, &guid_prefix);
        self.reader_cache.add_change(cache_change).unwrap();
    }

    pub fn reader_cache(&self) -> &HistoryCache {
        &self.reader_cache
    }   

    pub fn try_process_message(&mut self, src_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>) {
        self.waiting_state(src_guid_prefix, submessage);
    }
}

impl RtpsEntity for StatelessReader {
    fn guid(&self) -> GUID {
        self.guid
    }
}

impl RtpsEndpoint for StatelessReader {
    fn unicast_locator_list(&self) -> Vec<Locator> {
        todo!()
    }

    fn multicast_locator_list(&self) -> Vec<Locator> {
        todo!()
    }

    fn reliability_level(&self) -> ReliabilityKind {
        todo!()
    }

    fn topic_kind(&self) -> &TopicKind {
        todo!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::types::ChangeKind;
    // use crate::types::constants::{ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR};
    // use crate::serialized_payload::ParameterList;
    // use crate::messages::Endianness;
    // use crate::messages::submessages::Data;
    // use crate::messages::submessages::data_submessage::Payload;
    // use crate::inline_qos_types::KeyHash;
    // use crate::structure::CacheChange;
    // use crate::behavior::change_kind_to_status_info;
    
    // #[test]
    // fn run() {
    //     let reader_guid_prefix = [0;12];
    //     let source_locator = Locator::new(0, 7400, [0;16]);
    //     let mut reader = StatelessReader::new(
    //         GUID::new(reader_guid_prefix, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER),
    //         TopicKind::WithKey,
    //         ReliabilityKind::BestEffort,
    //         vec![source_locator],
    //         vec![],
    //         false,
    //         HistoryCacheResourceLimits::default(),
    //        );

    //     let mut inline_qos = ParameterList::new();
    //     let instance_handle = [1;16];
    //     inline_qos.push(KeyHash(instance_handle));
    //     inline_qos.push(change_kind_to_status_info(ChangeKind::Alive));

    //     let data1 = Data::new(
    //         Endianness::LittleEndian,
    //         ENTITYID_UNKNOWN,
    //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
    //         1,
    //         Some(inline_qos),
    //         Payload::Data(vec![0,1,2]),
    //     );

    //     let source_guid_prefix  = [2;12];
    //     reader.input_queue.push_back((source_guid_prefix, RtpsSubmessage::Data(data1)));

    //     let expected_cache_change = CacheChange::new(
    //         ChangeKind::Alive,
    //         GUID::new(source_guid_prefix, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
    //         instance_handle,
    //         1,
    //         Some(vec![0,1,2]),
    //         None);

    //     assert_eq!(reader.reader_cache.changes().len(), 0);
    //     let expected_data = vec![0,1,2];
    //     reader.run(|cc| assert_eq!(cc.data_value(),&expected_data) );
    //     assert_eq!(reader.reader_cache.changes().len(), 1);
    //     assert!(reader.reader_cache.changes().contains(&expected_cache_change));
    //     reader.run(|_cc| assert!(false, "Callback shouldn't execute") );
    // }

    // #[test]
    // fn submessage_destination() {
    //     let reader_guid_prefix = [0;12];
    //     let source_locator_unicast1 = Locator::new(0, 7400, [0;16]);
    //     let source_locator_unicast2 = Locator::new(0, 7400, [1;16]);
    //     let source_locator_multicast = Locator::new(0, 7401, [2;16]);
    //     let reader = StatelessReader::new(
    //         GUID::new(reader_guid_prefix, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER),
    //         TopicKind::WithKey,
    //         ReliabilityKind::BestEffort,
    //         vec![source_locator_unicast1, source_locator_unicast2],
    //         vec![source_locator_multicast],
    //         false,
    //         HistoryCacheResourceLimits::default(),
    //        );
        
    //     let data_to_unknown_reader = RtpsSubmessage::Data(Data::new(
    //         Endianness::LittleEndian,
    //         ENTITYID_UNKNOWN,
    //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
    //         1,
    //         None,
    //         Payload::Data(vec![0,1,2]),
    //     ));

    //     let data_to_this_reader = RtpsSubmessage::Data(Data::new(
    //         Endianness::LittleEndian,
    //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
    //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
    //         1,
    //         None,
    //         Payload::Data(vec![0,1,2]),
    //     ));

    //     let data_to_other_reader = RtpsSubmessage::Data(Data::new(
    //         Endianness::LittleEndian,
    //         ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
    //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
    //         1,
    //         None,
    //         Payload::Data(vec![0,1,2]),
    //     ));

    //     let source_guid_prefix = [1;12];

        // Check that messages from different valid locators are received
        // assert!(reader.is_submessage_destination(&source_locator_unicast1, &source_guid_prefix, &data_to_unknown_reader));
        // assert!(reader.is_submessage_destination(&source_locator_unicast2, &source_guid_prefix, &data_to_unknown_reader));
        // assert!(reader.is_submessage_destination(&source_locator_multicast, &source_guid_prefix, &data_to_unknown_reader));

        // // Check that messages with reader id unknown and the correct reader id are received
        // assert!(reader.is_submessage_destination(&source_locator_unicast1, &source_guid_prefix, &data_to_unknown_reader));
        // assert!(reader.is_submessage_destination(&source_locator_unicast1, &source_guid_prefix, &data_to_this_reader));

        // // Check that messages with other source locator and mean for other reader are NOT received
        // let other_source_locator = Locator::new(1, 1111, [11;16]);
        // assert!(!reader.is_submessage_destination(&other_source_locator, &source_guid_prefix, &data_to_unknown_reader));
        // assert!(!reader.is_submessage_destination(&source_locator_unicast1, &source_guid_prefix, &data_to_other_reader));
    // }
}
