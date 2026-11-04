use super::{message_creator::write_submessages_into_transport, reader_locator::RtpsReaderLocator};
use crate::{
    rtps::cache_change::CacheChangeWrite,
    rtps_messages::{
        submessage_elements::SequenceNumberSet,
        submessages::{gap::GapSubmessage, info_timestamp::InfoTimestampSubmessage},
    },
    transport::{
        interface::{RtpsParticipant, Write},
        types::{ENTITYID_UNKNOWN, Guid, Locator, SequenceNumber},
    },
};
use alloc::vec::Vec;

pub struct RtpsStatelessWriter {
    guid: Guid,
    changes: Vec<CacheChangeWrite>,
    reader_locators: Vec<RtpsReaderLocator>,
}

impl RtpsStatelessWriter {
    pub fn new(guid: Guid) -> Self {
        Self {
            guid,
            changes: Vec::new(),
            reader_locators: Vec::new(),
        }
    }

    pub fn guid(&self) -> Guid {
        self.guid
    }

    pub fn add_change(&mut self, cache_change: CacheChangeWrite) {
        self.changes.push(cache_change);
    }

    pub fn write_message(&mut self, transport_participant: &mut impl RtpsParticipant) {
        if self.changes.is_empty() || self.reader_locators.is_empty() {
            return;
        }

        for reader_locator in &mut self.reader_locators {
            while let Some(unsent_change_seq_num) =
                reader_locator.next_unsent_change(self.changes.iter())
            {
                // The post-condition:
                // "( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
                // should be full-filled by next_unsent_change()
                let locators = [reader_locator.locator()];
                let mut transport_writer = transport_participant.writer(&locators);

                if let Some(cache_change) = self
                    .changes
                    .iter()
                    .find(|cc| cc.sequence_number == unsent_change_seq_num)
                {
                    let info_ts_submessage =
                        InfoTimestampSubmessage::new(false, cache_change.source_timestamp.into());
                    let data_submessage =
                        cache_change.as_data_submessage(ENTITYID_UNKNOWN, self.guid.entity_id());

                    write_submessages_into_transport(
                        &[&info_ts_submessage, &data_submessage],
                        self.guid.prefix(),
                        &mut transport_writer,
                    );
                    transport_writer.flush();
                } else {
                    let gap_submessage = GapSubmessage::new(
                        ENTITYID_UNKNOWN,
                        self.guid.entity_id(),
                        unsent_change_seq_num,
                        SequenceNumberSet::new(unsent_change_seq_num + 1, []),
                    );
                    write_submessages_into_transport(
                        &[&gap_submessage],
                        self.guid.prefix(),
                        &mut transport_writer,
                    );
                    transport_writer.flush();
                }
                reader_locator.set_highest_sent_change_sn(unsent_change_seq_num);
            }
        }
    }

    pub fn remove_change(&mut self, sequence_number: SequenceNumber) {
        self.changes
            .retain(|cc| cc.sequence_number != sequence_number);
    }

    pub fn reader_locator_add(&mut self, locator: Locator) {
        self.reader_locators
            .push(RtpsReaderLocator::new(locator, false));
    }

    pub fn reader_locator_remove(&mut self, locator: Locator) {
        self.reader_locators.retain(|x| x.locator() != locator);
    }

    pub fn reader_locator_list(&mut self) -> &mut [RtpsReaderLocator] {
        &mut self.reader_locators
    }

    pub fn changes(&self) -> &[CacheChangeWrite] {
        &self.changes
    }

    pub fn changes_mut(&mut self) -> &mut Vec<CacheChangeWrite> {
        &mut self.changes
    }
}
