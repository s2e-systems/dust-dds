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
        history_cache::RtpsHistoryCacheOperations,
        types::{EntityId, SequenceNumber, ENTITYID_UNKNOWN},
    },
};

use super::writer::reader_locator::RtpsReaderLocatorOperations;

pub trait RtpsStatelessWriterSendSubmessages<'a, P, D, S> {
    type ReaderLocatorType;

    fn send_submessages(
        &'a mut self,
        send_data: impl FnMut(&Self::ReaderLocatorType, DataSubmessage<P, D>),
        send_gap: impl FnMut(&Self::ReaderLocatorType, GapSubmessage<S>),
        send_heartbeat: impl FnMut(&Self::ReaderLocatorType, HeartbeatSubmessage),
    );
}

pub trait RtpsStatelessWriterReceiveAckNackSubmessage<S> {
    fn on_acknack_submessage_received(&mut self, acknack_submessage: &AckNackSubmessage<S>);
}

pub trait ChangeInHistoryCache {
    fn is_in_cache(&self) -> bool;
}

pub enum BestEffortStatelessWriterSendSubmessage<P, D, S> {
    Data(DataSubmessage<P, D>),
    Gap(GapSubmessage<S>),
}

pub trait BestEffortReaderLocatorUnsentChangesBehavior<P, D, S> {
    fn send_unsent_changes(&mut self) -> Option<BestEffortStatelessWriterSendSubmessage<P, D, S>>;
}

impl<T, P, D, S> BestEffortReaderLocatorUnsentChangesBehavior<P, D, S> for T
where
    T: RtpsReaderLocatorOperations,
    T::CacheChangeListType: IntoIterator,
    T::CacheChangeType: Into<DataSubmessage<P, D>> + Into<GapSubmessage<S>> + ChangeInHistoryCache,
{
    /// 8.4.8.1.4 Transition T4
    fn send_unsent_changes(&mut self) -> Option<BestEffortStatelessWriterSendSubmessage<P, D, S>> {
        if self.unsent_changes().into_iter().next().is_some() {
            let change = self.next_unsent_change();
            // The post-condition:
            // "( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
            // should be full-filled by next_unsent_change()
            if change.is_in_cache() {
                let data_submessage = change.into();
                Some(BestEffortStatelessWriterSendSubmessage::Data(
                    data_submessage,
                ))
            } else {
                let gap_submessage = change.into();
                Some(BestEffortStatelessWriterSendSubmessage::Gap(gap_submessage))
            }
        } else {
            None
        }
    }
}

pub enum ReliableStatelessWriterSendSubmessage<P, D, S> {
    Data(DataSubmessage<P, D>),
    Gap(GapSubmessage<S>),
}

/// This struct is a wrapper for the implementation of the behaviors described in 8.4.8.2 Reliable StatelessWriter Behavior
pub trait ReliableReaderLocatorUnsentChangesBehavior<P, D> {
    fn send_unsent_changes(&mut self) -> Option<DataSubmessage<P, D>>;
}

impl<T, P, D> ReliableReaderLocatorUnsentChangesBehavior<P, D> for T
where
    T: RtpsReaderLocatorOperations,
    T::CacheChangeListType: IntoIterator,
    T::CacheChangeType: Into<DataSubmessage<P, D>>,
{
    /// 8.4.8.2.4 Transition T4
    fn send_unsent_changes(&mut self) -> Option<DataSubmessage<P, D>> {
        if self.unsent_changes().into_iter().next().is_some() {
            let change = self.next_unsent_change();
            // The post-condition:
            // "( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
            // should be full-filled by next_unsent_change()
            let data_submessage = change.into();
            Some(data_submessage)
        } else {
            None
        }
    }
}

pub trait ReliableReaderLocatorSendHeartbeatBehavior {
    fn send_heartbeat(&mut self, writer_id: EntityId) -> HeartbeatSubmessage;
}

impl<T> ReliableReaderLocatorSendHeartbeatBehavior for T
where
    T: RtpsHistoryCacheOperations,
{
    fn send_heartbeat(&mut self, writer_id: EntityId) -> HeartbeatSubmessage {
        let endianness_flag = true;
        let final_flag = false;
        let liveliness_flag = false;
        let reader_id = EntityIdSubmessageElement {
            value: ENTITYID_UNKNOWN,
        };
        let writer_id = EntityIdSubmessageElement { value: writer_id };
        let first_sn = SequenceNumberSubmessageElement {
            value: self.get_seq_num_min().unwrap_or(0),
        };
        let last_sn = SequenceNumberSubmessageElement {
            value: self.get_seq_num_max().unwrap_or(0),
        };
        let count = CountSubmessageElement { value: Count(0) };

        HeartbeatSubmessage {
            endianness_flag,
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        }
    }
}

pub trait ReliableReaderLocatorReceiveAcknackBehavior<S> {
    fn receive_acknack(&mut self, acknack: &AckNackSubmessage<S>);
}

impl<S, T> ReliableReaderLocatorReceiveAcknackBehavior<S> for T
where
    T: RtpsReaderLocatorOperations,

    S: AsRef<[SequenceNumber]>,
{
    /// 8.4.8.2.5 Transition T6
    /// Implementation does not include the part corresponding to searching the reader locator
    /// on the stateless writer
    fn receive_acknack(&mut self, acknack: &AckNackSubmessage<S>) {
        self.requested_changes_set(acknack.reader_sn_state.set.as_ref());
    }
}

pub trait ReliableReaderLocatorRequestedChangesBehavior<P, D, S> {
    fn send_requested_changes(&mut self) -> Option<ReliableStatelessWriterSendSubmessage<P, D, S>>;
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

        impl ChangeInHistoryCache for CacheChange {
            fn is_in_cache(&self) -> bool;
        }
    }

    impl Into<GapSubmessage<()>> for MockCacheChange {
        fn into(self) -> GapSubmessage<()> {
            todo!()
        }
    }

    mock! {
        ReaderLocator{}

        impl RtpsReaderLocatorOperations for ReaderLocator {
            type CacheChangeType = MockCacheChange;
            type CacheChangeListType = Vec<i64>;

            fn next_requested_change(&mut self) -> MockCacheChange;
            fn next_unsent_change(&mut self) -> MockCacheChange;
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

    #[test]
    fn best_effort_stateless_writer_send_unsent_changes_single_data_submessage() {
        let mut seq = mockall::Sequence::new();

        let mut reader_locator = MockReaderLocator::new();

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
            .expect_unsent_changes()
            .once()
            .returning(|| vec![1])
            .in_sequence(&mut seq);

        reader_locator
            .expect_next_unsent_change()
            .once()
            .returning(|| {
                let mut cache_change = MockCacheChange::new();
                cache_change.expect_is_in_cache().return_const(true);
                cache_change.expect_into().returning(|| DATA_SUBMESSAGE);
                cache_change
            })
            .in_sequence(&mut seq);

        assert!(matches!(
            BestEffortReaderLocatorUnsentChangesBehavior::send_unsent_changes(&mut reader_locator,),
            Some(BestEffortStatelessWriterSendSubmessage::Data(_))
        ));
    }
}
