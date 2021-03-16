use crate::{
    behavior::{data_from_cache_change, RTPSWriter},
    messages::submessages::{
        self,
        submessage_elements::{SequenceNumberSet, SerializedData},
        Gap,
    },
    structure::{RTPSCacheChange, RTPSHistoryCache},
    types::{constants::ENTITYID_UNKNOWN, SequenceNumber},
};

use super::reader_locator::RTPSReaderLocator;

pub struct BestEffortReaderLocatorBehavior;

pub enum BestEffortReaderLocatorSendSubmessages<'a> {
    Data(submessages::Data<'a>),
    Gap(submessages::Gap),
}

impl BestEffortReaderLocatorBehavior {
    pub fn produce_message<'a, 'b>(
        reader_locator: &'b mut impl RTPSReaderLocator<CacheChangeRepresentation = SequenceNumber>,
        writer: &'a impl RTPSWriter<
            HistoryCacheType = impl RTPSHistoryCache<
                CacheChangeType = impl RTPSCacheChange<Data = impl AsRef<SerializedData> + 'a> + 'a,
            > + 'a,
        >,
    ) -> Option<BestEffortReaderLocatorSendSubmessages<'a>> {
        if reader_locator
            .unsent_changes(writer)
            .into_iter()
            .next()
            .is_some()
        {
            Self::pushing_state(reader_locator, writer)
        } else {
            None
        }
    }

    fn pushing_state<'a, 'b>(
        reader_locator: &'b mut impl RTPSReaderLocator<CacheChangeRepresentation = SequenceNumber>,
        writer: &'a impl RTPSWriter<
            HistoryCacheType = impl RTPSHistoryCache<
                CacheChangeType = impl RTPSCacheChange<Data = impl AsRef<SerializedData> + 'a> + 'a,
            > + 'a,
        >,
    ) -> Option<BestEffortReaderLocatorSendSubmessages<'a>> {
        // RL::can_send() is always true when this function is called
        // so we don't bother making an if here
        Self::transition_t4(reader_locator, writer)
    }

    fn transition_t4<'a, 'b>(
        reader_locator: &'b mut impl RTPSReaderLocator<CacheChangeRepresentation = SequenceNumber>,
        writer: &'a impl RTPSWriter<
            HistoryCacheType = impl RTPSHistoryCache<
                CacheChangeType = impl RTPSCacheChange<Data = impl AsRef<SerializedData> + 'a> + 'a,
            > + 'a,
        >,
    ) -> Option<BestEffortReaderLocatorSendSubmessages<'a>> {
        if let Some(next_unsent_seq_num) = reader_locator.next_unsent_change(writer) {
            if let Some(cache_change) = writer.writer_cache().get_change(next_unsent_seq_num) {
                Some(BestEffortReaderLocatorSendSubmessages::Data(
                    data_from_cache_change(cache_change, ENTITYID_UNKNOWN),
                ))
            } else {
                Some(BestEffortReaderLocatorSendSubmessages::Gap(Gap {
                    endianness_flag: false,
                    reader_id: ENTITYID_UNKNOWN,
                    writer_id: writer.guid().entity_id(),
                    gap_start: next_unsent_seq_num,
                    gap_list: SequenceNumberSet::new(next_unsent_seq_num, [0; 8]),
                }))
            }
        } else {
            None
        }
    }
}
#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::types::{constants::ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER, ChangeKind};
    // use crate::types::{Locator, GUID};

    // use crate::{structure::CacheChange, messages::submessages::submessage_elements::ParameterList};

    // #[derive(Clone)]
    // struct MockCacheChange;

    // impl CacheChange for MockCacheChange {
    //     fn new(
    //         kind: ChangeKind,
    //         writer_guid: GUID,
    //         instance_handle: crate::types::InstanceHandle,
    //         sequence_number: SequenceNumber,
    //         data_value: crate::messages::submessages::submessage_elements::SerializedData,
    //         inline_qos: crate::messages::submessages::submessage_elements::ParameterList,
    //     ) -> Self {
    //         todo!()
    //     }

    //     fn kind(&self) -> ChangeKind {
    //         todo!()
    //     }

    //     fn writer_guid(&self) -> GUID {
    //         todo!()
    //     }

    //     fn instance_handle(&self) -> crate::types::InstanceHandle {
    //         todo!()
    //     }

    //     fn sequence_number(&self) -> SequenceNumber {
    //         todo!()
    //     }

    //     fn data_value(&self) -> &crate::messages::submessages::submessage_elements::SerializedData {
    //         todo!()
    //     }

    //     fn inline_qos(&self) -> &crate::messages::submessages::submessage_elements::ParameterList {
    //         todo!()
    //     }
    // }

    // struct MockHistoryCache;

    // impl HistoryCache for MockHistoryCache {
    //     type CacheChangeType = MockCacheChange;

    //     fn add_change(&mut self, change: Self::CacheChangeType) {
    //         todo!()
    //     }

    //     fn remove_change(&mut self, seq_num: SequenceNumber) {
    //         todo!()
    //     }

    //     fn get_change(&self, seq_num: SequenceNumber) -> Option<&Self::CacheChangeType> {
    //         todo!()
    //     }

    //     fn get_seq_num_min(&self) -> Option<SequenceNumber> {
    //         todo!()
    //     }

    //     fn get_seq_num_max(&self) -> Option<SequenceNumber> {
    //         todo!()
    //     }
    // }

    // #[test]
    // fn produce_empty() {
    //     let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
    //     let mut reader_locator = ReaderLocator::new(locator);
    //     let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
    //     let history_cache = MockHistoryCache;

    //     // Run without any change being created or added in the cache
    //     let last_change_sequence_number = 0;
    //     let messages_vec = BestEffortReaderLocatorBehavior::produce_messages(
    //         &mut reader_locator,
    //         &history_cache,
    //         writer_entity_id,
    //         last_change_sequence_number,
    //     );

    //     assert!(messages_vec.is_empty());
    // }

    // #[test]
    // fn produce_data_message() {
    //     let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
    //     let mut reader_locator = ReaderLocator::new(locator);
    //     let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
    //     let mut history_cache = MockHistoryCache;

    //     // Add one change to the history cache
    //     let writer_guid = GUID::new([5; 12], writer_entity_id);
    //     let instance_handle = [1; 16];
    //     let cache_change1 = MockCacheChange::new(
    //         ChangeKind::Alive,
    //         writer_guid.into(),
    //         instance_handle,
    //         1,
    //         vec![1, 2, 3],
    //         ParameterList::new(),
    //     );
    //     history_cache.add_change(cache_change1.clone());

    //     // Run with the last change sequence number equal to the added cache change
    //     let last_change_sequence_number = 1;
    //     let messages_vec = BestEffortReaderLocatorBehavior::produce_messages(
    //         &mut reader_locator,
    //         &history_cache,
    //         writer_entity_id,
    //         last_change_sequence_number,
    //     );

    //     let expected_data_submessage =
    //         RtpsSubmessage::Data(data_from_cache_change(&cache_change1, ENTITYID_UNKNOWN));
    //     assert_eq!(messages_vec.len(), 1);
    //     assert!(messages_vec.contains(&expected_data_submessage));
    // }

    // #[test]
    // fn produce_gap_message() {
    //     let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
    //     let mut reader_locator = ReaderLocator::new(locator);
    //     let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
    //     let history_cache = MockHistoryCache;

    //     // Run with the a sequence number of 1 without adding any change to the history cache
    //     let last_change_sequence_number = 1;
    //     let messages_vec = BestEffortReaderLocatorBehavior::produce_messages(
    //         &mut reader_locator,
    //         &history_cache,
    //         writer_entity_id,
    //         last_change_sequence_number,
    //     );

    //     let expected_gap_submessage = RtpsSubmessage::Gap(Gap::new(
    //         BEHAVIOR_ENDIANNESS,
    //         ENTITYID_UNKNOWN,
    //         writer_entity_id,
    //         1,
    //         &[],
    //     ));
    //     assert_eq!(messages_vec.len(), 1);
    //     assert!(messages_vec.contains(&expected_gap_submessage));
    // }

    // #[test]
    // fn produce_data_and_gap_messages() {
    //     let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
    //     let mut reader_locator = ReaderLocator::new(locator);
    //     let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
    //     let mut history_cache = MockHistoryCache;

    //     // Add one change to the history cache
    //     let writer_guid = GUID::new([5; 12], writer_entity_id);
    //     let instance_handle = [1; 16];
    //     let cache_change1 = CacheChange::new(
    //         ChangeKind::Alive,
    //         writer_guid.into(),
    //         instance_handle,
    //         1,
    //         Some(vec![1, 2, 3]),
    //         None,
    //     );
    //     history_cache.add_change(cache_change1.clone());

    //     // Run with the last change sequence number one above the added cache change
    //     let last_change_sequence_number = 2;
    //     let messages_vec = BestEffortReaderLocatorBehavior::produce_messages(
    //         &mut reader_locator,
    //         &history_cache,
    //         writer_entity_id,
    //         last_change_sequence_number,
    //     );

    //     let expected_data_submessage =
    //         RtpsSubmessage::Data(data_from_cache_change(&cache_change1, ENTITYID_UNKNOWN));
    //     let expected_gap_submessage = RtpsSubmessage::Gap(Gap::new(
    //         BEHAVIOR_ENDIANNESS,
    //         ENTITYID_UNKNOWN,
    //         writer_entity_id,
    //         2,
    //         &[],
    //     ));
    //     assert_eq!(messages_vec.len(), 2);
    //     assert!(messages_vec.contains(&expected_data_submessage));
    //     assert!(messages_vec.contains(&expected_gap_submessage));
    // }
}
