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
        types::{EntityId, SequenceNumber, ENTITYID_UNKNOWN},
    },
};

use super::writer::{
    change_for_reader::RtpsChangeForReaderAttributes,
    reader_proxy::{RtpsReaderProxyAttributes, RtpsReaderProxyOperations},
};

pub trait StatefulWriterSendSubmessages<'a, P, D, S> {
    type ReaderProxyType;

    fn send_submessages(
        &'a mut self,
        send_data: impl FnMut(&Self::ReaderProxyType, DataSubmessage<P, D>),
        send_gap: impl FnMut(&Self::ReaderProxyType, GapSubmessage<S>),
        send_heartbeat: impl FnMut(&Self::ReaderProxyType, HeartbeatSubmessage),
    );
}

trait IsEmpty {
    fn is_empty(self) -> bool;
}

impl<T: IntoIterator> IsEmpty for T {
    fn is_empty(self) -> bool {
        self.into_iter().next().is_none()
    }
}

pub trait BestEffortReaderProxyUnsentChangesBehavior<P, D, S> {
    fn send_unsent_changes(
        &mut self,
        send_data: impl FnMut(&Self, DataSubmessage<P, D>),
        send_gap: impl FnMut(&Self, GapSubmessage<S>),
    );
}

impl<T, P, D, S> BestEffortReaderProxyUnsentChangesBehavior<P, D, S> for T
where
    T: RtpsReaderProxyOperations + RtpsReaderProxyAttributes,
    T::ChangeForReaderListType: IntoIterator,
    <T as RtpsReaderProxyOperations>::ChangeForReaderType:
        Into<DataSubmessage<P, D>> + Into<GapSubmessage<S>> + RtpsChangeForReaderAttributes,
{
    fn send_unsent_changes(
        &mut self,
        mut send_data: impl FnMut(&Self, DataSubmessage<P, D>),
        mut send_gap: impl FnMut(&Self, GapSubmessage<S>),
    ) {
        // Note: The readerId is set to the remote reader ID as described in 8.4.9.2.12 Transition T12
        // in confront to ENTITYID_UNKNOWN as described in 8.4.9.1.4 Transition T4
        let reader_id = self.remote_reader_guid().entity_id();

        while !self.unsent_changes().is_empty() {
            let change = self.next_unsent_change();
            // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
            // it's not done here to avoid the change being a mutable reference
            // Also the post-condition:
            // "( a_change BELONGS-TO the_reader_proxy.unsent_changes() ) == FALSE"
            // should be full-filled by next_unsent_change()
            if change.is_relevant() {
                let mut data_submessage: DataSubmessage<P, D> = change.into();
                data_submessage.reader_id.value = reader_id;
                send_data(self, data_submessage)
            } else {
                let mut gap_submessage: GapSubmessage<S> = change.into();
                gap_submessage.reader_id.value = reader_id;
                send_gap(self, gap_submessage)
            }
        }
    }
}

pub trait ReliableReaderProxyUnsentChangesBehavior<P, D, S> {
    fn send_unsent_changes(
        &mut self,
        send_data: impl FnMut(&Self, DataSubmessage<P, D>),
        send_gap: impl FnMut(&Self, GapSubmessage<S>),
    );
}

impl<T, P, D, S> ReliableReaderProxyUnsentChangesBehavior<P, D, S> for T
where
    T: RtpsReaderProxyOperations + RtpsReaderProxyAttributes,
    T::ChangeForReaderListType: IntoIterator,
    <T as RtpsReaderProxyOperations>::ChangeForReaderType:
        Into<DataSubmessage<P, D>> + Into<GapSubmessage<S>> + RtpsChangeForReaderAttributes,
{
    fn send_unsent_changes(
        &mut self,
        mut send_data: impl FnMut(&Self, DataSubmessage<P, D>),
        mut send_gap: impl FnMut(&Self, GapSubmessage<S>),
    ) {
        // Note: The readerId is set to the remote reader ID as described in 8.4.9.2.12 Transition T12
        // in confront to ENTITYID_UNKNOWN as described in 8.4.9.2.4 Transition T4
        let reader_id = self.remote_reader_guid().entity_id();

        while !self.unsent_changes().is_empty() {
            let change = self.next_unsent_change();
            // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
            // it's not done here to avoid the change being a mutable reference
            // Also the post-condition:
            // "( a_change BELONGS-TO the_reader_proxy.unsent_changes() ) == FALSE"
            // should be full-filled by next_unsent_change()
            if change.is_relevant() {
                let mut data_submessage: DataSubmessage<P, D> = change.into();
                data_submessage.reader_id.value = reader_id;
                send_data(self, data_submessage)
            } else {
                let mut gap_submessage: GapSubmessage<S> = change.into();
                gap_submessage.reader_id.value = reader_id;
                send_gap(self, gap_submessage)
            }
        }
    }
}

pub trait ReliableReaderProxySendHeartbeatBehavior {
    fn send_heartbeat(
        &self,
        writer_id: EntityId,
        heartbeat_count: Count,
        send_heartbeat: impl FnMut(&Self, HeartbeatSubmessage),
    );
}

impl<T> ReliableReaderProxySendHeartbeatBehavior for T
where
    T: RtpsHistoryCacheOperations,
{
    fn send_heartbeat(
        &self,
        writer_id: EntityId,
        heartbeat_count: Count,
        mut send_heartbeat: impl FnMut(&Self, HeartbeatSubmessage),
    ) {
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
        send_heartbeat(self, heartbeat_submessage)
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
    fn send_requested_changes(
        &mut self,
        send_data: impl FnMut(&Self, DataSubmessage<P, D>),
        send_gap: impl FnMut(&Self, GapSubmessage<S>),
    );
}

impl<T, P, D, S> ReliableReaderProxyRequestedChangesBehavior<P, D, S> for T
where
    T: RtpsReaderProxyOperations + RtpsReaderProxyAttributes,
    T::ChangeForReaderListType: IntoIterator,
    <T as RtpsReaderProxyOperations>::ChangeForReaderType:
        Into<DataSubmessage<P, D>> + Into<GapSubmessage<S>> + RtpsChangeForReaderAttributes,
{
    fn send_requested_changes(
        &mut self,
        mut send_data: impl FnMut(&Self, DataSubmessage<P, D>),
        mut send_gap: impl FnMut(&Self, GapSubmessage<S>),
    ) {
        let reader_id = self.remote_reader_guid().entity_id();

        while !self.requested_changes().is_empty() {
            let change_for_reader = self.next_requested_change();
            // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
            // it's not done here to avoid the change being a mutable reference
            // Also the post-condition:
            // a_change BELONGS-TO the_reader_proxy.requested_changes() ) == FALSE
            // should be full-filled by next_requested_change()
            if change_for_reader.is_relevant() {
                let mut data_submessage: DataSubmessage<P, D> = change_for_reader.into();
                data_submessage.reader_id.value = reader_id;
                send_data(self, data_submessage)
            } else {
                let mut gap_submessage: GapSubmessage<S> = change_for_reader.into();
                gap_submessage.reader_id.value = reader_id;
                send_gap(self, gap_submessage)
            }
        }
    }
}
