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

trait IsEmpty {
    fn is_empty(self) -> bool;
}

impl<T: IntoIterator> IsEmpty for T {
    fn is_empty(self) -> bool {
        self.into_iter().next().is_none()
    }
}
pub struct BestEffortStatefulWriterBehavior;

impl BestEffortStatefulWriterBehavior {
    /// 8.4.9.1.4 Transition T4
    pub fn send_unsent_changes<P, D, S>(
        reader_proxy: &mut (impl RtpsReaderProxyOperations<
            ChangeForReaderListType = impl IntoIterator,
            ChangeForReaderType = (impl Into<DataSubmessage<P, D>>
                                       + Into<GapSubmessage<S>>
                                       + RtpsChangeForReaderAttributes),
        > + RtpsReaderProxyAttributes),
        mut send_data: impl FnMut(DataSubmessage<P, D>),
        mut send_gap: impl FnMut(GapSubmessage<S>),
    ) {
        // Note: The readerId is set to the remote reader ID as described in 8.4.9.2.12 Transition T12
        // in confront to ENTITYID_UNKNOWN as described in 8.4.9.1.4 Transition T4
        let reader_id = reader_proxy.remote_reader_guid().entity_id();

        while !reader_proxy.unsent_changes().is_empty() {
            let change = reader_proxy.next_unsent_change();
            // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
            // it's not done here to avoid the change being a mutable reference
            // Also the post-condition:
            // "( a_change BELONGS-TO the_reader_proxy.unsent_changes() ) == FALSE"
            // should be full-filled by next_unsent_change()
            if change.is_relevant() {
                let mut data_submessage: DataSubmessage<P, D> = change.into();
                data_submessage.reader_id.value = reader_id;
                send_data(data_submessage)
            } else {
                let mut gap_submessage: GapSubmessage<S> = change.into();
                gap_submessage.reader_id.value = reader_id;
                send_gap(gap_submessage)
            }
        }
    }
}


/// This struct is a wrapper for the implementation of the behaviors described in 8.4.9.2 Reliable StatefulWriter Behavior
pub struct ReliableStatefulWriterBehavior;

impl ReliableStatefulWriterBehavior {
    /// Implement 8.4.9.2.4 Transition T4
    pub fn send_unsent_changes<P, D, S>(
        reader_proxy: &mut (impl RtpsReaderProxyOperations<
            ChangeForReaderListType = impl IntoIterator,
            ChangeForReaderType = (impl Into<DataSubmessage<P, D>>
                                       + Into<GapSubmessage<S>>
                                       + RtpsChangeForReaderAttributes),
        > + RtpsReaderProxyAttributes),
        mut send_data: impl FnMut(DataSubmessage<P, D>),
        mut send_gap: impl FnMut(GapSubmessage<S>),
    ) {
        // Note: The readerId is set to the remote reader ID as described in 8.4.9.2.12 Transition T12
        // in confront to ENTITYID_UNKNOWN as described in 8.4.9.2.4 Transition T4
        let reader_id = reader_proxy.remote_reader_guid().entity_id();

        while !reader_proxy.unsent_changes().is_empty() {
            let change = reader_proxy.next_unsent_change();
            // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
            // it's not done here to avoid the change being a mutable reference
            // Also the post-condition:
            // "( a_change BELONGS-TO the_reader_proxy.unsent_changes() ) == FALSE"
            // should be full-filled by next_unsent_change()
            if change.is_relevant() {
                let mut data_submessage: DataSubmessage<P, D> = change.into();
                data_submessage.reader_id.value = reader_id;
                send_data(data_submessage)
            } else {
                let mut gap_submessage: GapSubmessage<S> = change.into();
                gap_submessage.reader_id.value = reader_id;
                send_gap(gap_submessage)
            }
        }
    }

    /// 8.4.9.2.7 Transition T7
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
            value: writer_cache.get_seq_num_min().unwrap_or(1),
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

    /// 8.4.9.2.8 Transition T8
    pub fn receive_acknack<S>(
        reader_proxy: &mut impl RtpsReaderProxyOperations,
        acknack: &AckNackSubmessage<S>,
    ) where
        S: AsRef<[SequenceNumber]>,
    {
        reader_proxy.acked_changes_set(acknack.reader_sn_state.base - 1);
        reader_proxy.requested_changes_set(acknack.reader_sn_state.set.as_ref());
    }

    /// 8.4.9.2.12 Transition T12
    pub fn send_requested_changes<P, D, S>(
        reader_proxy: &mut (impl RtpsReaderProxyOperations<
            ChangeForReaderListType = impl IntoIterator,
            ChangeForReaderType = (impl Into<DataSubmessage<P, D>>
                                       + Into<GapSubmessage<S>>
                                       + RtpsChangeForReaderAttributes),
        > + RtpsReaderProxyAttributes),
        mut send_data: impl FnMut(DataSubmessage<P, D>),
        mut send_gap: impl FnMut(GapSubmessage<S>),
    ) {
        let reader_id = reader_proxy.remote_reader_guid().entity_id();

        while !reader_proxy.requested_changes().is_empty() {
            let change_for_reader = reader_proxy.next_requested_change();
            // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
            // it's not done here to avoid the change being a mutable reference
            // Also the post-condition:
            // a_change BELONGS-TO the_reader_proxy.requested_changes() ) == FALSE
            // should be full-filled by next_requested_change()
            if change_for_reader.is_relevant() {
                let mut data_submessage: DataSubmessage<P, D> = change_for_reader.into();
                data_submessage.reader_id.value = reader_id;
                send_data(data_submessage)
            } else {
                let mut gap_submessage: GapSubmessage<S> = change_for_reader.into();
                gap_submessage.reader_id.value = reader_id;
                send_gap(gap_submessage)
            }
        }
    }
}
