use std::collections::BTreeSet;

use crate::types::SequenceNumber;
use crate::types::constants::{ENTITYID_UNKNOWN, GUID_UNKNOWN};
use crate::messages::RtpsSubmessage;
use crate::messages::submessages::Gap;
use crate::behavior::{ReaderLocator, StatelessWriter};
use crate::messages::message_sender::Sender;

use super::{data_from_cache_change, BEHAVIOR_ENDIANNESS};
pub struct BestEffortStatelessWriterBehavior {}

impl BestEffortStatelessWriterBehavior{
    pub fn run(reader_locator: &ReaderLocator, stateless_writer: &StatelessWriter) {
        if !reader_locator.unsent_changes(stateless_writer.last_change_sequence_number()).is_empty() {
            Self::pushing_state(reader_locator, stateless_writer);
        }
    }

    fn pushing_state(reader_locator: &ReaderLocator, stateless_writer: &StatelessWriter) {
        // This state is only valid if there are unsent changes
        assert!(!reader_locator.unsent_changes(stateless_writer.last_change_sequence_number()).is_empty());
    
        while let Some(next_unsent_seq_num) = reader_locator.next_unsent_change(stateless_writer.last_change_sequence_number()) {
            Self::transition_t4(reader_locator, stateless_writer, next_unsent_seq_num);
        }
    }

    fn transition_t4(reader_locator: &ReaderLocator, stateless_writer: &StatelessWriter, next_unsent_seq_num: SequenceNumber) {

        if let Some(cache_change) = stateless_writer.writer_cache()
            .changes().iter().find(|cc| cc.sequence_number() == next_unsent_seq_num)
        {
            let data = data_from_cache_change(cache_change, ENTITYID_UNKNOWN);
            stateless_writer.push_send_message(reader_locator.locator(), &GUID_UNKNOWN, RtpsSubmessage::Data(data));
        } else {
            let gap = Gap::new(
                BEHAVIOR_ENDIANNESS,
                ENTITYID_UNKNOWN, 
                stateless_writer.guid().entity_id(),
                next_unsent_seq_num,
            BTreeSet::new());

            stateless_writer.push_send_message(reader_locator.locator(), &GUID_UNKNOWN, RtpsSubmessage::Gap(gap));
        }
    }
}
