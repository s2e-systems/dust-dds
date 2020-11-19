use crate::types::{ReliabilityKind, GuidPrefix };
use crate::types::constants::ENTITYID_UNKNOWN;
use crate::messages::RtpsSubmessage;
use crate::messages::submessages::Data;
use crate::behavior::RtpsReader;
use crate::behavior::cache_change_from_data;
use crate::behavior::endpoint_traits::CacheChangeReceiver;
use crate::behavior::cache_change_receiver_listener::CacheChangeReceiverListener;

pub struct StatelessReader {
    pub reader: RtpsReader,
}

impl StatelessReader {
    pub fn new(reader: RtpsReader) -> Self {

        assert!(reader.endpoint.reliability_level == ReliabilityKind::BestEffort, "Only BestEffort is supported on stateless reader");

        Self {
            reader
        }
    }

    fn waiting_state(&mut self, source_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>) {
        if let Some(inner_submessage) = submessage {
            if let RtpsSubmessage::Data(data) = inner_submessage { 
                if self.reader.endpoint.entity.guid.entity_id() == data.reader_id() || data.reader_id() == ENTITYID_UNKNOWN {
                    if let RtpsSubmessage::Data(data) = submessage.take().unwrap() {
                        self.transition_t2(source_guid_prefix, data)
                    }
                }
            }              
        }
    }

    fn transition_t2(&mut self, guid_prefix: GuidPrefix, data: Data) {
        let cache_change = cache_change_from_data(data, &guid_prefix);
        // listener.on_add_change(&cache_change);
        self.reader.reader_cache.add_change(cache_change).unwrap();
    }
}

impl CacheChangeReceiver for StatelessReader {
    fn try_process_message(&mut self, source_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>) {
        self.waiting_state(source_guid_prefix, submessage);
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
