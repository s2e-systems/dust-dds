use super::{
    history_cache::{RtpsWriterCacheChange, WriterHistoryCache},
    messages::{
        overall_structure::RtpsMessageHeader,
        submessages::{DataSubmessage, GapSubmessage, InfoTimestampSubmessage},
        types::Time,
        RtpsMessage, RtpsSubmessageKind,
    },
    transport::TransportWrite,
    types::{Locator, SequenceNumber, ENTITYID_UNKNOWN},
};

pub struct RtpsReaderLocator {
    unsent_changes: Vec<SequenceNumber>,
    locator: Locator,
    _expects_inline_qos: bool,
}

impl RtpsReaderLocator {
    pub fn new(locator: Locator, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            _expects_inline_qos: expects_inline_qos,
            unsent_changes: vec![],
        }
    }

    pub fn unsent_changes_mut(&mut self) -> &mut Vec<SequenceNumber> {
        &mut self.unsent_changes
    }

    pub fn next_unsent_change<'a>(
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
            .changes()
            .iter()
            .find(|c| c.sequence_number() == next_seq_num);

        RtpsReaderLocatorCacheChange { cache_change }
    }

    pub fn unsent_changes(&self) -> Vec<SequenceNumber> {
        self.unsent_changes.clone()
    }

    pub fn send_message(
        &mut self,
        writer_cache: &WriterHistoryCache,
        header: RtpsMessageHeader,
        transport: &mut impl TransportWrite,
    ) {
        let mut submessages = Vec::new();
        while !self.unsent_changes().is_empty() {
            let change = self.next_unsent_change(writer_cache);
            // The post-condition:
            // "( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
            // should be full-filled by next_unsent_change()
            if change.is_in_cache() {
                let (info_ts_submessage, data_submessage) = change.into();
                submessages.push(RtpsSubmessageKind::InfoTimestamp(info_ts_submessage));
                submessages.push(RtpsSubmessageKind::Data(data_submessage));
            } else {
                let gap_submessage = change.into();
                submessages.push(RtpsSubmessageKind::Gap(gap_submessage));
            }
        }
        if !submessages.is_empty() {
            transport.write(&RtpsMessage::new(header, submessages), &[self.locator])
        }
    }
}

pub struct RtpsReaderLocatorCacheChange<'a> {
    cache_change: Option<&'a RtpsWriterCacheChange>,
}

impl RtpsReaderLocatorCacheChange<'_> {
    pub fn is_in_cache(&self) -> bool {
        self.cache_change.is_some()
    }
}

impl<'a> From<RtpsReaderLocatorCacheChange<'a>> for GapSubmessage {
    fn from(_val: RtpsReaderLocatorCacheChange<'a>) -> Self {
        todo!()
    }
}

impl<'a> From<RtpsReaderLocatorCacheChange<'a>> for (InfoTimestampSubmessage, DataSubmessage<'a>) {
    fn from(val: RtpsReaderLocatorCacheChange<'a>) -> Self {
        let cache_change = val
            .cache_change
            .expect("Can only convert to data if it exists in the writer cache");
        let info_ts_submessage = InfoTimestampSubmessage {
            endianness_flag: true,
            invalidate_flag: false,
            timestamp: Time::new(
                cache_change.timestamp().sec(),
                cache_change.timestamp().nanosec(),
            ),
        };
        let data_submessage = cache_change.as_data_submessage(ENTITYID_UNKNOWN);
        (info_ts_submessage, data_submessage)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        implementation::rtps::{
            history_cache::RtpsWriterCacheChange,
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
            vec![],
        ));
        hc.add_change(RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            SequenceNumber::new(2),
            TIME_INVALID,
            vec![],
            vec![],
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
