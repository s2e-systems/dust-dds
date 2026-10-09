use super::{reader_locator::RtpsReaderLocator, submessage_writer::SubmessageWriter};
use crate::{
    rtps_messages::{
        submessage_elements::SequenceNumberSet,
        submessages::{gap::GapSubmessageWrite, info_timestamp::InfoTimestampSubmessageWrite},
        types::TIME_INVALID,
    },
    transport::types::{CacheChange, ENTITYID_UNKNOWN, Guid, Locator, SequenceNumber},
};
use alloc::vec::Vec;

pub struct RtpsStatelessWriter {
    guid: Guid,
    changes: Vec<CacheChange>,
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

    pub fn add_change(&mut self, cache_change: CacheChange) {
        self.changes.push(cache_change);
    }

    pub fn write_message(&mut self, submessage_writer: &mut (impl SubmessageWriter + ?Sized)) {
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

                if let Some(cache_change) = self
                    .changes
                    .iter()
                    .find(|cc| cc.sequence_number == unsent_change_seq_num)
                {
                    let info_ts_submessage = cache_change
                        .source_timestamp
                        .map_or(InfoTimestampSubmessageWrite::new(true, TIME_INVALID), |t| {
                            InfoTimestampSubmessageWrite::new(false, t.into())
                        });

                    let inline_qos = match (
                        cache_change.status_info_parameter(),
                        cache_change.key_hash_parameter(),
                    ) {
                        (Some(s), Some(k)) => &[s, k][..],
                        (Some(s), None) => &[s][..],
                        (None, Some(k)) => &[k][..],
                        (None, None) => &[],
                    };
                    let data_submessage = cache_change.as_data_submessage(
                        ENTITYID_UNKNOWN,
                        self.guid.entity_id(),
                        inline_qos,
                    );

                    submessage_writer.write_submessages(
                        &[info_ts_submessage.into(), data_submessage.into()],
                        &[reader_locator.locator()],
                    );
                } else {
                    let gap_submessage = GapSubmessageWrite::new(
                        ENTITYID_UNKNOWN,
                        self.guid.entity_id(),
                        unsent_change_seq_num,
                        SequenceNumberSet::new(unsent_change_seq_num + 1, []),
                    );
                    submessage_writer
                        .write_submessages(&[gap_submessage.into()], &[reader_locator.locator()]);
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

    pub fn changes(&self) -> &[CacheChange] {
        &self.changes
    }

    pub fn changes_mut(&mut self) -> &mut Vec<CacheChange> {
        &mut self.changes
    }
}
