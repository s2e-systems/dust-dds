use crate::types::{GUID, SequenceNumber, };
use crate::types::constants::ENTITYID_UNKNOWN;
use crate::messages::{RtpsSubmessage, InfoTs, Gap};
use crate::cache::{HistoryCache};
use crate::stateless_writer::ReaderLocator;
use crate::messages::Endianness;
use crate::messages::types::{Time};
use super::data_from_cache_change;
pub struct StatelessWriterBehavior {}

impl StatelessWriterBehavior{
    pub fn run_best_effort(reader_locator: &mut ReaderLocator, writer_guid: &GUID, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) -> Option<Vec<RtpsSubmessage>> {
        if !reader_locator.unsent_changes(last_change_sequence_number).is_empty() {
            Some(StatelessWriterBehavior::run_pushing_state(reader_locator, writer_guid, history_cache, last_change_sequence_number))
        } else {
            None
        }
    }

    fn run_pushing_state(reader_locator: &mut ReaderLocator, writer_guid: &GUID, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) -> Vec<RtpsSubmessage> {

        // This state is only valid if there are unsent changes
        assert!(!reader_locator.unsent_changes(last_change_sequence_number).is_empty());
    
        let endianness = Endianness::LittleEndian;
        let mut submessages = Vec::with_capacity(2); // TODO: Probably can be preallocated with the correct size
    
        let time = Time::now();
        let infots = InfoTs::new(Some(time), endianness);
        submessages.push(RtpsSubmessage::InfoTs(infots));
    
        while let Some(next_unsent_seq_num) = reader_locator.next_unsent_change(last_change_sequence_number) {
            if let Some(cache_change) = history_cache
                .get_change_with_sequence_number(&next_unsent_seq_num)
            {
                let data = data_from_cache_change(cache_change, endianness, ENTITYID_UNKNOWN);    
                submessages.push(RtpsSubmessage::Data(data));
            } else {
                let gap = Gap::new(
                    ENTITYID_UNKNOWN, 
                    *writer_guid.entity_id(),
                    next_unsent_seq_num,
                    Endianness::LittleEndian);
    
                submessages.push(RtpsSubmessage::Gap(gap));
            }
        }
    
        submessages
    }
}
