use crate::types::{GUID, SequenceNumber, };
use crate::types::constants::ENTITYID_UNKNOWN;
use crate::messages::{Gap};
use crate::cache::{HistoryCache};
use crate::stateless_writer::ReaderLocator;
use crate::messages::Endianness;
use crate::messages::receiver::WriterSendMessage;

use super::data_from_cache_change;
pub struct BestEffortStatelessWriterBehavior {}

impl BestEffortStatelessWriterBehavior{
    pub fn run(reader_locator: &ReaderLocator, writer_guid: &GUID, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) {
        if !reader_locator.unsent_changes(last_change_sequence_number).is_empty() {
            Self::pushing_state(reader_locator, writer_guid, history_cache, last_change_sequence_number)
        } else {
            ()
        }
    }

    fn pushing_state(reader_locator: &ReaderLocator, writer_guid: &GUID, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) {

        // This state is only valid if there are unsent changes
        assert!(!reader_locator.unsent_changes(last_change_sequence_number).is_empty());
    
        while let Some(next_unsent_seq_num) = reader_locator.next_unsent_change(last_change_sequence_number) {
            Self::transition_t4(reader_locator, writer_guid, history_cache, &next_unsent_seq_num);
        }
    }

    fn transition_t4(reader_locator: &ReaderLocator, writer_guid: &GUID, history_cache: &HistoryCache, next_unsent_seq_num: &SequenceNumber) {
        let endianness = Endianness::LittleEndian;

        if let Some(cache_change) = history_cache
            .changes().iter().find(|cc| cc.sequence_number() == next_unsent_seq_num)
        {
            let data = data_from_cache_change(cache_change, endianness, ENTITYID_UNKNOWN);
            reader_locator.push_send_message(WriterSendMessage::Data(data));
        } else {
            let gap = Gap::new(
                ENTITYID_UNKNOWN, 
                *writer_guid.entity_id(),
                *next_unsent_seq_num,
                endianness);

            reader_locator.push_send_message(WriterSendMessage::Gap(gap));
        }
    }
}
