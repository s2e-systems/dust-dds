use crate::types::{SequenceNumber, };
use crate::types::constants::ENTITYID_UNKNOWN;
use crate::messages::{Gap};
use crate::structure::stateless_writer::{ReaderLocator, StatelessWriter};
use crate::messages::Endianness;
use crate::messages::message_sender::WriterSendMessage;

use super::data_from_cache_change;
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
        let endianness = Endianness::LittleEndian;

        if let Some(cache_change) = stateless_writer.writer_cache()
            .changes().iter().find(|cc| cc.sequence_number() == &next_unsent_seq_num)
        {
            let data = data_from_cache_change(cache_change, endianness, ENTITYID_UNKNOWN);
            reader_locator.push_send_message(WriterSendMessage::Data(data));
        } else {
            let gap = Gap::new(
                ENTITYID_UNKNOWN, 
                stateless_writer.guid().entity_id(),
                next_unsent_seq_num,
                endianness);

            reader_locator.push_send_message(WriterSendMessage::Gap(gap));
        }
    }
}
