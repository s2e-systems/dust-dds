use crate::transport::{
    history_cache::CacheChange,
    types::{Guid, Locator, SequenceNumber, ENTITYID_UNKNOWN},
};

use super::messages::{
    overall_structure::RtpsMessageWrite,
    submessage_elements::SequenceNumberSet,
    submessages::{gap::GapSubmessage, info_timestamp::InfoTimestampSubmessage},
    types::TIME_INVALID,
};

#[derive(Clone)]
pub struct RtpsReaderLocator {
    locator: Locator,
    _expects_inline_qos: bool,
    highest_sent_change_sn: SequenceNumber,
}

impl RtpsReaderLocator {
    pub fn new(locator: Locator, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            _expects_inline_qos: expects_inline_qos,
            highest_sent_change_sn: 0,
        }
    }

    pub fn locator(&self) -> Locator {
        self.locator
    }

    pub fn next_unsent_change<'a>(
        &'a mut self,
        writer_history_cache: impl Iterator<Item = &'a CacheChange>,
    ) -> Option<SequenceNumber> {
        // unsent_changes := { changes SUCH_THAT change.sequenceNumber > this.highestSentChangeSN }
        // IF unsent_changes == <empty> return SEQUENCE_NUMBER_INVALID
        // ELSE return MIN { unsent_changes.sequenceNumber }

        writer_history_cache
            .map(|cc| cc.sequence_number())
            .filter(|sn| sn > &self.highest_sent_change_sn)
            .min()
    }

    pub fn set_highest_sent_change_sn(&mut self, highest_sent_change_sn: SequenceNumber) {
        self.highest_sent_change_sn = highest_sent_change_sn;
    }

    pub fn create_next_message(
        &mut self,
        changes: &[CacheChange],
        guid: Guid,
    ) -> Option<RtpsMessageWrite> {
        // The post-condition:
        // "( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
        // should be full-filled by next_unsent_change()
        if let Some(unsent_change_seq_num) = self.next_unsent_change(changes.iter()) {
            let rtps_message = if let Some(cache_change) = changes
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

                let data_submessage =
                    Box::new(cache_change.as_data_submessage(ENTITYID_UNKNOWN, guid.entity_id()));

                RtpsMessageWrite::from_submessages(
                    &[info_ts_submessage, data_submessage],
                    guid.prefix(),
                )
            } else {
                let gap_submessage = Box::new(GapSubmessage::new(
                    ENTITYID_UNKNOWN,
                    guid.entity_id(),
                    unsent_change_seq_num,
                    SequenceNumberSet::new(unsent_change_seq_num + 1, []),
                ));
                RtpsMessageWrite::from_submessages(&[gap_submessage], guid.prefix())
            };
            self.set_highest_sent_change_sn(unsent_change_seq_num);
            Some(rtps_message)
        } else {
            None
        }
    }
}
