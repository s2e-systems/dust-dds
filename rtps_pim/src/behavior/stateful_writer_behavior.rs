/// This file implements the behaviors described in 8.4.9 RTPS StatefulWriter Behavior
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
        types::{EntityId, GuidPrefix, SequenceNumber, ENTITYID_UNKNOWN},
    },
};

use super::writer::{
    change_for_reader::RtpsChangeForReaderAttributes,
    reader_proxy::{RtpsReaderProxyAttributes, RtpsReaderProxyOperations},
};

pub trait RtpsStatefulWriterReceiveAckNackSubmessage<S> {
    fn on_acknack_submessage_received(
        &mut self,
        acknack_submessage: &AckNackSubmessage<S>,
        source_guid_prefix: GuidPrefix,
    );
}

pub trait RtpsStatefulWriterSendSubmessages<'a, P, D, S> {
    type ReaderProxyType;

    fn send_submessages(
        &'a mut self,
        send_data: impl FnMut(&Self::ReaderProxyType, DataSubmessage<P, D>),
        send_gap: impl FnMut(&Self::ReaderProxyType, GapSubmessage<S>),
        send_heartbeat: impl FnMut(&Self::ReaderProxyType, HeartbeatSubmessage),
    );
}

pub enum BestEffortStatefulWriterSendSubmessage<P, D, S> {
    Data(DataSubmessage<P, D>),
    Gap(GapSubmessage<S>),
}

pub trait BestEffortReaderProxyUnsentChangesBehavior<P, D, S> {
    fn send_unsent_changes(&mut self) -> Option<BestEffortStatefulWriterSendSubmessage<P, D, S>>;
}

impl<T, P, D, S> BestEffortReaderProxyUnsentChangesBehavior<P, D, S> for T
where
    T: RtpsReaderProxyOperations + RtpsReaderProxyAttributes,
    T::ChangeForReaderListType: IntoIterator,
    <T as RtpsReaderProxyOperations>::ChangeForReaderType:
        Into<DataSubmessage<P, D>> + Into<GapSubmessage<S>> + RtpsChangeForReaderAttributes,
{
    fn send_unsent_changes(&mut self) -> Option<BestEffortStatefulWriterSendSubmessage<P, D, S>> {
        // Note: The readerId is set to the remote reader ID as described in 8.4.9.2.12 Transition T12
        // in confront to ENTITYID_UNKNOWN as described in 8.4.9.1.4 Transition T4
        let reader_id = self.remote_reader_guid().entity_id();

        if self.unsent_changes().into_iter().next().is_some() {
            let change = self.next_unsent_change();
            // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
            // it's not done here to avoid the change being a mutable reference
            // Also the post-condition:
            // "( a_change BELONGS-TO the_reader_proxy.unsent_changes() ) == FALSE"
            // should be full-filled by next_unsent_change()
            if change.is_relevant() {
                let mut data_submessage: DataSubmessage<P, D> = change.into();
                data_submessage.reader_id.value = reader_id;
                Some(BestEffortStatefulWriterSendSubmessage::Data(
                    data_submessage,
                ))
            } else {
                let mut gap_submessage: GapSubmessage<S> = change.into();
                gap_submessage.reader_id.value = reader_id;
                Some(BestEffortStatefulWriterSendSubmessage::Gap(gap_submessage))
            }
        } else {
            None
        }
    }
}

pub enum ReliableStatefulWriterSendSubmessage<P, D, S> {
    Data(DataSubmessage<P, D>),
    Gap(GapSubmessage<S>),
}

pub trait ReliableReaderProxyUnsentChangesBehavior<P, D, S> {
    fn send_unsent_changes(&mut self) -> Option<ReliableStatefulWriterSendSubmessage<P, D, S>>;
}

impl<T, P, D, S> ReliableReaderProxyUnsentChangesBehavior<P, D, S> for T
where
    T: RtpsReaderProxyOperations + RtpsReaderProxyAttributes,
    T::ChangeForReaderListType: IntoIterator,
    <T as RtpsReaderProxyOperations>::ChangeForReaderType:
        Into<DataSubmessage<P, D>> + Into<GapSubmessage<S>> + RtpsChangeForReaderAttributes,
{
    fn send_unsent_changes(&mut self) -> Option<ReliableStatefulWriterSendSubmessage<P, D, S>> {
        // Note: The readerId is set to the remote reader ID as described in 8.4.9.2.12 Transition T12
        // in confront to ENTITYID_UNKNOWN as described in 8.4.9.2.4 Transition T4
        let reader_id = self.remote_reader_guid().entity_id();

        if self.unsent_changes().into_iter().next().is_some() {
            let change = self.next_unsent_change();
            // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
            // it's not done here to avoid the change being a mutable reference
            // Also the post-condition:
            // "( a_change BELONGS-TO the_reader_proxy.unsent_changes() ) == FALSE"
            // should be full-filled by next_unsent_change()
            if change.is_relevant() {
                let mut data_submessage: DataSubmessage<P, D> = change.into();
                data_submessage.reader_id.value = reader_id;
                Some(ReliableStatefulWriterSendSubmessage::Data(data_submessage))
            } else {
                let mut gap_submessage: GapSubmessage<S> = change.into();
                gap_submessage.reader_id.value = reader_id;
                Some(ReliableStatefulWriterSendSubmessage::Gap(gap_submessage))
            }
        } else {
            None
        }
    }
}

pub trait ReliableReaderProxySendHeartbeatBehavior {
    fn send_heartbeat(&self, writer_id: EntityId) -> HeartbeatSubmessage;
}

impl<T> ReliableReaderProxySendHeartbeatBehavior for T
where
    T: RtpsHistoryCacheOperations,
{
    fn send_heartbeat(&self, writer_id: EntityId) -> HeartbeatSubmessage {
        let endianness_flag = true;
        let final_flag = false;
        let liveliness_flag = false;
        let reader_id = EntityIdSubmessageElement {
            value: ENTITYID_UNKNOWN,
        };
        let writer_id = EntityIdSubmessageElement { value: writer_id };
        let first_sn = SequenceNumberSubmessageElement {
            value: self.get_seq_num_min().unwrap_or(1),
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

pub trait ReliableReaderProxyReceiveAcknackBehavior<S> {
    fn receive_acknack(&mut self, acknack: &AckNackSubmessage<S>);
}

impl<S, T> ReliableReaderProxyReceiveAcknackBehavior<S> for T
where
    T: RtpsReaderProxyOperations,
    S: AsRef<[SequenceNumber]>,
{
    fn receive_acknack(&mut self, acknack: &AckNackSubmessage<S>) {
        self.acked_changes_set(acknack.reader_sn_state.base - 1);
        self.requested_changes_set(acknack.reader_sn_state.set.as_ref());
    }
}
pub trait ReliableReaderProxyRequestedChangesBehavior<P, D, S> {
    fn send_requested_changes(&mut self) -> Option<ReliableStatefulWriterSendSubmessage<P, D, S>>;
}

impl<T, P, D, S> ReliableReaderProxyRequestedChangesBehavior<P, D, S> for T
where
    T: RtpsReaderProxyOperations + RtpsReaderProxyAttributes,
    T::ChangeForReaderListType: IntoIterator,
    <T as RtpsReaderProxyOperations>::ChangeForReaderType:
        Into<DataSubmessage<P, D>> + Into<GapSubmessage<S>> + RtpsChangeForReaderAttributes,
{
    fn send_requested_changes(&mut self) -> Option<ReliableStatefulWriterSendSubmessage<P, D, S>> {
        let reader_id = self.remote_reader_guid().entity_id();

        if self.requested_changes().into_iter().next().is_some() {
            let change_for_reader = self.next_requested_change();
            // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
            // it's not done here to avoid the change being a mutable reference
            // Also the post-condition:
            // a_change BELONGS-TO the_reader_proxy.requested_changes() ) == FALSE
            // should be full-filled by next_requested_change()
            if change_for_reader.is_relevant() {
                let mut data_submessage: DataSubmessage<P, D> = change_for_reader.into();
                data_submessage.reader_id.value = reader_id;
                Some(ReliableStatefulWriterSendSubmessage::Data(data_submessage))
            } else {
                let mut gap_submessage: GapSubmessage<S> = change_for_reader.into();
                gap_submessage.reader_id.value = reader_id;
                Some(ReliableStatefulWriterSendSubmessage::Gap(gap_submessage))
            }
        } else {
            None
        }
    }
}
