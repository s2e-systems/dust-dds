use super::{
    types::{Locator, SequenceNumber},
    writer_history_cache::WriterHistoryCache,
};

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

    pub fn next_unsent_change(
        &mut self,
        writer_history_cache: &WriterHistoryCache,
    ) -> Option<SequenceNumber> {
        // unsent_changes := { changes SUCH_THAT change.sequenceNumber > this.highestSentChangeSN }
        // IF unsent_changes == <empty> return SEQUENCE_NUMBER_INVALID
        // ELSE return MIN { unsent_changes.sequenceNumber }

        writer_history_cache
            .change_list()
            .map(|cc| cc.sequence_number())
            .filter(|sn| sn > &self.highest_sent_change_sn)
            .min()
    }

    pub fn set_highest_sent_change_sn(&mut self, highest_sent_change_sn: SequenceNumber) {
        self.highest_sent_change_sn = highest_sent_change_sn;
    }
}
