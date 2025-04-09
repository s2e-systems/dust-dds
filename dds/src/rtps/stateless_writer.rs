use crate::transport::{
    history_cache::CacheChange,
    types::{Guid, GuidPrefix, Locator, SequenceNumber, ENTITYID_UNKNOWN},
};

use super::{
    message_sender::MessageSender,
    messages::{
        overall_structure::RtpsMessageWrite,
        submessage_elements::SequenceNumberSet,
        submessages::{gap::GapSubmessage, info_timestamp::InfoTimestampSubmessage},
        types::TIME_INVALID,
    },
    reader_locator::RtpsReaderLocator,
};

pub trait WriteMessage {
    fn write_message(&self, message: &RtpsMessageWrite, locator_list: &[Locator]);
    fn guid_prefix(&self) -> GuidPrefix;
}

pub struct RtpsStatelessWriter {
    guid: Guid,
    pub changes: Vec<CacheChange>,
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

    pub fn remove_change(&mut self, sequence_number: SequenceNumber) {
        self.changes
            .retain(|cc| cc.sequence_number() != sequence_number);
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

    pub fn write_message(&mut self, message_writer: &impl WriteMessage) {
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
                    .find(|cc| cc.sequence_number() == unsent_change_seq_num)
                {
                    let info_ts_submessage = Box::new(
                        cache_change
                            .source_timestamp()
                            .map_or(InfoTimestampSubmessage::new(true, TIME_INVALID), |t| {
                                InfoTimestampSubmessage::new(false, t.into())
                            }),
                    );

                    let data_submessage = Box::new(
                        cache_change.as_data_submessage(ENTITYID_UNKNOWN, self.guid.entity_id()),
                    );

                    let rtps_message = RtpsMessageWrite::from_submessages(
                        &[info_ts_submessage, data_submessage],
                        message_writer.guid_prefix(),
                    );
                    message_writer.write_message(&rtps_message, &[reader_locator.locator()]);
                } else {
                    let gap_submessage = Box::new(GapSubmessage::new(
                        ENTITYID_UNKNOWN,
                        self.guid.entity_id(),
                        unsent_change_seq_num,
                        SequenceNumberSet::new(unsent_change_seq_num + 1, []),
                    ));
                    let rtps_message = RtpsMessageWrite::from_submessages(
                        &[gap_submessage],
                        message_writer.guid_prefix(),
                    );
                    message_writer.write_message(&rtps_message, &[reader_locator.locator()]);
                }
                reader_locator.set_highest_sent_change_sn(unsent_change_seq_num);
            }
        }
    }

    pub fn send_message(&mut self, message_sender: &MessageSender) {
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
                    .find(|cc| cc.sequence_number() == unsent_change_seq_num)
                {
                    let info_ts_submessage = Box::new(
                        cache_change
                            .source_timestamp()
                            .map_or(InfoTimestampSubmessage::new(true, TIME_INVALID), |t| {
                                InfoTimestampSubmessage::new(false, t.into())
                            }),
                    );

                    let data_submessage = Box::new(
                        cache_change.as_data_submessage(ENTITYID_UNKNOWN, self.guid.entity_id()),
                    );

                    message_sender.write_message(
                        &[info_ts_submessage, data_submessage],
                        vec![reader_locator.locator()],
                    );
                } else {
                    let gap_submessage = Box::new(GapSubmessage::new(
                        ENTITYID_UNKNOWN,
                        self.guid.entity_id(),
                        unsent_change_seq_num,
                        SequenceNumberSet::new(unsent_change_seq_num + 1, []),
                    ));

                    message_sender.write_message(&[gap_submessage], vec![reader_locator.locator()]);
                }
                reader_locator.set_highest_sent_change_sn(unsent_change_seq_num);
            }
        }
    }
}
