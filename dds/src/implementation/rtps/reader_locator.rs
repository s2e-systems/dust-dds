use crate::infrastructure::time::Time;

use super::{
    history_cache::{RtpsWriterCacheChange, WriterHistoryCache},
    messages::{
        overall_structure::{RtpsMessageHeader, RtpsMessageWrite, RtpsSubmessageWriteKind},
        submessage_elements::SequenceNumberSet,
        submessages::{gap::GapSubmessageWrite, info_timestamp::InfoTimestampSubmessageWrite},
    },
    transport::TransportWrite,
    types::{EntityId, Locator, SequenceNumber, ENTITYID_UNKNOWN},
    writer::RtpsWriter,
};

pub struct RtpsReaderLocator {
    unsent_changes: Vec<SequenceNumber>,
    locator: Locator,
    _expects_inline_qos: bool,
}

fn info_timestamp_submessage<'a>(timestamp: Time) -> RtpsSubmessageWriteKind<'a> {
    RtpsSubmessageWriteKind::InfoTimestamp(InfoTimestampSubmessageWrite {
        endianness_flag: true,
        invalidate_flag: false,
        timestamp: super::messages::types::Time::new(timestamp.sec(), timestamp.nanosec()),
    })
}

fn gap_submessage<'a>(
    writer_id: EntityId,
    gap_sequence_number: SequenceNumber,
) -> RtpsSubmessageWriteKind<'a> {
    RtpsSubmessageWriteKind::Gap(GapSubmessageWrite::new(
        true,
        ENTITYID_UNKNOWN,
        writer_id,
        gap_sequence_number,
        SequenceNumberSet {
            base: gap_sequence_number,
            set: vec![],
        },
    ))
}

impl RtpsReaderLocator {
    pub fn new(locator: Locator, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            _expects_inline_qos: expects_inline_qos,
            unsent_changes: vec![],
        }
    }

    pub fn locator(&self) -> Locator {
        self.locator
    }

    pub fn unsent_changes_mut(&mut self) -> &mut Vec<SequenceNumber> {
        &mut self.unsent_changes
    }

    fn next_unsent_change<'a>(
        &mut self,
        writer_cache: &'a WriterHistoryCache,
    ) -> RtpsReaderLocatorCacheChange<'a> {
        // "next_seq_num := MIN { change.sequenceNumber
        //     SUCH-THAT change IN this.unsent_changes() };
        // return change IN this.unsent_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"

        let next_seq_num = self.unsent_changes.iter().min().cloned().unwrap();

        // 8.4.8.2.10 Transition T10
        // "After the transition, the following post-conditions hold:
        //   ( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
        self.unsent_changes.retain(|c| *c != next_seq_num);

        let cache_change = writer_cache
            .change_list()
            .iter()
            .find(|c| c.sequence_number() == next_seq_num);

        RtpsReaderLocatorCacheChange {
            sequence_number: next_seq_num,
            cache_change,
        }
    }

    pub fn send_message(
        &mut self,
        writer_cache: &WriterHistoryCache,
        writer_id: EntityId,
        header: RtpsMessageHeader,
        transport: &mut impl TransportWrite,
    ) {
        let mut submessages = Vec::new();
        while !self.unsent_changes.is_empty() {
            let change = self.next_unsent_change(writer_cache);
            // The post-condition:
            // "( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
            // should be full-filled by next_unsent_change()
            if let Some(cache_change) = change.cache_change {
                let info_ts_submessage = info_timestamp_submessage(cache_change.timestamp());
                let data_submessage = cache_change.as_data_submessage(ENTITYID_UNKNOWN);
                submessages.push(info_ts_submessage);
                submessages.push(RtpsSubmessageWriteKind::Data(data_submessage));
            } else {
                let gap_submessage = gap_submessage(writer_id, change.sequence_number);
                submessages.push(gap_submessage);
            }
        }
        if !submessages.is_empty() {
            transport.write(&RtpsMessageWrite::new(header, submessages), &[self.locator])
        }
    }
}

pub struct WriterAssociatedReaderLocator<'a> {
    writer: &'a RtpsWriter,
    reader_locator: &'a mut RtpsReaderLocator,
}

impl<'a> WriterAssociatedReaderLocator<'a> {
    pub fn new(writer: &'a RtpsWriter, reader_locator: &'a mut RtpsReaderLocator) -> Self {
        Self {
            writer,
            reader_locator,
        }
    }

    pub fn locator(&self) -> Locator {
        self.reader_locator.locator()
    }

    pub fn next_unsent_change(&mut self) -> Option<RtpsReaderLocatorCacheChange<'a>> {
        // "next_seq_num := MIN { change.sequenceNumber
        //     SUCH-THAT change IN this.unsent_changes() };
        // return change IN this.unsent_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"

        let next_seq_num = *self.reader_locator.unsent_changes.iter().min()?;

        // 8.4.8.2.10 Transition T10
        // "After the transition, the following post-conditions hold:
        //   ( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
        self.reader_locator
            .unsent_changes
            .retain(|c| *c != next_seq_num);

        let cache_change = self
            .writer
            .change_list()
            .iter()
            .find(|c| c.sequence_number() == next_seq_num);

        Some(RtpsReaderLocatorCacheChange {
            sequence_number: next_seq_num,
            cache_change,
        })
    }
}

pub struct RtpsReaderLocatorCacheChange<'a> {
    sequence_number: SequenceNumber,
    cache_change: Option<&'a RtpsWriterCacheChange>,
}

impl<'a> RtpsReaderLocatorCacheChange<'a> {
    pub fn cache_change(&self) -> Option<&'a RtpsWriterCacheChange> {
        self.cache_change
    }

    pub fn sequence_number(&self) -> SequenceNumber {
        self.sequence_number
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        implementation::rtps::{
            history_cache::RtpsWriterCacheChange,
            messages::submessage_elements::ParameterList,
            types::{ChangeKind, GUID_UNKNOWN, LOCATOR_INVALID},
        },
        infrastructure::{instance::HANDLE_NIL, time::TIME_INVALID},
    };

    use super::*;

    #[test]
    fn reader_locator_next_unsent_change() {
        let mut hc = WriterHistoryCache::new();
        hc.add_change(RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            SequenceNumber::new(1),
            TIME_INVALID,
            vec![],
            ParameterList::empty(),
        ));
        hc.add_change(RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            SequenceNumber::new(2),
            TIME_INVALID,
            vec![],
            ParameterList::empty(),
        ));
        let mut reader_locator_attributes = RtpsReaderLocator::new(LOCATOR_INVALID, false);
        reader_locator_attributes.unsent_changes =
            vec![SequenceNumber::new(1), SequenceNumber::new(2)];

        assert_eq!(
            reader_locator_attributes
                .next_unsent_change(&hc)
                .cache_change
                .unwrap()
                .sequence_number(),
            SequenceNumber::new(1)
        );
        assert_eq!(
            reader_locator_attributes
                .next_unsent_change(&hc)
                .cache_change
                .unwrap()
                .sequence_number(),
            SequenceNumber::new(2)
        );
    }
}
