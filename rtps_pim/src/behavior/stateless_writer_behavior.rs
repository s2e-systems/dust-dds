use core::borrow::Borrow;

/// This file implements the behaviors described in 8.4.8 RTPS StatelessWriter Behavior
use crate::{
    messages::{
        submessage_elements::{
            CountSubmessageElement, EntityIdSubmessageElement, SequenceNumberSubmessageElement,
        },
        submessages::{AckNackSubmessage, DataSubmessage, GapSubmessage, HeartbeatSubmessage},
        types::Count,
    },
    structure::{
        history_cache::{RtpsHistoryCacheAttributes, RtpsHistoryCacheOperations},
        types::{EntityId, SequenceNumber, ENTITYID_UNKNOWN},
    },
};

use super::writer::reader_locator::RtpsReaderLocatorOperations;

pub struct BestEffortStatelessWriterBehavior;

impl BestEffortStatelessWriterBehavior {
    /// 8.4.8.1.4 Transition T4
    pub fn send_unsent_changes<CacheChange, WriterCacheChange, P, D>(
        reader_locator: &mut impl RtpsReaderLocatorOperations<CacheChangeType = CacheChange>,
        writer_cache: &impl RtpsHistoryCacheAttributes<CacheChangeType = WriterCacheChange>,
        mut send_data: impl FnMut(DataSubmessage<P, D>),
    ) where
        CacheChange: Into<DataSubmessage<P, D>> + Borrow<WriterCacheChange>,
        WriterCacheChange: PartialEq,
    {
        while let Some(change) = reader_locator.next_unsent_change() {
            if writer_cache.changes().iter().any(|c| c == change.borrow()) {
                let data_submessage = change.into();
                send_data(data_submessage);
            }
        }
    }
}

/// This struct is a wrapper for the implementation of the behaviors described in 8.4.8.2 Reliable StatelessWriter Behavior
pub struct ReliableStatelessWriterBehavior;

impl ReliableStatelessWriterBehavior {
    /// 8.4.8.2.4 Transition T4
    pub fn send_unsent_changes<'a, CacheChange, P, D>(
        reader_locator: &mut impl RtpsReaderLocatorOperations<CacheChangeType = CacheChange>,
        mut send_data: impl FnMut(DataSubmessage<P, D>),
    ) where
        CacheChange: Into<DataSubmessage<P, D>>,
    {
        while let Some(change) = reader_locator.next_unsent_change() {
            let data_submessage = change.into();
            send_data(data_submessage)
        }
    }

    /// 8.4.8.2.5 Transition T5
    pub fn send_heartbeat(
        writer_cache: &impl RtpsHistoryCacheOperations,
        writer_id: EntityId,
        heartbeat_count: Count,
        mut send_heartbeat: impl FnMut(HeartbeatSubmessage),
    ) {
        let endianness_flag = true;
        let final_flag = false;
        let liveliness_flag = false;
        let reader_id = EntityIdSubmessageElement {
            value: ENTITYID_UNKNOWN,
        };
        let writer_id = EntityIdSubmessageElement { value: writer_id };
        let first_sn = SequenceNumberSubmessageElement {
            value: writer_cache.get_seq_num_min().unwrap_or(0),
        };
        let last_sn = SequenceNumberSubmessageElement {
            value: writer_cache.get_seq_num_max().unwrap_or(0),
        };
        let count = CountSubmessageElement {
            value: heartbeat_count,
        };
        let heartbeat_submessage = HeartbeatSubmessage {
            endianness_flag,
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        };
        send_heartbeat(heartbeat_submessage)
    }

    /// 8.4.8.2.5 Transition T6
    /// Implementation does not include the part correponding to searching the reader locator
    /// on the stateless writer
    pub fn receive_acknack<S>(
        reader_locator: &mut impl RtpsReaderLocatorOperations,
        acknack: &AckNackSubmessage<S>,
    ) where
        S: AsRef<[SequenceNumber]>,
    {
        reader_locator.requested_changes_set(acknack.reader_sn_state.set.as_ref());
    }

    /// 8.4.9.2.12 Transition T10
    pub fn send_requested_changes<'a, P, D, CacheChange, S>(
        _reader_locator: &mut impl RtpsReaderLocatorOperations,
        _writer_cache: &'a impl RtpsHistoryCacheAttributes<CacheChangeType = CacheChange>,
        mut _send_data: impl FnMut(DataSubmessage<P, D>),
        mut _send_gap: impl FnMut(GapSubmessage<S>),
    ) {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use mockall::mock;

    use crate::messages::submessage_elements::{
        ParameterListSubmessageElement, SerializedDataSubmessageElement,
    };

    use super::*;

    mock! {
        CacheChange{}

        impl Into<DataSubmessage<(), ()>> for CacheChange {
            fn into(self) -> DataSubmessage<(), ()>;
        }

        impl PartialEq<MockCacheChange> for CacheChange {
            fn eq(&self, other: &MockCacheChange) -> bool;
        }
    }

    mock! {
        ReaderLocator{}

        impl RtpsReaderLocatorOperations for ReaderLocator {
            type CacheChangeType = MockCacheChange;
            type CacheChangeListType = Vec<i64>;

            fn next_requested_change(&mut self) -> Option<MockCacheChange>;
            fn next_unsent_change(&mut self) -> Option<MockCacheChange>;
            fn requested_changes(&self) -> Vec<i64>;
            fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]);
            fn unsent_changes(&self) -> Vec<i64>;
        }
    }

    mock! {
        DataMessageSender<'a>{
            fn send_data(&mut self, data: DataSubmessage<(), ()> );
        }
    }

    mock! {
        HistoryCache{}
        impl RtpsHistoryCacheAttributes for HistoryCache{
            type CacheChangeType = MockCacheChange;

            fn changes(&self) -> &[MockCacheChange];
        }
    }

    #[test]
    fn best_effort_stateless_writer_send_unsent_changes_single_data_submessage() {
        let mut seq = mockall::Sequence::new();

        let mut reader_locator = MockReaderLocator::new();
        let mut data_message_sender = MockDataMessageSender::new();
        let mut writer_cache = MockHistoryCache::new();
        writer_cache.expect_changes().return_const({
            let mut c = MockCacheChange::new();
            c.expect_eq().return_const(true);
            vec![MockCacheChange::new()]
        });

        const DATA_SUBMESSAGE: DataSubmessage<(), ()> = DataSubmessage {
            endianness_flag: false,
            inline_qos_flag: false,
            data_flag: false,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_sn: SequenceNumberSubmessageElement { value: 1 },
            inline_qos: ParameterListSubmessageElement { parameter: () },
            serialized_payload: SerializedDataSubmessageElement { value: () },
        };

        reader_locator
            .expect_next_unsent_change()
            .once()
            .returning(|| {
                let mut c = MockCacheChange::new();
                c.expect_eq().return_const(true);
                c.expect_into().returning(|| DATA_SUBMESSAGE);
                Some(c)
            })
            .in_sequence(&mut seq);

        data_message_sender
            .expect_send_data()
            .once()
            .withf(|data| data.writer_sn.value == 1)
            .return_const(())
            .in_sequence(&mut seq);

        reader_locator
            .expect_next_unsent_change()
            .once()
            .returning(|| None)
            .in_sequence(&mut seq);

        BestEffortStatelessWriterBehavior::send_unsent_changes(
            &mut reader_locator,
            &writer_cache,
            |data| data_message_sender.send_data(data),
        )
    }
}
